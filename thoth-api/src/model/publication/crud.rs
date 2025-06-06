use super::{
    NewPublication, NewPublicationHistory, PatchPublication, Publication, PublicationField,
    PublicationHistory, PublicationOrderBy, PublicationType,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{publication, publication_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Publication {
    type NewEntity = NewPublication;
    type PatchEntity = PatchPublication;
    type OrderByEntity = PublicationOrderBy;
    type FilterParameter1 = PublicationType;
    type FilterParameter2 = ();
    type FilterParameter3 = ();

    fn pk(&self) -> Uuid {
        self.publication_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        parent_id_1: Option<Uuid>,
        _: Option<Uuid>,
        publication_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Publication>> {
        use crate::schema::publication::dsl::*;
        let mut connection = db.get()?;
        let mut query = publication
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::publication::all_columns)
            .into_boxed();

        query = match order.field {
            PublicationField::PublicationId => match order.direction {
                Direction::Asc => query.order(publication_id.asc()),
                Direction::Desc => query.order(publication_id.desc()),
            },
            PublicationField::PublicationType => match order.direction {
                Direction::Asc => query.order(publication_type.asc()),
                Direction::Desc => query.order(publication_type.desc()),
            },
            PublicationField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            PublicationField::Isbn => match order.direction {
                Direction::Asc => query.order(isbn.asc()),
                Direction::Desc => query.order(isbn.desc()),
            },
            PublicationField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            PublicationField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
            PublicationField::WidthMm => match order.direction {
                Direction::Asc => query.order(width_mm.asc()),
                Direction::Desc => query.order(width_mm.desc()),
            },
            PublicationField::WidthIn => match order.direction {
                Direction::Asc => query.order(width_in.asc()),
                Direction::Desc => query.order(width_in.desc()),
            },
            PublicationField::HeightMm => match order.direction {
                Direction::Asc => query.order(height_mm.asc()),
                Direction::Desc => query.order(height_mm.desc()),
            },
            PublicationField::HeightIn => match order.direction {
                Direction::Asc => query.order(height_in.asc()),
                Direction::Desc => query.order(height_in.desc()),
            },
            PublicationField::DepthMm => match order.direction {
                Direction::Asc => query.order(depth_mm.asc()),
                Direction::Desc => query.order(depth_mm.desc()),
            },
            PublicationField::DepthIn => match order.direction {
                Direction::Asc => query.order(depth_in.asc()),
                Direction::Desc => query.order(depth_in.desc()),
            },
            PublicationField::WeightG => match order.direction {
                Direction::Asc => query.order(weight_g.asc()),
                Direction::Desc => query.order(weight_g.desc()),
            },
            PublicationField::WeightOz => match order.direction {
                Direction::Asc => query.order(weight_oz.asc()),
                Direction::Desc => query.order(weight_oz.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if !publication_types.is_empty() {
            query = query.filter(publication_type.eq_any(publication_types));
        }
        if let Some(filter) = filter {
            // ISBN field is nullable, so searching with an empty filter could fail
            if !filter.is_empty() {
                query = query.filter(isbn.ilike(format!("%{filter}%")));
            }
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Publication>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        publication_types: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32> {
        use crate::schema::publication::dsl::*;
        let mut connection = db.get()?;
        let mut query = publication
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if !publication_types.is_empty() {
            query = query.filter(publication_type.eq_any(publication_types));
        }
        if let Some(filter) = filter {
            // ISBN field is nullable, so searching with an empty filter could fail
            if !filter.is_empty() {
                query = query.filter(isbn.ilike(format!("%{filter}%")));
            }
        }

        // `SELECT COUNT(*)` in postgres returns a BIGINT, which diesel parses as i64. Juniper does
        // not implement i64 yet, only i32. The only sensible way, albeit shameful, to solve this
        // is converting i64 to string and then parsing it as i32. This should work until we reach
        // 2147483647 records - if you are fixing this bug, congratulations on book number 2147483647!
        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.work_id)?.publisher_id(db)
    }

    crud_methods!(publication::table, publication::dsl::publication);
}

impl HistoryEntry for Publication {
    type NewHistoryEntity = NewPublicationHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            publication_id: self.publication_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewPublicationHistory {
    type MainEntity = PublicationHistory;

    db_insert!(publication_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publication_pk() {
        let publication: Publication = Default::default();
        assert_eq!(publication.pk(), publication.publication_id);
    }

    #[test]
    fn test_new_publication_history_from_publication() {
        let publication: Publication = Default::default();
        let user_id = "123456".to_string();
        let new_publication_history = publication.new_history_entry(&user_id);
        assert_eq!(
            new_publication_history.publication_id,
            publication.publication_id
        );
        assert_eq!(new_publication_history.user_id, user_id);
        assert_eq!(
            new_publication_history.data,
            serde_json::Value::String(serde_json::to_string(&publication).unwrap())
        );
    }
}
