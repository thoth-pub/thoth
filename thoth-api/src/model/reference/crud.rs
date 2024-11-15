use super::{
    NewReference, NewReferenceHistory, PatchReference, Reference, ReferenceField, ReferenceHistory,
    ReferenceOrderBy,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{reference, reference_history};
use crate::{crud_methods, db_insert};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Reference {
    type NewEntity = NewReference;
    type PatchEntity = PatchReference;
    type OrderByEntity = ReferenceOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();

    fn pk(&self) -> Uuid {
        self.reference_id
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
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Reference>> {
        use crate::schema::reference::dsl::*;
        let mut connection = db.get()?;
        let mut query = reference
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::reference::all_columns)
            .into_boxed();

        query = match order.field {
            ReferenceField::ReferenceId => match order.direction {
                Direction::Asc => query.order(reference_id.asc()),
                Direction::Desc => query.order(reference_id.desc()),
            },
            ReferenceField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            ReferenceField::ReferenceOrdinal => match order.direction {
                Direction::Asc => query.order(reference_ordinal.asc()),
                Direction::Desc => query.order(reference_ordinal.desc()),
            },
            ReferenceField::Doi => match order.direction {
                Direction::Asc => query.order(doi.asc()),
                Direction::Desc => query.order(doi.desc()),
            },
            ReferenceField::UnstructuredCitation => match order.direction {
                Direction::Asc => query.order(unstructured_citation.asc()),
                Direction::Desc => query.order(unstructured_citation.desc()),
            },
            ReferenceField::Issn => match order.direction {
                Direction::Asc => query.order(issn.asc()),
                Direction::Desc => query.order(issn.desc()),
            },
            ReferenceField::Isbn => match order.direction {
                Direction::Asc => query.order(isbn.asc()),
                Direction::Desc => query.order(isbn.desc()),
            },
            ReferenceField::JournalTitle => match order.direction {
                Direction::Asc => query.order(journal_title.asc()),
                Direction::Desc => query.order(journal_title.desc()),
            },
            ReferenceField::ArticleTitle => match order.direction {
                Direction::Asc => query.order(article_title.asc()),
                Direction::Desc => query.order(article_title.desc()),
            },
            ReferenceField::SeriesTitle => match order.direction {
                Direction::Asc => query.order(series_title.asc()),
                Direction::Desc => query.order(series_title.desc()),
            },
            ReferenceField::VolumeTitle => match order.direction {
                Direction::Asc => query.order(volume_title.asc()),
                Direction::Desc => query.order(volume_title.desc()),
            },
            ReferenceField::Edition => match order.direction {
                Direction::Asc => query.order(edition.asc()),
                Direction::Desc => query.order(edition.desc()),
            },
            ReferenceField::Author => match order.direction {
                Direction::Asc => query.order(author.asc()),
                Direction::Desc => query.order(author.desc()),
            },
            ReferenceField::Volume => match order.direction {
                Direction::Asc => query.order(volume.asc()),
                Direction::Desc => query.order(volume.desc()),
            },
            ReferenceField::Issue => match order.direction {
                Direction::Asc => query.order(issue.asc()),
                Direction::Desc => query.order(issue.desc()),
            },
            ReferenceField::FirstPage => match order.direction {
                Direction::Asc => query.order(first_page.asc()),
                Direction::Desc => query.order(first_page.desc()),
            },
            ReferenceField::ComponentNumber => match order.direction {
                Direction::Asc => query.order(component_number.asc()),
                Direction::Desc => query.order(component_number.desc()),
            },
            ReferenceField::StandardDesignator => match order.direction {
                Direction::Asc => query.order(standard_designator.asc()),
                Direction::Desc => query.order(standard_designator.desc()),
            },
            ReferenceField::StandardsBodyName => match order.direction {
                Direction::Asc => query.order(standards_body_name.asc()),
                Direction::Desc => query.order(standards_body_name.desc()),
            },
            ReferenceField::StandardsBodyAcronym => match order.direction {
                Direction::Asc => query.order(standards_body_acronym.asc()),
                Direction::Desc => query.order(standards_body_acronym.desc()),
            },
            ReferenceField::Url => match order.direction {
                Direction::Asc => query.order(url.asc()),
                Direction::Desc => query.order(url.desc()),
            },
            ReferenceField::PublicationDate => match order.direction {
                Direction::Asc => query.order(publication_date.asc()),
                Direction::Desc => query.order(publication_date.desc()),
            },
            ReferenceField::RetrievalDate => match order.direction {
                Direction::Asc => query.order(retrieval_date.asc()),
                Direction::Desc => query.order(retrieval_date.desc()),
            },
            ReferenceField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            ReferenceField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if let Some(filter) = filter {
            // All searchable fields are nullable, so searching with an empty filter could fail
            if !filter.is_empty() {
                query = query.filter(
                    doi.ilike(format!("%{filter}%"))
                        .or(unstructured_citation.ilike(format!("%{filter}%")))
                        .or(issn.ilike(format!("%{filter}%")))
                        .or(isbn.ilike(format!("%{filter}%")))
                        .or(journal_title.ilike(format!("%{filter}%")))
                        .or(article_title.ilike(format!("%{filter}%")))
                        .or(series_title.ilike(format!("%{filter}%")))
                        .or(volume_title.ilike(format!("%{filter}%")))
                        .or(author.ilike(format!("%{filter}%")))
                        .or(standard_designator.ilike(format!("%{filter}%")))
                        .or(standards_body_name.ilike(format!("%{filter}%")))
                        .or(url.ilike(format!("%{filter}%")))
                        .or(standards_body_acronym.ilike(format!("%{filter}%"))),
                );
            }
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Reference>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        publishers: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32> {
        use crate::schema::reference::dsl::*;
        let mut connection = db.get()?;
        let mut query = reference
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            // All searchable fields are nullable, so searching with an empty filter could fail
            if !filter.is_empty() {
                query = query.filter(
                    doi.ilike(format!("%{filter}%"))
                        .or(unstructured_citation.ilike(format!("%{filter}%")))
                        .or(issn.ilike(format!("%{filter}%")))
                        .or(isbn.ilike(format!("%{filter}%")))
                        .or(journal_title.ilike(format!("%{filter}%")))
                        .or(article_title.ilike(format!("%{filter}%")))
                        .or(series_title.ilike(format!("%{filter}%")))
                        .or(volume_title.ilike(format!("%{filter}%")))
                        .or(author.ilike(format!("%{filter}%")))
                        .or(standard_designator.ilike(format!("%{filter}%")))
                        .or(standards_body_name.ilike(format!("%{filter}%")))
                        .or(url.ilike(format!("%{filter}%")))
                        .or(standards_body_acronym.ilike(format!("%{filter}%"))),
                );
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
    crud_methods!(reference::table, reference::dsl::reference);
}

impl HistoryEntry for Reference {
    type NewHistoryEntity = NewReferenceHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            reference_id: self.reference_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewReferenceHistory {
    type MainEntity = ReferenceHistory;

    db_insert!(reference_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_pk() {
        let reference: Reference = Default::default();
        assert_eq!(reference.pk(), reference.reference_id);
    }

    #[test]
    fn test_new_publisher_history_from_publisher() {
        let reference: Reference = Default::default();
        let account_id: Uuid = Default::default();
        let new_reference_history = reference.new_history_entry(&account_id);
        assert_eq!(new_reference_history.reference_id, reference.reference_id);
        assert_eq!(new_reference_history.account_id, account_id);
        assert_eq!(
            new_reference_history.data,
            serde_json::Value::String(serde_json::to_string(&reference).unwrap())
        );
    }
}
