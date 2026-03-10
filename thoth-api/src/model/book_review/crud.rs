use super::{
    BookReview, BookReviewField, BookReviewHistory, BookReviewOrderBy, NewBookReview,
    NewBookReviewHistory, PatchBookReview,
};
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{book_review, book_review_history};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for BookReview {
    type NewEntity = NewBookReview;
    type PatchEntity = PatchBookReview;
    type OrderByEntity = BookReviewOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.book_review_id
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
    ) -> ThothResult<Vec<BookReview>> {
        use crate::schema::book_review::dsl::*;
        let mut connection = db.get()?;
        let mut query = book_review
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::book_review::all_columns)
            .into_boxed();

        query = match order.field {
            BookReviewField::BookReviewId => apply_directional_order!(query, order.direction, order, book_review_id),
            BookReviewField::WorkId => apply_directional_order!(query, order.direction, order, work_id),
            BookReviewField::ReviewOrdinal => apply_directional_order!(query, order.direction, order, review_ordinal),
            BookReviewField::Title => apply_directional_order!(query, order.direction, order, title),
            BookReviewField::AuthorName => apply_directional_order!(query, order.direction, order, author_name),
            BookReviewField::JournalName => apply_directional_order!(query, order.direction, order, journal_name),
            BookReviewField::ReviewDate => apply_directional_order!(query, order.direction, order, review_date),
            BookReviewField::CreatedAt => apply_directional_order!(query, order.direction, order, created_at),
            BookReviewField::UpdatedAt => apply_directional_order!(query, order.direction, order, updated_at),
        };

        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(work_id.eq(pid));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    title
                        .ilike(format!("%{filter}%"))
                        .or(author_name.ilike(format!("%{filter}%")))
                        .or(journal_name.ilike(format!("%{filter}%")))
                        .or(journal_volume.ilike(format!("%{filter}%")))
                        .or(journal_number.ilike(format!("%{filter}%")))
                        .or(journal_issn.ilike(format!("%{filter}%")))
                        .or(text.ilike(format!("%{filter}%")))
                        .or(doi.ilike(format!("%{filter}%")))
                        .or(url.ilike(format!("%{filter}%"))),
                );
            }
        }

        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<BookReview>(&mut connection)
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
        use crate::schema::book_review::dsl::*;
        let mut connection = db.get()?;
        let mut query = book_review
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .into_boxed();

        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(filter) = filter {
            if !filter.is_empty() {
                query = query.filter(
                    title
                        .ilike(format!("%{filter}%"))
                        .or(author_name.ilike(format!("%{filter}%")))
                        .or(journal_name.ilike(format!("%{filter}%")))
                        .or(journal_volume.ilike(format!("%{filter}%")))
                        .or(journal_number.ilike(format!("%{filter}%")))
                        .or(journal_issn.ilike(format!("%{filter}%")))
                        .or(text.ilike(format!("%{filter}%")))
                        .or(doi.ilike(format!("%{filter}%")))
                        .or(url.ilike(format!("%{filter}%"))),
                );
            }
        }

        query
            .count()
            .get_result::<i64>(&mut connection)
            .map(|t| t.to_string().parse::<i32>().unwrap())
            .map_err(Into::into)
    }

    crud_methods!(book_review::table, book_review::dsl::book_review);
}

publisher_id_impls!(BookReview, NewBookReview, PatchBookReview, |s, db| {
    crate::model::work::Work::from_id(db, &s.work_id)?.publisher_id(db)
});

impl HistoryEntry for BookReview {
    type NewHistoryEntity = NewBookReviewHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            book_review_id: self.book_review_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewBookReviewHistory {
    type MainEntity = BookReviewHistory;

    db_insert!(book_review_history::table);
}

impl Reorder for BookReview {
    db_change_ordinal!(
        book_review::table,
        book_review::review_ordinal,
        "book_review_review_ordinal_work_id_uniq"
    );

    fn get_other_objects(
        &self,
        connection: &mut diesel::PgConnection,
    ) -> ThothResult<Vec<(Uuid, i32)>> {
        book_review::table
            .select((book_review::book_review_id, book_review::review_ordinal))
            .filter(
                book_review::work_id
                    .eq(self.work_id)
                    .and(book_review::book_review_id.ne(self.book_review_id)),
            )
            .load::<(Uuid, i32)>(connection)
            .map_err(Into::into)
    }
}
