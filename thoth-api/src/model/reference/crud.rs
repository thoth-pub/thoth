use super::{
    NewReference, NewReferenceHistory, PatchReference, Reference, ReferenceField, ReferenceHistory,
    ReferenceOrderBy,
};
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{reference, reference_history};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
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
    type FilterParameter4 = ();

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
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Reference>> {
        use crate::schema::reference::dsl::*;
        let mut connection = db.get()?;
        let mut query = reference
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::reference::all_columns)
            .into_boxed();

        query = match order.field {
            ReferenceField::ReferenceId => apply_directional_order!(query, order.direction, order, reference_id),
            ReferenceField::WorkId => apply_directional_order!(query, order.direction, order, work_id),
            ReferenceField::ReferenceOrdinal => apply_directional_order!(query, order.direction, order, reference_ordinal),
            ReferenceField::Doi => apply_directional_order!(query, order.direction, order, doi),
            ReferenceField::UnstructuredCitation => apply_directional_order!(query, order.direction, order, unstructured_citation),
            ReferenceField::Issn => apply_directional_order!(query, order.direction, order, issn),
            ReferenceField::Isbn => apply_directional_order!(query, order.direction, order, isbn),
            ReferenceField::JournalTitle => apply_directional_order!(query, order.direction, order, journal_title),
            ReferenceField::ArticleTitle => apply_directional_order!(query, order.direction, order, article_title),
            ReferenceField::SeriesTitle => apply_directional_order!(query, order.direction, order, series_title),
            ReferenceField::VolumeTitle => apply_directional_order!(query, order.direction, order, volume_title),
            ReferenceField::Edition => apply_directional_order!(query, order.direction, order, edition),
            ReferenceField::Author => apply_directional_order!(query, order.direction, order, author),
            ReferenceField::Volume => apply_directional_order!(query, order.direction, order, volume),
            ReferenceField::Issue => apply_directional_order!(query, order.direction, order, issue),
            ReferenceField::FirstPage => apply_directional_order!(query, order.direction, order, first_page),
            ReferenceField::ComponentNumber => apply_directional_order!(query, order.direction, order, component_number),
            ReferenceField::StandardDesignator => apply_directional_order!(query, order.direction, order, standard_designator),
            ReferenceField::StandardsBodyName => apply_directional_order!(query, order.direction, order, standards_body_name),
            ReferenceField::StandardsBodyAcronym => apply_directional_order!(query, order.direction, order, standards_body_acronym),
            ReferenceField::Url => apply_directional_order!(query, order.direction, order, url),
            ReferenceField::PublicationDate => apply_directional_order!(query, order.direction, order, publication_date),
            ReferenceField::RetrievalDate => apply_directional_order!(query, order.direction, order, retrieval_date),
            ReferenceField::CreatedAt => apply_directional_order!(query, order.direction, order, created_at),
            ReferenceField::UpdatedAt => apply_directional_order!(query, order.direction, order, updated_at),
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
        _: Option<Self::FilterParameter4>,
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

    crud_methods!(reference::table, reference::dsl::reference);
}

publisher_id_impls!(Reference, NewReference, PatchReference, |s, db| {
    crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db)
});

impl HistoryEntry for Reference {
    type NewHistoryEntity = NewReferenceHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            reference_id: self.reference_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewReferenceHistory {
    type MainEntity = ReferenceHistory;

    db_insert!(reference_history::table);
}

impl Reorder for Reference {
    db_change_ordinal!(
        reference::table,
        reference::reference_ordinal,
        "reference_reference_ordinal_work_id_uniq"
    );

    fn get_other_objects(
        &self,
        connection: &mut diesel::PgConnection,
    ) -> ThothResult<Vec<(Uuid, i32)>> {
        reference::table
            .select((reference::reference_id, reference::reference_ordinal))
            .filter(
                reference::work_id
                    .eq(self.work_id)
                    .and(reference::reference_id.ne(self.reference_id)),
            )
            .load::<(Uuid, i32)>(connection)
            .map_err(Into::into)
    }
}
