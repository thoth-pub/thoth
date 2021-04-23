#[cfg(feature = "backend")]
/// Common functionality to perform basic CRUD actions on Thoth entities
pub trait Crud where Self: Sized {
    /// The structure used to create a new entity, e.g. `NewImprint`
    type NewEntity;
    /// The structure used to modify an existing entity, e.g. `PatchImprint`
    type PatchEntity;

    /// Specify the entity's primary key
    fn pk(&self) -> uuid::Uuid;

    /// Query the database to obtain an instance of the entity given its ID
    fn from_id(db: &crate::db::PgPool, entity_id: &uuid::Uuid) -> crate::errors::ThothResult<Self>;

    /// Insert a new record in the database and obtain the resulting instance
    fn create(db: &crate::db::PgPool, data: &Self::NewEntity) -> crate::errors::ThothResult<Self>;

    /// Modify the record in the database and obtain the resulting instance
    fn update(&self, db: &crate::db::PgPool, data: &Self::PatchEntity, account_id: &uuid::Uuid) -> crate::errors::ThothResult<Self>;

    /// Delete the record from the database and obtain the deleted instance
    fn delete(self, db: &crate::db::PgPool) -> crate::errors::ThothResult<Self>;
}

/// Declares function implementations for the `Crud` trait, reducing the boilerplate needed to define
/// the CRUD functionality for each entity.
///
/// Example usage
/// -------------
///
/// ```ignore
/// use crate::imprint::model::{NewImprint, PatchImprint, Imprint, NewImprintHistory};
/// use crate::crud_methods;
/// use crate::model::Crud;
///
/// impl Crud for Imprint {
///     type NewEntity = NewImprint;
///     type PatchEntity = PatchImprint;
///
///     fn pk(&self) -> uuid::Uuid {
///         self.imprint_id
///     }
///
///     crud_methods!(imprint::table, imprint::dsl::imprint, NewImprintHistory);
/// }
/// ```
///
///
#[cfg(feature = "backend")]
#[macro_export]
macro_rules! crud_methods {
    ($table_dsl:expr, $entity_dsl:expr, $history_entity:ident) => {
        fn from_id(db: &crate::db::PgPool, entity_id: &uuid::Uuid) -> crate::errors::ThothResult<Self> {
            use diesel::{RunQueryDsl, QueryDsl};

            let connection = db.get().unwrap();
            match $entity_dsl.find(entity_id).get_result::<Self>(&connection) {
                Ok(t) => Ok(t),
                Err(e) => Err(crate::errors::ThothError::from(e)),
            }
        }

        fn create(db: &crate::db::PgPool, data: &Self::NewEntity) -> crate::errors::ThothResult<Self> {
            use diesel::{RunQueryDsl};

            let connection = db.get().unwrap();
            match diesel::insert_into($table_dsl)
                .values(data)
                .get_result::<Self>(&connection)
            {
                Ok(t) => Ok(t),
                Err(e) => Err(crate::errors::ThothError::from(e)),
            }
        }

        /// Makes a database transaction that first updates the entity and then creates a new
        /// history entity record.
        fn update(&self, db: &crate::db::PgPool, data: &Self::PatchEntity, account_id: &uuid::Uuid) -> crate::errors::ThothResult<Self> {
            use diesel::{RunQueryDsl, QueryDsl, Connection};

            let connection = db.get().unwrap();
            connection.transaction(
                || match diesel::update($entity_dsl.find(&self.pk())).set(data).get_result(&connection) {
                    Ok(c) => {
                        match $history_entity::new(self, account_id.clone()).insert(&connection) {
                            Ok(_) => Ok(c),
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(crate::errors::ThothError::from(e)),
                },
            )
        }

        fn delete(self, db: &crate::db::PgPool) -> crate::errors::ThothResult<Self> {
            use diesel::{RunQueryDsl, QueryDsl};

            let connection = db.get().unwrap();
            match diesel::delete($entity_dsl.find(&self.pk())).execute(&connection) {
                Ok(_) => Ok(self),
                Err(e) => Err(crate::errors::ThothError::from(e)),
            }
        }
    }
}