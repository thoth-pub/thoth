use super::*;

// ---------------------------------------------------------------------------
// Normalisation (no database)
// ---------------------------------------------------------------------------

fn digits(input: &str) -> Option<String> {
    normalise_isbn(input).map(|n| n.digits)
}

#[test]
fn isbn13_forms_normalise_to_the_same_digits() {
    for input in [
        "9781783745005",
        "978-1-78374-500-5",
        "978 1 78374 500 5",
        " ISBN 978-1-78374-500-5 ",
        "isbn:9781783745005",
        "ISBN-13: 978-1-78374-500-5",
        "978\u{2010}1\u{2010}78374\u{2010}500\u{2010}5",
        "978\u{a0}1783745005",
    ] {
        assert_eq!(digits(input).as_deref(), Some("9781783745005"), "{input}");
    }
}

#[test]
fn isbn10_converts_to_isbn13() {
    assert_eq!(digits("1783745002").as_deref(), Some("9781783745005"));
    assert_eq!(digits("1-78374-500-2").as_deref(), Some("9781783745005"));
    assert_eq!(
        digits("ISBN 0-19-853453-1").as_deref(),
        Some("9780198534532")
    );
}

#[test]
fn isbn10_with_x_check_digit() {
    assert_eq!(digits("080442957X").as_deref(), Some("9780804429573"));
    assert_eq!(digits("080442957x").as_deref(), Some("9780804429573"));
}

#[test]
fn bad_input_is_invalid() {
    for input in [
        "",
        "   ",
        "ISBN",
        "9781783745006",  // bad ISBN-13 check digit
        "1783745003",     // bad ISBN-10 check digit
        "978178374500",   // 12 digits
        "97817837450055", // 14 digits
        "9771783745005",  // not 978/979 prefix (ISSN range)
        "978178374500X",  // X only valid in ISBN-10
        "abcdefghijklm",
        "9781783745005 9781783745005",
        "978-1-78374-500-5 (pdf)",
    ] {
        assert_eq!(normalise_isbn(input), None, "{input}");
    }
}

#[test]
fn display_form_is_hyphenated_when_possible() {
    let n = normalise_isbn("9781783745005").unwrap();
    assert_eq!(n.display, "978-1-78374-500-5");
}

// ---------------------------------------------------------------------------
// Matching (database)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod db {
    use super::*;
    use crate::model::publication::{NewPublication, Publication, PublicationType};
    use crate::model::tests::db::{create_imprint, create_publisher, create_work, setup_test_db};
    use crate::model::work::WorkType;
    use crate::model::Crud;
    use diesel::sql_types::Text;

    fn isbn13(body: &str) -> String {
        // body: first 12 digits; append the check digit.
        let sum: u32 = body
            .chars()
            .filter_map(|c| c.to_digit(10))
            .enumerate()
            .map(|(i, d)| if i % 2 == 0 { d } else { d * 3 })
            .sum();
        format!("{body}{}", (10 - sum % 10) % 10)
    }

    fn add_publication(
        pool: &PgPool,
        work_id: Uuid,
        publication_type: PublicationType,
        isbn: Option<&str>,
    ) {
        let new = NewPublication {
            publication_type,
            work_id,
            isbn: isbn.map(|i| Isbn::from_str(i).expect("valid test ISBN")),
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
        Publication::create(pool, &new).expect("publication");
    }

    fn set_doi(pool: &PgPool, work_id: Uuid, doi: &str) {
        let mut c = pool.get().unwrap();
        sql_query("UPDATE work SET doi = $1 WHERE work_id = $2")
            .bind::<Text, _>(doi)
            .bind::<SqlUuid, _>(work_id)
            .execute(&mut c)
            .expect("set doi");
    }

    #[test]
    fn matches_every_status_in_input_order() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);

        // w1: print ISBN on a paperback, PDF ISBN on a PDF, one DOI.
        let w1 = create_work(&pool, &imprint);
        set_doi(&pool, w1.work_id, "https://doi.org/10.1234/w1");
        let paper = isbn13("978178374500");
        let pdf = isbn13("978178374501");
        add_publication(&pool, w1.work_id, PublicationType::Paperback, Some(&paper));
        add_publication(&pool, w1.work_id, PublicationType::Pdf, Some(&pdf));

        // w2: no DOI.
        let w2 = create_work(&pool, &imprint);
        let nodoi = isbn13("978178374502");
        add_publication(&pool, w2.work_id, PublicationType::Hardback, Some(&nodoi));

        // w3 and w4 share an ISBN (data error): ambiguous.
        let w3 = create_work(&pool, &imprint);
        let w4 = create_work(&pool, &imprint);
        set_doi(&pool, w3.work_id, "https://doi.org/10.1234/w3");
        set_doi(&pool, w4.work_id, "https://doi.org/10.1234/w4");
        let dup = isbn13("978178374503");
        add_publication(&pool, w3.work_id, PublicationType::Epub, Some(&dup));
        add_publication(&pool, w4.work_id, PublicationType::Html, Some(&dup));

        // w5: the same ISBN on two publications of one work: one match.
        let w5 = create_work(&pool, &imprint);
        set_doi(&pool, w5.work_id, "https://doi.org/10.1234/w5");
        let both = isbn13("978178374504");
        add_publication(&pool, w5.work_id, PublicationType::Xml, Some(&both));
        add_publication(&pool, w5.work_id, PublicationType::Mobi, Some(&both));

        let unknown = isbn13("978178374599");
        let hyphenated_pdf = Isbn::from_str(&pdf).unwrap().to_string();
        let isbn10_of_paper = {
            // ISBN-10 for 978-1-78374-500-5 is 1783745002
            "1783745002".to_string()
        };
        assert_eq!(isbn13("978178374500"), "9781783745005");

        let inputs: Vec<String> = vec![
            hyphenated_pdf.clone(),
            isbn10_of_paper,
            nodoi.clone(),
            dup.clone(),
            both.clone(),
            unknown.clone(),
            "not an isbn".to_string(),
            pdf.clone(),
        ];
        let out = work_isbn_matches(&pool, &inputs, &[]).expect("lookup");
        assert_eq!(out.len(), inputs.len());
        for (o, i) in out.iter().zip(&inputs) {
            assert_eq!(&o.isbn, i);
        }
        assert_eq!(out[0].status, IsbnMatchStatus::Matched);
        assert_eq!(out[0].work_id, Some(w1.work_id));
        assert_eq!(
            out[0].work_doi,
            Some(Doi::from_str("https://doi.org/10.1234/w1").unwrap())
        );
        assert_eq!(out[0].work_type, Some(WorkType::Monograph));
        assert_eq!(out[0].publisher_id, Some(publisher.publisher_id));
        // ISBN-10 of the paperback resolves to the same work.
        assert_eq!(out[1].status, IsbnMatchStatus::Matched);
        assert_eq!(out[1].work_id, Some(w1.work_id));
        assert_eq!(out[2].status, IsbnMatchStatus::NoDoi);
        assert_eq!(out[2].work_id, Some(w2.work_id));
        assert_eq!(out[2].work_doi, None);
        assert_eq!(out[3].status, IsbnMatchStatus::Ambiguous);
        assert_eq!(out[3].work_id, None);
        assert_eq!(out[4].status, IsbnMatchStatus::Matched);
        assert_eq!(out[4].work_id, Some(w5.work_id));
        assert_eq!(out[5].status, IsbnMatchStatus::Unknown);
        assert!(out[5].normalised_isbn.is_some());
        assert_eq!(out[6].status, IsbnMatchStatus::Invalid);
        assert_eq!(out[6].normalised_isbn, None);
        // Duplicate input gives a duplicate entry (hyphenated and plain).
        assert_eq!(out[7].work_id, out[0].work_id);
    }

    #[test]
    fn publisher_filter_limits_scope() {
        let (_guard, pool) = setup_test_db();
        let p1 = create_publisher(&pool);
        let p2 = create_publisher(&pool);
        let i1 = create_imprint(&pool, &p1);
        let i2 = create_imprint(&pool, &p2);
        let w1 = create_work(&pool, &i1);
        let w2 = create_work(&pool, &i2);
        set_doi(&pool, w1.work_id, "https://doi.org/10.1234/p1");
        set_doi(&pool, w2.work_id, "https://doi.org/10.1234/p2");
        let a = isbn13("978178374510");
        let b = isbn13("978178374511");
        add_publication(&pool, w1.work_id, PublicationType::Pdf, Some(&a));
        add_publication(&pool, w2.work_id, PublicationType::Pdf, Some(&b));

        let inputs = vec![a.clone(), b.clone()];
        let scoped = work_isbn_matches(&pool, &inputs, &[p1.publisher_id]).unwrap();
        assert_eq!(scoped[0].status, IsbnMatchStatus::Matched);
        assert_eq!(scoped[1].status, IsbnMatchStatus::Unknown);

        // The same ISBN held by two publishers is ambiguous without a filter
        // and unambiguous with one.
        let c = isbn13("978178374512");
        add_publication(&pool, w1.work_id, PublicationType::Epub, Some(&c));
        add_publication(&pool, w2.work_id, PublicationType::Epub, Some(&c));
        let c_in = vec![c.clone()];
        assert_eq!(
            work_isbn_matches(&pool, &c_in, &[]).unwrap()[0].status,
            IsbnMatchStatus::Ambiguous
        );
        assert_eq!(
            work_isbn_matches(&pool, &c_in, &[p2.publisher_id]).unwrap()[0].work_id,
            Some(w2.work_id)
        );
    }

    #[test]
    fn empty_and_oversized_inputs() {
        let (_guard, pool) = setup_test_db();
        assert!(work_isbn_matches(&pool, &[], &[]).unwrap().is_empty());
        let too_many: Vec<String> = (0..=ISBN_LOOKUP_MAX_INPUTS)
            .map(|_| "9781783745005".to_string())
            .collect();
        assert!(work_isbn_matches(&pool, &too_many, &[]).is_err());
        let exactly: Vec<String> = too_many[..ISBN_LOOKUP_MAX_INPUTS].to_vec();
        assert_eq!(
            work_isbn_matches(&pool, &exactly, &[]).unwrap().len(),
            ISBN_LOOKUP_MAX_INPUTS
        );
    }

    #[test]
    fn unhyphenated_storage_still_matches() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(&pool);
        let imprint = create_imprint(&pool, &publisher);
        let w = create_work(&pool, &imprint);
        set_doi(&pool, w.work_id, "https://doi.org/10.1234/raw");
        let isbn = isbn13("978178374520");
        add_publication(&pool, w.work_id, PublicationType::Pdf, Some(&isbn));
        let mut c = pool.get().unwrap();
        sql_query("UPDATE publication SET isbn = replace(isbn, '-', '') WHERE work_id = $1")
            .bind::<SqlUuid, _>(w.work_id)
            .execute(&mut c)
            .ok();
        drop(c);
        let out = work_isbn_matches(&pool, &[isbn], &[]).unwrap();
        assert_eq!(out[0].status, IsbnMatchStatus::Matched);
    }
}
