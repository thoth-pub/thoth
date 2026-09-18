use super::*;

#[cfg(feature = "backend")]
use crate::db::PgPool;

#[cfg(feature = "backend")]
pub(crate) mod db {
    use std::collections::HashMap;
    use std::env;
    use std::fs::OpenOptions;
    use std::sync::{Arc, OnceLock};
    use std::time::Duration;

    use diesel::pg::PgConnection;
    use diesel::r2d2::ConnectionManager;
    use diesel::RunQueryDsl;
    use fs2::FileExt;
    use uuid::Uuid;
    use zitadel::actix::introspection::IntrospectedUser;

    use crate::db::{init_pool, run_migrations, PgPool};
    use crate::graphql::Context;
    use crate::model::contribution::{Contribution, ContributionType, NewContribution};
    use crate::model::contributor::{Contributor, NewContributor};
    use crate::model::distribution_job::DistributionJobCreation;
    use crate::model::imprint::{Imprint, NewImprint};
    use crate::model::institution::{Institution, NewInstitution};
    use crate::model::publication::{NewPublication, Publication, PublicationType};
    use crate::model::publisher::{NewPublisher, Publisher};
    use crate::model::series::{NewSeries, Series, SeriesType};
    use crate::model::work::{NewWork, Work, WorkStatus, WorkType};
    use crate::model::{CountryCode, Crud};
    use crate::policy::Role;
    use crate::storage::{create_cloudfront_client, create_s3_client, CloudFrontClient, S3Client};

    static MIGRATIONS: OnceLock<Result<(), String>> = OnceLock::new();
    static POOL: OnceLock<Arc<PgPool>> = OnceLock::new();
    static CLIENTS: OnceLock<(Arc<S3Client>, Arc<CloudFrontClient>)> = OnceLock::new();

    pub(crate) struct TestDbGuard {
        _file: std::fs::File,
    }

    pub(crate) fn test_lock() -> TestDbGuard {
        let mut path = env::temp_dir();
        path.push("thoth_test_db.lock");
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .unwrap_or_else(|err| panic!("Failed to open lock file {path:?}: {err}"));
        file.lock_exclusive()
            .unwrap_or_else(|err| panic!("Failed to lock test DB file {path:?}: {err}"));
        TestDbGuard { _file: file }
    }

    pub(crate) fn test_db_url() -> String {
        dotenv::dotenv().ok();
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for backend tests")
    }

    pub(crate) fn db_pool() -> Arc<PgPool> {
        let url = test_db_url();
        let migrations = MIGRATIONS
            .get_or_init(|| run_migrations(&url).map_err(|err| err.to_string()))
            .clone();
        migrations.expect("Failed to run migrations for test DB");
        let pool = POOL.get_or_init(|| Arc::new(init_pool(&url)));
        pool.clone()
    }

    pub(crate) fn failing_pool() -> PgPool {
        let manager = ConnectionManager::<PgConnection>::new(
            "postgres://invalid:invalid@localhost:1/invalid",
        );
        diesel::r2d2::Pool::builder()
            .max_size(1)
            .connection_timeout(Duration::from_millis(100))
            .build_unchecked(manager)
    }

    fn test_clients() -> (Arc<S3Client>, Arc<CloudFrontClient>) {
        let (s3_client, cloudfront_client) = CLIENTS.get_or_init(|| {
            std::thread::spawn(|| {
                let runtime =
                    tokio::runtime::Runtime::new().expect("Failed to build Tokio runtime");
                runtime.block_on(async {
                    let s3 =
                        create_s3_client("test-access-key", "test-secret-key", "us-east-1").await;
                    let cloudfront =
                        create_cloudfront_client("test-access-key", "test-secret-key", "us-east-1")
                            .await;
                    (Arc::new(s3), Arc::new(cloudfront))
                })
            })
            .join()
            .expect("Failed to initialize AWS clients")
        });
        (Arc::clone(s3_client), Arc::clone(cloudfront_client))
    }

    /// The one test-harness reset statement (BE-06 Amendment 3 section 5.2).
    ///
    /// It verifies the 15-trigger permanence manifest before any bypass, truncates
    /// every public table under a transaction-local `replica` mode, restores
    /// `origin`, reseeds the two BE-06 permanent rows and verifies that no
    /// protection was left disabled. `thoth-api/tests/support/mod.rs` carries a
    /// byte-identical copy, asserted by test H6.
    pub(crate) const TEST_RESET_SQL: &str = r#"
DO $$
DECLARE
    tbls TEXT;
BEGIN
    IF current_setting('session_replication_role') <> 'origin' THEN
        RAISE EXCEPTION 'BE06_TEST_RESET_PROTECTIONS_NOT_RESTORED';
    END IF;

    IF (SELECT count(*)
          FROM pg_trigger t
          JOIN pg_class c ON c.oid = t.tgrelid
          JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'public' AND NOT t.tgisinternal AND t.tgenabled = 'O'
           AND (c.relname, t.tgname) IN (
               ('crossref_write_permit', 'crossref_write_permit_insert_guard'),
               ('crossref_write_permit', 'crossref_write_permit_fsm'),
               ('crossref_write_permit', 'crossref_write_permit_no_truncate'),
               ('crossref_write_permit', 'crossref_permit_membership_agreement_p'),
               ('crossref_write_permit_doi', 'crossref_write_permit_doi_immutable'),
               ('crossref_write_permit_doi', 'crossref_write_permit_doi_no_truncate'),
               ('crossref_write_permit_doi', 'crossref_permit_membership_agreement_d'),
               ('work_crossref_version_floor', 'work_crossref_version_floor_guard'),
               ('work_crossref_version_floor', 'work_crossref_version_floor_no_truncate'),
               ('crossref_version_floor_audit', 'crossref_version_floor_audit_append_only'),
               ('crossref_version_floor_audit', 'crossref_version_floor_audit_no_truncate'),
               ('work_upsert_control', 'work_upsert_control_guard'),
               ('work_upsert_control', 'work_upsert_control_no_truncate'),
               ('work_upsert_admission', 'work_upsert_admission_guard'),
               ('work_upsert_admission', 'work_upsert_admission_no_truncate'))) <> 15 THEN
        RAISE EXCEPTION 'BE06_TEST_RESET_PROTECTIONS_NOT_RESTORED';
    END IF;

    SELECT string_agg(format('%I.%I', schemaname, tablename), ', ')
    INTO tbls
    FROM pg_tables
    WHERE schemaname = 'public'
      AND tablename != '__diesel_schema_migrations';

    IF tbls IS NOT NULL THEN
        PERFORM set_config('session_replication_role', 'replica', true);
        EXECUTE 'TRUNCATE TABLE ' || tbls || ' RESTART IDENTITY CASCADE';
        PERFORM set_config('session_replication_role', 'origin', true);
    END IF;

    INSERT INTO public.work_upsert_control (execution_profile, capture_enabled, execution_enabled)
    VALUES ('CROSSREF', false, false);
    INSERT INTO public.work_crossref_version_floor (floor_id, floor_value)
    VALUES (true, 0);

    IF current_setting('session_replication_role') <> 'origin'
       OR EXISTS (SELECT 1
                    FROM pg_trigger t
                    JOIN pg_class c ON c.oid = t.tgrelid
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                   WHERE n.nspname = 'public' AND NOT t.tgisinternal AND t.tgenabled <> 'O') THEN
        RAISE EXCEPTION 'BE06_TEST_RESET_PROTECTIONS_NOT_RESTORED';
    END IF;
END $$;
"#;

    pub(crate) fn reset_db(pool: &PgPool) -> Result<(), diesel::result::Error> {
        let mut connection = pool.get().expect("Failed to get DB connection");
        diesel::sql_query(TEST_RESET_SQL)
            .execute(&mut connection)
            .map(|_| ())
    }

    pub(crate) fn setup_test_db() -> (TestDbGuard, Arc<PgPool>) {
        let guard = test_lock();
        let pool = db_pool();
        reset_db(&pool).expect("Failed to reset DB");
        (guard, pool)
    }

    fn test_user(user_id: &str) -> IntrospectedUser {
        IntrospectedUser {
            user_id: user_id.to_string(),
            username: None,
            name: None,
            given_name: None,
            family_name: None,
            preferred_username: None,
            email: None,
            email_verified: None,
            locale: None,
            project_roles: None,
            metadata: None,
        }
    }

    pub(crate) fn test_context(pool: Arc<PgPool>, user_id: &str) -> Context {
        let (s3_client, cloudfront_client) = test_clients();
        Context::new(
            pool,
            Some(test_user(user_id)),
            s3_client,
            cloudfront_client,
            DistributionJobCreation::default(),
        )
    }

    pub(crate) fn test_user_with_role(user_id: &str, role: Role, org_id: &str) -> IntrospectedUser {
        let mut scoped = HashMap::new();
        scoped.insert(org_id.to_string(), "role".to_string());
        let mut project_roles = HashMap::new();
        project_roles.insert(role.as_ref().to_string(), scoped);

        IntrospectedUser {
            user_id: user_id.to_string(),
            username: None,
            name: None,
            given_name: None,
            family_name: None,
            preferred_username: None,
            email: None,
            email_verified: None,
            locale: None,
            project_roles: Some(project_roles),
            metadata: None,
        }
    }

    pub(crate) fn test_superuser(user_id: &str) -> IntrospectedUser {
        let mut project_roles = HashMap::new();
        project_roles.insert(Role::Superuser.as_ref().to_string(), HashMap::new());

        IntrospectedUser {
            user_id: user_id.to_string(),
            username: None,
            name: None,
            given_name: None,
            family_name: None,
            preferred_username: None,
            email: None,
            email_verified: None,
            locale: None,
            project_roles: Some(project_roles),
            metadata: None,
        }
    }

    pub(crate) fn test_context_with_user(pool: Arc<PgPool>, user: IntrospectedUser) -> Context {
        test_context_with_job_creation(pool, Some(user), DistributionJobCreation::default())
    }

    pub(crate) fn test_context_anonymous(pool: Arc<PgPool>) -> Context {
        test_context_with_job_creation(pool, None, DistributionJobCreation::default())
    }

    /// A context whose automatic distribution-job creation setting is chosen by
    /// the test.
    ///
    /// This is how `BE-04`'s switch evidence is produced: the value travels
    /// through the request context and the write context exactly as it does in
    /// production, so an `OFF` case and an `ON` case differ in exactly one
    /// value and **no environment variable is mutated by any test**.
    pub(crate) fn test_context_with_job_creation(
        pool: Arc<PgPool>,
        user: Option<IntrospectedUser>,
        job_creation: DistributionJobCreation,
    ) -> Context {
        let (s3_client, cloudfront_client) = test_clients();
        Context::new(pool, user, s3_client, cloudfront_client, job_creation)
    }

    pub(crate) fn create_publisher(pool: &PgPool) -> Publisher {
        let org_id = format!("org-{}", Uuid::new_v4());
        let new_publisher = NewPublisher {
            publisher_name: format!("DB Publisher {}", Uuid::new_v4()),
            publisher_shortname: None,
            publisher_url: None,
            zitadel_id: Some(org_id),
            accessibility_statement: None,
            accessibility_report_url: None,
        };

        Publisher::create(pool, &new_publisher).expect("Failed to create publisher in DB")
    }

    pub(crate) fn create_imprint(pool: &PgPool, publisher: &Publisher) -> Imprint {
        let new_imprint = NewImprint {
            publisher_id: publisher.publisher_id,
            imprint_name: format!("DB Imprint {}", Uuid::new_v4()),
            imprint_url: None,
            crossmark_doi: None,
            s3_bucket: None,
            cdn_domain: None,
            cloudfront_dist_id: None,
            default_currency: None,
            default_place: None,
            default_locale: None,
        };

        Imprint::create(pool, &new_imprint).expect("Failed to create imprint in DB")
    }

    pub(crate) fn create_contributor(pool: &PgPool) -> Contributor {
        let suffix = Uuid::new_v4();
        let new_contributor = NewContributor {
            first_name: Some("Test".to_string()),
            last_name: format!("Contributor {suffix}"),
            full_name: format!("Test Contributor {suffix}"),
            orcid: None,
            website: None,
        };

        Contributor::create(pool, &new_contributor).expect("Failed to create contributor in DB")
    }

    pub(crate) fn create_institution(pool: &PgPool) -> Institution {
        let new_institution = NewInstitution {
            institution_name: format!("Institution {}", Uuid::new_v4()),
            institution_doi: None,
            ror: None,
            country_code: Some(CountryCode::Gbr),
        };

        Institution::create(pool, &new_institution).expect("Failed to create institution in DB")
    }

    pub(crate) fn create_series(pool: &PgPool, imprint: &Imprint) -> Series {
        let new_series = NewSeries {
            series_type: SeriesType::Journal,
            series_name: format!("Series {}", Uuid::new_v4()),
            issn_print: None,
            issn_digital: None,
            series_url: None,
            series_description: None,
            series_cfp_url: None,
            imprint_id: imprint.imprint_id,
        };

        Series::create(pool, &new_series).expect("Failed to create series in DB")
    }

    pub(crate) fn create_work(pool: &PgPool, imprint: &Imprint) -> Work {
        let new_work = NewWork {
            work_type: WorkType::Monograph,
            work_status: WorkStatus::Forthcoming,
            reference: None,
            edition: Some(1),
            imprint_id: imprint.imprint_id,
            doi: None,
            publication_date: None,
            withdrawn_date: None,
            place: None,
            page_count: None,
            page_breakdown: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            license: None,
            copyright_holder: None,
            landing_page: None,
            lccn: None,
            oclc: None,
            general_note: None,
            bibliography_note: None,
            toc: None,
            resources_description: None,
            cover_url: None,
            cover_caption: None,
            first_page: None,
            last_page: None,
            page_interval: None,
        };

        Work::create(pool, &new_work).expect("Failed to create work in DB")
    }

    pub(crate) fn create_contribution(
        pool: &PgPool,
        work: &Work,
        contributor: &Contributor,
    ) -> Contribution {
        let new_contribution = NewContribution {
            work_id: work.work_id,
            contributor_id: contributor.contributor_id,
            contribution_type: ContributionType::Author,
            main_contribution: true,
            first_name: contributor.first_name.clone(),
            last_name: contributor.last_name.clone(),
            full_name: contributor.full_name.clone(),
            contribution_ordinal: 1,
        };

        Contribution::create(pool, &new_contribution).expect("Failed to create contribution in DB")
    }

    pub(crate) fn create_publication(pool: &PgPool, work: &Work) -> Publication {
        let new_publication = NewPublication {
            publication_type: PublicationType::Paperback,
            work_id: work.work_id,
            isbn: None,
            width_mm: None,
            width_in: None,
            height_mm: None,
            height_in: None,
            depth_mm: None,
            depth_in: None,
            weight_g: None,
            weight_oz: None,
            accessibility_standard: None,
            accessibility_additional_standard: None,
            accessibility_exception: None,
            accessibility_report_url: None,
        };

        Publication::create(pool, &new_publication).expect("Failed to create publication in DB")
    }
}

#[cfg(feature = "backend")]
pub(crate) fn assert_graphql_enum_roundtrip<E>(value: E)
where
    E: juniper::FromInputValue<juniper::DefaultScalarValue>
        + juniper::ToInputValue<juniper::DefaultScalarValue>
        + juniper::GraphQLType<juniper::DefaultScalarValue>
        + juniper::GraphQLValue<juniper::DefaultScalarValue, Context = (), TypeInfo = ()>
        + PartialEq
        + std::fmt::Debug
        + Clone,
    <E as juniper::FromInputValue<juniper::DefaultScalarValue>>::Error: std::fmt::Debug,
{
    let _ = <E as juniper::GraphQLType<juniper::DefaultScalarValue>>::name(&());
    let mut registry = juniper::Registry::new(Default::default());
    let _ = <E as juniper::GraphQLType<juniper::DefaultScalarValue>>::meta(&(), &mut registry);
    let _ = <E as juniper::GraphQLValue<juniper::DefaultScalarValue>>::type_name(&value, &());

    let input = value.to_input_value();
    let parsed = E::from_input_value(&input).expect("GraphQL enum should parse");
    assert_eq!(parsed, value);
}

#[cfg(feature = "backend")]
pub(crate) fn assert_db_enum_to_sql<E, ST>(pool: &PgPool, value: &E)
where
    E: diesel::serialize::ToSql<ST, diesel::pg::Pg>
        + diesel::serialize::ToSql<diesel::sql_types::Nullable<ST>, diesel::pg::Pg>
        + std::fmt::Debug,
    ST: diesel::sql_types::SingleValue + diesel::sql_types::SqlType,
    diesel::pg::Pg: diesel::sql_types::HasSqlType<ST>
        + diesel::sql_types::HasSqlType<diesel::sql_types::Nullable<ST>>,
{
    use diesel::pg::PgMetadataLookup;
    use diesel::query_builder::bind_collector::RawBytesBindCollector;
    use diesel::query_builder::BindCollector;

    let mut connection = pool.get().expect("Failed to get DB connection");
    let mut collector = RawBytesBindCollector::<diesel::pg::Pg>::new();
    let metadata_lookup: &mut dyn PgMetadataLookup = &mut *connection;
    collector
        .push_bound_value::<ST, _>(value, metadata_lookup)
        .expect("Failed to serialize DB enum");
    collector
        .push_bound_value::<diesel::sql_types::Nullable<ST>, _>(value, metadata_lookup)
        .expect("Failed to serialize DB enum (nullable)");
}

#[cfg(feature = "backend")]
pub(crate) fn assert_db_enum_as_expression<E, ST>(value: E)
where
    E: diesel::expression::AsExpression<ST>
        + diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>
        + Copy,
    for<'a> &'a E: diesel::expression::AsExpression<ST>
        + diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>,
    for<'a> &'a &'a E: diesel::expression::AsExpression<ST>
        + diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>,
    ST: diesel::sql_types::SqlType
        + diesel::expression::TypedExpressionType
        + diesel::sql_types::SingleValue,
{
    let _ = <E as diesel::expression::AsExpression<ST>>::as_expression(value);
    let _ = <E as diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>>::as_expression(
        value,
    );
    let value_ref = &value;
    let _ = <&E as diesel::expression::AsExpression<ST>>::as_expression(value_ref);
    let _ =
        <&E as diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>>::as_expression(
            value_ref,
        );
    let value_ref_ref = &value_ref;
    let _ = <&&E as diesel::expression::AsExpression<ST>>::as_expression(value_ref_ref);
    let _ =
        <&&E as diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>>::as_expression(
            value_ref_ref,
        );
}

#[cfg(feature = "backend")]
pub(crate) fn assert_db_enum_queryable<E, ST>(value: E)
where
    E: diesel::Queryable<ST, diesel::pg::Pg, Row = E> + Copy,
{
    let _ = <E as diesel::Queryable<ST, diesel::pg::Pg>>::build(value)
        .expect("Failed to build DB enum via Queryable");
}

#[cfg(feature = "backend")]
pub(crate) fn assert_db_enum_roundtrip<E, ST>(pool: &PgPool, literal: &str, expected: E)
where
    E: diesel::deserialize::FromSqlRow<ST, diesel::pg::Pg>
        + diesel::serialize::ToSql<ST, diesel::pg::Pg>
        + diesel::serialize::ToSql<diesel::sql_types::Nullable<ST>, diesel::pg::Pg>
        + diesel::expression::AsExpression<ST>
        + diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>
        + diesel::Queryable<ST, diesel::pg::Pg, Row = E>
        + Copy
        + PartialEq
        + std::fmt::Debug
        + 'static,
    for<'a> &'a E: diesel::expression::AsExpression<ST>
        + diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>,
    for<'a> &'a &'a E: diesel::expression::AsExpression<ST>
        + diesel::expression::AsExpression<diesel::sql_types::Nullable<ST>>,
    ST: diesel::sql_types::SingleValue
        + diesel::sql_types::SqlType
        + diesel::expression::TypedExpressionType,
    diesel::pg::Pg: diesel::sql_types::HasSqlType<ST>,
{
    use diesel::dsl::sql;
    use diesel::prelude::*;

    assert_db_enum_as_expression::<E, ST>(expected);
    assert_db_enum_queryable::<E, ST>(expected);
    assert_db_enum_to_sql::<E, ST>(pool, &expected);

    let mut connection = pool.get().expect("Failed to get DB connection");
    let fetched: E = diesel::select(sql::<ST>(literal))
        .get_result(&mut connection)
        .expect("Failed to roundtrip DB enum");

    assert_eq!(fetched, expected);
}

mod publisher_ids {
    use crate::model::tests::db::{create_imprint, create_publisher, create_work, setup_test_db};
    use crate::model::work_relation::{NewWorkRelation, RelationType, WorkRelation};
    use crate::model::{Crud, PublisherId, PublisherIds};

    #[test]
    fn publisher_id_zitadel_id_resolves_from_related_publisher() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let zitadel_id = work
            .zitadel_id(pool.as_ref())
            .expect("Failed to resolve publisher zitadel id");
        assert_eq!(zitadel_id, publisher.zitadel_id.clone().unwrap());
    }

    #[test]
    fn publisher_ids_zitadel_ids_returns_sorted_unique_ids() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        let relator = create_work(pool.as_ref(), &imprint);
        let related = create_work(pool.as_ref(), &other_imprint);

        let new_relation = NewWorkRelation {
            relator_work_id: relator.work_id,
            related_work_id: related.work_id,
            relation_type: RelationType::HasPart,
            relation_ordinal: 1,
        };
        let relation =
            WorkRelation::create(pool.as_ref(), &new_relation).expect("Failed to create relation");

        let mut expected = vec![
            publisher.zitadel_id.clone().unwrap(),
            other_publisher.zitadel_id.clone().unwrap(),
        ];
        expected.sort();

        let ids = relation
            .zitadel_ids(pool.as_ref())
            .expect("Failed to resolve publisher zitadel ids");
        assert_eq!(ids, expected);
    }
}

#[cfg(feature = "backend")]
mod db_errors {
    use crate::model::publisher::Publisher;
    use crate::model::tests::db::failing_pool;
    use crate::model::Crud;
    use uuid::Uuid;

    #[test]
    fn failing_pool_returns_error() {
        let pool = failing_pool();
        let result = Publisher::from_id(&pool, &Uuid::new_v4());
        assert!(result.is_err());
    }
}

#[test]
fn test_doi_default() {
    let doi: Doi = Default::default();
    assert_eq!(doi, Doi("".to_string()));
}

#[test]
fn test_isbn_default() {
    let isbn: Isbn = Default::default();
    assert_eq!(isbn, Isbn("".to_string()));
}

#[test]
fn test_orcid_default() {
    let orcid: Orcid = Default::default();
    assert_eq!(orcid, Orcid("".to_string()));
}

#[test]
fn test_ror_default() {
    let ror: Ror = Default::default();
    assert_eq!(ror, Ror("".to_string()));
}

#[test]
fn test_timestamp_default() {
    let stamp: Timestamp = Default::default();
    assert_eq!(
        stamp,
        Timestamp(TimeZone::timestamp_opt(&Utc, 0, 0).unwrap())
    );
}

#[test]
fn test_doi_display() {
    let doi = Doi("https://doi.org/10.12345/Test-Suffix.01".to_string());
    assert_eq!(format!("{doi}"), "10.12345/Test-Suffix.01");
}

#[test]
fn test_isbn_display() {
    let isbn = Isbn("978-3-16-148410-0".to_string());
    assert_eq!(format!("{isbn}"), "978-3-16-148410-0");
}

#[test]
fn test_orcid_display() {
    let orcid = Orcid("https://orcid.org/0000-0002-1234-5678".to_string());
    assert_eq!(format!("{orcid}"), "0000-0002-1234-5678");
}

#[test]
fn test_ror_display() {
    let ror = Ror("https://ror.org/0abcdef12".to_string());
    assert_eq!(format!("{ror}"), "0abcdef12");
}

#[test]
fn test_timestamp_display() {
    let stamp: Timestamp = Default::default();
    assert_eq!(format!("{stamp}"), "1970-01-01 00:00:00");
}

#[test]
fn test_doi_fromstr() {
    let standardised = Doi("https://doi.org/10.12345/Test-Suffix.01".to_string());
    assert_eq!(
        Doi::from_str("https://doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("http://doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("HTTPS://DOI.ORG/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("Https://DOI.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("https://www.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("http://www.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("www.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("https://dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("http://dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("https://www.dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("http://www.dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("www.dx.doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert!(Doi::from_str("htts://doi.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("https://10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("https://test.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("http://test.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("test.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("//doi.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("https://doi-org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("10.https://doi.org/12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("http://dx.doi.org/10.2990/1471-5457(2005)24[2:tmpwac]2.0.co;2").is_ok());
    assert!(Doi::from_str(
        "https://doi.org/10.1002/(SICI)1098-2736(199908)36:6<637::AID-TEA4>3.0.CO;2-9"
    )
    .is_ok());
    assert!(Doi::from_str(
        "https://doi.org/10.1002/(sici)1096-8644(1996)23+<91::aid-ajpa4>3.0.co;2-c"
    )
    .is_ok());
}

#[test]
fn doi_fromstr_rejects_empty_input() {
    assert!(matches!(Doi::from_str(""), Err(ThothError::DoiEmptyError)));
}

#[test]
fn doi_fromstr_rejects_invalid_input() {
    let result = Doi::from_str("not-a-doi");
    assert!(matches!(result, Err(ThothError::DoiParseError(_))));
}

#[test]
fn test_isbn_fromstr() {
    // Note the `isbn2` crate contains tests of valid/invalid ISBN values -
    // this focuses on testing that a valid ISBN in any format is standardised
    let standardised = Isbn("978-3-16-148410-0".to_string());
    assert_eq!(Isbn::from_str("978-3-16-148410-0").unwrap(), standardised);
    assert_eq!(Isbn::from_str("9783161484100").unwrap(), standardised);
    assert_eq!(Isbn::from_str("978 3 16 148410 0").unwrap(), standardised);
    assert_eq!(Isbn::from_str("978 3 16-148410-0").unwrap(), standardised);
    assert_eq!(Isbn::from_str("9-7-831614-8-4-100").unwrap(), standardised);
    assert_eq!(
        Isbn::from_str("   97831    614 84  100    ").unwrap(),
        standardised
    );
    assert_eq!(
        Isbn::from_str("---97--831614----8-4100--").unwrap(),
        standardised
    );
    assert!(Isbn::from_str("978-3-16-148410-1").is_err());
    assert!(Isbn::from_str("1234567890123").is_err());
    assert!(Isbn::from_str("0-684-84328-5").is_err());
    assert!(Isbn::from_str("abcdef").is_err());
}

#[test]
fn isbn_fromstr_rejects_empty_input() {
    assert!(matches!(
        Isbn::from_str(""),
        Err(ThothError::IsbnEmptyError)
    ));
}

#[test]
fn isbn_fromstr_rejects_garbage_input() {
    let result = Isbn::from_str("not-an-isbn");
    assert!(matches!(result, Err(ThothError::IsbnParseError(_))));
}

#[test]
fn test_orcid_fromstr() {
    let standardised = Orcid("https://orcid.org/0000-0002-1234-5678".to_string());
    assert_eq!(
        Orcid::from_str("https://orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("http://orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("HTTPS://ORCID.ORG/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("Https://ORCiD.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("https://www.orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("http://www.orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("www.orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert!(Orcid::from_str("htts://orcid.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("https://0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("https://test.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("http://test.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("test.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("//orcid.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("https://orcid-org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("0000-0002-1234-5678https://orcid.org/").is_err());
    assert!(Orcid::from_str("0009-0002-1234-567X").is_ok());
}

#[test]
fn orcid_fromstr_rejects_empty_input() {
    assert!(matches!(
        Orcid::from_str(""),
        Err(ThothError::OrcidEmptyError)
    ));
}

#[test]
fn orcid_fromstr_rejects_invalid_input() {
    let result = Orcid::from_str("0000-0002-1234-567");
    assert!(matches!(result, Err(ThothError::OrcidParseError(_))));
}

#[test]
fn test_ror_fromstr() {
    let standardised = Ror("https://ror.org/0abcdef12".to_string());
    assert_eq!(
        Ror::from_str("https://ror.org/0abcdef12").unwrap(),
        standardised
    );
    assert_eq!(
        Ror::from_str("http://ror.org/0abcdef12").unwrap(),
        standardised
    );
    assert_eq!(Ror::from_str("ror.org/0abcdef12").unwrap(), standardised);
    assert_eq!(Ror::from_str("0abcdef12").unwrap(), standardised);
    assert_eq!(
        Ror::from_str("HTTPS://ROR.ORG/0abcdef12").unwrap(),
        standardised
    );
    assert_eq!(
        Ror::from_str("Https://Ror.org/0abcdef12").unwrap(),
        standardised
    );
    assert_eq!(
        Ror::from_str("https://www.ror.org/0abcdef12").unwrap(),
        standardised
    );
    // Testing shows that while leading http://ror and https://www.ror
    // resolve successfully, leading www.ror and http://www.ror do not.
    assert!(Ror::from_str("http://www.ror.org/0abcdef12").is_err());
    assert!(Ror::from_str("www.ror.org/0abcdef12").is_err());
    assert!(Ror::from_str("htts://ror.org/0abcdef12").is_err());
    assert!(Ror::from_str("https://0abcdef12").is_err());
    assert!(Ror::from_str("https://test.org/0abcdef12").is_err());
    assert!(Ror::from_str("http://test.org/0abcdef12").is_err());
    assert!(Ror::from_str("test.org/0abcdef12").is_err());
    assert!(Ror::from_str("//ror.org/0abcdef12").is_err());
    assert!(Ror::from_str("https://ror-org/0abcdef12").is_err());
    assert!(Ror::from_str("0abcdef12https://ror.org/").is_err());
}

#[test]
fn ror_fromstr_rejects_empty_input() {
    assert!(matches!(Ror::from_str(""), Err(ThothError::RorEmptyError)));
}

#[test]
fn ror_fromstr_rejects_invalid_input() {
    let result = Ror::from_str("not-a-ror");
    assert!(matches!(result, Err(ThothError::RorParseError(_))));
}

#[test]
fn test_isbn_to_hyphenless_string() {
    let hyphenless_isbn = Isbn("978-3-16-148410-0".to_string()).to_hyphenless_string();
    assert_eq!(hyphenless_isbn, "9783161484100");
}

#[test]
fn test_orcid_to_hyphenless_string() {
    let hyphenless_orcid =
        Orcid("https://orcid.org/0000-0002-1234-5678".to_string()).to_hyphenless_string();
    assert_eq!(hyphenless_orcid, "0000000212345678");
}

#[test]
fn test_doi_with_domain() {
    let doi = "https://doi.org/10.12345/Test-Suffix.01";
    assert_eq!(Doi(doi.to_string()).with_domain().to_string(), doi);
}

#[test]
fn test_orcid_with_domain() {
    let orcid = "https://orcid.org/0000-0002-1234-5678";
    assert_eq!(Orcid(orcid.to_string()).with_domain().to_string(), orcid);
}

#[test]
fn test_ror_with_domain() {
    let ror = "https://ror.org/0abcdef12";
    assert_eq!(Ror(ror.to_string()).with_domain().to_string(), ror);
}

#[test]
fn test_timestamp_parse_from_rfc3339_valid() {
    let input = "1999-12-31T23:59:00Z";
    let timestamp = Timestamp::parse_from_rfc3339(input);
    assert!(timestamp.is_ok());

    let expected = Timestamp(Utc.with_ymd_and_hms(1999, 12, 31, 23, 59, 0).unwrap());
    assert_eq!(timestamp.unwrap(), expected);
}

#[test]
fn test_timestamp_parse_from_rfc3339_invalid_format() {
    let input = "1999-12-31 23:59:00"; // Missing 'T' and 'Z'
    let timestamp = Timestamp::parse_from_rfc3339(input);
    assert!(timestamp.is_err());
}

#[test]
fn test_timestamp_parse_from_rfc3339_invalid_date() {
    let input = "1999-02-30T23:59:00Z"; // Invalid date
    let timestamp = Timestamp::parse_from_rfc3339(input);
    assert!(timestamp.is_err());
}

#[test]
fn test_timestamp_to_rfc3339() {
    let timestamp = Timestamp(Utc.with_ymd_and_hms(1999, 12, 31, 23, 59, 0).unwrap());
    assert_eq!(timestamp.to_rfc3339(), "1999-12-31T23:59:00+00:00");
}

#[test]
fn test_timestamp_round_trip_rfc3339_conversion() {
    let original_string = "2023-11-13T12:34:56Z";
    let timestamp = Timestamp::parse_from_rfc3339(original_string).unwrap();
    let converted_string = timestamp.to_rfc3339();

    let round_trip_timestamp = Timestamp::parse_from_rfc3339(&converted_string).unwrap();
    assert_eq!(timestamp, round_trip_timestamp);
}

/// BE-06 test-harness reset contract (Amendment 3 section 5, tests H1-H7 and H9).
///
/// Every database test here resets through the harness statement, never through
/// ad-hoc SQL, and every statement that changes the replication mode or a
/// trigger runs inside a transaction that is rolled back.
#[cfg(feature = "backend")]
mod be06_reset {
    use diesel::connection::SimpleConnection;
    use diesel::pg::PgConnection;
    use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
    use diesel::{Connection, QueryableByName, RunQueryDsl};
    use uuid::Uuid;

    use crate::model::tests::db::{create_publisher, reset_db, setup_test_db, TEST_RESET_SQL};

    const INJECTED: &str = "BE06_H4_INJECTED_FAILURE";
    const NOT_RESTORED: &str = "BE06_TEST_RESET_PROTECTIONS_NOT_RESTORED";

    #[derive(QueryableByName)]
    struct TextRow {
        #[diesel(sql_type = Text)]
        value: String,
    }

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    fn text(connection: &mut PgConnection, sql: &str) -> String {
        diesel::sql_query(sql)
            .get_result::<TextRow>(connection)
            .unwrap_or_else(|error| panic!("query `{sql}` failed: {error}"))
            .value
    }

    fn count(connection: &mut PgConnection, sql: &str) -> i64 {
        diesel::sql_query(sql)
            .get_result::<CountRow>(connection)
            .unwrap_or_else(|error| panic!("query `{sql}` failed: {error}"))
            .count
    }

    fn error_message(error: &diesel::result::Error) -> String {
        match error {
            diesel::result::Error::DatabaseError(_, info) => info.message().to_string(),
            other => other.to_string(),
        }
    }

    /// Run `sql` in a transaction that is always rolled back, and return the
    /// database error it raised. Panics if the statement succeeded.
    fn refusal(connection: &mut PgConnection, sql: &str) -> String {
        let outcome = connection.transaction::<(), diesel::result::Error, _>(|connection| {
            connection.batch_execute(sql)?;
            Err(diesel::result::Error::RollbackTransaction)
        });
        match outcome {
            Err(diesel::result::Error::RollbackTransaction) => {
                panic!("`{sql}` was expected to be refused but succeeded")
            }
            Err(error) => error_message(&error),
            Ok(()) => unreachable!("the closure never commits"),
        }
    }

    fn assert_refused(connection: &mut PgConnection, sql: &str, code: &str) {
        let message = refusal(connection, sql);
        assert!(
            message.contains(code),
            "`{sql}` must be refused with {code}, got: {message}"
        );
    }

    /// Residue in every BE-06 table: a permit with membership, the floor at the
    /// G-7 target, an audit row, control `(true, true)`, a generation row, an
    /// admission row and an orphaned capture-queue row.
    fn seed_residue(connection: &mut PgConnection, publisher_id: Uuid) {
        connection
            .transaction::<(), diesel::result::Error, _>(|connection| {
                diesel::sql_query(
                    "INSERT INTO crossref_write_permit \
                         (route, scope, publisher_id, publisher_identity, root_work_identity, \
                          source_generation_witness, doi_set_digest, doi_set_cardinality, \
                          crossref_timestamp, doi_batch_id) \
                     VALUES ('LEGACY_SCHEDULED', 'SINGLE_ROOT_WORK', $1, $1, gen_random_uuid(), 0, \
                             public.crossref_doi_set_digest(ARRAY['https://doi.org/10.12345/be06-residue']), \
                             1, 20260904120000000, 'be06-residue')",
                )
                .bind::<SqlUuid, _>(publisher_id)
                .execute(connection)?;
                connection.batch_execute(
                    "INSERT INTO crossref_write_permit_doi (permit_id, doi) \
                     SELECT permit_id, 'https://doi.org/10.12345/be06-residue' FROM crossref_write_permit",
                )?;
                Ok(())
            })
            .expect("seed a permit with membership");
        connection
            .batch_execute(
                "UPDATE work_crossref_version_floor SET floor_value = 99999999999999; \
                 INSERT INTO crossref_version_floor_audit \
                     (mutation_kind, before_value, after_value, g6_attempt_id, observation_id, \
                      g7_authorization_reference, authorization_register_digest, actor) \
                 VALUES ('ADVANCE_VERSION_FLOOR', 0, 99999999999999, gen_random_uuid(), gen_random_uuid(), \
                         'G7-AUTH-RESIDUE', repeat('a', 64), 'be06-residue'); \
                 UPDATE work_upsert_control SET capture_enabled = true, execution_enabled = true \
                  WHERE execution_profile = 'CROSSREF'; \
                 INSERT INTO work_upsert_generation (work_id, execution_profile, source_generation) \
                 VALUES (gen_random_uuid(), 'CROSSREF', 3); \
                 INSERT INTO work_upsert_capture_queue (entry_kind, work_ids) \
                 VALUES ('OWNERS', ARRAY[gen_random_uuid()]);",
            )
            .expect("seed floor, audit, control, generation and queue residue");
        diesel::sql_query(
            "INSERT INTO work_upsert_admission \
                 (execution_profile, publisher_id, activation_id, evidence_reference, actor) \
             VALUES ('CROSSREF', $1, gen_random_uuid(), 'EV-RESIDUE', 'be06-residue')",
        )
        .bind::<SqlUuid, _>(publisher_id)
        .execute(connection)
        .expect("seed an admission row");
    }

    fn residue_fingerprint(connection: &mut PgConnection) -> String {
        text(
            connection,
            "SELECT concat_ws('|', \
                 (SELECT count(*) FROM crossref_write_permit), \
                 (SELECT count(*) FROM crossref_write_permit_doi), \
                 (SELECT floor_value FROM work_crossref_version_floor), \
                 (SELECT count(*) FROM crossref_version_floor_audit), \
                 (SELECT capture_enabled::text || execution_enabled::text FROM work_upsert_control), \
                 (SELECT count(*) FROM work_upsert_generation), \
                 (SELECT count(*) FROM work_upsert_admission), \
                 (SELECT count(*) FROM work_upsert_capture_queue)) AS value",
        )
    }

    const RESIDUE: &str = "1|1|99999999999999|1|truetrue|1|1|1";

    fn assert_protections_active(connection: &mut PgConnection) {
        assert_eq!(
            text(
                connection,
                "SELECT current_setting('session_replication_role') AS value"
            ),
            "origin"
        );
        assert_eq!(
            count(
                connection,
                "SELECT count(*) AS count FROM pg_trigger t \
                 JOIN pg_class c ON c.oid = t.tgrelid \
                 JOIN pg_namespace n ON n.oid = c.relnamespace \
                 WHERE n.nspname = 'public' AND NOT t.tgisinternal AND t.tgenabled <> 'O'"
            ),
            0,
            "no public user trigger may be disabled after a reset"
        );
    }

    /// Section 5.1 items 5 and 6 exactly: the two seed rows, every other public
    /// table empty, identities restarted, the migration ledger untouched.
    fn assert_clean_state(connection: &mut PgConnection) {
        assert_eq!(
            text(
                connection,
                "SELECT string_agg(execution_profile::text || ':' || capture_enabled::text || ':' \
                     || execution_enabled::text, ',') AS value FROM work_upsert_control"
            ),
            "CROSSREF:false:false"
        );
        assert_eq!(
            text(
                connection,
                "SELECT string_agg(floor_id::text || ':' || floor_value::text, ',') AS value \
                 FROM work_crossref_version_floor"
            ),
            "true:0"
        );
        assert_eq!(
            text(
                connection,
                "SELECT coalesce(string_agg(table_name || '=' || n, ',' ORDER BY table_name), '') AS value \
                 FROM (SELECT table_name::text AS table_name, \
                              (xpath('/row/n/text()', query_to_xml( \
                                  format('SELECT count(*) AS n FROM public.%I', table_name), \
                                  false, true, '')))[1]::text::bigint AS n \
                         FROM information_schema.tables \
                        WHERE table_schema = 'public' AND table_type = 'BASE TABLE' \
                          AND table_name NOT IN ('__diesel_schema_migrations', \
                                                 'work_upsert_control', \
                                                 'work_crossref_version_floor')) s \
                WHERE n > 0"
            ),
            "",
            "every other public table must be empty after a reset"
        );
        assert_eq!(
            text(
                connection,
                "SELECT coalesce(pg_sequence_last_value( \
                     pg_get_serial_sequence('public.work_upsert_capture_queue', 'entry_id'))::text, \
                     'restarted') AS value"
            ),
            "restarted",
            "identities are restarted"
        );
        assert_eq!(
            count(
                connection,
                "SELECT count(*) AS count FROM __diesel_schema_migrations \
                 WHERE version IN ('20260910', '20260911')"
            ),
            2,
            "the migration ledger is untouched"
        );
        assert_protections_active(connection);
    }

    #[test]
    fn h1_the_reset_removes_residue_from_every_be06_table_and_restores_the_seed_rows() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let mut connection = pool.get().expect("connection");
        seed_residue(&mut connection, publisher.publisher_id);
        assert_eq!(residue_fingerprint(&mut connection), RESIDUE);

        reset_db(pool.as_ref()).expect("the reset succeeds over BE-06 residue");
        assert_clean_state(&mut connection);
    }

    #[test]
    fn h2_consecutive_resets_and_a_reset_between_two_writers_are_identical() {
        let (_guard, pool) = setup_test_db();
        let mut connection = pool.get().expect("connection");
        let publisher = create_publisher(pool.as_ref());
        seed_residue(&mut connection, publisher.publisher_id);

        reset_db(pool.as_ref()).expect("first reset");
        reset_db(pool.as_ref()).expect("second reset");
        assert_clean_state(&mut connection);

        connection
            .batch_execute(
                "INSERT INTO work_upsert_generation (work_id, execution_profile, source_generation) \
                 VALUES (gen_random_uuid(), 'CROSSREF', 1)",
            )
            .expect("first writer");
        reset_db(pool.as_ref()).expect("reset between writers");
        assert_clean_state(&mut connection);
        connection
            .batch_execute(
                "INSERT INTO work_upsert_generation (work_id, execution_profile, source_generation) \
                 VALUES (gen_random_uuid(), 'CROSSREF', 2)",
            )
            .expect("second writer");
        reset_db(pool.as_ref()).expect("reset after the second writer");
        assert_clean_state(&mut connection);
    }

    #[test]
    fn h3_after_a_reset_every_permanence_protection_refuses_ordinary_sql() {
        let (_guard, pool) = setup_test_db();
        let mut reset_connection = pool.get().expect("reset connection");
        diesel::sql_query(TEST_RESET_SQL)
            .execute(&mut reset_connection)
            .expect("reset on a held connection");
        assert_protections_active(&mut reset_connection);

        let mut other = pool.get().expect("another pooled connection");
        assert_protections_active(&mut other);

        let publisher = create_publisher(pool.as_ref());
        seed_residue(&mut other, publisher.publisher_id);

        // Permits and membership.
        assert_refused(
            &mut other,
            "DELETE FROM crossref_write_permit",
            "CROSSREF_PERMIT_DELETE_REFUSED",
        );
        assert_refused(
            &mut other,
            "TRUNCATE crossref_write_permit CASCADE",
            "CROSSREF_PERMIT_DELETE_REFUSED",
        );
        assert_refused(
            &mut other,
            "TRUNCATE crossref_write_permit, crossref_write_permit_doi",
            "CROSSREF_PERMIT_DELETE_REFUSED",
        );
        assert_refused(
            &mut other,
            "TRUNCATE crossref_write_permit_doi",
            "CROSSREF_PERMIT_DELETE_REFUSED",
        );
        assert_refused(
            &mut other,
            "TRUNCATE work CASCADE",
            "CROSSREF_PERMIT_DELETE_REFUSED",
        );
        assert_refused(
            &mut other,
            "DO $$ BEGIN TRUNCATE crossref_write_permit; \
             EXCEPTION WHEN SQLSTATE '0A000' THEN RAISE EXCEPTION 'SQLSTATE_0A000'; END $$",
            "SQLSTATE_0A000",
        );
        assert_refused(
            &mut other,
            "UPDATE crossref_write_permit_doi SET doi = doi",
            "CROSSREF_PERMIT_MEMBERSHIP_IMMUTABLE",
        );
        assert_refused(
            &mut other,
            "DELETE FROM crossref_write_permit_doi",
            "CROSSREF_PERMIT_MEMBERSHIP_IMMUTABLE",
        );
        // The version floor and its audit.
        assert_refused(
            &mut other,
            "DELETE FROM work_crossref_version_floor",
            "CROSSREF_VERSION_FLOOR_PERMANENT",
        );
        assert_refused(
            &mut other,
            "TRUNCATE work_crossref_version_floor",
            "CROSSREF_VERSION_FLOOR_PERMANENT",
        );
        assert_refused(
            &mut other,
            "UPDATE work_crossref_version_floor SET floor_value = 0",
            "CROSSREF_VERSION_FLOOR_NOT_DECREASING",
        );
        for statement in [
            "UPDATE crossref_version_floor_audit SET actor = 'changed'",
            "DELETE FROM crossref_version_floor_audit",
            "TRUNCATE crossref_version_floor_audit",
        ] {
            assert_refused(
                &mut other,
                statement,
                "CROSSREF_VERSION_FLOOR_AUDIT_APPEND_ONLY",
            );
        }
        // The generic control and admission tables (R-8 included).
        for statement in [
            "DELETE FROM work_upsert_control",
            "TRUNCATE work_upsert_control",
        ] {
            assert_refused(
                &mut other,
                statement,
                "WORK_UPSERT_CONTROL_ROW_IS_PERMANENT",
            );
        }
        assert_refused(
            &mut other,
            "UPDATE work_upsert_admission SET evidence_reference = 'changed'",
            "WORK_UPSERT_ADMISSION_IMMUTABLE",
        );
        for statement in [
            "DELETE FROM work_upsert_admission",
            "TRUNCATE work_upsert_admission",
        ] {
            assert_refused(
                &mut other,
                statement,
                "WORK_UPSERT_ADMISSION_DELETE_ONLY_BY_PUBLISHER_CASCADE",
            );
        }

        assert_eq!(residue_fingerprint(&mut other), RESIDUE);
        assert_protections_active(&mut reset_connection);
        assert_protections_active(&mut other);
    }

    #[test]
    fn h4_a_failure_after_the_truncate_rolls_everything_back_with_protections_active() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let mut connection = pool.get().expect("connection");
        seed_residue(&mut connection, publisher.publisher_id);

        let truncate = "EXECUTE 'TRUNCATE TABLE ' || tbls || ' RESTART IDENTITY CASCADE';";
        assert_eq!(TEST_RESET_SQL.matches(truncate).count(), 1);
        let injected = TEST_RESET_SQL.replacen(
            truncate,
            &format!("{truncate}\n        RAISE EXCEPTION '{INJECTED}';"),
            1,
        );
        assert_eq!(injected.matches(INJECTED).count(), 1);

        let error = diesel::sql_query(injected.as_str())
            .execute(&mut connection)
            .expect_err("the injected failure aborts the reset");
        assert!(error_message(&error).contains(INJECTED));

        assert_protections_active(&mut connection);
        assert_refused(
            &mut connection,
            "TRUNCATE work_upsert_control",
            "WORK_UPSERT_CONTROL_ROW_IS_PERMANENT",
        );
        assert_eq!(residue_fingerprint(&mut connection), RESIDUE);

        reset_db(pool.as_ref()).expect("the next reset succeeds");
        assert_clean_state(&mut connection);
    }

    #[test]
    fn h5_the_reset_fails_closed_when_a_protection_is_missing_or_the_mode_is_wrong() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let mut connection = pool.get().expect("connection");
        seed_residue(&mut connection, publisher.publisher_id);

        // A permanence guard disabled inside the reset's own transaction.
        let message = refusal(
            &mut connection,
            &format!(
                "ALTER TABLE work_upsert_control DISABLE TRIGGER work_upsert_control_no_truncate;\n{TEST_RESET_SQL}"
            ),
        );
        assert!(message.contains(NOT_RESTORED), "got: {message}");
        assert_eq!(residue_fingerprint(&mut connection), RESIDUE);
        assert_protections_active(&mut connection);

        // The statement without its origin restore.
        let restore = "        PERFORM set_config('session_replication_role', 'origin', true);\n";
        assert_eq!(TEST_RESET_SQL.matches(restore).count(), 1);
        let without_restore = TEST_RESET_SQL.replacen(restore, "", 1);
        let message = refusal(&mut connection, &without_restore);
        assert!(message.contains(NOT_RESTORED), "got: {message}");
        assert_eq!(residue_fingerprint(&mut connection), RESIDUE);
        assert_protections_active(&mut connection);

        // A connection left in session-level replica mode, reset afterwards.
        connection
            .batch_execute("SET session_replication_role = replica")
            .expect("enter replica mode for the negative control");
        let outcome = diesel::sql_query(TEST_RESET_SQL).execute(&mut connection);
        connection
            .batch_execute("SET session_replication_role = origin")
            .expect("leave replica mode");
        let message = error_message(&outcome.expect_err("refused in replica mode"));
        assert!(message.contains(NOT_RESTORED), "got: {message}");
        assert_eq!(residue_fingerprint(&mut connection), RESIDUE);
        assert_protections_active(&mut connection);

        // One manifest trigger dropped in a rolled-back transaction.
        let message = refusal(
            &mut connection,
            &format!(
                "DROP TRIGGER crossref_version_floor_audit_no_truncate ON crossref_version_floor_audit;\n{TEST_RESET_SQL}"
            ),
        );
        assert!(message.contains(NOT_RESTORED), "got: {message}");
        assert_eq!(
            count(
                &mut connection,
                "SELECT count(*) AS count FROM pg_trigger \
                 WHERE tgname = 'crossref_version_floor_audit_no_truncate'"
            ),
            1,
            "the rolled-back drop restored the trigger"
        );
        assert_eq!(residue_fingerprint(&mut connection), RESIDUE);
        assert_protections_active(&mut connection);

        reset_db(pool.as_ref()).expect("a good reset succeeds afterwards");
        assert_clean_state(&mut connection);
    }

    #[test]
    fn h6_the_integration_test_reset_statement_is_byte_identical() {
        let support =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mod.rs"))
                .expect("read tests/support/mod.rs");
        let body = support
            .split_once("let sql = r#\"")
            .expect("tests/support/mod.rs declares its reset statement as a raw string")
            .1;
        let statement = body
            .split_once("\"#;")
            .expect("the raw string is terminated")
            .0;
        assert_eq!(statement, TEST_RESET_SQL);
    }

    fn files_under(root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(root).expect("read directory") {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                files_under(&path, out);
            } else {
                out.push(path);
            }
        }
    }

    #[test]
    fn h7_the_replication_mode_bypass_exists_only_in_test_sources() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut sources = Vec::new();
        files_under(&manifest.join("src"), &mut sources);
        for path in sources {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if content.contains("session_replication_role") {
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                assert!(
                    name == "tests.rs" || name.ends_with("_tests.rs"),
                    "{} mentions session_replication_role outside a test source",
                    path.display()
                );
            }
        }
        let mut migrations = Vec::new();
        files_under(&manifest.join("migrations"), &mut migrations);
        for path in migrations {
            let content = std::fs::read_to_string(&path).expect("read migration file");
            assert!(
                !content.contains("session_replication_role"),
                "{} mentions session_replication_role",
                path.display()
            );
        }
    }

    #[test]
    fn h9_the_test_database_runs_the_suite_over_the_migrated_schema() {
        let (_guard, pool) = setup_test_db();
        let mut connection = pool.get().expect("connection");
        assert_eq!(
            count(
                &mut connection,
                "SELECT count(*) AS count FROM __diesel_schema_migrations \
                 WHERE version IN ('20260910', '20260911')"
            ),
            2
        );
        assert_eq!(
            count(
                &mut connection,
                "SELECT count(*) AS count FROM pg_tables WHERE schemaname = 'public' \
                 AND tablename IN ('work_upsert_generation', 'work_upsert_capture_queue', \
                                   'work_upsert_control', 'work_upsert_admission', \
                                   'crossref_write_permit', 'crossref_write_permit_doi', \
                                   'work_crossref_version_floor', 'crossref_version_floor_audit')"
            ),
            8
        );
        assert_clean_state(&mut connection);
    }
}
