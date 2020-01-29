extern crate dotenv;

use diesel::prelude::*;
use juniper::RootNode;
use uuid::Uuid;
use chrono::naive::NaiveDate;

use crate::db::PgPool;
use crate::schema::work;
use crate::sql_types::*;

#[derive(Clone)]
pub struct Context {
  pub db: PgPool,
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::object(Context = Context)]
impl QueryRoot {
  fn works(context: &Context) -> Vec<Work> {
    use crate::schema::work::dsl::*;
    let connection = context.db.get().unwrap();
    work
      .limit(100)
      .load::<Work>(&connection)
      .expect("Error loading works")
  }

  fn publications(context: &Context) -> Vec<Publication> {
    use crate::schema::publication::dsl::*;
    let connection = context.db.get().unwrap();
    publication
      .limit(10)
      .load::<Publication>(&connection)
      .expect("Error loading publications")
  }
}

pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {
  fn create_work(context: &Context, data: NewWork) -> Work {
    let connection = context.db.get().unwrap();
    diesel::insert_into(work::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new work")
  }
}

#[derive(Queryable)]
struct Work {
    work_id: Uuid,
    work_type: WorkType,
    title: String,
    subtitle: Option<String>,
    publisher: Option<String>,
    doi: Option<String>,
    publication_date: Option<NaiveDate>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "work"]
pub struct NewWork {
    work_id: Uuid,
    work_type: WorkType,
    title: String,
    subtitle: Option<String>,
    publisher: Option<String>,
    doi: Option<String>,
    publication_date: Option<NaiveDate>,
}

#[juniper::object(Context = Context, description = "A written text that can be published")]
impl Work {
    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn work_type(&self) -> &WorkType {
        &self.work_type
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn subtitle(&self) -> Option<&String> {
        self.subtitle.as_ref()
    }

    pub fn publisher(&self) -> Option<&String> {
        self.publisher.as_ref()
    }

    pub fn doi(&self) -> Option<&String> {
        self.doi.as_ref()
    }

    pub fn publication_date(&self) -> Option<NaiveDate> {
        self.publication_date
    }

    pub fn publications(&self, context: &Context) -> Vec<Publication> {
        use crate::schema::publication::dsl::*;
        let connection = context.db.get().unwrap();
        publication
            .filter(work_id.eq(self.work_id))
            .limit(100)
            .load::<Publication>(&connection)
            .expect("Error loading publications")
    }
}

#[derive(Queryable)]
struct Publication {
    publication_id: Uuid,
    publication_type: PublicationType,
    work_id: Uuid,
    isbn: Option<String>,
    publication_url: Option<String>,
}

#[juniper::object(description = "A manifestation of a written text")]
impl Publication {
    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    pub fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn isbn(&self) -> Option<&String> {
        self.isbn.as_ref()
    }

    pub fn publication_url(&self) -> Option<&String> {
        self.publication_url.as_ref()
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
  Schema::new(QueryRoot {}, MutationRoot {})
}
