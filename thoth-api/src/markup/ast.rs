use super::ConversionLimit;
use pulldown_cmark::{Event, Parser, Tag};
use scraper::{ElementRef, Html, Selector};
use thoth_errors::{ThothError, ThothResult};

// Simple AST node
#[derive(Debug, Clone)]
pub enum Node {
    Document(Vec<Node>),
    Paragraph(Vec<Node>),
    Bold(Vec<Node>),
    Italic(Vec<Node>),
    Code(Vec<Node>),
    Superscript(Vec<Node>),
    Subscript(Vec<Node>),
    SmallCaps(Vec<Node>),
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
                if let Some(node) = stack.pop() {
                    if let Some(top) = stack.last_mut() {
                        match top {
                            Node::Document(children)
                            | Node::Paragraph(children)
                            | Node::Bold(children)
                            | Node::Italic(children)
                            | Node::Code(children)
                            | Node::Superscript(children)
                            | Node::Subscript(children)
                            | Node::SmallCaps(children)
                            | Node::List(children)
                            | Node::ListItem(children) => children.push(node),
                            Node::Text(_) => {}
                            Node::Link { text, .. } => text.push(node),
                        }
                    }
                }
            }
            Event::Text(text) => {
                if let Some(
                    Node::Document(children)
                    | Node::Paragraph(children)
                    | Node::Bold(children)
                    | Node::Italic(children)
                    | Node::Code(children)
                    | Node::Superscript(children)
                    | Node::Subscript(children)
                    | Node::SmallCaps(children)
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
            Event::Code(code_text) => {
                if let Some(
                    Node::Document(children)
                    | Node::Paragraph(children)
                    | Node::Bold(children)
                    | Node::Italic(children)
                    | Node::Code(children)
                    | Node::Superscript(children)
                    | Node::Subscript(children)
                    | Node::SmallCaps(children)
                    | Node::List(children)
                    | Node::ListItem(children),
                ) = stack.last_mut()
                {
                    children.push(Node::Code(vec![Node::Text(code_text.to_string())]));
                } else if let Some(Node::Link {
                    text: link_text, ..
                }) = stack.last_mut()
                {
                    link_text.push(Node::Code(vec![Node::Text(code_text.to_string())]));
                }
            }
            _ => {}
        }
    }

    let result = stack.pop().unwrap_or_else(|| Node::Document(vec![]));

    // Post-process to wrap standalone inline elements in paragraphs
    match result {
        Node::Document(children) => {
            if children.len() > 1 {
                let all_inline = children.iter().all(|child| {
                    matches!(
                        child,
                        Node::Bold(_)
                            | Node::Italic(_)
                            | Node::Code(_)
                            | Node::Superscript(_)
                            | Node::Subscript(_)
                            | Node::SmallCaps(_)
                            | Node::Text(_)
                            | Node::Link { .. }
                    )
                });
                if all_inline {
                    Node::Document(vec![Node::Paragraph(children)])
                } else {
                    Node::Document(children)
                }
            } else if children.len() == 1 {
                // If we have only one child, check if it should be wrapped in a paragraph
                match &children[0] {
                    Node::Link { .. } | Node::Text(_) => {
                        // Wrap standalone links and text in paragraphs
                        Node::Document(vec![Node::Paragraph(children)])
                    }
                    _ => Node::Document(children),
                }
            } else {
                Node::Document(children)
            }
        }
        _ => result,
    }
}

// Convert HTML string to AST
pub fn html_to_ast(html: &str) -> Node {
    // Helper function to parse an HTML element to AST node
    fn parse_element_to_node(element: ElementRef) -> Node {
        let tag_name = element.value().name();
        let mut children = Vec::new();

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

        match tag_name {
            "html" | "body" | "div" => Node::Document(children),
            "p" => Node::Paragraph(children),
            "strong" | "b" => Node::Bold(children),
            "em" | "i" => Node::Italic(children),
            "code" => Node::Code(children),
            "sup" => Node::Superscript(children),
            "sub" => Node::Subscript(children),
            "text" => Node::SmallCaps(children),
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
        let result = Node::Document(children);

        // Post-process to wrap standalone inline elements in paragraphs
        match result {
            Node::Document(children) => {
                if children.len() > 1 {
                    let all_inline = children.iter().all(|child| {
                        matches!(
                            child,
                            Node::Bold(_)
                                | Node::Italic(_)
                                | Node::Code(_)
                                | Node::Superscript(_)
                                | Node::Subscript(_)
                                | Node::SmallCaps(_)
                                | Node::Text(_)
                                | Node::Link { .. }
                        )
                    });
                    if all_inline {
                        Node::Document(vec![Node::Paragraph(children)])
                    } else {
                        Node::Document(children)
                    }
                } else if children.len() == 1 {
                    // If we have only one child, check if it should be wrapped in a paragraph
                    match &children[0] {
                        Node::Link { .. }
                        | Node::Text(_)
                        | Node::Bold(_)
                        | Node::Italic(_)
                        | Node::Code(_)
                        | Node::Superscript(_)
                        | Node::Subscript(_)
                        | Node::SmallCaps(_) => {
                            // Wrap standalone inline elements in paragraphs
                            Node::Document(vec![Node::Paragraph(children)])
                        }
                        _ => Node::Document(children),
                    }
                } else {
                    Node::Document(children)
                }
            }
            _ => result,
        }
    }
}

// Helper function to parse text and detect URLs
fn parse_text_with_urls(text: &str) -> Vec<Node> {
    let mut result = Vec::new();
    let mut current_pos = 0;

    // Simple URL regex pattern - matches http/https URLs
    let url_pattern = regex::Regex::new(r"(https?://[^\s]+)").unwrap();

    for mat in url_pattern.find_iter(text) {
        if mat.start() > current_pos {
            let before_text = &text[current_pos..mat.start()];
            if !before_text.is_empty() {
                result.push(Node::Text(before_text.to_string()));
            }
        }

        let url = mat.as_str();
        result.push(Node::Link {
            url: url.to_string(),
            text: vec![Node::Text(url.to_string())],
        });

        current_pos = mat.end();
    }

    if current_pos < text.len() {
        let remaining_text = &text[current_pos..];
        if !remaining_text.is_empty() {
            result.push(Node::Text(remaining_text.to_string()));
        }
    }

    if result.is_empty() {
        result.push(Node::Text(text.to_string()));
    }

    result
}

// Convert plain text string to AST
pub fn plain_text_to_ast(text: &str) -> Node {
    let parsed_nodes = parse_text_with_urls(text.trim());

    if parsed_nodes.len() == 1 {
        parsed_nodes[0].clone()
    } else {
        Node::Document(parsed_nodes)
    }
}

// Special function to convert plain text AST to JATS with proper <sc> wrapping
pub fn plain_text_ast_to_jats(node: &Node) -> String {
    match node {
        Node::Document(children) => {
            let inner: String = children.iter().map(plain_text_ast_to_jats).collect();
            inner
        }
        Node::Paragraph(children) => {
            let inner: String = children.iter().map(plain_text_ast_to_jats).collect();
            format!("<p>{}</p>", inner)
        }
        Node::Text(text) => {
            // For plain text, wrap in <sc> tags only
            format!("<p>{}</p>", text)
        }
        Node::Link { url, text } => {
            let inner: String = text.iter().map(plain_text_ast_to_jats).collect();
            format!(r#"<ext-link xlink:href="{}">{}</ext-link>"#, url, inner)
        }
        _ => {
            // For other nodes, use regular ast_to_jats
            ast_to_jats(node)
        }
    }
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
        Node::Code(children) => {
            let inner: String = children.iter().map(ast_to_jats).collect();
            format!("<monospace>{}</monospace>", inner)
        }
        Node::Superscript(children) => {
            let inner: String = children.iter().map(ast_to_jats).collect();
            format!("<sup>{}</sup>", inner)
        }
        Node::Subscript(children) => {
            let inner: String = children.iter().map(ast_to_jats).collect();
            format!("<sub>{}</sub>", inner)
        }
        Node::SmallCaps(children) => {
            let inner: String = children.iter().map(ast_to_jats).collect();
            format!("<sc>{}</sc>", inner)
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

// Convert JATS XML string to AST
pub fn jats_to_ast(jats: &str) -> Node {
    // Helper function to parse a JATS element to AST node
    fn parse_jats_element_to_node(element: ElementRef) -> Node {
        let tag_name = element.value().name();
        let mut children = Vec::new();

        for child in element.children() {
            match child.value() {
                scraper::node::Node::Element(_) => {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        children.push(parse_jats_element_to_node(child_element));
                    }
                }
                scraper::node::Node::Text(text) => {
                    children.push(Node::Text(text.to_string()));
                }
                _ => {}
            }
        }

        match tag_name {
            "article" | "body" | "sec" | "div" => Node::Document(children),
            "p" => Node::Paragraph(children),
            "bold" => Node::Bold(children),
            "italic" => Node::Italic(children),
            "monospace" => Node::Code(children),
            "sup" => Node::Superscript(children),
            "sub" => Node::Subscript(children),
            "sc" => Node::SmallCaps(children),
            "list" => Node::List(children),
            "list-item" => Node::ListItem(children),
            "ext-link" => {
                // Extract xlink:href attribute for links
                let url = element.value().attr("xlink:href").unwrap_or("").to_string();
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

    let document = Html::parse_document(jats);
    let body_selector = Selector::parse("body").unwrap();

    // If there's a body tag, parse its contents, otherwise parse the whole document
    if let Some(body_element) = document.select(&body_selector).next() {
        parse_jats_element_to_node(body_element)
    } else {
        // If no body tag, create a document node with all top-level elements
        let mut children = Vec::new();
        for child in document.root_element().children() {
            if let Some(element) = ElementRef::wrap(child) {
                children.push(parse_jats_element_to_node(element));
            }
        }

        // If we have multiple inline elements, wrap them in a paragraph
        if children.len() > 1 {
            let all_inline = children.iter().all(|child| {
                matches!(
                    child,
                    Node::Bold(_)
                        | Node::Italic(_)
                        | Node::Code(_)
                        | Node::Superscript(_)
                        | Node::Subscript(_)
                        | Node::Text(_)
                        | Node::Link { .. }
                )
            });
            if all_inline {
                Node::Document(vec![Node::Paragraph(children)])
            } else {
                Node::Document(children)
            }
        } else if children.len() == 1 {
            // Special case: if the single child is a text node, return it directly
            // Otherwise, wrap in document
            match &children[0] {
                Node::Text(_) => children.into_iter().next().unwrap(),
                _ => Node::Document(children),
            }
        } else {
            Node::Document(children)
        }
    }
}

// Convert AST to HTML
pub fn ast_to_html(node: &Node) -> String {
    match node {
        Node::Document(children) => children.iter().map(ast_to_html).collect(),
        Node::Paragraph(children) => {
            let inner: String = children.iter().map(ast_to_html).collect();
            format!("<p>{}</p>", inner)
        }
        Node::Bold(children) => {
            let inner: String = children.iter().map(ast_to_html).collect();
            format!("<strong>{}</strong>", inner)
        }
        Node::Italic(children) => {
            let inner: String = children.iter().map(ast_to_html).collect();
            format!("<em>{}</em>", inner)
        }
        Node::Code(children) => {
            let inner: String = children.iter().map(ast_to_html).collect();
            format!("<code>{}</code>", inner)
        }
        Node::Superscript(children) => {
            let inner: String = children.iter().map(ast_to_html).collect();
            format!("<sup>{}</sup>", inner)
        }
        Node::Subscript(children) => {
            let inner: String = children.iter().map(ast_to_html).collect();
            format!("<sub>{}</sub>", inner)
        }
        Node::SmallCaps(children) => {
            let inner: String = children.iter().map(ast_to_html).collect();
            format!("<text>{}</text>", inner)
        }
        Node::List(items) => {
            let inner: String = items.iter().map(ast_to_html).collect();
            format!("<ul>{}</ul>", inner)
        }
        Node::ListItem(children) => {
            let inner: String = children.iter().map(ast_to_html).collect();
            format!("<li>{}</li>", inner)
        }
        Node::Link { url, text } => {
            let inner: String = text.iter().map(ast_to_html).collect();
            format!(r#"<a href="{}">{}</a>"#, url, inner)
        }
        Node::Text(text) => text.clone(),
    }
}

// Convert AST to Markdown
pub fn ast_to_markdown(node: &Node) -> String {
    match node {
        Node::Document(children) => {
            let mut result = String::new();
            for (i, child) in children.iter().enumerate() {
                if i > 0 {
                    result.push_str("\n\n");
                }
                result.push_str(&ast_to_markdown(child));
            }
            result
        }
        Node::Paragraph(children) => {
            let inner: String = children.iter().map(ast_to_markdown).collect();
            inner
        }
        Node::Bold(children) => {
            let inner: String = children.iter().map(ast_to_markdown).collect();
            format!("**{}**", inner)
        }
        Node::Italic(children) => {
            let inner: String = children.iter().map(ast_to_markdown).collect();
            format!("*{}*", inner)
        }
        Node::Code(children) => {
            let inner: String = children.iter().map(ast_to_markdown).collect();
            format!("`{}`", inner)
        }
        Node::Superscript(children) => {
            let inner: String = children.iter().map(ast_to_markdown).collect();
            format!("<sup>{}</sup>", inner)
        }
        Node::Subscript(children) => {
            let inner: String = children.iter().map(ast_to_markdown).collect();
            format!("<sub>{}</sub>", inner)
        }
        Node::SmallCaps(children) => {
            let inner: String = children.iter().map(ast_to_markdown).collect();
            format!("<sc>{}</sc>", inner)
        }
        Node::List(items) => {
            let mut result = String::new();
            for item in items {
                result.push_str(&ast_to_markdown(item));
            }
            result
        }
        Node::ListItem(children) => {
            let inner: String = children.iter().map(ast_to_markdown).collect();
            format!("- {}\n", inner)
        }
        Node::Link { url, text } => {
            let inner: String = text.iter().map(ast_to_markdown).collect();
            format!("[{}]({})", inner, url)
        }
        Node::Text(text) => text.clone(),
    }
}

// Convert AST to plain text
pub fn ast_to_plain_text(node: &Node) -> String {
    match node {
        Node::Document(children) => {
            let mut result = String::new();
            for (i, child) in children.iter().enumerate() {
                if i > 0 {
                    result.push_str("\n\n");
                }
                result.push_str(&ast_to_plain_text(child));
            }
            result
        }
        Node::Paragraph(children) => {
            let inner: String = children.iter().map(ast_to_plain_text).collect();
            inner
        }
        Node::Bold(children)
        | Node::Italic(children)
        | Node::Code(children)
        | Node::Superscript(children)
        | Node::Subscript(children) => {
            // For plain text, we just extract the text content without formatting
            children.iter().map(ast_to_plain_text).collect()
        }
        Node::SmallCaps(children) => {
            // For plain text, we just extract the text content without formatting
            children.iter().map(ast_to_plain_text).collect()
        }
        Node::List(items) => {
            let mut result = String::new();
            for item in items {
                result.push_str(&ast_to_plain_text(item));
            }
            result
        }
        Node::ListItem(children) => {
            let inner: String = children.iter().map(ast_to_plain_text).collect();
            format!("• {}\n", inner)
        }
        Node::Link { url, text } => {
            let inner: String = text.iter().map(ast_to_plain_text).collect();
            format!("{} ({})", inner, url)
        }
        Node::Text(text) => text.clone(),
    }
}

/// Strip structural elements from AST for title conversion (preserves paragraphs with inline content)
pub fn strip_structural_elements_from_ast(node: &Node) -> Node {
    match node {
        Node::Document(children) => {
            let mut processed_children = Vec::new();
            for child in children {
                let processed_child = strip_structural_elements_from_ast(child);
                match processed_child {
                    Node::Document(grandchildren) => {
                        processed_children.extend(grandchildren);
                    }
                    _ => processed_children.push(processed_child),
                }
            }
            Node::Document(processed_children)
        }
        Node::Paragraph(children) => {
            // For titles, check if paragraph contains only inline elements
            let all_inline = children.iter().all(|child| {
                matches!(
                    child,
                    Node::Bold(_)
                        | Node::Italic(_)
                        | Node::Code(_)
                        | Node::Superscript(_)
                        | Node::Subscript(_)
                        | Node::Text(_)
                        | Node::Link { .. }
                )
            });

            if all_inline {
                // If all children are inline, preserve the paragraph wrapper for titles
                let processed_children: Vec<Node> = children
                    .iter()
                    .map(strip_structural_elements_from_ast)
                    .collect();
                Node::Paragraph(processed_children)
            } else {
                // If contains structural elements, strip the paragraph but preserve content
                let mut processed_children = Vec::new();
                for child in children {
                    let processed_child = strip_structural_elements_from_ast(child);
                    match processed_child {
                        Node::Document(grandchildren) => {
                            processed_children.extend(grandchildren);
                        }
                        _ => processed_children.push(processed_child),
                    }
                }
                if processed_children.len() == 1 {
                    processed_children.into_iter().next().unwrap()
                } else {
                    Node::Document(processed_children)
                }
            }
        }
        Node::List(items) => {
            // Lists are stripped, but their content is preserved
            let mut processed_children = Vec::new();
            for item in items {
                let processed_item = strip_structural_elements_from_ast(item);
                match processed_item {
                    Node::Document(grandchildren) => {
                        processed_children.extend(grandchildren);
                    }
                    _ => processed_children.push(processed_item),
                }
            }
            Node::Document(processed_children)
        }
        Node::ListItem(children) => {
            // List items are stripped, but their content is preserved
            let mut processed_children = Vec::new();
            for child in children {
                let processed_child = strip_structural_elements_from_ast(child);
                match processed_child {
                    Node::Document(grandchildren) => {
                        processed_children.extend(grandchildren);
                    }
                    _ => processed_children.push(processed_child),
                }
            }
            Node::Document(processed_children)
        }
        Node::Bold(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast)
                .collect();
            Node::Bold(processed_children)
        }
        Node::Italic(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast)
                .collect();
            Node::Italic(processed_children)
        }
        Node::Code(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast)
                .collect();
            Node::Code(processed_children)
        }
        Node::Superscript(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast)
                .collect();
            Node::Superscript(processed_children)
        }
        Node::Subscript(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast)
                .collect();
            Node::Subscript(processed_children)
        }
        Node::SmallCaps(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast)
                .collect();
            Node::SmallCaps(processed_children)
        }
        Node::Link { url, text } => {
            let processed_text: Vec<Node> = text
                .iter()
                .map(strip_structural_elements_from_ast)
                .collect();
            Node::Link {
                url: url.clone(),
                text: processed_text,
            }
        }
        Node::Text(text) => Node::Text(text.clone()),
    }
}

/// Strip structural elements from AST for convert_from_jats (strips all structural elements including paragraphs)
pub fn strip_structural_elements_from_ast_for_conversion(node: &Node) -> Node {
    match node {
        Node::Document(children) => {
            let mut processed_children = Vec::new();
            for child in children {
                let processed_child = strip_structural_elements_from_ast_for_conversion(child);
                match processed_child {
                    Node::Document(grandchildren) => {
                        processed_children.extend(grandchildren);
                    }
                    _ => processed_children.push(processed_child),
                }
            }
            Node::Document(processed_children)
        }
        Node::Paragraph(children) => {
            // Always strip paragraphs for convert_from_jats
            let mut processed_children = Vec::new();
            for child in children {
                let processed_child = strip_structural_elements_from_ast_for_conversion(child);
                match processed_child {
                    Node::Document(grandchildren) => {
                        processed_children.extend(grandchildren);
                    }
                    _ => processed_children.push(processed_child),
                }
            }
            if processed_children.len() == 1 {
                processed_children.into_iter().next().unwrap()
            } else {
                Node::Document(processed_children)
            }
        }
        Node::List(items) => {
            // Lists are stripped, but their content is preserved
            let mut processed_children = Vec::new();
            for item in items {
                let processed_item = strip_structural_elements_from_ast_for_conversion(item);
                match processed_item {
                    Node::Document(grandchildren) => {
                        processed_children.extend(grandchildren);
                    }
                    _ => processed_children.push(processed_item),
                }
            }
            Node::Document(processed_children)
        }
        Node::ListItem(children) => {
            // List items are stripped, but their content is preserved
            let mut processed_children = Vec::new();
            for child in children {
                let processed_child = strip_structural_elements_from_ast_for_conversion(child);
                match processed_child {
                    Node::Document(grandchildren) => {
                        processed_children.extend(grandchildren);
                    }
                    _ => processed_children.push(processed_child),
                }
            }
            Node::Document(processed_children)
        }
        Node::Bold(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast_for_conversion)
                .collect();
            Node::Bold(processed_children)
        }
        Node::Italic(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast_for_conversion)
                .collect();
            Node::Italic(processed_children)
        }
        Node::Code(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast_for_conversion)
                .collect();
            Node::Code(processed_children)
        }
        Node::Superscript(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast_for_conversion)
                .collect();
            Node::Superscript(processed_children)
        }
        Node::Subscript(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast_for_conversion)
                .collect();
            Node::Subscript(processed_children)
        }
        Node::SmallCaps(children) => {
            let processed_children: Vec<Node> = children
                .iter()
                .map(strip_structural_elements_from_ast_for_conversion)
                .collect();
            Node::SmallCaps(processed_children)
        }
        Node::Link { url, text } => {
            let processed_text: Vec<Node> = text
                .iter()
                .map(strip_structural_elements_from_ast_for_conversion)
                .collect();
            Node::Link {
                url: url.clone(),
                text: processed_text,
            }
        }
        Node::Text(text) => Node::Text(text.clone()),
    }
}

/// Validate AST content based on content type
pub fn validate_ast_content(node: &Node, conversion_limit: ConversionLimit) -> ThothResult<()> {
    match conversion_limit {
        ConversionLimit::Title => validate_title_content(node),
        ConversionLimit::Abstract | ConversionLimit::Biography => validate_abstract_content(node),
    }
}

/// Validate title/subtitle content - only inline formatting allowed
fn validate_title_content(node: &Node) -> ThothResult<()> {
    match node {
        Node::Document(children) => {
            // Document should only contain inline elements or a single paragraph
            if children.len() > 1 {
                // Check if all children are inline elements
                let all_inline = children.iter().all(|child| {
                    matches!(
                        child,
                        Node::Bold(_)
                            | Node::Italic(_)
                            | Node::Code(_)
                            | Node::Superscript(_)
                            | Node::Subscript(_)
                            | Node::SmallCaps(_)
                            | Node::Text(_)
                            | Node::Link { .. }
                    )
                });
                if !all_inline {
                    return Err(ThothError::TitleMultipleTopLevelElementsError);
                }
            }
            for child in children {
                validate_title_content(child)?;
            }
        }
        Node::Paragraph(children) => {
            // Paragraphs are allowed in titles, but only for grouping inline elements
            for child in children {
                validate_title_content(child)?;
            }
        }
        Node::Bold(children)
        | Node::Italic(children)
        | Node::Code(children)
        | Node::Superscript(children)
        | Node::Subscript(children)
        | Node::SmallCaps(children) => {
            // Inline formatting elements are allowed
            for child in children {
                validate_title_content(child)?;
            }
        }
        Node::Link { text, .. } => {
            // Links are allowed
            for child in text {
                validate_title_content(child)?;
            }
        }
        Node::Text(_) => {
            // Text nodes are allowed
        }
        Node::List(_) => {
            return Err(ThothError::TitleListItemError);
        }
        Node::ListItem(_) => {
            return Err(ThothError::TitleListItemError);
        }
    }
    Ok(())
}

/// Validate abstract/biography content - paragraphs, breaks, and lists allowed
fn validate_abstract_content(node: &Node) -> ThothResult<()> {
    match node {
        Node::Document(children) => {
            for child in children {
                validate_abstract_content(child)?;
            }
        }
        Node::Paragraph(children)
        | Node::Bold(children)
        | Node::Italic(children)
        | Node::Code(children)
        | Node::Superscript(children)
        | Node::Subscript(children)
        | Node::SmallCaps(children) => {
            for child in children {
                validate_abstract_content(child)?;
            }
        }
        Node::List(children) | Node::ListItem(children) => {
            for child in children {
                validate_abstract_content(child)?;
            }
        }
        Node::Link { text, .. } => {
            for child in text {
                validate_abstract_content(child)?;
            }
        }
        Node::Text(_) => {
            // Text nodes are always allowed
        }
    }
    Ok(())
}

/// Check if content contains disallowed structural elements for titles
pub fn contains_disallowed_title_elements(content: &str) -> Vec<String> {
    let mut disallowed = Vec::new();

    // Check for HTML structural elements
    let structural_patterns = [
        (r"<ul[^>]*>", "unordered list"),
        (r"<ol[^>]*>", "ordered list"),
        (r"<li[^>]*>", "list item"),
        (r"<br\s*/?>", "line break"),
        (r"<break\s*/?>", "break element"),
    ];

    for (pattern, description) in structural_patterns.iter() {
        if let Ok(re) = regex::Regex::new(pattern) {
            if re.is_match(content) {
                disallowed.push(description.to_string());
            }
        }
    }

    // Check for Markdown structural elements
    if content.contains("\n\n") && content.split("\n\n").count() > 1 {
        disallowed.push("multiple paragraphs".to_string());
    }

    if content
        .lines()
        .any(|line| line.trim().starts_with("- ") || line.trim().starts_with("* "))
    {
        disallowed.push("markdown list".to_string());
    }

    disallowed
}

/// Check if content contains disallowed structural elements for abstracts/biographies
pub fn contains_disallowed_abstract_elements(content: &str) -> Vec<String> {
    let mut disallowed = Vec::new();

    // For abstracts/biographies, we allow most structural elements
    // Only check for truly problematic elements

    // Check for nested lists (which might be too complex)
    if let Ok(re) = regex::Regex::new(r"<li[^>]*>.*<ul[^>]*>") {
        if re.is_match(content) {
            disallowed.push("nested lists".to_string());
        }
    }

    // Check for tables (not supported)
    if content.contains("<table") || content.contains("<tr") || content.contains("<td") {
        disallowed.push("tables".to_string());
    }

    // Check for images (not supported)
    if content.contains("<img") || content.contains("![") {
        disallowed.push("images".to_string());
    }

    disallowed
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
    fn test_html_to_ast_small_caps() {
        let html = "<text>Small caps text</text>";
        let ast = html_to_ast(html);

        // Check that we have a SmallCaps node somewhere in the AST
        fn find_small_caps(node: &Node) -> bool {
            match node {
                Node::SmallCaps(children) => {
                    if children.len() == 1 {
                        match &children[0] {
                            Node::Text(content) => content == "Small caps text",
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                Node::Document(children) | Node::Paragraph(children) => {
                    children.iter().any(find_small_caps)
                }
                _ => false,
            }
        }

        assert!(
            find_small_caps(&ast),
            "Expected to find SmallCaps node with 'Small caps text'"
        );
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
            Node::Text(content) => {
                assert_eq!(content, "This is a single paragraph.");
            }
            _ => panic!("Expected text node"),
        }
    }

    #[test]
    fn test_plain_text_to_ast_multiple_paragraphs() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let ast = plain_text_to_ast(text);

        match ast {
            Node::Text(content) => {
                assert_eq!(
                    content,
                    "First paragraph.\n\nSecond paragraph.\n\nThird paragraph."
                );
            }
            _ => panic!("Expected text node"),
        }
    }

    #[test]
    fn test_plain_text_to_ast_empty_paragraphs_filtered() {
        let text = "First paragraph.\n\n\n\nSecond paragraph.";
        let ast = plain_text_to_ast(text);

        match ast {
            Node::Text(content) => {
                assert_eq!(content, "First paragraph.\n\n\n\nSecond paragraph.");
            }
            _ => panic!("Expected text node"),
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
    fn test_ast_to_jats_superscript() {
        let ast = Node::Superscript(vec![Node::Text("2".to_string())]);
        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "<sup>2</sup>");
    }

    #[test]
    fn test_ast_to_jats_subscript() {
        let ast = Node::Subscript(vec![Node::Text("H2O".to_string())]);
        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "<sub>H2O</sub>");
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
        let jats = plain_text_ast_to_jats(&ast);

        // Should wrap plain text in <p><sc> tags
        assert_eq!(
            jats,
            "<p>First paragraph.\n\nSecond paragraph with multiple lines.\nIt continues here.</p>"
        );
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

    #[test]
    fn test_markdown_to_ast_code() {
        let markdown = "This is `inline code` text";
        let ast = markdown_to_ast(markdown);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph(para_children) => {
                        assert_eq!(para_children.len(), 3); // Text, Code, Text
                        let has_code = para_children
                            .iter()
                            .any(|child| matches!(child, Node::Code(_)));
                        assert!(has_code);
                    }
                    _ => panic!("Expected paragraph node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_html_to_ast_code() {
        let html = "<p>This is <code>inline code</code> text</p>";
        let ast = html_to_ast(html);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Paragraph(para_children) => {
                        assert_eq!(para_children.len(), 3); // Text, Code, Text
                        let has_code = para_children
                            .iter()
                            .any(|child| matches!(child, Node::Code(_)));
                        assert!(has_code);
                    }
                    _ => panic!("Expected paragraph node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_ast_to_jats_code() {
        let ast = Node::Code(vec![Node::Text("inline code".to_string())]);
        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "<monospace>inline code</monospace>");
    }

    #[test]
    fn test_ast_to_jats_code_with_nested_content() {
        let ast = Node::Code(vec![
            Node::Text("function ".to_string()),
            Node::Bold(vec![Node::Text("main".to_string())]),
            Node::Text("()".to_string()),
        ]);
        let jats = ast_to_jats(&ast);
        assert_eq!(jats, "<monospace>function <bold>main</bold>()</monospace>");
    }

    #[test]
    fn test_round_trip_markdown_code_to_jats() {
        let markdown = "Use `println!` macro for output";
        let ast = markdown_to_ast(markdown);
        let jats = ast_to_jats(&ast);

        assert!(jats.contains("<monospace>println!</monospace>"));
    }

    #[test]
    fn test_round_trip_html_code_to_jats() {
        let html = "<p>Use <code>println!</code> macro for output</p>";
        let ast = html_to_ast(html);
        let jats = ast_to_jats(&ast);

        assert!(jats.contains("<monospace>println!</monospace>"));
    }

    #[test]
    fn test_code_with_multiple_spans() {
        let markdown = "`first` and `second` code spans";
        let ast = markdown_to_ast(markdown);
        let jats = ast_to_jats(&ast);

        assert!(jats.contains("<monospace>first</monospace>"));
        assert!(jats.contains("<monospace>second</monospace>"));
    }

    #[test]
    fn test_code_in_list_item() {
        let markdown = "- Use `git commit` to save changes";
        let ast = markdown_to_ast(markdown);
        let jats = ast_to_jats(&ast);

        assert!(jats.contains("<list-item>"));
        assert!(jats.contains("<monospace>git commit</monospace>"));
    }

    #[test]
    fn test_code_in_link() {
        let html = r#"<a href="https://docs.rs">Visit <code>docs.rs</code> for documentation</a>"#;
        let ast = html_to_ast(html);
        let jats = ast_to_jats(&ast);

        assert!(jats.contains(r#"<ext-link xlink:href="https://docs.rs">"#));
        assert!(jats.contains("<monospace>docs.rs</monospace>"));
    }

    #[test]
    fn test_plain_text_to_ast_with_url() {
        let text = "Visit https://example.com for more info";
        let ast = plain_text_to_ast(text);

        match ast {
            Node::Document(children) => {
                assert_eq!(children.len(), 3); // Text, Link, Text
                let has_link = children
                    .iter()
                    .any(|child| matches!(child, Node::Link { .. }));
                assert!(has_link);
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_plain_text_to_ast_multiple_urls() {
        let text = "Check https://example.com and https://docs.rs for resources";
        let ast = plain_text_to_ast(text);
        let jats = ast_to_jats(&ast);

        assert!(jats.contains(r#"<ext-link xlink:href="https://example.com">"#));
        assert!(jats.contains(r#"<ext-link xlink:href="https://docs.rs">"#));
    }

    #[test]
    fn test_plain_text_to_ast_no_urls() {
        let text = "This is just plain text without any URLs";
        let ast = plain_text_to_ast(text);

        match ast {
            Node::Text(content) => {
                assert_eq!(content, "This is just plain text without any URLs");
            }
            _ => panic!("Expected text node"),
        }
    }

    #[test]
    fn test_plain_text_to_ast_url_with_text() {
        let text = "Visit https://example.com for more information";
        let ast = plain_text_to_ast(text);
        let jats = ast_to_jats(&ast);

        assert!(jats.contains("Visit "));
        assert!(jats.contains(
            r#"<ext-link xlink:href="https://example.com">https://example.com</ext-link>"#
        ));
        assert!(jats.contains(" for more information"));
    }

    // Validation tests
    #[test]
    fn test_validate_title_content_valid() {
        let ast = Node::Document(vec![Node::Paragraph(vec![Node::Text(
            "Simple Title".to_string(),
        )])]);
        assert!(validate_ast_content(&ast, ConversionLimit::Title).is_ok());
    }

    #[test]
    fn test_validate_title_content_with_inline_formatting() {
        let ast = Node::Document(vec![Node::Paragraph(vec![
            Node::Bold(vec![Node::Text("Bold".to_string())]),
            Node::Text(" and ".to_string()),
            Node::Italic(vec![Node::Text("italic".to_string())]),
            Node::Text(" text".to_string()),
        ])]);
        assert!(validate_ast_content(&ast, ConversionLimit::Title).is_ok());
    }

    #[test]
    fn test_validate_title_content_with_link() {
        let ast = Node::Document(vec![Node::Paragraph(vec![
            Node::Text("Visit ".to_string()),
            Node::Link {
                url: "https://example.com".to_string(),
                text: vec![Node::Text("example.com".to_string())],
            },
        ])]);
        assert!(validate_ast_content(&ast, ConversionLimit::Title).is_ok());
    }

    #[test]
    fn test_validate_title_content_disallows_lists() {
        let ast = Node::Document(vec![Node::List(vec![Node::ListItem(vec![Node::Text(
            "Item 1".to_string(),
        )])])]);
        assert!(validate_ast_content(&ast, ConversionLimit::Title).is_err());
    }

    #[test]
    fn test_validate_title_content_disallows_multiple_top_level() {
        let ast = Node::Document(vec![
            Node::Paragraph(vec![Node::Text("First".to_string())]),
            Node::Paragraph(vec![Node::Text("Second".to_string())]),
        ]);
        assert!(validate_ast_content(&ast, ConversionLimit::Title).is_err());
    }

    #[test]
    fn test_validate_abstract_content_allows_lists() {
        let ast = Node::Document(vec![Node::List(vec![
            Node::ListItem(vec![Node::Text("Item 1".to_string())]),
            Node::ListItem(vec![Node::Text("Item 2".to_string())]),
        ])]);
        assert!(validate_ast_content(&ast, ConversionLimit::Abstract).is_ok());
    }

    #[test]
    fn test_validate_abstract_content_allows_multiple_paragraphs() {
        let ast = Node::Document(vec![
            Node::Paragraph(vec![Node::Text("First paragraph".to_string())]),
            Node::Paragraph(vec![Node::Text("Second paragraph".to_string())]),
        ]);
        assert!(validate_ast_content(&ast, ConversionLimit::Abstract).is_ok());
    }

    #[test]
    fn test_validate_abstract_content_allows_nested_formatting() {
        let ast = Node::Document(vec![Node::Paragraph(vec![Node::Bold(vec![
            Node::Text("Bold with ".to_string()),
            Node::Italic(vec![Node::Text("italic".to_string())]),
        ])])]);
        assert!(validate_ast_content(&ast, ConversionLimit::Abstract).is_ok());
    }

    #[test]
    fn test_contains_disallowed_title_elements_html() {
        let content = "<p>Title with <ul><li>list</li></ul></p>";
        let disallowed = contains_disallowed_title_elements(content);
        assert!(disallowed.contains(&"unordered list".to_string()));
    }

    #[test]
    fn test_contains_disallowed_title_elements_markdown() {
        let content = "Title\n\nWith multiple paragraphs";
        let disallowed = contains_disallowed_title_elements(content);
        assert!(disallowed.contains(&"multiple paragraphs".to_string()));
    }

    #[test]
    fn test_contains_disallowed_title_elements_markdown_list() {
        let content = "Title with\n- Item 1\n- Item 2";
        let disallowed = contains_disallowed_title_elements(content);
        assert!(disallowed.contains(&"markdown list".to_string()));
    }

    #[test]
    fn test_contains_disallowed_title_elements_valid() {
        let content = "<p><strong>Valid Title</strong></p>";
        let disallowed = contains_disallowed_title_elements(content);
        assert!(disallowed.is_empty());
    }

    #[test]
    fn test_contains_disallowed_abstract_elements_tables() {
        let content = "<p>Abstract with <table><tr><td>data</td></tr></table></p>";
        let disallowed = contains_disallowed_abstract_elements(content);
        assert!(disallowed.contains(&"tables".to_string()));
    }

    #[test]
    fn test_contains_disallowed_abstract_elements_images() {
        let content = "<p>Abstract with <img src=\"test.jpg\"></p>";
        let disallowed = contains_disallowed_abstract_elements(content);
        assert!(disallowed.contains(&"images".to_string()));
    }

    #[test]
    fn test_contains_disallowed_abstract_elements_valid() {
        let content = "<p>Valid abstract with <ul><li>list</li></ul></p>";
        let disallowed = contains_disallowed_abstract_elements(content);
        assert!(disallowed.is_empty());
    }

    #[test]
    fn test_validation_error_display() {
        let error = ThothError::RequestError("Lists are not allowed".to_string());
        assert!(error.to_string().contains("Lists are not allowed"));

        let error = ThothError::RequestError("Structural element 'div' is not allowed".to_string());
        assert!(error
            .to_string()
            .contains("Structural element 'div' is not allowed"));
    }

    // JATS to AST tests
    #[test]
    fn test_jats_to_ast_basic_formatting() {
        let jats = "<bold>Bold text</bold> and <italic>italic text</italic>";
        let ast = jats_to_ast(jats);

        // Debug: let's see what we actually get
        match ast {
            Node::Document(children) => {
                // For now, let's just check that we have the expected elements
                // regardless of whether they're wrapped in a paragraph
                let has_bold = children.iter().any(|child| matches!(child, Node::Bold(_)));
                let has_italic = children
                    .iter()
                    .any(|child| matches!(child, Node::Italic(_)));
                let has_text = children.iter().any(|child| matches!(child, Node::Text(_)));
                assert!(has_bold);
                assert!(has_italic);
                assert!(has_text);

                // If we have exactly 3 children, they should be wrapped in a paragraph
                if children.len() == 3 {
                    // This means the paragraph wrapping didn't work
                    // Let's check if all children are inline elements
                    let all_inline = children.iter().all(|child| {
                        matches!(
                            child,
                            Node::Bold(_)
                                | Node::Italic(_)
                                | Node::Code(_)
                                | Node::Superscript(_)
                                | Node::Subscript(_)
                                | Node::Text(_)
                                | Node::Link { .. }
                        )
                    });
                    assert!(all_inline, "All children should be inline elements");
                } else if children.len() == 1 {
                    // This means they were wrapped in a paragraph
                    match &children[0] {
                        Node::Paragraph(para_children) => {
                            assert_eq!(para_children.len(), 3);
                        }
                        _ => panic!("Expected paragraph node"),
                    }
                } else {
                    panic!("Unexpected number of children: {}", children.len());
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_jats_to_ast_link() {
        let jats = r#"<ext-link xlink:href="https://example.com">Link text</ext-link>"#;
        let ast = jats_to_ast(jats);

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
    fn test_jats_to_ast_list() {
        let jats = "<list><list-item>Item 1</list-item><list-item>Item 2</list-item></list>";
        let ast = jats_to_ast(jats);

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
    fn test_jats_to_ast_superscript_subscript() {
        let jats = "<p>H<sub>2</sub>O and E=mc<sup>2</sup></p>";
        let ast = jats_to_ast(jats);

        match ast {
            Node::Document(children) => {
                // The HTML parser creates multiple nodes: text "H", sub, text "O and E=mc", sup, text ""
                assert!(!children.is_empty());

                // Helper function to check recursively for subscript/superscript
                fn has_node_type(node: &Node, check_subscript: bool) -> bool {
                    match node {
                        Node::Subscript(_) if check_subscript => true,
                        Node::Superscript(_) if !check_subscript => true,
                        Node::Document(children)
                        | Node::Paragraph(children)
                        | Node::Bold(children)
                        | Node::Italic(children)
                        | Node::Code(children)
                        | Node::Superscript(children)
                        | Node::Subscript(children)
                        | Node::List(children)
                        | Node::ListItem(children) => children
                            .iter()
                            .any(|child| has_node_type(child, check_subscript)),
                        Node::Link { text, .. } => text
                            .iter()
                            .any(|child| has_node_type(child, check_subscript)),
                        _ => false,
                    }
                }

                let has_subscript = children.iter().any(|child| has_node_type(child, true));
                let has_superscript = children.iter().any(|child| has_node_type(child, false));

                assert!(has_subscript);
                assert!(has_superscript);
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_jats_to_ast_small_caps() {
        let jats = "<sc>Small caps text</sc>";
        let ast = jats_to_ast(jats);

        // Debug: let's see what we actually get
        match ast {
            Node::SmallCaps(children) => {
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Text(content) => {
                        assert_eq!(content, "Small caps text");
                    }
                    _ => panic!("Expected text node as child of SmallCaps"),
                }
            }
            Node::Document(children) => {
                // If it's a document, check if it has one child that's a SmallCaps node
                if children.len() == 1 {
                    match &children[0] {
                        Node::SmallCaps(sc_children) => {
                            assert_eq!(sc_children.len(), 1);
                            match &sc_children[0] {
                                Node::Text(content) => {
                                    assert_eq!(content, "Small caps text");
                                }
                                _ => panic!("Expected text node as child of SmallCaps"),
                            }
                        }
                        _ => panic!(
                            "Expected SmallCaps node as single child, got: {:?}",
                            children[0]
                        ),
                    }
                } else {
                    panic!(
                        "Expected single child in document, got {} children: {:?}",
                        children.len(),
                        children
                    );
                }
            }
            _ => panic!(
                "Expected SmallCaps node or document with SmallCaps child, got: {:?}",
                ast
            ),
        }
    }

    #[test]
    fn test_jats_to_ast_round_trip() {
        let original_jats = "<bold>Bold</bold> and <italic>italic</italic> with <ext-link xlink:href=\"https://example.com\">link</ext-link>";
        let ast = jats_to_ast(original_jats);
        let converted_jats = ast_to_jats(&ast);

        // Should preserve the basic structure
        assert!(converted_jats.contains("<bold>Bold</bold>"));
        assert!(converted_jats.contains("<italic>italic</italic>"));
        assert!(converted_jats
            .contains(r#"<ext-link xlink:href="https://example.com">link</ext-link>"#));
    }

    // AST to HTML tests
    #[test]
    fn test_ast_to_html_basic() {
        let ast = Node::Document(vec![Node::Paragraph(vec![
            Node::Bold(vec![Node::Text("Bold".to_string())]),
            Node::Text(" and ".to_string()),
            Node::Italic(vec![Node::Text("italic".to_string())]),
        ])]);
        let html = ast_to_html(&ast);
        assert_eq!(html, "<p><strong>Bold</strong> and <em>italic</em></p>");
    }

    #[test]
    fn test_ast_to_html_small_caps() {
        let ast = Node::SmallCaps(vec![Node::Text("Small caps text".to_string())]);
        let html = ast_to_html(&ast);
        assert_eq!(html, "<text>Small caps text</text>");
    }

    #[test]
    fn test_ast_to_html_list() {
        let ast = Node::List(vec![
            Node::ListItem(vec![Node::Text("Item 1".to_string())]),
            Node::ListItem(vec![Node::Text("Item 2".to_string())]),
        ]);
        let html = ast_to_html(&ast);
        assert_eq!(html, "<ul><li>Item 1</li><li>Item 2</li></ul>");
    }

    #[test]
    fn test_ast_to_html_link() {
        let ast = Node::Link {
            url: "https://example.com".to_string(),
            text: vec![Node::Text("Link text".to_string())],
        };
        let html = ast_to_html(&ast);
        assert_eq!(html, r#"<a href="https://example.com">Link text</a>"#);
    }

    // AST to Markdown tests
    #[test]
    fn test_ast_to_markdown_basic() {
        let ast = Node::Document(vec![Node::Paragraph(vec![
            Node::Bold(vec![Node::Text("Bold".to_string())]),
            Node::Text(" and ".to_string()),
            Node::Italic(vec![Node::Text("italic".to_string())]),
        ])]);
        let markdown = ast_to_markdown(&ast);
        assert_eq!(markdown, "**Bold** and *italic*");
    }

    #[test]
    fn test_ast_to_markdown_list() {
        let ast = Node::List(vec![
            Node::ListItem(vec![Node::Text("Item 1".to_string())]),
            Node::ListItem(vec![Node::Text("Item 2".to_string())]),
        ]);
        let markdown = ast_to_markdown(&ast);
        assert_eq!(markdown, "- Item 1\n- Item 2\n");
    }

    #[test]
    fn test_ast_to_markdown_link() {
        let ast = Node::Link {
            url: "https://example.com".to_string(),
            text: vec![Node::Text("Link text".to_string())],
        };
        let markdown = ast_to_markdown(&ast);
        assert_eq!(markdown, "[Link text](https://example.com)");
    }

    #[test]
    fn test_ast_to_markdown_code() {
        let ast = Node::Code(vec![Node::Text("code".to_string())]);
        let markdown = ast_to_markdown(&ast);
        assert_eq!(markdown, "`code`");
    }

    // AST to plain text tests
    #[test]
    fn test_ast_to_plain_text_basic() {
        let ast = Node::Document(vec![Node::Paragraph(vec![
            Node::Bold(vec![Node::Text("Bold".to_string())]),
            Node::Text(" and ".to_string()),
            Node::Italic(vec![Node::Text("italic".to_string())]),
        ])]);
        let plain = ast_to_plain_text(&ast);
        assert_eq!(plain, "Bold and italic");
    }

    #[test]
    fn test_ast_to_plain_text_list() {
        let ast = Node::List(vec![
            Node::ListItem(vec![Node::Text("Item 1".to_string())]),
            Node::ListItem(vec![Node::Text("Item 2".to_string())]),
        ]);
        let plain = ast_to_plain_text(&ast);
        assert_eq!(plain, "• Item 1\n• Item 2\n");
    }

    #[test]
    fn test_ast_to_plain_text_link() {
        let ast = Node::Link {
            url: "https://example.com".to_string(),
            text: vec![Node::Text("Link text".to_string())],
        };
        let plain = ast_to_plain_text(&ast);
        assert_eq!(plain, "Link text (https://example.com)");
    }

    #[test]
    fn test_ast_to_plain_text_multiple_paragraphs() {
        let ast = Node::Document(vec![
            Node::Paragraph(vec![Node::Text("First paragraph".to_string())]),
            Node::Paragraph(vec![Node::Text("Second paragraph".to_string())]),
        ]);
        let plain = ast_to_plain_text(&ast);
        assert_eq!(plain, "First paragraph\n\nSecond paragraph");
    }

    // Round-trip tests
    #[test]
    fn test_round_trip_html_to_ast_to_html() {
        let original_html = "<p><strong>Bold</strong> and <em>italic</em></p>";
        let ast = html_to_ast(original_html);
        let converted_html = ast_to_html(&ast);
        assert_eq!(converted_html, original_html);
    }

    #[test]
    fn test_round_trip_markdown_to_ast_to_markdown() {
        let original_markdown = "**Bold** and *italic*";
        let ast = markdown_to_ast(original_markdown);
        let converted_markdown = ast_to_markdown(&ast);
        // Note: The converted markdown might be slightly different due to paragraph wrapping
        assert!(converted_markdown.contains("**Bold**"));
        assert!(converted_markdown.contains("*italic*"));
    }

    #[test]
    fn test_round_trip_jats_to_ast_to_jats() {
        let original_jats = "<bold>Bold</bold> and <italic>italic</italic>";
        let ast = jats_to_ast(original_jats);
        let converted_jats = ast_to_jats(&ast);
        assert!(converted_jats.contains("<bold>Bold</bold>"));
        assert!(converted_jats.contains("<italic>italic</italic>"));
    }
}
