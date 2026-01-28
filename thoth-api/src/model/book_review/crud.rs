use super::{
    BookReview, BookReviewField, BookReviewOrderBy, NewBookReview, NewBookReviewHistory,
    PatchBookReview,
};
use crate::graphql::utils::Direction;
use crate::model::work::WorkType;
use crate::model::{Crud, DbInsert, HistoryEntry, Reorder};
use crate::schema::{book_review, book_review::dsl, book_review_history, work};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

fn validate_book_only(db: &crate::db::PgPool, work_id: &Uuid) -> ThothResult<()> {
    let mut connection = db.get()?;
    let work_type = work::table
        .select(work::work_type)
        .filter(work::work_id.eq(work_id))
        .first::<WorkType>(&mut connection)?;

    if work_type == WorkType::BookChapter {
        return Err(ThothError::InternalError(
            "Book reviews can only be attached to book records, not chapters".to_string(),
        ));
    }
    Ok(())
}

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
        use crate::schema::imprint;
        let mut connection = db.get()?;
        let mut query = book_review
            .inner_join(work::table.inner_join(imprint::table))
            .select(crate::schema::book_review::all_columns)
            .into_boxed();

        query = match order.field {
            BookReviewField::BookReviewId => match order.direction {
                Direction::Asc => query.order(book_review_id.asc()),
                Direction::Desc => query.order(book_review_id.desc()),
            },
            BookReviewField::WorkId => match order.direction {
                Direction::Asc => query.order(work_id.asc()),
                Direction::Desc => query.order(work_id.desc()),
            },
            BookReviewField::ReviewOrdinal => match order.direction {
                Direction::Asc => query.order(review_ordinal.asc()),
                Direction::Desc => query.order(review_ordinal.desc()),
            },
            BookReviewField::Title => match order.direction {
                Direction::Asc => query.order(title.asc()),
                Direction::Desc => query.order(title.desc()),
            },
            BookReviewField::AuthorName => match order.direction {
                Direction::Asc => query.order(author_name.asc()),
                Direction::Desc => query.order(author_name.desc()),
            },
            BookReviewField::JournalName => match order.direction {
                Direction::Asc => query.order(journal_name.asc()),
                Direction::Desc => query.order(journal_name.desc()),
            },
            BookReviewField::ReviewDate => match order.direction {
                Direction::Asc => query.order(review_date.asc()),
                Direction::Desc => query.order(review_date.desc()),
            },
            BookReviewField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            BookReviewField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };

        if !publishers.is_empty() {
            query = query.filter(imprint::publisher_id.eq_any(publishers));
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
        use crate::schema::imprint;
        let mut connection = db.get()?;
        let mut query = book_review
            .inner_join(work::table.inner_join(imprint::table))
            .into_boxed();
        if !publishers.is_empty() {
            query = query.filter(imprint::publisher_id.eq_any(publishers));
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

    fn publisher_id(&self, db: &crate::db::PgPool) -> ThothResult<Uuid> {
        crate::model::work::Work::from_id(db, &self.work_id)?.publisher_id(db)
    }

    fn create(db: &crate::db::PgPool, data: &Self::NewEntity) -> ThothResult<Self> {
        validate_book_only(db, &data.work_id)?;
        let mut connection = db.get()?;
        diesel::insert_into(book_review::table)
            .values(data)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    fn update(
        &self,
        db: &crate::db::PgPool,
        data: &Self::PatchEntity,
        account_id: &Uuid,
    ) -> ThothResult<Self> {
        validate_book_only(db, &self.work_id)?;

        let mut connection = db.get()?;
        connection.transaction(|connection| {
            diesel::update(dsl::book_review.find(&self.pk()))
                .set(data)
                .get_result(connection)
                .map_err(Into::into)
                .and_then(|c| {
                    self.new_history_entry(account_id)
                        .insert(connection)
                        .map(|_| c)
                })
        })
    }

    fn from_id(db: &crate::db::PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        use diesel::{QueryDsl, RunQueryDsl};
        let mut connection = db.get()?;
        dsl::book_review
            .find(entity_id)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    fn delete(self, db: &crate::db::PgPool) -> ThothResult<Self> {
        use diesel::{QueryDsl, RunQueryDsl};
        let mut connection = db.get()?;
        diesel::delete(dsl::book_review.find(&self.pk()))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(Into::into)
    }
}

impl Reorder for BookReview {
    fn change_ordinal(
        &self,
        db: &crate::db::PgPool,
        current_ordinal: i32,
        new_ordinal: i32,
        _account_id: &Uuid,
    ) -> ThothResult<Self> {
        let mut connection = db.get()?;
        connection.transaction(|connection| {
            if current_ordinal == new_ordinal {
                return Ok(self.clone());
            }

            let mut other_objects = self.get_other_objects(connection)?;
            other_objects.sort_by_key(|(_, ordinal)| *ordinal);

            diesel::sql_query("SET CONSTRAINTS idx_book_review_workid_ordinal DEFERRED")
                .execute(connection)?;
            for (id, ordinal) in other_objects {
                if new_ordinal > current_ordinal {
                    if ordinal > current_ordinal && ordinal <= new_ordinal {
                        let updated_ordinal = ordinal - 1;
                        diesel::update(book_review::table.find(id))
                            .set(book_review::review_ordinal.eq(&updated_ordinal))
                            .execute(connection)?;
                    }
                } else if ordinal >= new_ordinal && ordinal < current_ordinal {
                    let updated_ordinal = ordinal + 1;
                    diesel::update(book_review::table.find(id))
                        .set(book_review::review_ordinal.eq(&updated_ordinal))
                        .execute(connection)?;
                }
            }
            diesel::update(book_review::table.find(self.book_review_id))
                .set(book_review::review_ordinal.eq(&new_ordinal))
                .get_result::<Self>(connection)
                .map_err(Into::into)
        })
    }

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

impl HistoryEntry for BookReview {
    type NewHistoryEntity = NewBookReviewHistory;

    fn new_history_entry(&self, account_id: &Uuid) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            book_review_id: self.book_review_id,
            account_id: *account_id,
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewBookReviewHistory {
    type MainEntity = super::BookReviewHistory;

    db_insert!(book_review_history::table);
}
