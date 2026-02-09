use super::*;

#[cfg(feature = "backend")]
pub(crate) mod db {
    use std::collections::HashMap;
    use std::env;
    use std::fs::OpenOptions;
    use std::sync::{Arc, OnceLock};

    use diesel::RunQueryDsl;
    use fs2::FileExt;
    use uuid::Uuid;
    use zitadel::actix::introspection::IntrospectedUser;

    use crate::db::{init_pool, run_migrations, PgPool};
    use crate::graphql::Context;
    use crate::model::contribution::{Contribution, ContributionType, NewContribution};
    use crate::model::contributor::{Contributor, NewContributor};
    use crate::model::imprint::{Imprint, NewImprint};
    use crate::model::institution::{CountryCode, Institution, NewInstitution};
    use crate::model::publication::{NewPublication, Publication, PublicationType};
    use crate::model::publisher::{NewPublisher, Publisher};
    use crate::model::series::{NewSeries, Series, SeriesType};
    use crate::model::work::{NewWork, Work, WorkStatus, WorkType};
    use crate::model::Crud;
    use crate::policy::Role;

    static MIGRATIONS: OnceLock<Result<(), String>> = OnceLock::new();
    static POOL: OnceLock<Arc<PgPool>> = OnceLock::new();

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

    pub(crate) fn reset_db(pool: &PgPool) -> Result<(), diesel::result::Error> {
        let mut connection = pool.get().expect("Failed to get DB connection");
        let sql = r#"
DO $$
DECLARE
    tbls TEXT;
BEGIN
    SELECT string_agg(format('%I.%I', schemaname, tablename), ', ')
    INTO tbls
    FROM pg_tables
    WHERE schemaname = 'public'
      AND tablename != '__diesel_schema_migrations';

    IF tbls IS NOT NULL THEN
        EXECUTE 'TRUNCATE TABLE ' || tbls || ' RESTART IDENTITY CASCADE';
    END IF;
END $$;
"#;
        diesel::sql_query(sql).execute(&mut connection).map(|_| ())
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
        Context::new(pool, Some(test_user(user_id)))
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
        Context::new(pool, Some(user))
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
    assert_eq!(format!("{}", Doi(doi.to_string()).with_domain()), doi);
}

#[test]
fn test_orcid_with_domain() {
    let orcid = "https://orcid.org/0000-0002-1234-5678";
    assert_eq!(format!("{}", Orcid(orcid.to_string()).with_domain()), orcid);
}

#[test]
fn test_ror_with_domain() {
    let ror = "https://ror.org/0abcdef12";
    assert_eq!(format!("{}", Ror(ror.to_string()).with_domain()), ror);
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
