use super::{
    Language, LanguageCode, LanguageField, LanguageHistory, LanguageRelation, NewLanguage,
    NewLanguageHistory, PatchLanguage,
};
use crate::graphql::model::LanguageOrderBy;
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{language, language_history};
use crate::{crud_methods, db_insert};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
use uuid::Uuid;

impl Crud for Language {
    type NewEntity = NewLanguage;
    type PatchEntity = PatchLanguage;
    type OrderByEntity = LanguageOrderBy;
    type FilterParameter1 = LanguageCode;
    type FilterParameter2 = LanguageRelation;
    type FilterParameter3 = ();

    fn pk(&self) -> Uuid {
        self.language_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        _: Option<String>,
        order: Self::OrderByEntity,
        publishers: Vec<Uuid>,
        parent_id_1: Option<Uuid>,
        _: Option<Uuid>,
        language_codes: Vec<Self::FilterParameter1>,
        language_relations: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<Vec<Language>> {
        use crate::schema::language::dsl;
        let mut connection = db.get()?;
        let mut query = dsl::language
            .inner_join(crate::schema::work::table.inner_join(crate::schema::imprint::table))
            .select(crate::schema::language::all_columns)
            .into_boxed();

        query = match order.field {
            LanguageField::LanguageId => match order.direction {
                Direction::Asc => query.order(dsl::language_id.asc()),
                Direction::Desc => query.order(dsl::language_id.desc()),
            },
            LanguageField::WorkId => match order.direction {
                Direction::Asc => query.order(dsl::work_id.asc()),
                Direction::Desc => query.order(dsl::work_id.desc()),
            },
            LanguageField::LanguageCode => match order.direction {
                Direction::Asc => query.order(dsl::language_code.asc()),
                Direction::Desc => query.order(dsl::language_code.desc()),
            },
            LanguageField::LanguageRelation => match order.direction {
                Direction::Asc => query.order(dsl::language_relation.asc()),
                Direction::Desc => query.order(dsl::language_relation.desc()),
            },
            LanguageField::MainLanguage => match order.direction {
                Direction::Asc => query.order(dsl::main_language.asc()),
                Direction::Desc => query.order(dsl::main_language.desc()),
            },
            LanguageField::CreatedAt => match order.direction {
                Direction::Asc => query.order(dsl::created_at.asc()),
                Direction::Desc => query.order(dsl::created_at.desc()),
            },
            LanguageField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(dsl::updated_at.asc()),
                Direction::Desc => query.order(dsl::updated_at.desc()),
            },
        };
        if !publishers.is_empty() {
            query = query.filter(crate::schema::imprint::publisher_id.eq_any(publishers));
        }
        if let Some(pid) = parent_id_1 {
            query = query.filter(dsl::work_id.eq(pid));
        }
        if !language_codes.is_empty() {
            query = query.filter(dsl::language_code.eq_any(language_codes));
        }
        if !language_relations.is_empty() {
            query = query.filter(dsl::language_relation.eq_any(language_relations));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Language>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        _: Option<String>,
        _: Vec<Uuid>,
        language_codes: Vec<Self::FilterParameter1>,
        language_relations: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
    ) -> ThothResult<i32> {
        use crate::schema::language::dsl;
        let mut connection = db.get()?;
        let mut query = dsl::language.into_boxed();
        if !language_codes.is_empty() {
            query = query.filter(dsl::language_code.eq_any(language_codes));
        }
        if !language_relations.is_empty() {
            query = query.filter(dsl::language_relation.eq_any(language_relations));
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

    crud_methods!(language::table, language::dsl::language);
}

impl HistoryEntry for Language {
    type NewHistoryEntity = NewLanguageHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            language_id: self.language_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewLanguageHistory {
    type MainEntity = LanguageHistory;

    db_insert!(language_history::table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_pk() {
        let language: Language = Default::default();
        assert_eq!(language.pk(), language.language_id);
    }

    #[test]
    fn test_new_language_history_from_language() {
        let language: Language = Default::default();
        let user_id = "123456".to_string();
        let new_language_history = language.new_history_entry(&user_id);
        assert_eq!(new_language_history.language_id, language.language_id);
        assert_eq!(new_language_history.user_id, user_id);
        assert_eq!(
            new_language_history.data,
            serde_json::Value::String(serde_json::to_string(&language).unwrap())
        );
    }
}
