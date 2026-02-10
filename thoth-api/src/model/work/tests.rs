use super::*;
use uuid::Uuid;

fn make_work() -> Work {
    use std::str::FromStr;
    Work {
        work_id: Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
        work_type: WorkType::Monograph,
        work_status: WorkStatus::Active,
        reference: None,
        edition: Some(1),
        imprint_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
        doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
        publication_date: NaiveDate::from_ymd_opt(1999, 12, 31),
        withdrawn_date: None,
        place: Some("León, Spain".to_string()),
        page_count: Some(123),
        page_breakdown: None,
        image_count: Some(22),
        table_count: Some(3),
        audio_count: None,
        video_count: None,
        license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
        copyright_holder: Some("Author1".to_string()),
        landing_page: Some("https://book.page".to_string()),
        lccn: None,
        oclc: None,
        general_note: None,
        bibliography_note: None,
        toc: None,
        cover_url: Some("https://book.cover/image".to_string()),
        cover_caption: None,
        created_at: Default::default(),
        updated_at: Default::default(),
        first_page: None,
        last_page: None,
        page_interval: None,
        updated_at_with_relations: Default::default(),
    }
}

mod defaults {
    use super::*;

    #[test]
    fn worktype_default_is_monograph() {
        let worktype: WorkType = Default::default();
        assert_eq!(worktype, WorkType::Monograph);
    }

    #[test]
    fn workstatus_default_is_forthcoming() {
        let workstatus: WorkStatus = Default::default();
        assert_eq!(workstatus, WorkStatus::Forthcoming);
    }

    #[test]
    fn workfield_default_is_full_title() {
        let workfield: WorkField = Default::default();
        assert_eq!(workfield, WorkField::FullTitle);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn worktype_display_formats_expected_strings() {
        assert_eq!(format!("{}", WorkType::BookChapter), "Book Chapter");
        assert_eq!(format!("{}", WorkType::Monograph), "Monograph");
        assert_eq!(format!("{}", WorkType::EditedBook), "Edited Book");
        assert_eq!(format!("{}", WorkType::Textbook), "Textbook");
        assert_eq!(format!("{}", WorkType::JournalIssue), "Journal Issue");
        assert_eq!(format!("{}", WorkType::BookSet), "Book Set");
    }

    #[test]
    fn workstatus_display_formats_expected_strings() {
        assert_eq!(format!("{}", WorkStatus::Cancelled), "Cancelled");
        assert_eq!(format!("{}", WorkStatus::Forthcoming), "Forthcoming");
        assert_eq!(
            format!("{}", WorkStatus::PostponedIndefinitely),
            "Postponed Indefinitely"
        );
        assert_eq!(format!("{}", WorkStatus::Active), "Active");
        assert_eq!(format!("{}", WorkStatus::Withdrawn), "Withdrawn");
        assert_eq!(format!("{}", WorkStatus::Superseded), "Superseded");
    }

    #[test]
    fn workfield_display_formats_expected_strings() {
        assert_eq!(format!("{}", WorkField::WorkId), "ID");
        assert_eq!(format!("{}", WorkField::WorkType), "Type");
        assert_eq!(format!("{}", WorkField::WorkStatus), "WorkStatus");
        assert_eq!(format!("{}", WorkField::Reference), "Reference");
        assert_eq!(format!("{}", WorkField::Edition), "Edition");
        assert_eq!(format!("{}", WorkField::Doi), "DOI");
        assert_eq!(format!("{}", WorkField::PublicationDate), "PublicationDate");
        assert_eq!(format!("{}", WorkField::WithdrawnDate), "WithdrawnDate");
        assert_eq!(format!("{}", WorkField::Place), "Place");
        assert_eq!(format!("{}", WorkField::PageCount), "PageCount");
        assert_eq!(format!("{}", WorkField::PageBreakdown), "PageBreakdown");
        assert_eq!(format!("{}", WorkField::FirstPage), "FirstPage");
        assert_eq!(format!("{}", WorkField::LastPage), "LastPage");
        assert_eq!(format!("{}", WorkField::PageInterval), "PageInterval");
        assert_eq!(format!("{}", WorkField::ImageCount), "ImageCount");
        assert_eq!(format!("{}", WorkField::TableCount), "TableCount");
        assert_eq!(format!("{}", WorkField::AudioCount), "AudioCount");
        assert_eq!(format!("{}", WorkField::VideoCount), "VideoCount");
        assert_eq!(format!("{}", WorkField::License), "License");
        assert_eq!(format!("{}", WorkField::CopyrightHolder), "CopyrightHolder");
        assert_eq!(format!("{}", WorkField::LandingPage), "LandingPage");
        assert_eq!(format!("{}", WorkField::Lccn), "LCCN");
        assert_eq!(format!("{}", WorkField::Oclc), "OCLC");
        assert_eq!(format!("{}", WorkField::ShortAbstract), "ShortAbstract");
        assert_eq!(format!("{}", WorkField::LongAbstract), "LongAbstract");
        assert_eq!(format!("{}", WorkField::GeneralNote), "GeneralNote");
        assert_eq!(
            format!("{}", WorkField::BibliographyNote),
            "BibliographyNote"
        );
        assert_eq!(format!("{}", WorkField::Toc), "TOC");
        assert_eq!(format!("{}", WorkField::CoverUrl), "CoverURL");
        assert_eq!(format!("{}", WorkField::CoverCaption), "CoverCaption");
        assert_eq!(format!("{}", WorkField::CreatedAt), "CreatedAt");
        assert_eq!(format!("{}", WorkField::UpdatedAt), "UpdatedAt");
        assert_eq!(
            format!("{}", WorkField::UpdatedAtWithRelations),
            "UpdatedAtWithRelations"
        );
    }

    #[test]
    fn worktype_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            WorkType::from_str("Book Chapter").unwrap(),
            WorkType::BookChapter
        );
        assert_eq!(
            WorkType::from_str("Monograph").unwrap(),
            WorkType::Monograph
        );
        assert_eq!(
            WorkType::from_str("Edited Book").unwrap(),
            WorkType::EditedBook
        );
        assert_eq!(WorkType::from_str("Textbook").unwrap(), WorkType::Textbook);
        assert_eq!(
            WorkType::from_str("Journal Issue").unwrap(),
            WorkType::JournalIssue
        );
        assert_eq!(WorkType::from_str("Book Set").unwrap(), WorkType::BookSet);

        assert!(WorkType::from_str("Book Section").is_err());
        assert!(WorkType::from_str("Manuscript").is_err());
    }

    #[test]
    fn workstatus_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            WorkStatus::from_str("Cancelled").unwrap(),
            WorkStatus::Cancelled
        );
        assert_eq!(
            WorkStatus::from_str("Forthcoming").unwrap(),
            WorkStatus::Forthcoming
        );
        assert_eq!(
            WorkStatus::from_str("Postponed Indefinitely").unwrap(),
            WorkStatus::PostponedIndefinitely
        );
        assert_eq!(WorkStatus::from_str("Active").unwrap(), WorkStatus::Active);
        assert_eq!(
            WorkStatus::from_str("Withdrawn").unwrap(),
            WorkStatus::Withdrawn
        );
        assert_eq!(
            WorkStatus::from_str("Superseded").unwrap(),
            WorkStatus::Superseded
        );

        assert!(WorkStatus::from_str("Published").is_err());
        assert!(WorkStatus::from_str("Unpublished").is_err());
    }

    #[test]
    fn workfield_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(WorkField::from_str("ID").unwrap(), WorkField::WorkId);
        assert_eq!(WorkField::from_str("Type").unwrap(), WorkField::WorkType);
        assert_eq!(
            WorkField::from_str("WorkStatus").unwrap(),
            WorkField::WorkStatus
        );
        assert_eq!(WorkField::from_str("Title").unwrap(), WorkField::FullTitle);
        assert_eq!(WorkField::from_str("ShortTitle").unwrap(), WorkField::Title);
        assert_eq!(
            WorkField::from_str("Subtitle").unwrap(),
            WorkField::Subtitle
        );
        assert_eq!(
            WorkField::from_str("Reference").unwrap(),
            WorkField::Reference
        );
        assert_eq!(WorkField::from_str("Edition").unwrap(), WorkField::Edition);
        assert_eq!(WorkField::from_str("DOI").unwrap(), WorkField::Doi);
        assert_eq!(
            WorkField::from_str("PublicationDate").unwrap(),
            WorkField::PublicationDate
        );
        assert_eq!(
            WorkField::from_str("WithdrawnDate").unwrap(),
            WorkField::WithdrawnDate
        );
        assert_eq!(WorkField::from_str("Place").unwrap(), WorkField::Place);
        assert_eq!(
            WorkField::from_str("PageCount").unwrap(),
            WorkField::PageCount
        );
        assert_eq!(
            WorkField::from_str("PageBreakdown").unwrap(),
            WorkField::PageBreakdown
        );
        assert_eq!(
            WorkField::from_str("FirstPage").unwrap(),
            WorkField::FirstPage
        );
        assert_eq!(
            WorkField::from_str("LastPage").unwrap(),
            WorkField::LastPage
        );
        assert_eq!(
            WorkField::from_str("PageInterval").unwrap(),
            WorkField::PageInterval
        );
        assert_eq!(
            WorkField::from_str("ImageCount").unwrap(),
            WorkField::ImageCount
        );
        assert_eq!(
            WorkField::from_str("TableCount").unwrap(),
            WorkField::TableCount
        );
        assert_eq!(
            WorkField::from_str("AudioCount").unwrap(),
            WorkField::AudioCount
        );
        assert_eq!(
            WorkField::from_str("VideoCount").unwrap(),
            WorkField::VideoCount
        );
        assert_eq!(WorkField::from_str("License").unwrap(), WorkField::License);
        assert_eq!(
            WorkField::from_str("CopyrightHolder").unwrap(),
            WorkField::CopyrightHolder
        );
        assert_eq!(
            WorkField::from_str("LandingPage").unwrap(),
            WorkField::LandingPage
        );
        assert_eq!(WorkField::from_str("LCCN").unwrap(), WorkField::Lccn);
        assert_eq!(WorkField::from_str("OCLC").unwrap(), WorkField::Oclc);
        assert_eq!(
            WorkField::from_str("ShortAbstract").unwrap(),
            WorkField::ShortAbstract
        );
        assert_eq!(
            WorkField::from_str("LongAbstract").unwrap(),
            WorkField::LongAbstract
        );
        assert_eq!(
            WorkField::from_str("GeneralNote").unwrap(),
            WorkField::GeneralNote
        );
        assert_eq!(
            WorkField::from_str("BibliographyNote").unwrap(),
            WorkField::BibliographyNote
        );
        assert_eq!(WorkField::from_str("TOC").unwrap(), WorkField::Toc);
        assert_eq!(
            WorkField::from_str("CoverURL").unwrap(),
            WorkField::CoverUrl
        );
        assert_eq!(
            WorkField::from_str("CoverCaption").unwrap(),
            WorkField::CoverCaption
        );
        assert_eq!(
            WorkField::from_str("CreatedAt").unwrap(),
            WorkField::CreatedAt
        );
        assert_eq!(
            WorkField::from_str("UpdatedAt").unwrap(),
            WorkField::UpdatedAt
        );
        assert_eq!(
            WorkField::from_str("UpdatedAtWithRelations").unwrap(),
            WorkField::UpdatedAtWithRelations
        );
        assert!(WorkField::from_str("WorkID").is_err());
        assert!(WorkField::from_str("Contributors").is_err());
        assert!(WorkField::from_str("Publisher").is_err());
    }
}

mod conversions {
    use super::*;
    #[cfg(feature = "backend")]
    use crate::model::tests::db::setup_test_db;
    #[cfg(feature = "backend")]
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[test]
    fn work_into_patchwork_copies_fields() {
        let work = make_work();
        let patch_work: PatchWork = work.clone().into();

        macro_rules! assert_fields_eq {
            ($($field:ident),+) => {
                $(
                    assert_eq!(work.$field, patch_work.$field);
                )+
            };
        }
        assert_fields_eq!(
            work_id,
            work_type,
            work_status,
            reference,
            edition,
            imprint_id,
            doi,
            publication_date,
            withdrawn_date,
            place,
            page_count,
            page_breakdown,
            image_count,
            table_count,
            audio_count,
            video_count,
            license,
            copyright_holder,
            landing_page,
            lccn,
            oclc,
            general_note,
            bibliography_note,
            toc,
            cover_url,
            cover_caption,
            first_page,
            last_page,
            page_interval
        );
    }

    #[cfg(feature = "backend")]
    #[test]
    fn worktype_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(WorkType::Monograph);
    }

    #[cfg(feature = "backend")]
    #[test]
    fn workstatus_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(WorkStatus::Active);
    }

    #[cfg(feature = "backend")]
    #[test]
    fn workfield_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(WorkField::WorkId);
    }

    #[cfg(feature = "backend")]
    #[test]
    fn worktype_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<WorkType, crate::schema::sql_types::WorkType>(
            pool.as_ref(),
            "'monograph'::work_type",
            WorkType::Monograph,
        );
    }

    #[cfg(feature = "backend")]
    #[test]
    fn workstatus_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<WorkStatus, crate::schema::sql_types::WorkStatus>(
            pool.as_ref(),
            "'active'::work_status",
            WorkStatus::Active,
        );
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn compile_page_interval_returns_expected_interval() {
        let mut work = make_work();
        assert!(work.compile_page_interval().is_none());

        work.first_page = Some("1".to_string());
        work.last_page = Some("10".to_string());
        assert_eq!(work.compile_page_interval(), Some("1–10".to_string()));
    }

    #[test]
    fn is_published_returns_true_for_published_statuses() {
        let mut work = make_work();

        work.work_status = WorkStatus::Forthcoming;
        assert!(!work.is_published());
        work.work_status = WorkStatus::Cancelled;
        assert!(!work.is_published());
        work.work_status = WorkStatus::PostponedIndefinitely;
        assert!(!work.is_published());

        work.work_status = WorkStatus::Active;
        assert!(work.is_published());
        work.work_status = WorkStatus::Withdrawn;
        assert!(work.is_published());
        work.work_status = WorkStatus::Superseded;
        assert!(work.is_published());
    }

    #[test]
    fn is_out_of_print_returns_true_for_out_of_print_statuses() {
        let mut work = make_work();

        work.work_status = WorkStatus::Forthcoming;
        assert!(!work.is_out_of_print());
        work.work_status = WorkStatus::Cancelled;
        assert!(!work.is_out_of_print());
        work.work_status = WorkStatus::PostponedIndefinitely;
        assert!(!work.is_out_of_print());
        work.work_status = WorkStatus::Active;
        assert!(!work.is_out_of_print());

        work.work_status = WorkStatus::Withdrawn;
        assert!(work.is_out_of_print());
        work.work_status = WorkStatus::Superseded;
        assert!(work.is_out_of_print());
    }

    #[test]
    fn is_active_returns_true_for_active_status() {
        let mut work = make_work();
        assert!(work.is_active());

        work.work_status = WorkStatus::Forthcoming;
        assert!(!work.is_active());
        work.work_status = WorkStatus::Cancelled;
        assert!(!work.is_active());
        work.work_status = WorkStatus::PostponedIndefinitely;
        assert!(!work.is_active());
        work.work_status = WorkStatus::Withdrawn;
        assert!(!work.is_active());
        work.work_status = WorkStatus::Superseded;
        assert!(!work.is_active());
    }

    #[test]
    fn pk_returns_id() {
        let work: Work = Default::default();
        assert_eq!(work.pk(), work.work_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let work: Work = Default::default();
        let user_id = "123456".to_string();
        let new_work_history = work.new_history_entry(&user_id);
        assert_eq!(new_work_history.work_id, work.work_id);
        assert_eq!(new_work_history.user_id, user_id);
        assert_eq!(
            new_work_history.data,
            serde_json::Value::String(serde_json::to_string(&work).unwrap())
        );
    }
}

mod validation {
    use super::*;

    #[test]
    fn validate_fails_when_published_without_publication_date() {
        let mut work = make_work();
        work.work_status = WorkStatus::Active;
        work.publication_date = None;

        assert_eq!(work.validate(), Err(ThothError::PublicationDateError));
    }

    #[test]
    fn validate_fails_when_published_with_withdrawn_date() {
        let mut work = make_work();
        work.work_status = WorkStatus::Active;
        work.withdrawn_date = Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());

        assert_eq!(work.validate(), Err(ThothError::WithdrawnDateError));
    }

    #[test]
    fn validate_fails_when_out_of_print_without_withdrawn_date() {
        let mut work = make_work();
        work.work_status = WorkStatus::Withdrawn;
        work.withdrawn_date = None;

        assert_eq!(work.validate(), Err(ThothError::NoWithdrawnDateError));
        work.work_status = WorkStatus::Superseded;
        assert_eq!(work.validate(), Err(ThothError::NoWithdrawnDateError));
    }

    #[test]
    fn validate_fails_when_withdrawn_date_before_publication_date() {
        let mut work = make_work();
        work.work_status = WorkStatus::Withdrawn;
        work.publication_date = Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        work.withdrawn_date = Some(NaiveDate::from_ymd_opt(2019, 12, 31).unwrap());

        assert_eq!(
            work.validate(),
            Err(ThothError::WithdrawnDateBeforePublicationDateError)
        );
    }

    #[test]
    fn validate_succeeds_with_valid_dates() {
        let mut work = make_work();
        work.work_status = WorkStatus::Withdrawn;
        work.publication_date = Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        work.withdrawn_date = Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());

        assert_eq!(work.validate(), Ok(()));
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;
    use std::collections::HashMap;

    use chrono::NaiveDate;
    use zitadel::actix::introspection::IntrospectedUser;

    use crate::model::publication::{NewPublication, Publication, PublicationType};
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context_with_user,
        test_superuser, test_user_with_role,
    };
    use crate::model::work::policy::WorkPolicy;
    use crate::model::Crud;
    use crate::model::Isbn;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};
    use thoth_errors::ThothError;

    fn make_patch_from_work(work: &Work) -> PatchWork {
        PatchWork {
            work_id: work.work_id,
            work_type: work.work_type,
            work_status: work.work_status,
            reference: work.reference.clone(),
            edition: work.edition,
            imprint_id: work.imprint_id,
            doi: work.doi.clone(),
            publication_date: work.publication_date,
            withdrawn_date: work.withdrawn_date,
            place: work.place.clone(),
            page_count: work.page_count,
            page_breakdown: work.page_breakdown.clone(),
            image_count: work.image_count,
            table_count: work.table_count,
            audio_count: work.audio_count,
            video_count: work.video_count,
            license: work.license.clone(),
            copyright_holder: work.copyright_holder.clone(),
            landing_page: work.landing_page.clone(),
            lccn: work.lccn.clone(),
            oclc: work.oclc.clone(),
            general_note: work.general_note.clone(),
            bibliography_note: work.bibliography_note.clone(),
            toc: work.toc.clone(),
            cover_url: work.cover_url.clone(),
            cover_caption: work.cover_caption.clone(),
            first_page: work.first_page.clone(),
            last_page: work.last_page.clone(),
            page_interval: work.page_interval.clone(),
        }
    }

    fn make_new_work(imprint_id: Uuid) -> NewWork {
        NewWork {
            work_type: WorkType::Monograph,
            work_status: WorkStatus::Forthcoming,
            reference: None,
            edition: Some(1),
            imprint_id,
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
        }
    }

    fn make_new_publication_with_isbn(work_id: Uuid, isbn: Isbn) -> NewPublication {
        NewPublication {
            publication_type: PublicationType::Epub,
            work_id,
            isbn: Some(isbn),
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
        }
    }

    fn user_with_roles(user_id: &str, org_id: &str, roles: &[Role]) -> IntrospectedUser {
        let mut project_roles = HashMap::new();
        for role in roles {
            let mut scoped = HashMap::new();
            scoped.insert(org_id.to_string(), "role".to_string());
            project_roles.insert(role.as_ref().to_string(), scoped);
        }

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

    #[test]
    fn crud_policy_allows_publisher_user_for_create() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("work-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
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

        assert!(WorkPolicy::can_create(&ctx, &new_work, ()).is_ok());

        let other_user = test_user_with_role("work-user", Role::PublisherUser, "org-other");
        let other_ctx = test_context_with_user(pool.clone(), other_user);
        assert!(WorkPolicy::can_create(&other_ctx, &new_work, ()).is_err());
    }

    #[test]
    fn crud_policy_requires_work_lifecycle_role_for_status_change() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let mut patch = make_patch_from_work(&work);
        patch.work_status = WorkStatus::Active;
        patch.publication_date = NaiveDate::from_ymd_opt(2020, 1, 1);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let basic_user = test_user_with_role("work-user", Role::PublisherUser, &org_id);
        let basic_ctx = test_context_with_user(pool.clone(), basic_user);
        assert!(WorkPolicy::can_update(&basic_ctx, &work, &patch, ()).is_err());

        let lifecycle_user = user_with_roles(
            "work-user",
            &org_id,
            &[Role::PublisherUser, Role::WorkLifecycle],
        );
        let lifecycle_ctx = test_context_with_user(pool.clone(), lifecycle_user);
        assert!(WorkPolicy::can_update(&lifecycle_ctx, &work, &patch, ()).is_ok());
    }

    #[test]
    fn crud_policy_requires_work_lifecycle_role_for_withdrawn_date_change() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let mut new_work = make_new_work(imprint.imprint_id);
        new_work.work_status = WorkStatus::Withdrawn;
        new_work.publication_date = NaiveDate::from_ymd_opt(2020, 1, 1);
        new_work.withdrawn_date = NaiveDate::from_ymd_opt(2021, 1, 1);
        let work = Work::create(pool.as_ref(), &new_work).expect("Failed to create work");

        let mut patch = make_patch_from_work(&work);
        patch.withdrawn_date = NaiveDate::from_ymd_opt(2022, 1, 1);

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let basic_user = test_user_with_role("work-user", Role::PublisherUser, &org_id);
        let basic_ctx = test_context_with_user(pool.clone(), basic_user);
        assert!(WorkPolicy::can_update(&basic_ctx, &work, &patch, ()).is_err());

        let lifecycle_user = user_with_roles(
            "work-user",
            &org_id,
            &[Role::PublisherUser, Role::WorkLifecycle],
        );
        let lifecycle_ctx = test_context_with_user(pool.clone(), lifecycle_user);
        assert!(WorkPolicy::can_update(&lifecycle_ctx, &work, &patch, ()).is_ok());
    }

    #[test]
    fn crud_policy_allows_non_lifecycle_update_without_work_lifecycle_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let mut patch = make_patch_from_work(&work);
        patch.reference = Some("Updated reference".to_string());

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let basic_user = test_user_with_role("work-user", Role::PublisherUser, &org_id);
        let basic_ctx = test_context_with_user(pool.clone(), basic_user);

        assert!(WorkPolicy::can_update(&basic_ctx, &work, &patch, ()).is_ok());
    }

    #[test]
    fn crud_policy_rejects_chapter_when_isbn_publication_exists() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        let new_publication =
            make_new_publication_with_isbn(work.work_id, Isbn("978-3-16-148410-0".to_string()));

        Publication::create(pool.as_ref(), &new_publication)
            .expect("Failed to create publication with ISBN");

        let mut patch = make_patch_from_work(&work);
        patch.work_type = WorkType::BookChapter;

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("work-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let result = WorkPolicy::can_update(&ctx, &work, &patch, ());
        assert!(matches!(result, Err(ThothError::ChapterIsbnError)));
    }

    #[test]
    fn crud_policy_prevents_non_superuser_from_unpublishing_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let mut new_work = make_new_work(imprint.imprint_id);
        new_work.work_status = WorkStatus::Active;
        new_work.publication_date = NaiveDate::from_ymd_opt(2020, 1, 1);
        let work = Work::create(pool.as_ref(), &new_work).expect("Failed to create work");

        let mut patch = make_patch_from_work(&work);
        patch.work_status = WorkStatus::Forthcoming;

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let lifecycle_user = user_with_roles(
            "work-user",
            &org_id,
            &[Role::PublisherUser, Role::WorkLifecycle],
        );
        let ctx = test_context_with_user(pool.clone(), lifecycle_user);

        let result = WorkPolicy::can_update(&ctx, &work, &patch, ());
        assert!(matches!(result, Err(ThothError::ThothSetWorkStatusError)));
    }

    #[test]
    fn crud_policy_prevents_non_superuser_from_deleting_published_work() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let new_work = NewWork {
            work_type: WorkType::Monograph,
            work_status: WorkStatus::Active,
            reference: None,
            edition: Some(1),
            imprint_id: imprint.imprint_id,
            doi: None,
            publication_date: NaiveDate::from_ymd_opt(2020, 1, 1),
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

        let work = Work::create(pool.as_ref(), &new_work).expect("Failed to create work");

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("work-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);
        assert!(WorkPolicy::can_delete(&ctx, &work).is_err());

        let super_ctx = test_context_with_user(pool.clone(), test_superuser("work-super"));
        assert!(WorkPolicy::can_delete(&super_ctx, &work).is_ok());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;
    use chrono::NaiveDate;
    use std::str::FromStr;
    use uuid::Uuid;

    use crate::graphql::types::inputs::{Expression, TimeExpression};
    use crate::model::issue::{Issue, NewIssue};
    use crate::model::locale::LocaleCode;
    use crate::model::publication::{NewPublication, Publication, PublicationType};
    use crate::model::r#abstract::{Abstract, AbstractType, NewAbstract};
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_series, create_work, setup_test_db, test_context,
    };
    use crate::model::title::{NewTitle, Title};
    use crate::model::work_relation::{NewWorkRelation, RelationType, WorkRelation};
    use crate::model::{Crud, Doi, Isbn, Timestamp};

    fn make_new_work(imprint_id: Uuid) -> NewWork {
        NewWork {
            work_type: WorkType::Monograph,
            work_status: WorkStatus::Forthcoming,
            reference: None,
            edition: Some(1),
            imprint_id,
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
        }
    }

    fn make_work(
        pool: &crate::db::PgPool,
        imprint_id: Uuid,
        work_type: WorkType,
        work_status: WorkStatus,
        reference: Option<String>,
    ) -> Work {
        let new_work = NewWork {
            work_type,
            work_status,
            reference,
            ..make_new_work(imprint_id)
        };

        Work::create(pool, &new_work).expect("Failed to create work")
    }

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let fetched_work =
            Work::from_id(pool.as_ref(), &work.work_id).expect("Failed to fetch work");
        assert_eq!(work.work_id, fetched_work.work_id);

        let patch = PatchWork {
            work_id: work.work_id,
            work_type: work.work_type,
            work_status: work.work_status,
            reference: Some(format!("Updated {}", Uuid::new_v4())),
            edition: work.edition,
            imprint_id: work.imprint_id,
            doi: work.doi.clone(),
            publication_date: work.publication_date,
            withdrawn_date: work.withdrawn_date,
            place: work.place.clone(),
            page_count: work.page_count,
            page_breakdown: work.page_breakdown.clone(),
            image_count: work.image_count,
            table_count: work.table_count,
            audio_count: work.audio_count,
            video_count: work.video_count,
            license: work.license.clone(),
            copyright_holder: work.copyright_holder.clone(),
            landing_page: work.landing_page.clone(),
            lccn: work.lccn.clone(),
            oclc: work.oclc.clone(),
            general_note: work.general_note.clone(),
            bibliography_note: work.bibliography_note.clone(),
            toc: work.toc.clone(),
            cover_url: work.cover_url.clone(),
            cover_caption: work.cover_caption.clone(),
            first_page: work.first_page.clone(),
            last_page: work.last_page.clone(),
            page_interval: work.page_interval.clone(),
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = work.update(&ctx, &patch).expect("Failed to update work");
        assert_eq!(updated.reference, patch.reference);

        let deleted = updated
            .delete(pool.as_ref())
            .expect("Failed to delete work");
        assert!(Work::from_id(pool.as_ref(), &deleted.work_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        create_work(pool.as_ref(), &imprint);
        create_work(pool.as_ref(), &imprint);

        let order = WorkOrderBy {
            field: WorkField::WorkId,
            direction: Direction::Asc,
        };

        let first = Work::all(
            pool.as_ref(),
            1,
            0,
            None,
            order.clone(),
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch works");
        let second = Work::all(
            pool.as_ref(),
            1,
            1,
            None,
            order,
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch works");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].work_id, second[0].work_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        create_work(pool.as_ref(), &imprint);
        create_work(pool.as_ref(), &imprint);

        let count = Work::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count works");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_count_filters_by_publisher_type_status_and_publication_date() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);

        Work::create(
            pool.as_ref(),
            &NewWork {
                work_type: WorkType::Monograph,
                work_status: WorkStatus::Active,
                publication_date: NaiveDate::from_ymd_opt(2000, 1, 1),
                ..make_new_work(imprint.imprint_id)
            },
        )
        .expect("Failed to create work");
        Work::create(
            pool.as_ref(),
            &NewWork {
                work_type: WorkType::BookChapter,
                work_status: WorkStatus::Forthcoming,
                edition: None,
                publication_date: NaiveDate::from_ymd_opt(2020, 1, 1),
                ..make_new_work(other_imprint.imprint_id)
            },
        )
        .expect("Failed to create work");

        let count_by_publisher = Work::count(
            pool.as_ref(),
            None,
            vec![publisher.publisher_id],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count works by publisher");
        assert_eq!(count_by_publisher, 1);

        let count_by_type = Work::count(
            pool.as_ref(),
            None,
            vec![],
            vec![WorkType::BookChapter],
            vec![],
            None,
            None,
        )
        .expect("Failed to count works by type");
        assert_eq!(count_by_type, 1);

        let count_by_status = Work::count(
            pool.as_ref(),
            None,
            vec![],
            vec![],
            vec![WorkStatus::Active],
            None,
            None,
        )
        .expect("Failed to count works by status");
        assert_eq!(count_by_status, 1);

        let newer_than = TimeExpression {
            timestamp: Timestamp::parse_from_rfc3339("2010-01-01T00:00:00Z").unwrap(),
            expression: Expression::GreaterThan,
        };
        let count_by_date = Work::count(
            pool.as_ref(),
            None,
            vec![],
            vec![],
            vec![],
            Some(newer_than),
            None,
        )
        .expect("Failed to count works by publication date");
        assert_eq!(count_by_date, 1);
    }

    #[test]
    fn crud_filter_matches_reference() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let marker = format!("Ref {}", Uuid::new_v4());
        let matches = make_work(
            pool.as_ref(),
            imprint.imprint_id,
            WorkType::Monograph,
            WorkStatus::Forthcoming,
            Some(marker.clone()),
        );
        make_work(
            pool.as_ref(),
            imprint.imprint_id,
            WorkType::Monograph,
            WorkStatus::Forthcoming,
            Some("Other Ref".to_string()),
        );

        let filtered = Work::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter works");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].work_id, matches.work_id);
    }

    #[test]
    fn crud_filter_param_limits_work_types() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let matches = make_work(
            pool.as_ref(),
            imprint.imprint_id,
            WorkType::Monograph,
            WorkStatus::Forthcoming,
            None,
        );
        make_work(
            pool.as_ref(),
            imprint.imprint_id,
            WorkType::EditedBook,
            WorkStatus::Forthcoming,
            None,
        );

        let filtered = Work::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![WorkType::Monograph],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter works by type");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].work_id, matches.work_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let first = create_work(pool.as_ref(), &imprint);
        let second = create_work(pool.as_ref(), &imprint);
        let mut ids = [first.work_id, second.work_id];
        ids.sort();

        let asc = Work::all(
            pool.as_ref(),
            2,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to order works (asc)");

        let desc = Work::all(
            pool.as_ref(),
            2,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Desc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to order works (desc)");

        assert_eq!(asc[0].work_id, ids[0]);
        assert_eq!(desc[0].work_id, ids[1]);
    }

    #[test]
    fn crud_from_doi_respects_case_and_type_filter() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let doi = Doi::from_str("https://doi.org/10.1234/TEST.DOI").unwrap();
        let new_work = NewWork {
            doi: Some(doi.clone()),
            ..make_new_work(imprint.imprint_id)
        };
        let work = Work::create(pool.as_ref(), &new_work).expect("Failed to create work");

        let lookup = Doi::from_str("https://doi.org/10.1234/test.doi").unwrap();
        let found = Work::from_doi(pool.as_ref(), lookup.clone(), vec![])
            .expect("Failed to fetch work by DOI");
        assert_eq!(found.work_id, work.work_id);

        let filtered_ok = Work::from_doi(pool.as_ref(), lookup.clone(), vec![WorkType::Monograph])
            .expect("Failed to fetch work by DOI with type filter");
        assert_eq!(filtered_ok.work_id, work.work_id);

        let filtered_err = Work::from_doi(pool.as_ref(), lookup, vec![WorkType::EditedBook]);
        assert!(filtered_err.is_err());
    }

    #[test]
    fn crud_can_update_imprint_rejects_work_with_issue() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        assert!(work.can_update_imprint(pool.as_ref()).is_ok());

        let series = create_series(pool.as_ref(), &imprint);
        Issue::create(
            pool.as_ref(),
            &NewIssue {
                series_id: series.series_id,
                work_id: work.work_id,
                issue_ordinal: 1,
            },
        )
        .expect("Failed to create issue");

        assert!(work.can_update_imprint(pool.as_ref()).is_err());
    }

    #[test]
    fn crud_can_be_chapter_rejects_work_with_isbn_publication() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);

        assert!(work.can_be_chapter(pool.as_ref()).is_ok());

        Publication::create(
            pool.as_ref(),
            &NewPublication {
                publication_type: PublicationType::Paperback,
                work_id: work.work_id,
                isbn: Some(Isbn::from_str("9780131103627").unwrap()),
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
            },
        )
        .expect("Failed to create publication");

        assert!(work.can_be_chapter(pool.as_ref()).is_err());
    }

    #[test]
    fn crud_children_returns_has_child_relations() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let parent = create_work(pool.as_ref(), &imprint);
        let child = create_work(pool.as_ref(), &imprint);
        let other = create_work(pool.as_ref(), &imprint);

        WorkRelation::create(
            pool.as_ref(),
            &NewWorkRelation {
                relator_work_id: parent.work_id,
                related_work_id: child.work_id,
                relation_type: RelationType::HasChild,
                relation_ordinal: 1,
            },
        )
        .expect("Failed to create work relation");
        WorkRelation::create(
            pool.as_ref(),
            &NewWorkRelation {
                relator_work_id: parent.work_id,
                related_work_id: other.work_id,
                relation_type: RelationType::HasPart,
                relation_ordinal: 2,
            },
        )
        .expect("Failed to create work relation");

        let children = parent
            .children(pool.as_ref())
            .expect("Failed to load children");

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].work_id, child.work_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let matches = create_work(pool.as_ref(), &imprint);

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        create_work(pool.as_ref(), &other_imprint);

        let filtered = Work::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![publisher.publisher_id],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter works by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].work_id, matches.work_id);
    }

    #[test]
    fn crud_filter_parent_imprint_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let matches = create_work(pool.as_ref(), &imprint);

        let other_imprint = create_imprint(pool.as_ref(), &publisher);
        create_work(pool.as_ref(), &other_imprint);

        let filtered = Work::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            Some(imprint.imprint_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter works by imprint");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].work_id, matches.work_id);
    }

    #[test]
    fn crud_filter_param_limits_work_statuses() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let matches = make_work(
            pool.as_ref(),
            imprint.imprint_id,
            WorkType::Monograph,
            WorkStatus::Forthcoming,
            None,
        );
        Work::create(
            pool.as_ref(),
            &NewWork {
                work_status: WorkStatus::Active,
                publication_date: NaiveDate::from_ymd_opt(2020, 1, 1),
                ..make_new_work(imprint.imprint_id)
            },
        )
        .expect("Failed to create work");

        let filtered = Work::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![WorkStatus::Forthcoming],
            None,
            None,
        )
        .expect("Failed to filter works by status");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].work_id, matches.work_id);
    }

    #[test]
    fn crud_filter_matches_title_and_abstract() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let marker = format!("Marker {}", Uuid::new_v4());

        let work_with_title = create_work(pool.as_ref(), &imprint);
        Title::create(
            pool.as_ref(),
            &NewTitle {
                work_id: work_with_title.work_id,
                locale_code: LocaleCode::En,
                full_title: format!("Title {marker}"),
                title: "Title".to_string(),
                subtitle: None,
                canonical: true,
            },
        )
        .expect("Failed to create title");

        let work_with_abstract = create_work(pool.as_ref(), &imprint);
        Abstract::create(
            pool.as_ref(),
            &NewAbstract {
                work_id: work_with_abstract.work_id,
                content: format!("Abstract {marker}"),
                locale_code: LocaleCode::En,
                abstract_type: AbstractType::Long,
                canonical: true,
            },
        )
        .expect("Failed to create abstract");

        let filtered = Work::all(
            pool.as_ref(),
            10,
            0,
            Some(marker.clone()),
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter works by title/abstract");

        assert_eq!(filtered.len(), 2);

        let count = Work::count(
            pool.as_ref(),
            Some(marker),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count works by title/abstract");

        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_param_limits_publication_date() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);

        Work::create(
            pool.as_ref(),
            &NewWork {
                publication_date: NaiveDate::from_ymd_opt(2000, 1, 1),
                ..make_new_work(imprint.imprint_id)
            },
        )
        .expect("Failed to create work");
        Work::create(
            pool.as_ref(),
            &NewWork {
                publication_date: NaiveDate::from_ymd_opt(2020, 1, 1),
                ..make_new_work(imprint.imprint_id)
            },
        )
        .expect("Failed to create work");

        let greater_than = TimeExpression {
            timestamp: Timestamp::parse_from_rfc3339("2010-01-01T00:00:00Z").unwrap(),
            expression: Expression::GreaterThan,
        };
        let less_than = TimeExpression {
            timestamp: Timestamp::parse_from_rfc3339("2010-01-01T00:00:00Z").unwrap(),
            expression: Expression::LessThan,
        };

        let newer = Work::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            Some(greater_than),
            None,
        )
        .expect("Failed to filter works by publication date (gt)");

        let older = Work::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            Some(less_than),
            None,
        )
        .expect("Failed to filter works by publication date (lt)");

        assert_eq!(newer.len(), 1);
        assert_eq!(older.len(), 1);
    }

    #[test]
    fn crud_filter_param_limits_updated_at_with_relations() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        create_work(pool.as_ref(), &imprint);
        create_work(pool.as_ref(), &imprint);

        let greater_than = TimeExpression {
            timestamp: Timestamp::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap(),
            expression: Expression::GreaterThan,
        };
        let less_than = TimeExpression {
            timestamp: Timestamp::parse_from_rfc3339("3000-01-01T00:00:00Z").unwrap(),
            expression: Expression::LessThan,
        };

        let count_newer = Work::count(
            pool.as_ref(),
            None,
            vec![],
            vec![],
            vec![],
            None,
            Some(greater_than),
        )
        .expect("Failed to count works by updated_at_with_relations (gt)");

        let count_older = Work::count(
            pool.as_ref(),
            None,
            vec![],
            vec![],
            vec![],
            None,
            Some(less_than),
        )
        .expect("Failed to count works by updated_at_with_relations (lt)");

        assert_eq!(count_newer, 2);
        assert_eq!(count_older, 2);
    }

    #[test]
    fn crud_filter_param_limits_updated_at_with_relations_in_all() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        create_work(pool.as_ref(), &imprint);
        create_work(pool.as_ref(), &imprint);

        let greater_than = TimeExpression {
            timestamp: Timestamp::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap(),
            expression: Expression::GreaterThan,
        };
        let less_than = TimeExpression {
            timestamp: Timestamp::parse_from_rfc3339("3000-01-01T00:00:00Z").unwrap(),
            expression: Expression::LessThan,
        };

        let newer = Work::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            Some(greater_than),
        )
        .expect("Failed to filter works by updated_at_with_relations (gt)");

        let older = Work::all(
            pool.as_ref(),
            10,
            0,
            None,
            WorkOrderBy {
                field: WorkField::WorkId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            Some(less_than),
        )
        .expect("Failed to filter works by updated_at_with_relations (lt)");

        assert_eq!(newer.len(), 2);
        assert_eq!(older.len(), 2);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        create_work(pool.as_ref(), &imprint);
        create_work(pool.as_ref(), &imprint);

        let fields: Vec<fn() -> WorkField> = vec![
            || WorkField::WorkId,
            || WorkField::WorkType,
            || WorkField::WorkStatus,
            || WorkField::FullTitle,
            || WorkField::Title,
            || WorkField::Subtitle,
            || WorkField::Reference,
            || WorkField::Edition,
            || WorkField::Doi,
            || WorkField::PublicationDate,
            || WorkField::WithdrawnDate,
            || WorkField::Place,
            || WorkField::PageCount,
            || WorkField::PageBreakdown,
            || WorkField::FirstPage,
            || WorkField::LastPage,
            || WorkField::PageInterval,
            || WorkField::ImageCount,
            || WorkField::TableCount,
            || WorkField::AudioCount,
            || WorkField::VideoCount,
            || WorkField::License,
            || WorkField::CopyrightHolder,
            || WorkField::LandingPage,
            || WorkField::Lccn,
            || WorkField::Oclc,
            || WorkField::ShortAbstract,
            || WorkField::LongAbstract,
            || WorkField::GeneralNote,
            || WorkField::BibliographyNote,
            || WorkField::Toc,
            || WorkField::CoverUrl,
            || WorkField::CoverCaption,
            || WorkField::CreatedAt,
            || WorkField::UpdatedAt,
            || WorkField::UpdatedAtWithRelations,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Work::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    WorkOrderBy {
                        field: field(),
                        direction,
                    },
                    vec![],
                    None,
                    None,
                    vec![],
                    vec![],
                    None,
                    None,
                )
                .expect("Failed to order works");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
