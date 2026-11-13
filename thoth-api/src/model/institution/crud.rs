use super::{
    Institution, InstitutionChangeset, InstitutionField, InstitutionHistory, InstitutionOrderBy,
    InstitutionRorStatePatch, NewInstitution, NewInstitutionHistory, PatchInstitution, RorStatus,
};
use crate::db::PgPool;
use crate::model::{Crud, DbInsert, HistoryEntry, PublisherIds};
use crate::policy::{PolicyContext, UserAccess};
use crate::schema::{institution, institution_history};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

/// The message of the error `Crud::create` returns for `Institution` outside
/// thoth-api's own unit tests.
pub(crate) const CREATE_WITHOUT_HISTORY_ERROR: &str =
    "Institutions are created only through `Institution::create_with_history`, which records the created row in `institution_history`";

/// The lock taken before the row is read for update.
///
/// `ROW EXCLUSIVE` is the lock the `UPDATE` takes anyway, so only the order
/// changes: while a migration holds `SHARE ROW EXCLUSIVE` on `institution`,
/// an update waits here holding no row lock, and cannot deadlock with it.
pub(crate) const UPDATE_TABLE_LOCK_SQL: &str = "LOCK TABLE institution IN ROW EXCLUSIVE MODE";

/// Whether the caller may write `ror_status` and `ror_successor`.
///
/// The predicate is this stage's matrix decision (ROR-SYNC-01C, thoth#1024):
/// `SUPERUSER` or `ROR_SYNC_SERVICE`. It does not make `SUPERUSER` hold the
/// machine role: `is_ror_sync_service()` stays false for a superuser without it.
pub(crate) fn may_write_ror_state<C: PolicyContext>(ctx: &C) -> bool {
    ctx.user()
        .is_some_and(|user| user.is_superuser() || user.is_ror_sync_service())
}

impl InstitutionRorStatePatch {
    /// Build the status and successor changeset for one update.
    ///
    /// - `privileged` is [`may_write_ror_state`] for the caller. An
    ///   unprivileged caller's values are never copied: `InstitutionPolicy`
    ///   has already denied a differing value, and an equal one is dropped, so
    ///   a concurrent privileged write is not reverted.
    /// - `ror_changes` compares the patch's `ror` with the row re-read under
    ///   lock. When it changes, each field the caller did not supply, or was
    ///   not allowed to supply, is reset: the status to `ACTIVE`, the successor
    ///   to NULL.
    pub(crate) fn for_update(
        patch: &PatchInstitution,
        privileged: bool,
        ror_changes: bool,
    ) -> Self {
        let (status, successor) = if privileged {
            let successor = match &patch.ror_successor {
                juniper::Nullable::ImplicitNull => None,
                juniper::Nullable::ExplicitNull => Some(None),
                juniper::Nullable::Some(successor) => Some(Some(successor.clone())),
            };
            (patch.ror_status, successor)
        } else {
            (None, None)
        };

        if ror_changes {
            Self {
                ror_status: status.or(Some(RorStatus::Active)),
                ror_successor: successor.or(Some(None)),
            }
        } else {
            Self {
                ror_status: status,
                ror_successor: successor,
            }
        }
    }
}

impl Institution {
    /// Create an institution and record the created row in
    /// `institution_history`, under the caller's user ID, in one transaction.
    ///
    /// This is the only way to create an institution outside thoth-api's own
    /// unit tests: `Crud::create` for `Institution` writes no history and
    /// returns an error there.
    pub(crate) fn create_with_history<C: PolicyContext>(
        ctx: &C,
        data: &NewInstitution,
    ) -> ThothResult<Self> {
        let user_id = ctx.user_id()?.to_string();
        let mut connection = ctx.db().get()?;
        connection.transaction(|connection| {
            let created = diesel::insert_into(institution::table)
                .values(data)
                .get_result::<Institution>(connection)?;
            created.new_history_entry(&user_id).insert(connection)?;
            Ok(created)
        })
    }
}

/// Call `Crud::create` for `Institution`.
///
/// This exists only so that thoth-api's integration tests, which build the
/// library without `cfg(test)`, can prove that `Crud::create` refuses there
/// (the trait is crate-private). Outside thoth-api's unit tests it always
/// returns the error and never writes.
#[doc(hidden)]
pub fn crud_create_without_history(db: &PgPool, data: &NewInstitution) -> ThothResult<Institution> {
    <Institution as Crud>::create(db, data)
}

impl Crud for Institution {
    type NewEntity = NewInstitution;
    type PatchEntity = PatchInstitution;
    type OrderByEntity = InstitutionOrderBy;
    type FilterParameter1 = RorStatus;
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
        ror_statuses: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<Vec<Institution>> {
        use crate::schema::institution::dsl::*;
        let mut connection = db.get()?;
        let mut query = institution.into_boxed();

        query = match order.field {
            InstitutionField::InstitutionId => {
                apply_directional_order!(query, order.direction, order, institution_id)
            }
            InstitutionField::InstitutionName => {
                apply_directional_order!(query, order.direction, order, institution_name)
            }
            InstitutionField::InstitutionDoi => {
                apply_directional_order!(query, order.direction, order, institution_doi)
            }
            InstitutionField::Ror => apply_directional_order!(query, order.direction, order, ror),
            InstitutionField::CountryCode => {
                apply_directional_order!(query, order.direction, order, country_code)
            }
            InstitutionField::CreatedAt => {
                apply_directional_order!(query, order.direction, order, created_at)
            }
            InstitutionField::UpdatedAt => {
                apply_directional_order!(query, order.direction, order, updated_at)
            }
            InstitutionField::RorStatus => {
                apply_directional_order!(query, order.direction, order, ror_status, institution_id)
            }
        };
        if !ror_statuses.is_empty() {
            query = query.filter(ror_status.eq_any(ror_statuses));
        }
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
        ror_statuses: Vec<Self::FilterParameter1>,
        _: Vec<Self::FilterParameter2>,
        _: Option<Self::FilterParameter3>,
        _: Option<Self::FilterParameter4>,
    ) -> ThothResult<i32> {
        use crate::schema::institution::dsl::*;
        let mut connection = db.get()?;
        let mut query = institution.into_boxed();
        if !ror_statuses.is_empty() {
            query = query.filter(ror_status.eq_any(ror_statuses));
        }
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

    // `crud_methods!` is not used: it emits these four methods together, and
    // `create` and `update` differ here (ROR-SYNC-01C, thoth#1024).

    fn from_id(db: &PgPool, entity_id: &Uuid) -> ThothResult<Self> {
        let mut connection = db.get()?;
        institution::table
            .find(entity_id)
            .get_result::<Self>(&mut connection)
            .map_err(Into::into)
    }

    /// Writes no history. Outside thoth-api's own unit tests it returns an
    /// internal error naming [`Institution::create_with_history`], so that no
    /// caller, direct or through the trait, creates an institution without a
    /// history row. Unit-test fixtures may still use it.
    fn create(db: &PgPool, data: &NewInstitution) -> ThothResult<Self> {
        #[cfg(not(test))]
        {
            let _ = (db, data);
            Err(ThothError::InternalError(
                CREATE_WITHOUT_HISTORY_ERROR.to_string(),
            ))
        }
        #[cfg(test)]
        {
            let mut connection = db.get()?;
            diesel::insert_into(institution::table)
                .values(data)
                .get_result::<Self>(&mut connection)
                .map_err(Into::into)
        }
    }

    /// Update the institution in one transaction:
    ///
    /// 1. take `ROW EXCLUSIVE` on `institution` (see [`UPDATE_TABLE_LOCK_SQL`]),
    ///    then lock the row `FOR NO KEY UPDATE` and re-read it, so that new
    ///    affiliations and fundings referencing it are not blocked;
    /// 2. record the re-read row in `institution_history` under the caller's
    ///    user ID;
    /// 3. build [`InstitutionRorStatePatch`] from the caller's privilege and
    ///    from whether `ror` changes against the re-read row;
    /// 4. write both changesets in one `UPDATE`, because each CHECK is
    ///    evaluated on every row version written: two statements would write an
    ///    intermediate row that violates `institution_ror_state_check` when
    ///    `ror` is cleared on a non-active row.
    fn update<C: PolicyContext>(&self, ctx: &C, data: &PatchInstitution) -> ThothResult<Self> {
        let user_id = ctx.user_id()?.to_string();
        let privileged = may_write_ror_state(ctx);
        let changeset = InstitutionChangeset::from(data);
        let mut connection = ctx.db().get()?;
        connection.transaction(|connection| {
            diesel::sql_query(UPDATE_TABLE_LOCK_SQL).execute(connection)?;
            let current = institution::table
                .find(self.pk())
                .for_no_key_update()
                .get_result::<Institution>(connection)?;
            current.new_history_entry(&user_id).insert(connection)?;
            let state =
                InstitutionRorStatePatch::for_update(data, privileged, data.ror != current.ror);
            diesel::update(institution::table.find(self.pk()))
                .set((&changeset, &state))
                .get_result::<Institution>(connection)
                .map_err(Into::into)
        })
    }

    fn delete(self, db: &PgPool) -> ThothResult<Self> {
        let mut connection = db.get()?;
        diesel::delete(institution::table.find(self.pk()))
            .execute(&mut connection)
            .map(|_| self)
            .map_err(Into::into)
    }
}

impl PublisherIds for Institution {
    fn publisher_ids(&self, db: &PgPool) -> ThothResult<Vec<Uuid>> {
        let mut connection = db.get()?;
        let publishers_via_affiliation = crate::schema::publisher::table
            .inner_join(
                crate::schema::imprint::table.inner_join(
                    crate::schema::work::table.inner_join(
                        crate::schema::contribution::table
                            .inner_join(crate::schema::affiliation::table),
                    ),
                ),
            )
            .select(crate::schema::publisher::publisher_id)
            .filter(crate::schema::affiliation::institution_id.eq(self.institution_id))
            .distinct()
            .load::<Uuid>(&mut connection)
            .map_err(|_| ThothError::InternalError("Unable to load records".into()))?;
        let publishers_via_funding =
            crate::schema::publisher::table
                .inner_join(crate::schema::imprint::table.inner_join(
                    crate::schema::work::table.inner_join(crate::schema::funding::table),
                ))
                .select(crate::schema::publisher::publisher_id)
                .filter(crate::schema::funding::institution_id.eq(self.institution_id))
                .distinct()
                .load::<Uuid>(&mut connection)
                .map_err(|_| ThothError::InternalError("Unable to load records".into()))?;
        Ok([publishers_via_affiliation, publishers_via_funding].concat())
    }
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
