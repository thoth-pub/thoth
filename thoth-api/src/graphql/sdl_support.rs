//! Shared SDL extraction for the generated-schema guards.
//!
//! The `BE-02` and `BE-03` guards assert that a given generated type exposes no
//! forbidden field. Those assertions are only as strong as the block they
//! inspect, so the extraction below is the security-relevant part of them: a
//! guard that silently inspects half a type body still passes while the field it
//! was written to catch sits in the half it never looked at.

/// The complete body of one SDL type, input or enum declaration.
///
/// `declaration` must include the opening `{` (for example `"type Publisher {"`).
/// The returned slice is everything between that brace and its **matching**
/// closing brace.
///
/// Extraction is brace-balanced and string-aware. Both properties are required
/// by the real generated schema:
///
/// - a field argument may carry a nested object default, as
///   `Publisher.imprints` does with
///   `order: ImprintOrderBy = {direction: "ASC", field: "IMPRINT_NAME"}`;
/// - a description may contain a brace, as the `doi` description does with
///   `\d{4,9}`, or an escaped quote, as the `Timestamp` description does;
/// - a description may be a `"""` block string, as `Imprint.crossmarkDoi` is.
///
/// A naive `split_once('}')` stops at the first closing brace of any kind. On
/// `type Publisher` that is the brace closing the `imprints` order default, so
/// everything declared after `imprints` — `contacts` and
/// `distributionPlatforms` — would never be inspected, and a protected field
/// added there would pass the guard unnoticed.
pub(crate) fn sdl_block<'a>(sdl: &'a str, declaration: &str) -> &'a str {
    let body = sdl
        .split_once(declaration)
        .unwrap_or_else(|| panic!("SDL must declare `{declaration}`"))
        .1;

    let bytes = body.as_bytes();
    let mut index = 0;
    // The declaration consumed the opening brace, so the body starts one deep.
    let mut depth = 1usize;

    while index < bytes.len() {
        // Only ASCII bytes are matched below, and every byte of a multi-byte
        // UTF-8 sequence is >= 0x80, so `index` is always a char boundary.
        if bytes[index..].starts_with(br#"""""#) {
            index = skip_block_string(bytes, index);
        } else if bytes[index] == b'"' {
            index = skip_quoted_string(bytes, index);
        } else {
            match bytes[index] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return &body[..index];
                    }
                }
                _ => {}
            }
            index += 1;
        }
    }

    panic!("`{declaration}` body is not brace-balanced")
}

/// The index just past the `"""` block string opening at `start`.
fn skip_block_string(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 3;
    while index < bytes.len() {
        if bytes[index..].starts_with(br#"""""#) {
            return index + 3;
        }
        index += 1;
    }
    bytes.len()
}

/// The index just past the `"` string opening at `start`, honouring `\` escapes.
fn skip_quoted_string(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::sdl_block;
    use crate::graphql::create_schema;

    #[test]
    fn extraction_spans_a_nested_object_default() {
        let sdl = "type Publisher {\n  imprints(order: X = {direction: \"ASC\"}): [Imprint!]!\n  \
                   distributionPlatforms: [A!]!\n}\n\ntype Next {\n  other: Int!\n}\n";
        let block = sdl_block(sdl, "type Publisher {");

        assert!(block.contains("imprints("));
        assert!(
            block.contains("distributionPlatforms"),
            "extraction truncated at the nested default: {block}"
        );
        // The following declaration must not bleed in.
        assert!(!block.contains("other: Int!"));
    }

    #[test]
    fn extraction_ignores_braces_and_quotes_inside_descriptions() {
        let sdl = "type T {\n  \"Expressed as `\\\\d{4,9}` and \\\"quoted\\\"\"\n  a: Int!\n  \
                   \"\"\"\n  A block } description {\n  \"\"\"\n  b: Int!\n}\n";
        let block = sdl_block(sdl, "type T {");

        assert!(block.contains("a: Int!"));
        assert!(
            block.contains("b: Int!"),
            "a brace inside a description truncated the body: {block}"
        );
    }

    #[test]
    fn extraction_covers_the_whole_real_publisher_declaration() {
        let sdl = create_schema().as_sdl();
        let block = sdl_block(&sdl, "type Publisher {");

        // `distributionPlatforms` is declared after `imprints`, whose nested
        // order default is exactly what truncated the previous extraction.
        for post_imprints_sentinel in ["contacts(", "distributionPlatforms:"] {
            assert!(
                block.contains(post_imprints_sentinel),
                "extraction missed `{post_imprints_sentinel}`, declared after `imprints`: {block}"
            );
        }
        // The block stops at its own closing brace.
        assert!(!block.contains("type PublisherContext"));
    }

    #[test]
    fn a_forbidden_field_inserted_after_imprints_is_caught() {
        // The regression the previous extraction could not catch: a protected
        // field smuggled in *after* the nested order default.
        let sdl = create_schema().as_sdl();
        let (head, tail) = sdl
            .split_once("  \"Get contacts linked to this publisher\"")
            .expect("the real Publisher type declares `contacts` after `imprints`");
        let tampered = format!("{head}  subscriptionPackage: ThothPackage!\n  \"Get contacts linked to this publisher\"{tail}");

        let block = sdl_block(&tampered, "type Publisher {");
        assert!(
            block.contains("subscriptionPackage"),
            "the guard would not see a protected field inserted after `imprints`"
        );
    }

    #[test]
    #[should_panic(expected = "not brace-balanced")]
    fn an_unbalanced_declaration_fails_loudly() {
        sdl_block("type Broken {\n  a: Int!\n", "type Broken {");
    }

    #[test]
    #[should_panic(expected = "SDL must declare")]
    fn a_missing_declaration_fails_loudly() {
        sdl_block("type Other {\n}\n", "type Absent {");
    }
}
