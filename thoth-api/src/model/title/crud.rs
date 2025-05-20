use diesel::prelude::*;
use diesel::result::Error;
use uuid::Uuid;

use crate::schema::title;
use crate::schema::title::dsl::*;

use super::mode::{NewTitle, Title, PatchTitle};

pub fn create_title(conn: &mut PgConnection, new_title: NewTitle) -> Result<Title, Error> {
    diesel::insert_into(title::table)
        .values(&new_title)
        .get_result(conn)
}

pub fn get_title(conn: &mut PgConnection, title_id: Uuid) -> Result<Title, Error> {
    title.find(title_id).first(conn)
}

pub fn get_titles_by_work(conn: &mut PgConnection, work_id: Uuid) -> Result<Vec<Title>, Error> {
    title.filter(title::work_id.eq(work_id)).load(conn)
}

pub fn update_title(
    conn: &mut PgConnection,
    title_id: Uuid,
    update_title: PatchTitle,
) -> Result<Title, Error> {
    diesel::update(title.find(title_id))
        .set(update_title)
        .get_result(conn)
}

pub fn delete_title(conn: &mut PgConnection, title_id: Uuid) -> Result<(), Error> {
    diesel::delete(title.find(title_id)).execute(conn)?;
    Ok(())
}

pub fn delete_titles_by_work(conn: &mut PgConnection, work_id: Uuid) -> Result<(), Error> {
    diesel::delete(title.filter(title::work_id.eq(work_id))).execute(conn)?;
    Ok(())
}
