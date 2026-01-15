use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use thoth_errors::{ThothError, ThothResult};

pub mod ast;

use ast::{
    ast_to_html, ast_to_jats, ast_to_markdown, ast_to_plain_text, html_to_ast, jats_to_ast,
    markdown_to_ast, plain_text_ast_to_jats, plain_text_to_ast,
    strip_structural_elements_from_ast_for_conversion, validate_ast_content,
};

/// Enum to represent the markup format
#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
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

/// Validate content format based on markup format
pub fn validate_format(content: &str, format: &MarkupFormat) -> ThothResult<()> {
    match format {
        MarkupFormat::Html | MarkupFormat::JatsXml => {
            // Basic HTML validation - check for opening and closing tags
            if !content.contains('<') || !content.contains('>') || !content.contains("</") {
                return Err(ThothError::UnsupportedFileFormatError);
            }
        }
        MarkupFormat::Markdown => {
            // Basic Markdown validation - check for markdown syntax
            if content.contains('<') && content.contains('>') {
                // At least one markdown element should be present
                return Err(ThothError::UnsupportedFileFormatError);
            }
        }
        MarkupFormat::PlainText => {}
    }
    Ok(())
}

/// Convert content to JATS XML format with specified tag
pub fn convert_to_jats(
    content: String,
    format: MarkupFormat,
    conversion_limit: ConversionLimit,
) -> ThothResult<String> {
    validate_format(&content, &format)?;
    let mut output = content.clone();

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
            output = ast_to_jats(&processed_ast);
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
            output = ast_to_jats(&processed_ast);
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
            output = if conversion_limit == ConversionLimit::Title {
                // Title JATS should remain inline (no paragraph wrapper)
                ast_to_jats(&processed_ast)
            } else {
                plain_text_ast_to_jats(&processed_ast)
            };
        }

        MarkupFormat::JatsXml => {}
    }

    Ok(output)
}

/// Convert from JATS XML to specified format using a specific tag name
pub fn convert_from_jats(
    jats_xml: &str,
    format: MarkupFormat,
    conversion_limit: ConversionLimit,
) -> ThothResult<String> {
    // Allow plain-text content that was stored without JATS markup for titles.
    if !jats_xml.contains('<') || !jats_xml.contains("</") {
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

    validate_format(jats_xml, &MarkupFormat::JatsXml)?;

    // Parse JATS to AST first for better handling
    let ast = jats_to_ast(jats_xml);

    // For title conversion, strip structural elements before validation
    let processed_ast = if conversion_limit == ConversionLimit::Title {
        strip_structural_elements_from_ast_for_conversion(&ast)
    } else {
        ast
    };

    // Validate the AST content based on conversion limit
    validate_ast_content(&processed_ast, conversion_limit)?;

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

        MarkupFormat::JatsXml => {
            // Return the AST converted back to JATS (should be identical)
            jats_xml.to_string()
        }
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
        assert_eq!(output, "<italic>Italic</italic> and <bold>Bold</bold>");
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
            r#"<ext-link xlink:href="https://example.com">Link</ext-link>"#
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
        assert_eq!(
        output,
        "<p>Hello </p><ext-link xlink:href=\"https://example.com\"><p>https://example.com</p></ext-link><p> world</p>"
    );
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
