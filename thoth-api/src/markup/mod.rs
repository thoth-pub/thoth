use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use thoth_errors::{ThothError, ThothResult};

pub mod ast;

use ast::{
    ast_to_html, ast_to_jats, ast_to_markdown, ast_to_plain_text, html_to_ast, jats_to_ast,
    markdown_to_ast, normalise_crossref_abstract_ast, plain_text_ast_to_jats, plain_text_to_ast,
    strip_structural_elements_from_ast_for_conversion, validate_ast_content,
};

/// Enum to represent the markup format
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(
        description = "Allowed markup formats for text fields that support structured content"
    ),
    ExistingTypePath = "crate::schema::sql_types::MarkupFormat"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum MarkupFormat {
    #[cfg_attr(feature = "backend", graphql(description = "HTML format"))]
    Html,
    #[cfg_attr(feature = "backend", graphql(description = "Markdown format"))]
    Markdown,
    #[cfg_attr(feature = "backend", graphql(description = "Plain text format"))]
    PlainText,
    #[cfg_attr(feature = "backend", graphql(description = "JATS XML format"))]
    #[default]
    JatsXml,
}

/// Limits how much structure is preserved/allowed when converting to/from JATS.
///
/// - `Abstract`/`Biography`: allow basic structural elements (paragraphs, lists, emphasis, links).
/// - `Title`: disallow structure; structural tags are stripped to plain inline text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionLimit {
    Abstract,
    Biography,
    Title,
}

fn looks_like_markup(content: &str) -> bool {
    regex::Regex::new(r"</?[A-Za-z][^>]*>")
        .unwrap()
        .is_match(content)
}

fn validate_jats_subset(content: &str, conversion_limit: ConversionLimit) -> ThothResult<()> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    fn local_tag_name(raw_name: &str) -> &str {
        raw_name.rsplit(':').next().unwrap_or(raw_name)
    }

    let allowed_tags: &[&str] = match conversion_limit {
        ConversionLimit::Title => &[
            "bold",
            "italic",
            "underline",
            "strike",
            "monospace",
            "sup",
            "sub",
            "sc",
            "ext-link",
            "inline-formula",
            "tex-math",
            "email",
            "uri",
        ],
        ConversionLimit::Abstract | ConversionLimit::Biography => &[
            "p",
            "bold",
            "italic",
            "underline",
            "strike",
            "monospace",
            "sup",
            "sub",
            "sc",
            "list",
            "list-item",
            "ext-link",
            "inline-formula",
            "tex-math",
            "email",
            "uri",
        ],
    };

    let wrapped = format!(
        r#"<root xmlns:xlink="http://www.w3.org/1999/xlink">{}</root>"#,
        content
    );
    let mut reader = Reader::from_str(&wrapped);
    let mut buf = Vec::new();
    let mut open_tags: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let tag_name = local_tag_name(&raw_name).to_string();
                if raw_name != "root" {
                    if !allowed_tags.contains(&tag_name.as_str()) {
                        return Err(ThothError::RequestError(format!(
                            "Unsupported JATS element: <{}>",
                            tag_name
                        )));
                    }

                    if matches!(
                        conversion_limit,
                        ConversionLimit::Abstract | ConversionLimit::Biography
                    ) && matches!(open_tags.last().map(String::as_str), Some("p"))
                        && matches!(tag_name.as_str(), "p" | "list" | "list-item")
                    {
                        return Err(ThothError::RequestError(
                            "Abstracts and biographies cannot contain nested block elements inside paragraphs."
                                .to_string(),
                        ));
                    }
                }

                for attr in e.attributes() {
                    let attr = attr.map_err(|err| {
                        ThothError::RequestError(format!("Invalid JATS attribute: {}", err))
                    })?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    if raw_name == "root" {
                        if key != "xmlns:xlink" {
                            return Err(ThothError::RequestError(format!(
                                "Unsupported JATS attribute: {}",
                                key
                            )));
                        }
                        continue;
                    }

                    if raw_name.ends_with("ext-link") {
                        if key != "xlink:href" {
                            return Err(ThothError::RequestError(format!(
                                "Unsupported JATS attribute on ext-link: {}",
                                key
                            )));
                        }
                    } else {
                        return Err(ThothError::RequestError(format!(
                            "Unsupported JATS attribute on {}: {}",
                            raw_name, key
                        )));
                    }
                }
                if raw_name != "root" {
                    open_tags.push(tag_name);
                }
            }
            Ok(Event::Empty(e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let tag_name = local_tag_name(&raw_name).to_string();
                if raw_name != "root" {
                    if !allowed_tags.contains(&tag_name.as_str()) {
                        return Err(ThothError::RequestError(format!(
                            "Unsupported JATS element: <{}>",
                            tag_name
                        )));
                    }

                    if matches!(
                        conversion_limit,
                        ConversionLimit::Abstract | ConversionLimit::Biography
                    ) && matches!(open_tags.last().map(String::as_str), Some("p"))
                        && matches!(tag_name.as_str(), "p" | "list" | "list-item")
                    {
                        return Err(ThothError::RequestError(
                            "Abstracts and biographies cannot contain nested block elements inside paragraphs."
                                .to_string(),
                        ));
                    }
                }

                for attr in e.attributes() {
                    let attr = attr.map_err(|err| {
                        ThothError::RequestError(format!("Invalid JATS attribute: {}", err))
                    })?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    if raw_name == "root" {
                        if key != "xmlns:xlink" {
                            return Err(ThothError::RequestError(format!(
                                "Unsupported JATS attribute: {}",
                                key
                            )));
                        }
                        continue;
                    }

                    if raw_name.ends_with("ext-link") {
                        if key != "xlink:href" {
                            return Err(ThothError::RequestError(format!(
                                "Unsupported JATS attribute on ext-link: {}",
                                key
                            )));
                        }
                    } else {
                        return Err(ThothError::RequestError(format!(
                            "Unsupported JATS attribute on {}: {}",
                            raw_name, key
                        )));
                    }
                }
            }
            Ok(Event::End(e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if raw_name != "root" {
                    open_tags.pop();
                }
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(ThothError::RequestError(format!(
                    "Invalid JATS XML: {}",
                    err
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    let email_pattern = regex::Regex::new(r"(?s)<email>(.*?)</email>").unwrap();
    let bare_email_pattern =
        regex::Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").unwrap();
    for captures in email_pattern.captures_iter(content) {
        let inner = captures.get(1).unwrap().as_str().trim();
        if inner.contains('<') || !bare_email_pattern.is_match(inner) {
            return Err(ThothError::RequestError(
                "Email elements must contain a bare email address.".to_string(),
            ));
        }
    }

    let uri_pattern = regex::Regex::new(r"(?s)<uri>(.*?)</uri>").unwrap();
    let bare_uri_pattern = regex::Regex::new(r"^https?://\S+$").unwrap();
    for captures in uri_pattern.captures_iter(content) {
        let inner = captures.get(1).unwrap().as_str().trim();
        if inner.contains('<') || !bare_uri_pattern.is_match(inner) {
            return Err(ThothError::RequestError(
                "URI elements must contain a bare URI.".to_string(),
            ));
        }
    }

    let inline_formula_pattern =
        regex::Regex::new(r"(?s)<inline-formula>(.*?)</inline-formula>").unwrap();
    let tex_math_pattern = regex::Regex::new(r"(?s)^\s*<tex-math>(.*?)</tex-math>\s*$").unwrap();
    let nested_tag_pattern = regex::Regex::new(r"</?[A-Za-z!/]").unwrap();
    for captures in inline_formula_pattern.captures_iter(content) {
        let inner = captures.get(1).unwrap().as_str();
        let Some(tex_captures) = tex_math_pattern.captures(inner) else {
            return Err(ThothError::RequestError(
                "Inline formulas must use a single <tex-math> child.".to_string(),
            ));
        };
        let tex = tex_captures.get(1).unwrap().as_str();
        if nested_tag_pattern.is_match(tex) {
            return Err(ThothError::RequestError(
                "Inline formulas must contain TeX text only.".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate content format based on markup format
pub fn validate_format(content: &str, format: &MarkupFormat) -> ThothResult<()> {
    match format {
        MarkupFormat::Html | MarkupFormat::JatsXml => {
            // Basic HTML validation - check for opening and closing tags
            if !looks_like_markup(content) {
                return Err(ThothError::UnsupportedFileFormatError);
            }
        }
        MarkupFormat::Markdown => {
            let html_tag_pattern =
                regex::Regex::new(r"</?[A-Za-z][A-Za-z0-9-]*(\s[^>]*)?>").unwrap();
            if html_tag_pattern.is_match(content) {
                return Err(ThothError::UnsupportedFileFormatError);
            }
        }
        MarkupFormat::PlainText => {
            if looks_like_markup(content) {
                return Err(ThothError::RequestError(
                    "Plain text input cannot contain markup tags. Use HTML or JATS XML instead."
                        .to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Convert content to JATS XML format with specified tag
pub fn convert_to_jats(
    content: String,
    format: MarkupFormat,
    conversion_limit: ConversionLimit,
) -> ThothResult<String> {
    if format == MarkupFormat::JatsXml {
        let content_looks_like_jats = looks_like_markup(&content);
        let ast = if content_looks_like_jats {
            validate_jats_subset(&content, conversion_limit)?;
            jats_to_ast(&content)
        } else {
            plain_text_to_ast(&content)
        };

        let processed_ast = if conversion_limit == ConversionLimit::Title {
            strip_structural_elements_from_ast_for_conversion(&ast)
        } else {
            ast
        };

        validate_ast_content(&processed_ast, conversion_limit)?;

        return Ok(
            if content_looks_like_jats || conversion_limit == ConversionLimit::Title {
                ast_to_jats(&processed_ast)
            } else {
                plain_text_ast_to_jats(&processed_ast)
            },
        );
    }

    validate_format(&content, &format)?;

    match format {
        MarkupFormat::Html => {
            // Use ast library to parse HTML and convert to JATS
            let ast = html_to_ast(&content);

            // For title conversion, strip structural elements before validation
            let processed_ast = if conversion_limit == ConversionLimit::Title {
                strip_structural_elements_from_ast_for_conversion(&ast)
            } else {
                ast
            };

            validate_ast_content(&processed_ast, conversion_limit)?;
            Ok(ast_to_jats(&processed_ast))
        }

        MarkupFormat::Markdown => {
            // Use ast library to parse Markdown and convert to JATS
            let ast = markdown_to_ast(&content);

            // For title conversion, strip structural elements before validation
            let processed_ast = if conversion_limit == ConversionLimit::Title {
                strip_structural_elements_from_ast_for_conversion(&ast)
            } else {
                ast
            };

            validate_ast_content(&processed_ast, conversion_limit)?;
            Ok(ast_to_jats(&processed_ast))
        }

        MarkupFormat::PlainText => {
            // Use ast library to parse plain text and convert to JATS
            let ast = plain_text_to_ast(&content);

            // For title conversion, strip structural elements before validation
            let processed_ast = if conversion_limit == ConversionLimit::Title {
                strip_structural_elements_from_ast_for_conversion(&ast)
            } else {
                ast
            };

            validate_ast_content(&processed_ast, conversion_limit)?;
            Ok(if conversion_limit == ConversionLimit::Title {
                // Title JATS should remain inline (no paragraph wrapper)
                ast_to_jats(&processed_ast)
            } else {
                plain_text_ast_to_jats(&processed_ast)
            })
        }

        MarkupFormat::JatsXml => unreachable!("handled above"),
    }
}

/// normalise stored abstract-like markup into the subset we safely emit to Crossref.
pub fn normalise_crossref_abstract_jats(content: &str) -> ThothResult<String> {
    let ast = if looks_like_markup(content) {
        jats_to_ast(content)
    } else {
        plain_text_to_ast(content)
    };
    let normalised = normalise_crossref_abstract_ast(ast)?;
    Ok(ast_to_jats(&normalised))
}

/// Convert from JATS XML to specified format using a specific tag name
pub fn convert_from_jats(
    jats_xml: &str,
    format: MarkupFormat,
    conversion_limit: ConversionLimit,
) -> ThothResult<String> {
    if format == MarkupFormat::JatsXml {
        return Ok(jats_xml.to_string());
    }

    // Allow plain-text content that was stored without JATS markup for titles.
    if !looks_like_markup(jats_xml) {
        let ast = plain_text_to_ast(jats_xml);
        let processed_ast = if conversion_limit == ConversionLimit::Title {
            strip_structural_elements_from_ast_for_conversion(&ast)
        } else {
            ast
        };
        validate_ast_content(&processed_ast, conversion_limit)?;
        return Ok(match format {
            MarkupFormat::Html => ast_to_html(&processed_ast),
            MarkupFormat::Markdown => ast_to_markdown(&processed_ast),
            MarkupFormat::PlainText => ast_to_plain_text(&processed_ast),
            MarkupFormat::JatsXml => {
                if conversion_limit == ConversionLimit::Title {
                    ast_to_jats(&processed_ast)
                } else {
                    plain_text_ast_to_jats(&processed_ast)
                }
            }
        });
    }

    // Read paths need to tolerate legacy stored markup and normalise it on the fly.
    let ast = jats_to_ast(jats_xml);

    // For title conversion, strip structural elements before validation
    let processed_ast = if conversion_limit == ConversionLimit::Title {
        strip_structural_elements_from_ast_for_conversion(&ast)
    } else {
        ast
    };

    let output = match format {
        MarkupFormat::Html => {
            // Use the dedicated AST to HTML converter
            ast_to_html(&processed_ast)
        }

        MarkupFormat::Markdown => {
            // Use the dedicated AST to Markdown converter
            ast_to_markdown(&processed_ast)
        }

        MarkupFormat::PlainText => {
            // Use the dedicated AST to plain text converter
            ast_to_plain_text(&processed_ast)
        }

        MarkupFormat::JatsXml => unreachable!("handled above"),
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- convert_to_jats tests start ---
    #[test]
    fn test_html_basic_formatting() {
        let input = "<em>Italic</em> and <strong>Bold</strong>";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::Html,
            ConversionLimit::Biography,
        )
        .unwrap();
        assert_eq!(
            output,
            "<p><italic>Italic</italic> and <bold>Bold</bold></p>"
        );
    }

    #[test]
    fn test_html_link_conversion() {
        let input = r#"<a href="https://example.com">Link</a>"#;
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::Html,
            ConversionLimit::Abstract,
        )
        .unwrap();
        assert_eq!(
            output,
            r#"<p><ext-link xlink:href="https://example.com">Link</ext-link></p>"#
        );
    }

    #[test]
    fn test_html_with_structure_allowed() {
        let input = "<ul><li>One</li><li>Two</li></ul>";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::Html,
            ConversionLimit::Abstract,
        )
        .unwrap();
        assert_eq!(
            output,
            "<list><list-item>One</list-item><list-item>Two</list-item></list>"
        );
    }

    #[test]
    fn test_html_with_structure_stripped() {
        let input = "<ul><li>One</li></ul>";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::Html,
            ConversionLimit::Title,
        )
        .unwrap();
        assert_eq!(output, "One");
    }

    #[test]
    fn test_html_small_caps_conversion() {
        let input = "<text>Small caps text</text>";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::Html,
            ConversionLimit::Title,
        )
        .unwrap();
        assert_eq!(output, "<sc>Small caps text</sc>");
    }

    #[test]
    fn test_markdown_basic_formatting() {
        let input = "**Bold** and *Italic* and `code`";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::Markdown,
            ConversionLimit::Title,
        )
        .unwrap();
        assert_eq!(
            output,
            "<bold>Bold</bold> and <italic>Italic</italic> and <monospace>code</monospace>"
        );
    }

    #[test]
    fn test_markdown_link_conversion() {
        let input = "[text](https://example.com)";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::Markdown,
            ConversionLimit::Title,
        )
        .unwrap();
        assert_eq!(
            output,
            r#"<ext-link xlink:href="https://example.com">text</ext-link>"#
        );
    }

    #[test]
    fn test_markdown_with_structure() {
        let input = "- Item 1\n- Item 2\n\nParagraph text";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::Markdown,
            ConversionLimit::Abstract,
        )
        .unwrap();

        assert!(
            output.contains(
                "<list><list-item>Item 1</list-item><list-item>Item 2</list-item></list>"
            ) && output.contains("<p>Paragraph text</p>")
        );
    }

    #[test]
    fn test_plain_text_with_url() {
        let input = "Hello https://example.com world";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::PlainText,
            ConversionLimit::Biography,
        )
        .unwrap();
        assert_eq!(output, "<p>Hello <uri>https://example.com</uri> world</p>");
    }

    #[test]
    fn test_plain_text_no_url() {
        let input = "Just plain text.";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::PlainText,
            ConversionLimit::Title,
        )
        .unwrap();
        assert_eq!(output, "Just plain text.");
    }

    #[test]
    fn test_jatsxml_plain_text_title_is_accepted() {
        let input = "Second Expanded Edition";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::JatsXml,
            ConversionLimit::Title,
        )
        .unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_jatsxml_plain_text_abstract_is_wrapped() {
        let input = "Plain abstract content.";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::JatsXml,
            ConversionLimit::Abstract,
        )
        .unwrap();
        assert_eq!(output, "<p>Plain abstract content.</p>");
    }

    #[test]
    fn test_markdown_formula_email_uri_and_break_conversion_is_rejected_for_abstracts() {
        let input = "Formula $E=mc^2$  \n<user@example.org> <https://example.org>";
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::Markdown,
            ConversionLimit::Abstract,
        )
        .is_err());
    }

    #[test]
    fn test_html_break_formula_email_and_uri_conversion_is_rejected_for_biographies() {
        let input = r#"<p>Line<br/><span class="inline-formula">E=mc^2</span> <a href="mailto:user@example.org">user@example.org</a> <a href="https://example.org">https://example.org</a></p>"#;
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::Html,
            ConversionLimit::Biography,
        )
        .is_err());
    }

    #[test]
    fn test_jatsxml_title_rejects_breaks() {
        let input = "Title<break/>Subtitle";
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::JatsXml,
            ConversionLimit::Title,
        )
        .is_err());
    }

    #[test]
    fn test_jatsxml_rejects_non_tex_inline_formula() {
        let input = "<inline-formula><mml:math/></inline-formula>";
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::JatsXml,
            ConversionLimit::Abstract,
        )
        .is_err());
    }

    #[test]
    fn test_jatsxml_rejects_nested_email_markup() {
        let input = "<email><bold>user@example.org</bold></email>";
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::JatsXml,
            ConversionLimit::Abstract,
        )
        .is_err());
    }

    #[test]
    fn test_jatsxml_rejects_nested_uri_markup() {
        let input = "<uri><italic>https://example.org</italic></uri>";
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::JatsXml,
            ConversionLimit::Abstract,
        )
        .is_err());
    }

    #[test]
    fn test_plain_text_rejects_markup_like_input() {
        let input = "<p>Hello</p>";
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::PlainText,
            ConversionLimit::Abstract,
        )
        .is_err());
    }

    #[test]
    fn test_jatsxml_rejects_legacy_html_tags() {
        let input = "<i>Hello</i>";
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::JatsXml,
            ConversionLimit::Abstract,
        )
        .is_err());
    }

    #[test]
    fn test_jatsxml_rejects_nested_paragraphs() {
        let input = "<p><p>Hello</p></p>";
        assert!(convert_to_jats(
            input.to_string(),
            MarkupFormat::JatsXml,
            ConversionLimit::Abstract,
        )
        .is_err());
    }

    #[test]
    fn test_normalise_crossref_abstract_jats_splits_breaks_and_removes_empty_paragraphs() {
        let input = "<p></p><p>First line<break/>Second line</p>";
        let output = normalise_crossref_abstract_jats(input).unwrap();
        assert_eq!(output, "<p>First line</p><p>Second line</p>");
    }

    #[test]
    fn test_normalise_crossref_abstract_jats_flattens_inline_only_content() {
        let input = "This has <bold>bold</bold> text.";
        let output = normalise_crossref_abstract_jats(input).unwrap();
        assert_eq!(output, "<p>This has <bold>bold</bold> text.</p>");
    }
    // --- convert_to_jats tests end   ---

    // --- convert_from_jats tests start   ---
    #[test]
    fn test_convert_from_jats_html_with_structure() {
        let input = r#"
            <p>Paragraph text</p>
            <list><list-item>Item 1</list-item><list-item>Item 2</list-item></list>
            <italic>Italic</italic> and <bold>Bold</bold>
            <ext-link xlink:href="https://example.com">Link</ext-link>
        "#;
        let output =
            convert_from_jats(input, MarkupFormat::Html, ConversionLimit::Abstract).unwrap();

        assert!(output.contains("<p>Paragraph text</p>"));
        assert!(output.contains("<ul><li>Item 1</li><li>Item 2</li></ul>"));
        assert!(output.contains("<em>Italic</em>"));
        assert!(output.contains("<strong>Bold</strong>"));
        assert!(output.contains(r#"<a href="https://example.com">Link</a>"#));
    }

    #[test]
    fn test_convert_from_jats_html_no_structure() {
        let input = r#"
            <p>Text</p><list><list-item>Item</list-item></list><bold>Bold</bold>
        "#;
        let output = convert_from_jats(input, MarkupFormat::Html, ConversionLimit::Title).unwrap();

        assert!(!output.contains("<p>"));
        assert!(!output.contains("<ul>"));
        assert!(output.contains("<strong>Bold</strong>"));
    }

    #[test]
    fn test_convert_from_jats_html_title_limit() {
        let input = r#"<p>Title</p><bold>Bold</bold>"#;
        let output = convert_from_jats(input, MarkupFormat::Html, ConversionLimit::Title).unwrap();

        assert!(!output.contains("<p>"));
        assert!(output.contains("<strong>Bold</strong>"));
    }

    #[test]
    fn test_convert_from_jats_markdown_with_structure() {
        let input = r#"
            <p>Text</p><list><list-item>Item 1</list-item><list-item>Item 2</list-item></list>
            <italic>It</italic> and <bold>Bold</bold>
            <ext-link xlink:href="https://link.com">Here</ext-link>
        "#;
        let output =
            convert_from_jats(input, MarkupFormat::Markdown, ConversionLimit::Biography).unwrap();

        assert!(output.contains("Text"));
        assert!(output.contains("- Item 1"));
        assert!(output.contains("*It*"));
        assert!(output.contains("**Bold**"));
        assert!(output.contains("[Here](https://link.com)"));
    }

    #[test]
    fn test_convert_from_jats_markdown_title_limit() {
        let input = r#"<p>Title</p><italic>It</italic>"#;
        let output =
            convert_from_jats(input, MarkupFormat::Markdown, ConversionLimit::Title).unwrap();

        assert!(!output.contains("<p>"));
        assert!(output.contains("*It*"));
    }

    #[test]
    fn test_convert_from_jats_plain_text_basic() {
        let input = r#"
            <p>Text</p> and <ext-link xlink:href="https://ex.com">Link</ext-link> and <sc>SC</sc>
        "#;
        let output =
            convert_from_jats(input, MarkupFormat::PlainText, ConversionLimit::Abstract).unwrap();

        assert!(output.contains("Text"));
        assert!(output.contains("Link (https://ex.com)"));
        assert!(!output.contains("<sc>"));
        assert!(!output.contains("<"));
    }

    #[test]
    fn test_convert_from_jats_preserves_inline_html() {
        let input = r#"<italic>i</italic> <bold>b</bold> <monospace>code</monospace>"#;
        let output =
            convert_from_jats(input, MarkupFormat::Html, ConversionLimit::Abstract).unwrap();

        assert!(output.contains("<em>i</em>"));
        assert!(output.contains("<strong>b</strong>"));
        assert!(output.contains("<code>code</code>"));
    }

    #[test]
    fn test_convert_from_jats_jatsxml_noop() {
        let input = r#"<p>Do nothing</p>"#;
        let output =
            convert_from_jats(input, MarkupFormat::JatsXml, ConversionLimit::Biography).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_convert_from_jats_jatsxml_passes_through_legacy_markup() {
        let input =
            r#"<p><i>Italic</i> <u>Underline</u> <a href="https://example.org">Link</a></p>"#;
        let output =
            convert_from_jats(input, MarkupFormat::JatsXml, ConversionLimit::Abstract).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_convert_from_jats_html_accepts_legacy_inline_html_tags() {
        let input =
            r#"<p><i>Italic</i> <u>Underline</u> <a href="https://example.org">Link</a></p>"#;
        let output =
            convert_from_jats(input, MarkupFormat::Html, ConversionLimit::Abstract).unwrap();

        assert!(output.contains("<em>Italic</em>"));
        assert!(output.contains("<u>Underline</u>"));
        assert!(output.contains(r#"<a href="https://example.org">Link</a>"#));
    }

    #[test]
    fn test_convert_from_jats_html_title_flattens_multiple_top_level_nodes() {
        let input = r#"<p>Legacy Title</p><i> Supplement</i>"#;
        let output = convert_from_jats(input, MarkupFormat::Html, ConversionLimit::Title).unwrap();

        assert_eq!(output, "Legacy Title<em> Supplement</em>");
    }

    #[test]
    fn test_convert_from_jats_markdown_formula_email_uri_and_break() {
        let input = "<p>Line<break/><inline-formula><tex-math>E=mc^2</tex-math></inline-formula> <email>user@example.org</email> <uri>https://example.org</uri></p>";
        let output =
            convert_from_jats(input, MarkupFormat::Markdown, ConversionLimit::Biography).unwrap();
        assert_eq!(
            output,
            "Line  \n$E=mc^2$ <user@example.org> <https://example.org>"
        );
    }

    #[test]
    fn test_convert_from_jats_html_formula_email_uri_and_break() {
        let input = "<p>Line<break/><inline-formula><tex-math>E=mc^2</tex-math></inline-formula> <email>user@example.org</email> <uri>https://example.org</uri></p>";
        let output =
            convert_from_jats(input, MarkupFormat::Html, ConversionLimit::Biography).unwrap();
        assert_eq!(
            output,
            r#"<p>Line<br/><span class="inline-formula">E=mc^2</span> <a href="mailto:user@example.org">user@example.org</a> <a href="https://example.org">https://example.org</a></p>"#
        );
    }

    #[test]
    fn test_convert_from_jats_html_allow_structure_false() {
        let input = r#"<p>Para</p><list><list-item>Item</list-item></list>"#;
        let output = convert_from_jats(input, MarkupFormat::Html, ConversionLimit::Title).unwrap();

        assert!(!output.contains("<p>"));
        assert!(!output.contains("<ul>"));
        assert!(output.contains("Para"));
        assert!(output.contains("Item"));
    }

    #[test]
    fn test_title_plain_text_to_jats_has_no_paragraph() {
        let input = "Plain title";
        let output = convert_to_jats(
            input.to_string(),
            MarkupFormat::PlainText,
            ConversionLimit::Title,
        )
        .unwrap();
        assert_eq!(output, "Plain title");
    }

    #[test]
    fn test_title_plain_text_roundtrip_no_paragraphs() {
        let plain = "Another plain title";
        let jats = convert_to_jats(
            plain.to_string(),
            MarkupFormat::PlainText,
            ConversionLimit::Title,
        )
        .unwrap();
        assert!(!jats.contains("<p>"));

        let back = convert_from_jats(&jats, MarkupFormat::JatsXml, ConversionLimit::Title).unwrap();
        assert_eq!(back, plain);
    }
    // --- convert_from_jats tests end
}
