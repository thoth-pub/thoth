use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_new_series(
    imprint_id: Uuid,
    series_type: SeriesType,
    series_name: impl Into<String>,
) -> NewSeries {
    NewSeries {
        series_type,
        series_name: series_name.into(),
        issn_print: None,
        issn_digital: None,
        series_url: None,
        series_description: None,
        series_cfp_url: None,
        imprint_id,
    }
}

fn make_patch_series(
    series: &Series,
    series_type: SeriesType,
    series_name: impl Into<String>,
) -> PatchSeries {
    PatchSeries {
        series_id: series.series_id,
        series_type,
        series_name: series_name.into(),
        issn_print: series.issn_print.clone(),
        issn_digital: series.issn_digital.clone(),
        series_url: series.series_url.clone(),
        series_description: series.series_description.clone(),
        series_cfp_url: series.series_cfp_url.clone(),
        imprint_id: series.imprint_id,
    }
}

fn make_series(
    pool: &crate::db::PgPool,
    imprint_id: Uuid,
    series_type: SeriesType,
    name: String,
) -> Series {
    let new_series = make_new_series(imprint_id, series_type, name);

    Series::create(pool, &new_series).expect("Failed to create series")
}

mod defaults {
    use super::*;

    #[test]
    fn seriestype_default_is_book_series() {
        let seriestype: SeriesType = Default::default();
        assert_eq!(seriestype, SeriesType::BookSeries);
    }

    #[test]
    fn seriesfield_default_is_series_name() {
        let seriesfield: SeriesField = Default::default();
        assert_eq!(seriesfield, SeriesField::SeriesName);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn seriestype_display_formats_expected_strings() {
        assert_eq!(format!("{}", SeriesType::Journal), "Journal");
        assert_eq!(format!("{}", SeriesType::BookSeries), "Book Series");
    }

    #[test]
    fn seriesfield_display_formats_expected_strings() {
        assert_eq!(format!("{}", SeriesField::SeriesId), "ID");
        assert_eq!(format!("{}", SeriesField::SeriesType), "SeriesType");
        assert_eq!(format!("{}", SeriesField::SeriesName), "Series");
        assert_eq!(format!("{}", SeriesField::IssnPrint), "ISSNPrint");
        assert_eq!(format!("{}", SeriesField::IssnDigital), "ISSNDigital");
        assert_eq!(format!("{}", SeriesField::SeriesUrl), "SeriesURL");
        assert_eq!(
            format!("{}", SeriesField::SeriesDescription),
            "SeriesDescription"
        );
        assert_eq!(format!("{}", SeriesField::SeriesCfpUrl), "SeriesCFPURL");
        assert_eq!(format!("{}", SeriesField::CreatedAt), "CreatedAt");
        assert_eq!(format!("{}", SeriesField::UpdatedAt), "UpdatedAt");
    }

    #[test]
    fn seriestype_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(
            SeriesType::from_str("Journal").unwrap(),
            SeriesType::Journal
        );
        assert_eq!(
            SeriesType::from_str("Book Series").unwrap(),
            SeriesType::BookSeries
        );

        assert!(SeriesType::from_str("bookseries").is_err());
        assert!(SeriesType::from_str("Collection").is_err());
    }

    #[test]
    fn seriesfield_fromstr_parses_expected_values() {
        use std::str::FromStr;
        assert_eq!(SeriesField::from_str("ID").unwrap(), SeriesField::SeriesId);
        assert_eq!(
            SeriesField::from_str("SeriesType").unwrap(),
            SeriesField::SeriesType
        );
        assert_eq!(
            SeriesField::from_str("Series").unwrap(),
            SeriesField::SeriesName
        );
        assert_eq!(
            SeriesField::from_str("ISSNPrint").unwrap(),
            SeriesField::IssnPrint
        );
        assert_eq!(
            SeriesField::from_str("ISSNDigital").unwrap(),
            SeriesField::IssnDigital
        );
        assert_eq!(
            SeriesField::from_str("SeriesURL").unwrap(),
            SeriesField::SeriesUrl
        );
        assert_eq!(
            SeriesField::from_str("SeriesDescription").unwrap(),
            SeriesField::SeriesDescription
        );
        assert_eq!(
            SeriesField::from_str("SeriesCFPURL").unwrap(),
            SeriesField::SeriesCfpUrl
        );
        assert_eq!(
            SeriesField::from_str("CreatedAt").unwrap(),
            SeriesField::CreatedAt
        );
        assert_eq!(
            SeriesField::from_str("UpdatedAt").unwrap(),
            SeriesField::UpdatedAt
        );
        assert!(SeriesField::from_str("SeriesID").is_err());
        assert!(SeriesField::from_str("Publisher").is_err());
        assert!(SeriesField::from_str("Issues").is_err());
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let series: Series = Default::default();
        assert_eq!(series.pk(), series.series_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let series: Series = Default::default();
        let user_id = "123456".to_string();
        let new_series_history = series.new_history_entry(&user_id);
        assert_eq!(new_series_history.series_id, series.series_id);
        assert_eq!(new_series_history.user_id, user_id);
        assert_eq!(
            new_series_history.data,
            serde_json::Value::String(serde_json::to_string(&series).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::series::policy::SeriesPolicy;
    use crate::model::tests::db::{
        create_imprint, create_publisher, setup_test_db, test_context_with_user,
        test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("series-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let imprint = create_imprint(pool.as_ref(), &publisher);
        let new_series = make_new_series(imprint.imprint_id, SeriesType::Journal, "Policy Series");

        let series = Series::create(pool.as_ref(), &new_series).expect("Failed to create");
        let patch = make_patch_series(&series, series.series_type, "Updated Policy Series");

        assert!(SeriesPolicy::can_create(&ctx, &new_series, ()).is_ok());
        assert!(SeriesPolicy::can_update(&ctx, &series, &patch, ()).is_ok());
        assert!(SeriesPolicy::can_delete(&ctx, &series).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let series = make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            "Policy Series".to_string(),
        );
        let patch = make_patch_series(&series, series.series_type, "Updated Policy Series");

        let user = test_user_with_role("series-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_series = make_new_series(imprint.imprint_id, SeriesType::Journal, "Policy Series");

        assert!(SeriesPolicy::can_create(&ctx, &new_series, ()).is_err());
        assert!(SeriesPolicy::can_update(&ctx, &series, &patch, ()).is_err());
        assert!(SeriesPolicy::can_delete(&ctx, &series).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::model::tests::db::{create_imprint, create_publisher, setup_test_db, test_context};
    use crate::model::Crud;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);

        let new_series = make_new_series(
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );

        let series = Series::create(pool.as_ref(), &new_series).expect("Failed to create");
        let fetched = Series::from_id(pool.as_ref(), &series.series_id).expect("Failed to fetch");
        assert_eq!(series.series_id, fetched.series_id);

        let patch = make_patch_series(&series, SeriesType::BookSeries, "Updated Series");

        let ctx = test_context(pool.clone(), "test-user");
        let updated = series.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.series_name, patch.series_name);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Series::from_id(pool.as_ref(), &deleted.series_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );

        let order = SeriesOrderBy {
            field: SeriesField::SeriesId,
            direction: Direction::Asc,
        };

        let first = Series::all(
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
        .expect("Failed to fetch series");
        let second = Series::all(
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
        .expect("Failed to fetch series");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].series_id, second[0].series_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );

        let count = Series::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count series");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_count_filters_by_series_type() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::BookSeries,
            format!("Series {}", Uuid::new_v4()),
        );

        let count = Series::count(
            pool.as_ref(),
            None,
            vec![],
            vec![SeriesType::Journal],
            vec![],
            None,
            None,
        )
        .expect("Failed to count series by type");
        assert_eq!(count, 1);
    }

    #[test]
    fn crud_filter_matches_series_name() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let marker = format!("Filter {}", Uuid::new_v4());
        let matches = make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {marker}"),
        );
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            "Other Series".to_string(),
        );

        let filtered = Series::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            SeriesOrderBy {
                field: SeriesField::SeriesId,
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
        .expect("Failed to filter series");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].series_id, matches.series_id);
    }

    #[test]
    fn crud_filter_param_limits_series_types() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let matches = make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::BookSeries,
            format!("Series {}", Uuid::new_v4()),
        );

        let filtered = Series::all(
            pool.as_ref(),
            10,
            0,
            None,
            SeriesOrderBy {
                field: SeriesField::SeriesId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![SeriesType::Journal],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter series by type");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].series_id, matches.series_id);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let matches = make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );

        let other_publisher = create_publisher(pool.as_ref());
        let other_imprint = create_imprint(pool.as_ref(), &other_publisher);
        make_series(
            pool.as_ref(),
            other_imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );

        let filtered = Series::all(
            pool.as_ref(),
            10,
            0,
            None,
            SeriesOrderBy {
                field: SeriesField::SeriesId,
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
        .expect("Failed to filter series by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].series_id, matches.series_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let first = make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );
        let second = make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );
        let mut ids = [first.series_id, second.series_id];
        ids.sort();

        let asc = Series::all(
            pool.as_ref(),
            2,
            0,
            None,
            SeriesOrderBy {
                field: SeriesField::SeriesId,
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
        .expect("Failed to order series (asc)");

        let desc = Series::all(
            pool.as_ref(),
            2,
            0,
            None,
            SeriesOrderBy {
                field: SeriesField::SeriesId,
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
        .expect("Failed to order series (desc)");

        assert_eq!(asc[0].series_id, ids[0]);
        assert_eq!(desc[0].series_id, ids[1]);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::Journal,
            format!("Series {}", Uuid::new_v4()),
        );
        make_series(
            pool.as_ref(),
            imprint.imprint_id,
            SeriesType::BookSeries,
            format!("Series {}", Uuid::new_v4()),
        );

        let fields: Vec<fn() -> SeriesField> = vec![
            || SeriesField::SeriesId,
            || SeriesField::SeriesType,
            || SeriesField::SeriesName,
            || SeriesField::IssnPrint,
            || SeriesField::IssnDigital,
            || SeriesField::SeriesUrl,
            || SeriesField::SeriesDescription,
            || SeriesField::SeriesCfpUrl,
            || SeriesField::CreatedAt,
            || SeriesField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Series::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    SeriesOrderBy {
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
                .expect("Failed to order series");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
