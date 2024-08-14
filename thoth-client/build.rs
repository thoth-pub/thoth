use std::fs;
use std::path::Path;
use thoth_api::graphql::model::create_schema;

// Generate the GraphQL schema and store it in assets/schema.graphql
fn main() {
    let schema = create_schema().as_sdl();

    let out_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let schema_path = Path::new(&out_dir).join("assets").join("schema.graphql");

    fs::write(&schema_path, schema).expect("Unable to write schema file");
    println!("Generated schema file at: {:?}", &schema_path);
}
