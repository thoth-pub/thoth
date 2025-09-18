use ast::{ast_to_jats, html_to_ast, markdown_to_ast, plain_text_to_ast};

fn main() {
    // Test markdown to AST
    let markdown = "**Bold text** and *italic text*\n\n- Item 1\n- Item 2";
    let ast = markdown_to_ast(markdown);
    println!("Markdown AST: {:#?}", ast);

    let jats = ast_to_jats(&ast);
    println!("JATS XML:\n{}", jats);

    println!("\n{}\n", "=".repeat(50));

    // Test HTML to AST
    let html = r#"<div><p><strong>Bold text</strong> and <em>italic text</em></p><ul><li>Item 1</li><li>Item 2</li></ul></div>"#;
    let html_ast = html_to_ast(html);
    println!("HTML AST: {:#?}", html_ast);

    let html_jats = ast_to_jats(&html_ast);
    println!("HTML to JATS XML:\n{}", html_jats);

    println!("\n{}\n", "=".repeat(50));

    // Test plain text to AST
    let plain_text = "This is the first paragraph.\n\nThis is the second paragraph with multiple lines.\nIt continues here.\n\nAnd this is the third paragraph.";
    let text_ast = plain_text_to_ast(plain_text);
    println!("Plain Text AST: {:#?}", text_ast);

    let text_jats = ast_to_jats(&text_ast);
    println!("Plain Text to JATS XML:\n{}", text_jats);
}
