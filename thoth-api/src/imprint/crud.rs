use diesel::{RunQueryDsl, QueryDsl, Connection};
use uuid::Uuid;

use super::model::{NewImprint, PatchImprint, Imprint, NewImprintHistory};
use crate::db::PgPool;
use crate::errors::{ThothResult, ThothError};
use crate::schema::imprint;

macro_rules! crud_methods {
    ($table_dsl:expr, $entity_dsl:expr) => {
        fn from_id(db: &PgPool, entity_id: &Uuid) -> ThothResult<Self> {
            let connection = db.get().unwrap();
            match $entity_dsl.find(entity_id).get_result::<Self>(&connection) {
                Ok(t) => Ok(t),
                Err(e) => Err(ThothError::from(e)),
            }
        }

        fn create(db: &PgPool, data: &Self::NewEntity) -> ThothResult<Self> {
            let connection = db.get().unwrap();
            match diesel::insert_into($table_dsl)
                .values(data)
                .get_result::<Self>(&connection)
            {
                Ok(t) => Ok(t),
                Err(e) => Err(ThothError::from(e)),
            }
        }

        fn update(&self, db: &PgPool, data: &Self::PatchEntity, account_id: &Uuid) -> ThothResult<Self> {
            let connection = db.get().unwrap();

            connection.transaction(
                || match diesel::update($entity_dsl.find(&self.pk())).set(data).get_result(&connection) {
                    Ok(c) => {
                        match NewImprintHistory::new(self, account_id.clone()).insert(&connection) {
                            Ok(_) => Ok(c),
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(ThothError::from(e)),
                },
            )
        }

        fn delete(self, db: &PgPool) -> ThothResult<Self> {
            let connection = db.get().unwrap();
            match diesel::delete($entity_dsl.find(&self.pk())).execute(&connection) {
                Ok(_) => Ok(self),
                Err(e) => Err(ThothError::from(e)),
            }
        }
    }
}

pub trait Crud where Self: Sized {
    type NewEntity;
    type PatchEntity;

    fn pk(&self) -> Uuid;

    fn from_id(db: &PgPool, entity_id: &Uuid) -> ThothResult<Self>;

    fn create(db: &PgPool, data: &Self::NewEntity) -> ThothResult<Self>;

    fn update(&self, db: &PgPool, data: &Self::PatchEntity, account_id: &Uuid) -> ThothResult<Self>;

    fn delete(self, db: &PgPool) -> ThothResult<Self>;
}

impl Crud for Imprint {
    type NewEntity = NewImprint;
    type PatchEntity = PatchImprint;

    fn pk(&self) -> Uuid {
        self.imprint_id
    }

    crud_methods!(imprint::table, imprint::dsl::imprint);
}