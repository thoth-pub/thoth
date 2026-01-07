use super::{
    Institution, InstitutionField, InstitutionHistory, InstitutionOrderBy, NewInstitution,
    NewInstitutionHistory, PatchInstitution,
};
use crate::graphql::utils::Direction;
use crate::model::{Crud, DbInsert, HistoryEntry};
use crate::schema::{institution, institution_history};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

impl Crud for Institution {
    type NewEntity = NewInstitution;
    type PatchEntity = PatchInstitution;
    type OrderByEntity = InstitutionOrderBy;
    type FilterParameter1 = ();
    type FilterParameter2 = ();
    type FilterParameter3 = ();
    type FilterParameter4 = ();

    fn pk(&self) -> Uuid {
        self.institution_id
    }

    fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        filter: Option<String>,
        order: Self::OrderByEntity,
        _: Vec<Uuid>,
        _: Option<Uuid>,
        _: Option<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Institution>> {
        use crate::schema::institution::dsl::*;
        let mut connection = db.get()?;
        let mut query = institution.into_boxed();

        query = match order.field {
            InstitutionField::InstitutionId => match order.direction {
                Direction::Asc => query.order(institution_id.asc()),
                Direction::Desc => query.order(institution_id.desc()),
            },
            InstitutionField::InstitutionName => match order.direction {
                Direction::Asc => query.order(institution_name.asc()),
                Direction::Desc => query.order(institution_name.desc()),
            },
            InstitutionField::InstitutionDoi => match order.direction {
                Direction::Asc => query.order(institution_doi.asc()),
                Direction::Desc => query.order(institution_doi.desc()),
            },
            InstitutionField::Ror => match order.direction {
                Direction::Asc => query.order(ror.asc()),
                Direction::Desc => query.order(ror.desc()),
            },
            InstitutionField::CountryCode => match order.direction {
                Direction::Asc => query.order(country_code.asc()),
                Direction::Desc => query.order(country_code.desc()),
            },
            InstitutionField::CreatedAt => match order.direction {
                Direction::Asc => query.order(created_at.asc()),
                Direction::Desc => query.order(created_at.desc()),
            },
            InstitutionField::UpdatedAt => match order.direction {
                Direction::Asc => query.order(updated_at.asc()),
                Direction::Desc => query.order(updated_at.desc()),
            },
        };
        if let Some(filter) = filter {
            query = query.filter(
                institution_name
                    .ilike(format!("%{filter}%"))
                    .or(ror.ilike(format!("%{filter}%")))
                    .or(institution_doi.ilike(format!("%{filter}%"))),
            );
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Institution>(&mut connection)
            .map_err(Into::into)
    }

    fn count(
        db: &crate::db::PgPool,
        filter: Option<String>,
        _: Vec<Uuid>,
        _: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::institution::dsl::*;
        let mut connection = db.get()?;
        let mut query = institution.into_boxed();
        if let Some(filter) = filter {
            query = query.filter(
                institution_name
                    .ilike(format!("%{filter}%"))
                    .or(ror.ilike(format!("%{filter}%")))
                    .or(institution_doi.ilike(format!("%{filter}%"))),
            );
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

    crud_methods!(institution::table, institution::dsl::institution);
}

impl HistoryEntry for Institution {
    type NewHistoryEntity = NewInstitutionHistory;

    fn new_history_entry(&self, user_id: &str) -> Self::NewHistoryEntity {
        Self::NewHistoryEntity {
            institution_id: self.institution_id,
            user_id: user_id.to_string(),
            data: serde_json::Value::String(serde_json::to_string(&self).unwrap()),
        }
    }
}

impl DbInsert for NewInstitutionHistory {
    type MainEntity = InstitutionHistory;

    db_insert!(institution_history::table);
}

impl Institution {
    pub fn linked_publisher_ids(&self, db: &crate::db::PgPool) -> ThothResult<Vec<Uuid>> {
        institution_linked_publisher_ids(self.institution_id, db)
    }
}

fn institution_linked_publisher_ids(
    institution_id: Uuid,
    db: &crate::db::PgPool,
) -> ThothResult<Vec<Uuid>> {
    let mut connection = db.get()?;
    let publishers_via_affiliation = crate::schema::publisher::table
        .inner_join(crate::schema::imprint::table.inner_join(
            crate::schema::work::table.inner_join(
                crate::schema::contribution::table.inner_join(crate::schema::affiliation::table),
            ),
        ))
        .select(crate::schema::publisher::publisher_id)
        .filter(crate::schema::affiliation::institution_id.eq(institution_id))
        .distinct()
        .load::<Uuid>(&mut connection)
        .map_err(|_| ThothError::InternalError("Unable to load records".into()))?;
    let publishers_via_funding = crate::schema::publisher::table
        .inner_join(
            crate::schema::imprint::table
                .inner_join(crate::schema::work::table.inner_join(crate::schema::funding::table)),
        )
        .select(crate::schema::publisher::publisher_id)
        .filter(crate::schema::funding::institution_id.eq(institution_id))
        .distinct()
        .load::<Uuid>(&mut connection)
        .map_err(|_| ThothError::InternalError("Unable to load records".into()))?;
    Ok([publishers_via_affiliation, publishers_via_funding].concat())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_institution_pk() {
        let institution: Institution = Default::default();
        assert_eq!(institution.pk(), institution.institution_id);
    }

    #[test]
    fn test_new_institution_history_from_institution() {
        let institution: Institution = Default::default();
        let user_id = "123456".to_string();
        let new_institution_history = institution.new_history_entry(&user_id);
        assert_eq!(
            new_institution_history.institution_id,
            institution.institution_id
        );
        assert_eq!(new_institution_history.user_id, user_id);
        assert_eq!(
            new_institution_history.data,
            serde_json::Value::String(serde_json::to_string(&institution).unwrap())
        );
    }
}
