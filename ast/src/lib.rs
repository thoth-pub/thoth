use pulldown_cmark::{Event, Parser, Tag};
use scraper::{ElementRef, Html, Selector};

// Simple AST node
#[derive(Debug, Clone)]
pub enum Node {
    Document(Vec<Node>),
    Paragraph(Vec<Node>),
    Bold(Vec<Node>),
    Italic(Vec<Node>),
    List(Vec<Node>),
    ListItem(Vec<Node>),
    Link { url: String, text: Vec<Node> },
    Text(String),
}

// Convert Markdown string to AST
pub fn markdown_to_ast(markdown: &str) -> Node {
    let parser = Parser::new(markdown);
    let mut stack: Vec<Node> = vec![Node::Document(vec![])];

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => stack.push(Node::Paragraph(vec![])),
                Tag::Strong => stack.push(Node::Bold(vec![])),
                Tag::Emphasis => stack.push(Node::Italic(vec![])),
                Tag::List(_) => stack.push(Node::List(vec![])),
                Tag::Item => stack.push(Node::ListItem(vec![])),
                Tag::Link {
                    dest_url, title, ..
                } => stack.push(Node::Link {
                    url: dest_url.to_string(),
                    text: vec![Node::Text(title.to_string())],
                }),
                _ => {}
            },
            Event::End(_tag) => {
                if let Some(node) = stack.pop()
                    && let Some(top) = stack.last_mut()
                {
                    match top {
                        Node::Document(children)
                        | Node::Paragraph(children)
                        | Node::Bold(children)
                        | Node::Italic(children)
                        | Node::List(children)
                        | Node::ListItem(children) => children.push(node),
                        Node::Text(_) => {}
                        Node::Link { text, .. } => text.push(node),
                    }
                }
            }
            Event::Text(text) => {
                if let Some(
                    Node::Document(children)
                    | Node::Paragraph(children)
                    | Node::Bold(children)
                    | Node::Italic(children)
                    | Node::List(children)
                    | Node::ListItem(children),
                ) = stack.last_mut()
                {
                    children.push(Node::Text(text.to_string()));
                } else if let Some(Node::Link {
                    text: link_text, ..
                }) = stack.last_mut()
                {
                    link_text.push(Node::Text(text.to_string()));
                }
            }
            _ => {}
        }
    }

    stack.pop().unwrap_or_else(|| Node::Document(vec![]))
}

// Convert HTML string to AST
pub fn html_to_ast(html: &str) -> Node {
    // Helper function to parse an HTML element to AST node
    fn parse_element_to_node(element: ElementRef) -> Node {
        let tag_name = element.value().name();
        let mut children = Vec::new();

        // Process child elements and text nodes
        for child in element.children() {
            match child.value() {
                scraper::node::Node::Element(_) => {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        children.push(parse_element_to_node(child_element));
                    }
                }
                scraper::node::Node::Text(text) => {
                    children.push(Node::Text(text.to_string()));
                }
                _ => {}
            }
        }

        // Map HTML tags to AST nodes
        match tag_name {
            "html" | "body" | "div" => Node::Document(children),
            "p" => Node::Paragraph(children),
            "strong" | "b" => Node::Bold(children),
            "em" | "i" => Node::Italic(children),
            "ul" | "ol" => Node::List(children),
            "li" => Node::ListItem(children),
            "a" => {
                // Extract href attribute for links
                let url = element.value().attr("href").unwrap_or("").to_string();
                Node::Link {
                    url,
                    text: children,
                }
            }
            _ => {
                // For unknown tags, create a document node with the children
                if children.is_empty() {
                    Node::Text(String::new())
                } else {
                    Node::Document(children)
                }
            }
        }
    }

    let document = Html::parse_document(html);
    let body_selector = Selector::parse("body").unwrap();

    // If there's a body tag, parse its contents, otherwise parse the whole document
    if let Some(body_element) = document.select(&body_selector).next() {
        parse_element_to_node(body_element)
    } else {
        // If no body tag, create a document node with all top-level elements
        let mut children = Vec::new();
        for child in document.root_element().children() {
            if let Some(element) = ElementRef::wrap(child) {
                children.push(parse_element_to_node(element));
            }
        }
        Node::Document(children)
    }
}

// Convert plain text string to AST
pub fn plain_text_to_ast(text: &str) -> Node {
    // Split text into paragraphs (double newlines) and create AST structure
    let paragraphs: Vec<Node> = text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .map(|paragraph| {
            let lines: Vec<Node> = paragraph
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| Node::Text(line.trim().to_string()))
                .collect();

            if lines.len() == 1 {
                // Single line paragraph
                Node::Paragraph(vec![lines[0].clone()])
            } else {
                // Multi-line paragraph
                Node::Paragraph(lines)
            }
        })
        .collect();

    Node::Document(paragraphs)
}

// Render AST to JATS XML
pub fn ast_to_jats(node: &Node) -> String {
    match node {
        Node::Document(children) => children.iter().map(ast_to_jats).collect(),
        Node::Paragraph(children) => {
            let inner: String = children.iter().map(ast_to_jats).collect();
            format!("<p>{}</p>", inner)
        }
        Node::Bold(children) => {
            let inner: String = children.iter().map(ast_to_jats).collect();
            format!("<bold>{}</bold>", inner)
        }
        Node::Italic(children) => {
            let inner: String = children.iter().map(ast_to_jats).collect();
            format!("<italic>{}</italic>", inner)
        }
        Node::List(items) => {
            let inner: String = items.iter().map(ast_to_jats).collect();
            format!("<list>{}</list>", inner)
        }
        Node::ListItem(children) => {
            let inner: String = children.iter().map(ast_to_jats).collect();
            format!("<list-item>{}</list-item>", inner)
        }
        Node::Link { url, text } => {
            let inner: String = text.iter().map(ast_to_jats).collect();
            format!(r#"<ext-link xlink:href="{}">{}</ext-link>"#, url, inner)
        }
        Node::Text(text) => text.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_ast_basic() {
        let markdown = "**Bold** and *italic* text";
        let ast = markdown_to_ast(markdown);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph(para_children) => {
                        assert_eq!(para_children.len(), 4); // Bold, text " and ", italic, text
                        // Check for bold, text, and italic nodes
                        let has_bold = para_children
                            .iter()
                            .any(|child| matches!(child, Node::Bold(_)));
                        let has_italic = para_children
                            .iter()
                            .any(|child| matches!(child, Node::Italic(_)));
                        let has_text = para_children
                            .iter()
                            .any(|child| matches!(child, Node::Text(_)));
                        assert!(has_bold);
                        assert!(has_italic);
                        assert!(has_text);
                    }
                    _ => panic!("Expected paragraph node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_markdown_to_ast_list() {
        let markdown = "- Item 1\n- Item 2";
        let ast = markdown_to_ast(markdown);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::List(list_children) => {
                        assert_eq!(list_children.len(), 2);
                        for child in list_children {
                            match child {
                                Node::ListItem(_) => {} // Expected
                                _ => panic!("Expected list item node"),
                            }
                        }
                    }
                    _ => panic!("Expected list node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_html_to_ast_basic() {
        let html = "<p><strong>Bold</strong> and <em>italic</em> text</p>";
        let ast = html_to_ast(html);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph(para_children) => {
                        assert_eq!(para_children.len(), 4); // Bold, text " and ", italic, text
                        // Check for bold, text, and italic nodes
                        let has_bold = para_children
                            .iter()
                            .any(|child| matches!(child, Node::Bold(_)));
                        let has_italic = para_children
                            .iter()
                            .any(|child| matches!(child, Node::Italic(_)));
                        let has_text = para_children
                            .iter()
                            .any(|child| matches!(child, Node::Text(_)));
                        assert!(has_bold);
                        assert!(has_italic);
                        assert!(has_text);
                    }
                    _ => panic!("Expected paragraph node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_html_to_ast_list() {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let ast = html_to_ast(html);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::List(list_children) => {
                        assert_eq!(list_children.len(), 2);
                        for child in list_children {
                            match child {
                                Node::ListItem(_) => {} // Expected
                                _ => panic!("Expected list item node"),
                            }
                        }
                    }
                    _ => panic!("Expected list node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_html_to_ast_ordered_list() {
        let html = "<ol><li>First</li><li>Second</li></ol>";
        let ast = html_to_ast(html);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::List(list_children) => {
                        assert_eq!(list_children.len(), 2);
                    }
                    _ => panic!("Expected list node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_html_to_ast_link() {
        let html = r#"<a href="https://example.com">Link text</a>"#;
        let ast = html_to_ast(html);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Link { url, text } => {
                        assert_eq!(url, "https://example.com");
                        assert_eq!(text.len(), 1);
                        match &text[0] {
                            Node::Text(content) => assert_eq!(content, "Link text"),
                            _ => panic!("Expected text node"),
                        }
                    }
                    _ => panic!("Expected link node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_plain_text_to_ast_single_paragraph() {
        let text = "This is a single paragraph.";
        let ast = plain_text_to_ast(text);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph(para_children) => {
                        assert_eq!(para_children.len(), 1);
                        match &para_children[0] {
                            Node::Text(content) => {
                                assert_eq!(content, "This is a single paragraph.")
                            }
                            _ => panic!("Expected text node"),
                        }
                    }
                    _ => panic!("Expected paragraph node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_plain_text_to_ast_multiple_paragraphs() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let ast = plain_text_to_ast(text);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 3);
                for child in children {
                    match child {
                        Node::Paragraph(_) => {} // Expected
                        _ => panic!("Expected paragraph node"),
                    }
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_plain_text_to_ast_empty_paragraphs_filtered() {
        let text = "First paragraph.\n\n\n\nSecond paragraph.";
        let ast = plain_text_to_ast(text);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 2); // Empty paragraphs should be filtered out
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_ast_to_jats_document() {
        let ast = Node::Document(vec![
            Node::Paragraph(vec![Node::Text("Hello".to_string())]),
            Node::Bold(vec![Node::Text("Bold text".to_string())]),
        ]);

        let jats = ast_to_jats(&ast);
        assert!(jats.contains("<p>Hello</p>"));
        assert!(jats.contains("<bold>Bold text</bold>"));
    }

    #[test]
    fn test_ast_to_jats_paragraph() {
        let ast = Node::Paragraph(vec![
            Node::Text("Hello ".to_string()),
            Node::Bold(vec![Node::Text("world".to_string())]),
        ]);

        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "<p>Hello <bold>world</bold></p>");
    }

    #[test]
    fn test_ast_to_jats_list() {
        let ast = Node::List(vec![
            Node::ListItem(vec![Node::Text("Item 1".to_string())]),
            Node::ListItem(vec![Node::Text("Item 2".to_string())]),
        ]);

        let jats = ast_to_jats(&ast);
        assert_eq!(
            jats,
            "<list><list-item>Item 1</list-item><list-item>Item 2</list-item></list>"
        );
    }

    #[test]
    fn test_ast_to_jats_text() {
        let ast = Node::Text("Plain text".to_string());
        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "Plain text");
    }

    #[test]
    fn test_ast_to_jats_bold() {
        let ast = Node::Bold(vec![Node::Text("Bold text".to_string())]);
        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "<bold>Bold text</bold>");
    }

    #[test]
    fn test_ast_to_jats_italic() {
        let ast = Node::Italic(vec![Node::Text("Italic text".to_string())]);
        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "<italic>Italic text</italic>");
    }

    #[test]
    fn test_ast_to_jats_list_item() {
        let ast = Node::ListItem(vec![Node::Text("List item text".to_string())]);
        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "<list-item>List item text</list-item>");
    }

    #[test]
    fn test_ast_to_jats_link() {
        let ast = Node::Link {
            url: "https://example.com".to_string(),
            text: vec![Node::Text("Link text".to_string())],
        };
        let jats = ast_to_jats(&ast);
        assert_eq!(
            jats,
            r#"<ext-link xlink:href="https://example.com">Link text</ext-link>"#
        );
    }

    #[test]
    fn test_round_trip_markdown_to_jats() {
        let markdown = "**Bold** and *italic* text\n\n- Item 1\n- Item 2";
        let ast = markdown_to_ast(markdown);
        let jats = ast_to_jats(&ast);

        // Should contain the expected JATS elements
        assert!(jats.contains("<bold>Bold</bold>"));
        assert!(jats.contains("<italic>italic</italic>"));
        assert!(jats.contains("<list>"));
        assert!(jats.contains("<list-item>Item 1</list-item>"));
        assert!(jats.contains("<list-item>Item 2</list-item>"));
    }

    #[test]
    fn test_round_trip_html_to_jats() {
        let html = "<p><strong>Bold</strong> and <em>italic</em> text</p><ul><li>Item 1</li><li>Item 2</li></ul>";
        let ast = html_to_ast(html);
        let jats = ast_to_jats(&ast);

        // Should contain the expected JATS elements
        assert!(jats.contains("<bold>Bold</bold>"));
        assert!(jats.contains("<italic>italic</italic>"));
        assert!(jats.contains("<list>"));
        assert!(jats.contains("<list-item>Item 1</list-item>"));
        assert!(jats.contains("<list-item>Item 2</list-item>"));
    }

    #[test]
    fn test_round_trip_plain_text_to_jats() {
        let text = "First paragraph.\n\nSecond paragraph with multiple lines.\nIt continues here.";
        let ast = plain_text_to_ast(text);
        let jats = ast_to_jats(&ast);

        // Should contain paragraph elements
        assert!(jats.contains("<p>"));
        assert!(jats.contains("First paragraph"));
        assert!(jats.contains("Second paragraph with multiple lines"));
    }

    #[test]
    fn test_empty_input() {
        let empty_ast = markdown_to_ast("");
        let jats = ast_to_jats(&empty_ast);
        assert_eq!(jats, "");
    }

    #[test]
    fn test_nested_formatting() {
        let markdown = "**Bold with *italic* inside**";
        let ast = markdown_to_ast(markdown);
        let jats = ast_to_jats(&ast);

        // Should handle nested formatting
        assert!(jats.contains("<bold>"));
        assert!(jats.contains("<italic>"));
    }
}
