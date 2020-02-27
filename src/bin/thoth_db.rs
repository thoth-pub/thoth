#[macro_use]
extern crate diesel_migrations;

use std::io;
use diesel_migrations::embed_migrations;
use thoth::db::establish_connection;
use std::io::stdout;

fn main() -> io::Result<()> {
    embed_migrations!("migrations");
    let connection = establish_connection().get().unwrap();
    Ok(embedded_migrations::run_with_output(&connection, &mut stdout())
        .expect("Can't run migrations"))
}
