#![cfg(feature = "backend")]

use super::*;

use crate::graphql::types::inputs::{Convert, LengthUnit, WeightUnit};
use crate::markup::MarkupFormat;
use crate::model::tests::db as test_db;
use crate::model::{
    additional_resource::{AdditionalResource, NewAdditionalResource, ResourceType},
    affiliation::{Affiliation, NewAffiliation, PatchAffiliation},
    award::{Award, AwardRole, NewAward},
    biography::{Biography, NewBiography, PatchBiography},
    book_review::{BookReview, NewBookReview},
    contact::{Contact, ContactType, NewContact, PatchContact},
    contribution::{Contribution, ContributionType, NewContribution, PatchContribution},
    contributor::{Contributor, NewContributor, PatchContributor},
    endorsement::{Endorsement, NewEndorsement, PatchEndorsement},
    funding::{Funding, NewFunding, PatchFunding},
    imprint::{Imprint, NewImprint, PatchImprint},
    institution::{Institution, NewInstitution, PatchInstitution},
    issue::{Issue, NewIssue, PatchIssue},
    language::{Language, LanguageCode, LanguageRelation, NewLanguage, PatchLanguage},
    locale::LocaleCode,
    location::{Location, LocationPlatform, NewLocation, PatchLocation},
    price::{CurrencyCode, NewPrice, PatchPrice, Price},
    publication::{NewPublication, PatchPublication, Publication, PublicationType},
    publisher::{NewPublisher, PatchPublisher, Publisher},
    r#abstract::{Abstract, AbstractType, NewAbstract, PatchAbstract},
    reference::{NewReference, PatchReference, Reference},
    series::{NewSeries, PatchSeries, Series, SeriesType},
    subject::{NewSubject, PatchSubject, Subject, SubjectType},
    title::{NewTitle, PatchTitle, Title},
    work::{NewWork, PatchWork, Work, WorkStatus, WorkType},
    work_relation::{NewWorkRelation, PatchWorkRelation, RelationType, WorkRelation},
    CountryCode, Crud, Doi, Isbn, Orcid, Ror,
};
use crate::policy::{PolicyContext, Role};
use chrono::NaiveDate;
use juniper::{DefaultScalarValue, ToInputValue, Variables};
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;

fn execute_graphql(
    schema: &Schema,
    context: &Context,
    query: &str,
    variables: Option<Variables>,
) -> JsonValue {
    let vars = variables.unwrap_or_default();
    let (value, errors) = juniper::execute_sync(query, None, schema, &vars, context)
        .expect("GraphQL execution failed");
    if !errors.is_empty() {
        panic!("GraphQL errors: {errors:?}");
    }
    serde_json::to_value(value).expect("Failed to serialize GraphQL response")
}

fn insert_var<T>(vars: &mut Variables, name: &str, value: T)
where
    T: ToInputValue<DefaultScalarValue>,
{
    vars.insert(name.to_string(), value.to_input_value());
}

fn json_uuid(value: &JsonValue) -> Uuid {
    let raw = value
        .as_str()
        .unwrap_or_else(|| panic!("Expected uuid string, got {value:?}"));
    Uuid::parse_str(raw).expect("Failed to parse uuid")
}

fn create_with_data<T>(
    schema: &Schema,
    context: &Context,
    mutation: &str,
    input_type: &str,
    return_fields: &str,
    data: T,
) -> JsonValue
where
    T: ToInputValue<DefaultScalarValue>,
{
    let query = format!(
        "mutation($data: {input_type}!) {{ {mutation}(data: $data) {{ {return_fields} }} }}"
    );
    let mut vars = Variables::new();
    insert_var(&mut vars, "data", data);
    let data = execute_graphql(schema, context, &query, Some(vars));
    data.get(mutation)
        .cloned()
        .unwrap_or_else(|| panic!("Missing mutation result for {mutation}"))
}

fn create_with_data_and_markup<T>(
    schema: &Schema,
    context: &Context,
    mutation: &str,
    input_type: &str,
    return_fields: &str,
    data: T,
    markup_format: MarkupFormat,
) -> JsonValue
where
    T: ToInputValue<DefaultScalarValue>,
{
    let query = format!(
        "mutation($data: {input_type}!, $markup: MarkupFormat!) {{ {mutation}(markupFormat: $markup, data: $data) {{ {return_fields} }} }}"
    );
    let mut vars = Variables::new();
    insert_var(&mut vars, "data", data);
    insert_var(&mut vars, "markup", markup_format);
    let data = execute_graphql(schema, context, &query, Some(vars));
    data.get(mutation)
        .cloned()
        .unwrap_or_else(|| panic!("Missing mutation result for {mutation}"))
}

fn update_with_data<T>(
    schema: &Schema,
    context: &Context,
    mutation: &str,
    input_type: &str,
    return_fields: &str,
    data: T,
) -> JsonValue
where
    T: ToInputValue<DefaultScalarValue>,
{
    let query = format!(
        "mutation($data: {input_type}!) {{ {mutation}(data: $data) {{ {return_fields} }} }}"
    );
    let mut vars = Variables::new();
    insert_var(&mut vars, "data", data);
    let data = execute_graphql(schema, context, &query, Some(vars));
    data.get(mutation)
        .cloned()
        .unwrap_or_else(|| panic!("Missing mutation result for {mutation}"))
}

fn update_with_data_and_markup<T>(
    schema: &Schema,
    context: &Context,
    mutation: &str,
    input_type: &str,
    return_fields: &str,
    data: T,
    markup_format: MarkupFormat,
) -> JsonValue
where
    T: ToInputValue<DefaultScalarValue>,
{
    let query = format!(
        "mutation($data: {input_type}!, $markup: MarkupFormat!) {{ {mutation}(markupFormat: $markup, data: $data) {{ {return_fields} }} }}"
    );
    let mut vars = Variables::new();
    insert_var(&mut vars, "data", data);
    insert_var(&mut vars, "markup", markup_format);
    let data = execute_graphql(schema, context, &query, Some(vars));
    data.get(mutation)
        .cloned()
        .unwrap_or_else(|| panic!("Missing mutation result for {mutation}"))
}

fn delete_with_id(
    schema: &Schema,
    context: &Context,
    mutation: &str,
    arg_name: &str,
    id: Uuid,
    return_fields: &str,
) -> JsonValue {
    let query =
        format!("mutation($id: Uuid!) {{ {mutation}({arg_name}: $id) {{ {return_fields} }} }}");
    let mut vars = Variables::new();
    insert_var(&mut vars, "id", id);
    let data = execute_graphql(schema, context, &query, Some(vars));
    data.get(mutation)
        .cloned()
        .unwrap_or_else(|| panic!("Missing mutation result for {mutation}"))
}

fn move_with_ordinal(
    schema: &Schema,
    context: &Context,
    mutation: &str,
    arg_name: &str,
    id: Uuid,
    new_ordinal: i32,
    return_fields: &str,
) -> JsonValue {
    let query = format!(
        "mutation($id: Uuid!, $ordinal: Int!) {{ {mutation}({arg_name}: $id, newOrdinal: $ordinal) {{ {return_fields} }} }}"
    );
    let mut vars = Variables::new();
    insert_var(&mut vars, "id", id);
    insert_var(&mut vars, "ordinal", new_ordinal);
    let data = execute_graphql(schema, context, &query, Some(vars));
    data.get(mutation)
        .cloned()
        .unwrap_or_else(|| panic!("Missing mutation result for {mutation}"))
}

fn unique(label: &str) -> String {
    format!("{label}-{}", Uuid::new_v4())
}

fn make_new_publisher(org_id: &str) -> NewPublisher {
    NewPublisher {
        publisher_name: unique("Publisher"),
        publisher_shortname: Some("TP".to_string()),
        publisher_url: Some("https://example.com/publisher".to_string()),
        zitadel_id: Some(org_id.to_string()),
        accessibility_statement: Some("Accessibility statement".to_string()),
        accessibility_report_url: Some("https://example.com/report".to_string()),
    }
}

fn make_new_imprint(publisher_id: Uuid) -> NewImprint {
    NewImprint {
        publisher_id,
        imprint_name: unique("Imprint"),
        imprint_url: Some("https://example.com/imprint".to_string()),
        crossmark_doi: None,
        s3_bucket: None,
        cdn_domain: None,
        cloudfront_dist_id: None,
        default_currency: None,
        default_place: None,
        default_locale: None,
    }
}

fn make_new_book_work(imprint_id: Uuid, doi: Doi) -> NewWork {
    NewWork {
        work_type: WorkType::Monograph,
        work_status: WorkStatus::Active,
        reference: Some("REF-001".to_string()),
        edition: Some(1),
        imprint_id,
        doi: Some(doi),
        publication_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        withdrawn_date: None,
        place: Some("Test Place".to_string()),
        page_count: Some(123),
        page_breakdown: Some("xii+123".to_string()),
        image_count: Some(10),
        table_count: Some(2),
        audio_count: Some(0),
        video_count: Some(0),
        license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
        copyright_holder: Some("Test Holder".to_string()),
        landing_page: Some("https://example.com/book".to_string()),
        lccn: Some("LCCN123".to_string()),
        oclc: Some("OCLC123".to_string()),
        general_note: Some("General note".to_string()),
        bibliography_note: Some("Bibliography note".to_string()),
        toc: Some("TOC".to_string()),
        resources_description: None,
        cover_url: Some("https://example.com/cover".to_string()),
        cover_caption: Some("Cover caption".to_string()),
        first_page: None,
        last_page: None,
        page_interval: None,
    }
}

fn make_new_work(imprint_id: Uuid, work_type: WorkType, doi: Doi) -> NewWork {
    let edition = match work_type {
        WorkType::BookChapter => None,
        _ => Some(1),
    };

    NewWork {
        work_type,
        work_status: WorkStatus::Active,
        reference: None,
        edition,
        imprint_id,
        doi: Some(doi),
        publication_date: Some(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
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
    }
}

fn make_new_title(work_id: Uuid, canonical: bool, subtitle: Option<&str>) -> NewTitle {
    let title = unique("Title");
    let subtitle = subtitle.map(|s| s.to_string());
    let full_title = match &subtitle {
        Some(sub) => format!("{title}: {sub}"),
        None => title.clone(),
    };

    NewTitle {
        work_id,
        locale_code: LocaleCode::En,
        full_title,
        title,
        subtitle,
        canonical,
    }
}

fn make_new_abstract(
    work_id: Uuid,
    abstract_type: AbstractType,
    canonical: bool,
    content: &str,
) -> NewAbstract {
    NewAbstract {
        work_id,
        content: content.to_string(),
        locale_code: LocaleCode::En,
        abstract_type,
        canonical,
    }
}

fn make_new_contributor() -> NewContributor {
    let suffix = unique("Contributor");
    NewContributor {
        first_name: Some("Test".to_string()),
        last_name: suffix.clone(),
        full_name: format!("Test {suffix}"),
        orcid: None,
        website: Some("https://example.com/contributor".to_string()),
    }
}

fn make_new_contribution(
    work_id: Uuid,
    contributor_id: Uuid,
    contribution_type: ContributionType,
    contribution_ordinal: i32,
) -> NewContribution {
    let suffix = unique("Contribution");
    NewContribution {
        work_id,
        contributor_id,
        contribution_type,
        main_contribution: contribution_ordinal == 1,
        first_name: Some("Test".to_string()),
        last_name: suffix.clone(),
        full_name: format!("Test {suffix}"),
        contribution_ordinal,
    }
}

fn make_new_biography(contribution_id: Uuid, canonical: bool, content: &str) -> NewBiography {
    NewBiography {
        contribution_id,
        content: content.to_string(),
        canonical,
        locale_code: LocaleCode::En,
    }
}

fn make_new_institution() -> NewInstitution {
    NewInstitution {
        institution_name: unique("Institution"),
        institution_doi: None,
        ror: None,
        country_code: Some(CountryCode::Gbr),
    }
}

fn make_new_funding(work_id: Uuid, institution_id: Uuid) -> NewFunding {
    NewFunding {
        work_id,
        institution_id,
        program: Some("Program".to_string()),
        project_name: Some("Project".to_string()),
        project_shortname: Some("Proj".to_string()),
        grant_number: Some("Grant".to_string()),
    }
}

fn make_new_affiliation(
    contribution_id: Uuid,
    institution_id: Uuid,
    affiliation_ordinal: i32,
) -> NewAffiliation {
    NewAffiliation {
        contribution_id,
        institution_id,
        affiliation_ordinal,
        position: Some("Position".to_string()),
    }
}

fn make_new_series(imprint_id: Uuid) -> NewSeries {
    NewSeries {
        series_type: SeriesType::Journal,
        series_name: unique("Series"),
        issn_print: None,
        issn_digital: None,
        series_url: Some("https://example.com/series".to_string()),
        series_description: Some("Series description".to_string()),
        series_cfp_url: Some("https://example.com/cfp".to_string()),
        imprint_id,
    }
}

fn make_new_issue(series_id: Uuid, work_id: Uuid, issue_ordinal: i32) -> NewIssue {
    NewIssue {
        series_id,
        work_id,
        issue_ordinal,
        issue_number: None,
    }
}

fn make_new_language(work_id: Uuid) -> NewLanguage {
    NewLanguage {
        work_id,
        language_code: LanguageCode::Eng,
        language_relation: LanguageRelation::Original,
    }
}

fn make_new_publication(work_id: Uuid) -> NewPublication {
    NewPublication {
        publication_type: PublicationType::Paperback,
        work_id,
        isbn: Some(Isbn::from_str("978-3-16-148410-0").unwrap()),
        width_mm: Some(100.0),
        width_in: Some(3.94),
        height_mm: Some(200.0),
        height_in: Some(7.87),
        depth_mm: Some(30.0),
        depth_in: Some(1.18),
        weight_g: Some(500.0),
        weight_oz: Some(17.64),
        accessibility_standard: None,
        accessibility_additional_standard: None,
        accessibility_exception: None,
        accessibility_report_url: None,
    }
}

fn make_new_location(publication_id: Uuid, canonical: bool) -> NewLocation {
    NewLocation {
        publication_id,
        landing_page: Some("https://example.com/location".to_string()),
        full_text_url: Some("https://example.com/full".to_string()),
        location_platform: LocationPlatform::Other,
        canonical,
    }
}

fn make_new_price(publication_id: Uuid) -> NewPrice {
    NewPrice {
        publication_id,
        currency_code: CurrencyCode::Usd,
        unit_price: 12.34,
    }
}

fn make_new_subject(work_id: Uuid, subject_ordinal: i32) -> NewSubject {
    NewSubject {
        work_id,
        subject_type: SubjectType::Bic,
        subject_code: format!("CODE-{subject_ordinal}"),
        subject_ordinal,
    }
}

fn make_new_work_relation(
    relator_work_id: Uuid,
    related_work_id: Uuid,
    relation_ordinal: i32,
) -> NewWorkRelation {
    NewWorkRelation {
        relator_work_id,
        related_work_id,
        relation_type: RelationType::HasPart,
        relation_ordinal,
    }
}

fn make_new_reference(work_id: Uuid, reference_ordinal: i32) -> NewReference {
    NewReference {
        work_id,
        reference_ordinal,
        doi: None,
        unstructured_citation: Some("Citation".to_string()),
        issn: None,
        isbn: None,
        journal_title: Some("Journal".to_string()),
        article_title: Some("Article".to_string()),
        series_title: None,
        volume_title: None,
        edition: Some(1),
        author: Some("Author".to_string()),
        volume: Some("1".to_string()),
        issue: Some("2".to_string()),
        first_page: Some("1".to_string()),
        component_number: None,
        standard_designator: None,
        standards_body_name: None,
        standards_body_acronym: None,
        url: Some("https://example.com/ref".to_string()),
        publication_date: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        retrieval_date: None,
    }
}

fn make_new_contact(publisher_id: Uuid) -> NewContact {
    NewContact {
        publisher_id,
        contact_type: ContactType::Accessibility,
        email: "access@example.com".to_string(),
    }
}

struct SeedData {
    publisher_id: Uuid,
    publisher_org: String,
    imprint_id: Uuid,
    book_work_id: Uuid,
    chapter_work_id: Uuid,
    other_chapter_work_id: Uuid,
    issue_work_id: Uuid,
    issue_work_id_two: Uuid,
    title_id: Uuid,
    abstract_short_id: Uuid,
    abstract_long_id: Uuid,
    biography_id: Uuid,
    contributor_id: Uuid,
    contributor_id_two: Uuid,
    contribution_id: Uuid,
    contribution_id_two: Uuid,
    series_id: Uuid,
    issue_id: Uuid,
    issue_id_two: Uuid,
    language_id: Uuid,
    publication_id: Uuid,
    location_id: Uuid,
    price_id: Uuid,
    subject_id: Uuid,
    subject_id_two: Uuid,
    institution_id: Uuid,
    funding_id: Uuid,
    affiliation_id: Uuid,
    affiliation_id_two: Uuid,
    work_relation_id: Uuid,
    work_relation_id_two: Uuid,
    reference_id: Uuid,
    reference_id_two: Uuid,
    contact_id: Uuid,
    book_doi: Doi,
    chapter_doi: Doi,
}

fn seed_data(schema: &Schema, context: &Context) -> SeedData {
    let publisher_org = format!("org-{}", Uuid::new_v4());
    let publisher = create_with_data(
        schema,
        context,
        "createPublisher",
        "NewPublisher",
        "publisherId",
        make_new_publisher(&publisher_org),
    );
    let publisher_id = json_uuid(&publisher["publisherId"]);

    let imprint = create_with_data(
        schema,
        context,
        "createImprint",
        "NewImprint",
        "imprintId",
        make_new_imprint(publisher_id),
    );
    let imprint_id = json_uuid(&imprint["imprintId"]);

    let book_doi = Doi::from_str("10.1234/book").unwrap();
    let chapter_doi = Doi::from_str("10.1234/chapter").unwrap();
    let other_chapter_doi = Doi::from_str("10.1234/chapter-two").unwrap();
    let issue_doi = Doi::from_str("10.1234/issue-one").unwrap();
    let issue_doi_two = Doi::from_str("10.1234/issue-two").unwrap();

    let book_work = create_with_data(
        schema,
        context,
        "createWork",
        "NewWork",
        "workId",
        make_new_book_work(imprint_id, book_doi.clone()),
    );
    let book_work_id = json_uuid(&book_work["workId"]);

    let chapter_work = create_with_data(
        schema,
        context,
        "createWork",
        "NewWork",
        "workId",
        make_new_work(imprint_id, WorkType::BookChapter, chapter_doi.clone()),
    );
    let chapter_work_id = json_uuid(&chapter_work["workId"]);

    let other_chapter_work = create_with_data(
        schema,
        context,
        "createWork",
        "NewWork",
        "workId",
        make_new_work(imprint_id, WorkType::BookChapter, other_chapter_doi),
    );
    let other_chapter_work_id = json_uuid(&other_chapter_work["workId"]);

    let issue_work = create_with_data(
        schema,
        context,
        "createWork",
        "NewWork",
        "workId",
        make_new_work(imprint_id, WorkType::JournalIssue, issue_doi),
    );
    let issue_work_id = json_uuid(&issue_work["workId"]);

    let issue_work_two = create_with_data(
        schema,
        context,
        "createWork",
        "NewWork",
        "workId",
        make_new_work(imprint_id, WorkType::JournalIssue, issue_doi_two),
    );
    let issue_work_id_two = json_uuid(&issue_work_two["workId"]);

    let title = create_with_data_and_markup(
        schema,
        context,
        "createTitle",
        "NewTitle",
        "titleId",
        make_new_title(book_work_id, true, Some("Subtitle")),
        MarkupFormat::PlainText,
    );
    let title_id = json_uuid(&title["titleId"]);

    let abstract_short = create_with_data_and_markup(
        schema,
        context,
        "createAbstract",
        "NewAbstract",
        "abstractId",
        make_new_abstract(book_work_id, AbstractType::Short, true, "Short abstract"),
        MarkupFormat::PlainText,
    );
    let abstract_short_id = json_uuid(&abstract_short["abstractId"]);

    let abstract_long = create_with_data_and_markup(
        schema,
        context,
        "createAbstract",
        "NewAbstract",
        "abstractId",
        make_new_abstract(book_work_id, AbstractType::Long, false, "Long abstract"),
        MarkupFormat::PlainText,
    );
    let abstract_long_id = json_uuid(&abstract_long["abstractId"]);

    let contributor = create_with_data(
        schema,
        context,
        "createContributor",
        "NewContributor",
        "contributorId",
        make_new_contributor(),
    );
    let contributor_id = json_uuid(&contributor["contributorId"]);

    let contributor_two = create_with_data(
        schema,
        context,
        "createContributor",
        "NewContributor",
        "contributorId",
        make_new_contributor(),
    );
    let contributor_id_two = json_uuid(&contributor_two["contributorId"]);

    let contribution = create_with_data(
        schema,
        context,
        "createContribution",
        "NewContribution",
        "contributionId",
        make_new_contribution(book_work_id, contributor_id, ContributionType::Author, 1),
    );
    let contribution_id = json_uuid(&contribution["contributionId"]);

    let contribution_two = create_with_data(
        schema,
        context,
        "createContribution",
        "NewContribution",
        "contributionId",
        make_new_contribution(
            book_work_id,
            contributor_id_two,
            ContributionType::Editor,
            2,
        ),
    );
    let contribution_id_two = json_uuid(&contribution_two["contributionId"]);

    let biography = create_with_data_and_markup(
        schema,
        context,
        "createBiography",
        "NewBiography",
        "biographyId",
        make_new_biography(contribution_id, true, "Biography content"),
        MarkupFormat::PlainText,
    );
    let biography_id = json_uuid(&biography["biographyId"]);

    let institution = create_with_data(
        schema,
        context,
        "createInstitution",
        "NewInstitution",
        "institutionId",
        make_new_institution(),
    );
    let institution_id = json_uuid(&institution["institutionId"]);

    let funding = create_with_data(
        schema,
        context,
        "createFunding",
        "NewFunding",
        "fundingId",
        make_new_funding(book_work_id, institution_id),
    );
    let funding_id = json_uuid(&funding["fundingId"]);

    let affiliation = create_with_data(
        schema,
        context,
        "createAffiliation",
        "NewAffiliation",
        "affiliationId",
        make_new_affiliation(contribution_id, institution_id, 1),
    );
    let affiliation_id = json_uuid(&affiliation["affiliationId"]);

    let affiliation_two = create_with_data(
        schema,
        context,
        "createAffiliation",
        "NewAffiliation",
        "affiliationId",
        make_new_affiliation(contribution_id, institution_id, 2),
    );
    let affiliation_id_two = json_uuid(&affiliation_two["affiliationId"]);

    let series = create_with_data(
        schema,
        context,
        "createSeries",
        "NewSeries",
        "seriesId",
        make_new_series(imprint_id),
    );
    let series_id = json_uuid(&series["seriesId"]);

    let issue = create_with_data(
        schema,
        context,
        "createIssue",
        "NewIssue",
        "issueId",
        make_new_issue(series_id, issue_work_id, 1),
    );
    let issue_id = json_uuid(&issue["issueId"]);

    let issue_two = create_with_data(
        schema,
        context,
        "createIssue",
        "NewIssue",
        "issueId",
        make_new_issue(series_id, issue_work_id_two, 2),
    );
    let issue_id_two = json_uuid(&issue_two["issueId"]);

    let language = create_with_data(
        schema,
        context,
        "createLanguage",
        "NewLanguage",
        "languageId",
        make_new_language(book_work_id),
    );
    let language_id = json_uuid(&language["languageId"]);

    let publication = create_with_data(
        schema,
        context,
        "createPublication",
        "NewPublication",
        "publicationId",
        make_new_publication(book_work_id),
    );
    let publication_id = json_uuid(&publication["publicationId"]);

    let location = create_with_data(
        schema,
        context,
        "createLocation",
        "NewLocation",
        "locationId",
        make_new_location(publication_id, true),
    );
    let location_id = json_uuid(&location["locationId"]);

    let price = create_with_data(
        schema,
        context,
        "createPrice",
        "NewPrice",
        "priceId",
        make_new_price(publication_id),
    );
    let price_id = json_uuid(&price["priceId"]);

    let subject = create_with_data(
        schema,
        context,
        "createSubject",
        "NewSubject",
        "subjectId",
        make_new_subject(book_work_id, 1),
    );
    let subject_id = json_uuid(&subject["subjectId"]);

    let subject_two = create_with_data(
        schema,
        context,
        "createSubject",
        "NewSubject",
        "subjectId",
        make_new_subject(book_work_id, 2),
    );
    let subject_id_two = json_uuid(&subject_two["subjectId"]);

    let work_relation = create_with_data(
        schema,
        context,
        "createWorkRelation",
        "NewWorkRelation",
        "workRelationId",
        make_new_work_relation(book_work_id, chapter_work_id, 1),
    );
    let work_relation_id = json_uuid(&work_relation["workRelationId"]);

    let work_relation_two = create_with_data(
        schema,
        context,
        "createWorkRelation",
        "NewWorkRelation",
        "workRelationId",
        make_new_work_relation(book_work_id, other_chapter_work_id, 2),
    );
    let work_relation_id_two = json_uuid(&work_relation_two["workRelationId"]);

    let reference = create_with_data(
        schema,
        context,
        "createReference",
        "NewReference",
        "referenceId",
        make_new_reference(book_work_id, 1),
    );
    let reference_id = json_uuid(&reference["referenceId"]);

    let reference_two = create_with_data(
        schema,
        context,
        "createReference",
        "NewReference",
        "referenceId",
        make_new_reference(book_work_id, 2),
    );
    let reference_id_two = json_uuid(&reference_two["referenceId"]);

    let contact = create_with_data(
        schema,
        context,
        "createContact",
        "NewContact",
        "contactId",
        make_new_contact(publisher_id),
    );
    let contact_id = json_uuid(&contact["contactId"]);

    SeedData {
        publisher_id,
        publisher_org,
        imprint_id,
        book_work_id,
        chapter_work_id,
        other_chapter_work_id,
        issue_work_id,
        issue_work_id_two,
        title_id,
        abstract_short_id,
        abstract_long_id,
        biography_id,
        contributor_id,
        contributor_id_two,
        contribution_id,
        contribution_id_two,
        series_id,
        issue_id,
        issue_id_two,
        language_id,
        publication_id,
        location_id,
        price_id,
        subject_id,
        subject_id_two,
        institution_id,
        funding_id,
        affiliation_id,
        affiliation_id_two,
        work_relation_id,
        work_relation_id_two,
        reference_id,
        reference_id_two,
        contact_id,
        book_doi,
        chapter_doi,
    }
}

fn patch_publisher(publisher: &Publisher) -> PatchPublisher {
    PatchPublisher {
        publisher_id: publisher.publisher_id,
        publisher_name: format!("{} Updated", publisher.publisher_name),
        publisher_shortname: publisher.publisher_shortname.clone(),
        publisher_url: publisher.publisher_url.clone(),
        zitadel_id: publisher.zitadel_id.clone(),
        accessibility_statement: publisher.accessibility_statement.clone(),
        accessibility_report_url: publisher.accessibility_report_url.clone(),
    }
}

fn patch_imprint(imprint: &Imprint) -> PatchImprint {
    PatchImprint {
        imprint_id: imprint.imprint_id,
        publisher_id: imprint.publisher_id,
        imprint_name: format!("{} Updated", imprint.imprint_name),
        imprint_url: imprint.imprint_url.clone(),
        crossmark_doi: imprint.crossmark_doi.clone(),
        s3_bucket: imprint.s3_bucket.clone(),
        cdn_domain: imprint.cdn_domain.clone(),
        cloudfront_dist_id: imprint.cloudfront_dist_id.clone(),
        default_currency: imprint.default_currency,
        default_place: imprint.default_place.clone(),
        default_locale: imprint.default_locale,
    }
}

fn patch_contributor(contributor: &Contributor) -> PatchContributor {
    PatchContributor {
        contributor_id: contributor.contributor_id,
        first_name: contributor.first_name.clone(),
        last_name: contributor.last_name.clone(),
        full_name: format!("{} Updated", contributor.full_name),
        orcid: contributor.orcid.clone(),
        website: contributor.website.clone(),
    }
}

fn patch_contribution(contribution: &Contribution) -> PatchContribution {
    PatchContribution {
        contribution_id: contribution.contribution_id,
        work_id: contribution.work_id,
        contributor_id: contribution.contributor_id,
        contribution_type: contribution.contribution_type,
        main_contribution: contribution.main_contribution,
        first_name: contribution.first_name.clone(),
        last_name: contribution.last_name.clone(),
        full_name: format!("{} Updated", contribution.full_name),
        contribution_ordinal: contribution.contribution_ordinal,
    }
}

fn patch_publication(publication: &Publication) -> PatchPublication {
    PatchPublication {
        publication_id: publication.publication_id,
        publication_type: publication.publication_type,
        work_id: publication.work_id,
        isbn: publication.isbn.clone(),
        width_mm: publication.width_mm.map(|w| w + 1.0),
        width_in: publication.width_in,
        height_mm: publication.height_mm,
        height_in: publication.height_in,
        depth_mm: publication.depth_mm,
        depth_in: publication.depth_in,
        weight_g: publication.weight_g,
        weight_oz: publication.weight_oz,
        accessibility_standard: publication.accessibility_standard,
        accessibility_additional_standard: publication.accessibility_additional_standard,
        accessibility_exception: publication.accessibility_exception,
        accessibility_report_url: publication.accessibility_report_url.clone(),
    }
}

fn patch_series(series: &Series) -> PatchSeries {
    PatchSeries {
        series_id: series.series_id,
        series_type: series.series_type,
        series_name: format!("{} Updated", series.series_name),
        issn_print: series.issn_print.clone(),
        issn_digital: series.issn_digital.clone(),
        series_url: series.series_url.clone(),
        series_description: series.series_description.clone(),
        series_cfp_url: series.series_cfp_url.clone(),
        imprint_id: series.imprint_id,
    }
}

fn patch_issue(issue: &Issue) -> PatchIssue {
    PatchIssue {
        issue_id: issue.issue_id,
        series_id: issue.series_id,
        work_id: issue.work_id,
        issue_ordinal: issue.issue_ordinal,
        issue_number: issue.issue_number,
    }
}

fn patch_language(language: &Language) -> PatchLanguage {
    PatchLanguage {
        language_id: language.language_id,
        work_id: language.work_id,
        language_code: language.language_code,
        language_relation: language.language_relation,
    }
}

fn patch_institution(institution: &Institution) -> PatchInstitution {
    PatchInstitution {
        institution_id: institution.institution_id,
        institution_name: format!("{} Updated", institution.institution_name),
        institution_doi: institution.institution_doi.clone(),
        ror: institution.ror.clone(),
        country_code: institution.country_code,
    }
}

fn patch_funding(funding: &Funding) -> PatchFunding {
    PatchFunding {
        funding_id: funding.funding_id,
        work_id: funding.work_id,
        institution_id: funding.institution_id,
        program: funding.program.clone(),
        project_name: funding.project_name.clone(),
        project_shortname: funding.project_shortname.clone(),
        grant_number: funding.grant_number.clone(),
    }
}

fn patch_location(location: &Location) -> PatchLocation {
    PatchLocation {
        location_id: location.location_id,
        publication_id: location.publication_id,
        landing_page: location
            .landing_page
            .as_ref()
            .map(|url| format!("{url}?updated=1")),
        full_text_url: location.full_text_url.clone(),
        location_platform: location.location_platform,
        canonical: location.canonical,
    }
}

fn patch_price(price: &Price) -> PatchPrice {
    PatchPrice {
        price_id: price.price_id,
        publication_id: price.publication_id,
        currency_code: price.currency_code,
        unit_price: price.unit_price + 1.0,
    }
}

fn patch_subject(subject: &Subject) -> PatchSubject {
    PatchSubject {
        subject_id: subject.subject_id,
        work_id: subject.work_id,
        subject_type: subject.subject_type,
        subject_code: format!("{}-UPDATED", subject.subject_code),
        subject_ordinal: subject.subject_ordinal,
    }
}

fn patch_affiliation(affiliation: &Affiliation) -> PatchAffiliation {
    PatchAffiliation {
        affiliation_id: affiliation.affiliation_id,
        contribution_id: affiliation.contribution_id,
        institution_id: affiliation.institution_id,
        affiliation_ordinal: affiliation.affiliation_ordinal,
        position: affiliation.position.clone(),
    }
}

fn patch_work_relation(work_relation: &WorkRelation) -> PatchWorkRelation {
    PatchWorkRelation {
        work_relation_id: work_relation.work_relation_id,
        relator_work_id: work_relation.relator_work_id,
        related_work_id: work_relation.related_work_id,
        relation_type: work_relation.relation_type,
        relation_ordinal: work_relation.relation_ordinal,
    }
}

fn patch_reference(reference: &Reference) -> PatchReference {
    PatchReference {
        reference_id: reference.reference_id,
        work_id: reference.work_id,
        reference_ordinal: reference.reference_ordinal,
        doi: reference.doi.clone(),
        unstructured_citation: reference.unstructured_citation.clone(),
        issn: reference.issn.clone(),
        isbn: reference.isbn.clone(),
        journal_title: reference.journal_title.clone(),
        article_title: reference.article_title.clone(),
        series_title: reference.series_title.clone(),
        volume_title: reference.volume_title.clone(),
        edition: reference.edition,
        author: reference.author.clone(),
        volume: reference.volume.clone(),
        issue: reference.issue.clone(),
        first_page: reference.first_page.clone(),
        component_number: reference.component_number.clone(),
        standard_designator: reference.standard_designator.clone(),
        standards_body_name: reference.standards_body_name.clone(),
        standards_body_acronym: reference.standards_body_acronym.clone(),
        url: reference.url.clone(),
        publication_date: reference.publication_date,
        retrieval_date: reference.retrieval_date,
    }
}

fn patch_contact(contact: &Contact) -> PatchContact {
    PatchContact {
        contact_id: contact.contact_id,
        publisher_id: contact.publisher_id,
        contact_type: contact.contact_type,
        email: format!("updated-{}", contact.email),
    }
}

fn patch_title(title: &Title) -> PatchTitle {
    PatchTitle {
        title_id: title.title_id,
        work_id: title.work_id,
        locale_code: title.locale_code,
        full_title: format!("{} Updated", title.full_title),
        title: format!("{} Updated", title.title),
        subtitle: title.subtitle.clone(),
        canonical: title.canonical,
    }
}

fn patch_abstract(abstract_item: &Abstract) -> PatchAbstract {
    PatchAbstract {
        abstract_id: abstract_item.abstract_id,
        work_id: abstract_item.work_id,
        content: format!("{} Updated", abstract_item.content),
        locale_code: abstract_item.locale_code,
        abstract_type: abstract_item.abstract_type,
        canonical: abstract_item.canonical,
    }
}

fn patch_biography(biography: &Biography) -> PatchBiography {
    PatchBiography {
        biography_id: biography.biography_id,
        contribution_id: biography.contribution_id,
        content: format!("{} Updated", biography.content),
        canonical: biography.canonical,
        locale_code: biography.locale_code,
    }
}

fn assert_work_resolvers(
    work: &Work,
    context: &Context,
    title: &Title,
    short_abs: &Abstract,
    long_abs: &Abstract,
    expected_imprint_id: Uuid,
) {
    assert_eq!(work.work_id(), &work.work_id);
    assert_eq!(work.work_type(), &work.work_type);
    assert_eq!(work.work_status(), &work.work_status);
    assert_eq!(work.full_title(context).unwrap(), title.full_title);
    assert_eq!(work.title(context).unwrap(), title.title);
    assert_eq!(work.subtitle(context).unwrap(), title.subtitle);
    let expected_short = short_abs.canonical.then(|| short_abs.content.clone());
    let expected_long = long_abs.canonical.then(|| long_abs.content.clone());
    assert_eq!(work.short_abstract(context).unwrap(), expected_short);
    assert_eq!(work.long_abstract(context).unwrap(), expected_long);
    assert_eq!(work.reference(), work.reference.as_ref());
    assert_eq!(work.edition(), work.edition.as_ref());
    assert_eq!(work.imprint_id(), work.imprint_id);
    assert_eq!(work.doi(), work.doi.as_ref());
    assert_eq!(work.publication_date(), work.publication_date);
    assert_eq!(work.withdrawn_date(), work.withdrawn_date);
    assert_eq!(work.place(), work.place.as_ref());
    assert_eq!(work.page_count(), work.page_count.as_ref());
    assert_eq!(work.page_breakdown(), work.page_breakdown.as_ref());
    assert_eq!(work.image_count(), work.image_count.as_ref());
    assert_eq!(work.table_count(), work.table_count.as_ref());
    assert_eq!(work.audio_count(), work.audio_count.as_ref());
    assert_eq!(work.video_count(), work.video_count.as_ref());
    assert_eq!(work.license(), work.license.as_ref());
    assert_eq!(work.copyright_holder(), work.copyright_holder.as_ref());
    assert_eq!(work.landing_page(), work.landing_page.as_ref());
    assert_eq!(work.lccn(), work.lccn.as_ref());
    assert_eq!(work.oclc(), work.oclc.as_ref());
    assert_eq!(work.general_note(), work.general_note.as_ref());
    assert_eq!(work.bibliography_note(), work.bibliography_note.as_ref());
    assert_eq!(work.toc(), work.toc.as_ref());
    assert_eq!(work.cover_url(), work.cover_url.as_ref());
    assert_eq!(work.cover_caption(), work.cover_caption.as_ref());
    assert_eq!(work.created_at(), work.created_at);
    assert_eq!(work.updated_at(), work.updated_at);
    assert_eq!(work.first_page(), work.first_page.as_ref());
    assert_eq!(work.last_page(), work.last_page.as_ref());
    assert_eq!(work.page_interval(), work.page_interval.as_ref());
    assert_eq!(
        work.updated_at_with_relations(),
        work.updated_at_with_relations
    );

    let imprint = work.imprint(context).unwrap();
    assert_eq!(imprint.imprint_id, expected_imprint_id);

    assert!(!work
        .contributions(context, Some(10), Some(0), None, None)
        .unwrap()
        .is_empty());
    assert!(!work
        .languages(
            context,
            Some(10),
            Some(0),
            None,
            None,
            Some(LanguageRelation::Original),
            None
        )
        .unwrap()
        .is_empty());
    assert!(!work
        .publications(context, Some(10), Some(0), None, None, None)
        .unwrap()
        .is_empty());
    assert!(!work
        .subjects(context, Some(10), Some(0), None, None, None)
        .unwrap()
        .is_empty());
    assert!(!work
        .fundings(context, Some(10), Some(0), None)
        .unwrap()
        .is_empty());
    let _ = work.issues(context, Some(10), Some(0), None).unwrap();
    assert!(!work
        .relations(context, Some(10), Some(0), None, None)
        .unwrap()
        .is_empty());
    assert!(!work
        .references(context, Some(10), Some(0), None, None)
        .unwrap()
        .is_empty());
}

fn assert_publication_resolvers(publication: &Publication, context: &Context) {
    assert_eq!(publication.publication_id(), publication.publication_id);
    assert_eq!(
        publication.publication_type(),
        &publication.publication_type
    );
    assert_eq!(publication.work_id(), publication.work_id);
    assert_eq!(publication.isbn(), publication.isbn.as_ref());
    assert_eq!(publication.created_at(), publication.created_at);
    assert_eq!(publication.updated_at(), publication.updated_at);
    assert_eq!(publication.width(LengthUnit::Mm), publication.width_mm);
    assert_eq!(publication.width(LengthUnit::In), publication.width_in);
    assert_eq!(
        publication.width(LengthUnit::Cm),
        publication
            .width_mm
            .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm))
    );
    assert_eq!(publication.height(LengthUnit::Mm), publication.height_mm);
    assert_eq!(publication.height(LengthUnit::In), publication.height_in);
    assert_eq!(
        publication.height(LengthUnit::Cm),
        publication
            .height_mm
            .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm))
    );
    assert_eq!(publication.depth(LengthUnit::Mm), publication.depth_mm);
    assert_eq!(publication.depth(LengthUnit::In), publication.depth_in);
    assert_eq!(
        publication.depth(LengthUnit::Cm),
        publication
            .depth_mm
            .map(|w| w.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::Cm))
    );
    assert_eq!(publication.weight(WeightUnit::G), publication.weight_g);
    assert_eq!(publication.weight(WeightUnit::Oz), publication.weight_oz);
    assert_eq!(
        publication.accessibility_standard(),
        publication.accessibility_standard.as_ref()
    );
    assert_eq!(
        publication.accessibility_additional_standard(),
        publication.accessibility_additional_standard.as_ref()
    );
    assert_eq!(
        publication.accessibility_exception(),
        publication.accessibility_exception.as_ref()
    );
    assert_eq!(
        publication.accessibility_report_url(),
        publication.accessibility_report_url.as_ref()
    );
    assert!(!publication
        .prices(context, Some(10), Some(0), None, None)
        .unwrap()
        .is_empty());
    assert!(!publication
        .locations(context, Some(10), Some(0), None, None)
        .unwrap()
        .is_empty());
    let work = publication.work(context).unwrap();
    assert_eq!(work.work_id, publication.work_id);
}

fn assert_publisher_resolvers(publisher: &Publisher, context: &Context) {
    assert_eq!(publisher.publisher_id(), publisher.publisher_id);
    assert_eq!(publisher.publisher_name(), &publisher.publisher_name);
    assert_eq!(
        publisher.publisher_shortname(),
        publisher.publisher_shortname.as_ref()
    );
    assert_eq!(publisher.publisher_url(), publisher.publisher_url.as_ref());
    assert_eq!(publisher.zitadel_id(), publisher.zitadel_id.as_ref());
    assert_eq!(
        publisher.accessibility_statement(),
        publisher.accessibility_statement.as_ref()
    );
    assert_eq!(
        publisher.accessibility_report_url(),
        publisher.accessibility_report_url.as_ref()
    );
    assert_eq!(publisher.created_at(), publisher.created_at);
    assert_eq!(publisher.updated_at(), publisher.updated_at);
    assert!(!publisher
        .imprints(context, Some(10), Some(0), None, None)
        .unwrap()
        .is_empty());
    assert!(!publisher
        .contacts(context, Some(10), Some(0), None, None)
        .unwrap()
        .is_empty());
}

fn assert_imprint_resolvers(imprint: &Imprint, context: &Context) {
    assert_eq!(imprint.imprint_id(), imprint.imprint_id);
    assert_eq!(imprint.publisher_id(), imprint.publisher_id);
    assert_eq!(imprint.imprint_name(), &imprint.imprint_name);
    assert_eq!(imprint.imprint_url(), imprint.imprint_url.as_ref());
    assert_eq!(imprint.crossmark_doi(), imprint.crossmark_doi.as_ref());
    assert_eq!(imprint.created_at(), imprint.created_at);
    assert_eq!(imprint.updated_at(), imprint.updated_at);
    let publisher = imprint.publisher(context).unwrap();
    assert_eq!(publisher.publisher_id, imprint.publisher_id);
    assert!(!imprint
        .works(
            context,
            Some(10),
            Some(0),
            None,
            None,
            None,
            Some(WorkStatus::Active),
            None,
            None,
            None
        )
        .unwrap()
        .is_empty());
}

fn assert_contributor_resolvers(contributor: &Contributor, context: &Context) {
    assert_eq!(contributor.contributor_id(), contributor.contributor_id);
    assert_eq!(contributor.first_name(), contributor.first_name.as_ref());
    assert_eq!(contributor.last_name(), &contributor.last_name);
    assert_eq!(contributor.full_name(), &contributor.full_name);
    assert_eq!(contributor.orcid(), contributor.orcid.as_ref());
    assert_eq!(contributor.website(), contributor.website.as_ref());
    assert_eq!(contributor.created_at(), contributor.created_at);
    assert_eq!(contributor.updated_at(), contributor.updated_at);
    assert!(!contributor
        .contributions(context, Some(10), Some(0), None, None)
        .unwrap()
        .is_empty());
}

fn assert_contribution_resolvers(
    contribution: &Contribution,
    context: &Context,
    biography_content: &str,
) {
    assert_eq!(contribution.contribution_id(), contribution.contribution_id);
    assert_eq!(contribution.contributor_id(), contribution.contributor_id);
    assert_eq!(contribution.work_id(), contribution.work_id);
    assert_eq!(
        contribution.contribution_type(),
        &contribution.contribution_type
    );
    assert_eq!(
        contribution.main_contribution(),
        contribution.main_contribution
    );
    assert!(!contribution
        .biographies(
            context,
            Some(10),
            Some(0),
            None,
            None,
            None,
            Some(MarkupFormat::PlainText)
        )
        .unwrap()
        .is_empty());
    let biography = contribution.biography(context).unwrap();
    assert_eq!(biography, Some(biography_content.to_string()));
    assert_eq!(contribution.created_at(), contribution.created_at);
    assert_eq!(contribution.updated_at(), contribution.updated_at);
    assert_eq!(contribution.first_name(), contribution.first_name.as_ref());
    assert_eq!(contribution.last_name(), &contribution.last_name);
    assert_eq!(contribution.full_name(), &contribution.full_name);
    assert_eq!(
        contribution.contribution_ordinal(),
        &contribution.contribution_ordinal
    );
    let work = contribution.work(context).unwrap();
    assert_eq!(work.work_id, contribution.work_id);
    let contributor = contribution.contributor(context).unwrap();
    assert_eq!(contributor.contributor_id, contribution.contributor_id);
    assert!(!contribution
        .affiliations(context, Some(10), Some(0), None)
        .unwrap()
        .is_empty());
}

fn assert_series_resolvers(series: &Series, context: &Context) {
    assert_eq!(series.series_id(), series.series_id);
    assert_eq!(series.series_type(), &series.series_type);
    assert_eq!(series.series_name(), &series.series_name);
    assert_eq!(series.issn_print(), series.issn_print.as_ref());
    assert_eq!(series.issn_digital(), series.issn_digital.as_ref());
    assert_eq!(series.series_url(), series.series_url.as_ref());
    assert_eq!(
        series.series_description(),
        series.series_description.as_ref()
    );
    assert_eq!(series.series_cfp_url(), series.series_cfp_url.as_ref());
    assert_eq!(series.imprint_id(), series.imprint_id);
    assert_eq!(series.created_at(), series.created_at);
    assert_eq!(series.updated_at(), series.updated_at);
    let imprint = series.imprint(context).unwrap();
    assert_eq!(imprint.imprint_id, series.imprint_id);
    assert!(!series
        .issues(context, Some(10), Some(0), None)
        .unwrap()
        .is_empty());
}

fn assert_issue_resolvers(issue: &Issue, context: &Context) {
    assert_eq!(issue.issue_id(), issue.issue_id);
    assert_eq!(issue.work_id(), issue.work_id);
    assert_eq!(issue.series_id(), issue.series_id);
    assert_eq!(issue.issue_ordinal(), &issue.issue_ordinal);
    assert_eq!(issue.issue_number(), issue.issue_number.as_ref());
    assert_eq!(issue.created_at(), issue.created_at);
    assert_eq!(issue.updated_at(), issue.updated_at);
    let series = issue.series(context).unwrap();
    assert_eq!(series.series_id, issue.series_id);
    let work = issue.work(context).unwrap();
    assert_eq!(work.work_id, issue.work_id);
}

fn assert_language_resolvers(language: &Language, context: &Context) {
    assert_eq!(language.language_id(), language.language_id);
    assert_eq!(language.work_id(), language.work_id);
    assert_eq!(language.language_code(), &language.language_code);
    assert_eq!(language.language_relation(), &language.language_relation);
    assert_eq!(language.created_at(), language.created_at);
    assert_eq!(language.updated_at(), language.updated_at);
    let work = language.work(context).unwrap();
    assert_eq!(work.work_id, language.work_id);
}

fn assert_location_resolvers(location: &Location, context: &Context) {
    assert_eq!(location.location_id(), location.location_id);
    assert_eq!(location.publication_id(), location.publication_id);
    assert_eq!(location.landing_page(), location.landing_page.as_ref());
    assert_eq!(location.full_text_url(), location.full_text_url.as_ref());
    assert_eq!(location.location_platform(), &location.location_platform);
    assert_eq!(location.canonical(), location.canonical);
    assert_eq!(location.created_at(), location.created_at);
    assert_eq!(location.updated_at(), location.updated_at);
    let publication = location.publication(context).unwrap();
    assert_eq!(publication.publication_id, location.publication_id);
}

fn assert_price_resolvers(price: &Price, context: &Context) {
    assert_eq!(price.price_id(), price.price_id);
    assert_eq!(price.publication_id(), price.publication_id);
    assert_eq!(price.currency_code(), &price.currency_code);
    assert_eq!(price.unit_price(), price.unit_price);
    assert_eq!(price.created_at(), price.created_at);
    assert_eq!(price.updated_at(), price.updated_at);
    let publication = price.publication(context).unwrap();
    assert_eq!(publication.publication_id, price.publication_id);
}

fn assert_subject_resolvers(subject: &Subject, context: &Context) {
    assert_eq!(subject.subject_id(), &subject.subject_id);
    assert_eq!(subject.work_id(), &subject.work_id);
    assert_eq!(subject.subject_type(), &subject.subject_type);
    assert_eq!(subject.subject_code(), &subject.subject_code);
    assert_eq!(subject.subject_ordinal(), &subject.subject_ordinal);
    assert_eq!(subject.created_at(), subject.created_at);
    assert_eq!(subject.updated_at(), subject.updated_at);
    let work = subject.work(context).unwrap();
    assert_eq!(work.work_id, subject.work_id);
}

fn assert_institution_resolvers(institution: &Institution, context: &Context) {
    assert_eq!(institution.institution_id(), &institution.institution_id);
    assert_eq!(
        institution.institution_name(),
        &institution.institution_name
    );
    assert_eq!(
        institution.institution_doi(),
        institution.institution_doi.as_ref()
    );
    assert_eq!(
        institution.country_code(),
        institution.country_code.as_ref()
    );
    assert_eq!(institution.ror(), institution.ror.as_ref());
    assert_eq!(institution.created_at(), institution.created_at);
    assert_eq!(institution.updated_at(), institution.updated_at);
    assert!(!institution
        .fundings(context, Some(10), Some(0), None)
        .unwrap()
        .is_empty());
    assert!(!institution
        .affiliations(context, Some(10), Some(0), None)
        .unwrap()
        .is_empty());
}

fn assert_funding_resolvers(funding: &Funding, context: &Context) {
    assert_eq!(funding.funding_id(), &funding.funding_id);
    assert_eq!(funding.work_id(), &funding.work_id);
    assert_eq!(funding.institution_id(), &funding.institution_id);
    assert_eq!(funding.program(), funding.program.as_ref());
    assert_eq!(funding.project_name(), funding.project_name.as_ref());
    assert_eq!(
        funding.project_shortname(),
        funding.project_shortname.as_ref()
    );
    assert_eq!(funding.grant_number(), funding.grant_number.as_ref());
    assert_eq!(funding.created_at(), funding.created_at);
    assert_eq!(funding.updated_at(), funding.updated_at);
    let work = funding.work(context).unwrap();
    assert_eq!(work.work_id, funding.work_id);
    let institution = funding.institution(context).unwrap();
    assert_eq!(institution.institution_id, funding.institution_id);
}

fn assert_affiliation_resolvers(affiliation: &Affiliation, context: &Context) {
    assert_eq!(affiliation.affiliation_id(), affiliation.affiliation_id);
    assert_eq!(affiliation.contribution_id(), affiliation.contribution_id);
    assert_eq!(affiliation.institution_id(), affiliation.institution_id);
    assert_eq!(
        affiliation.affiliation_ordinal(),
        &affiliation.affiliation_ordinal
    );
    assert_eq!(affiliation.position(), affiliation.position.as_ref());
    assert_eq!(affiliation.created_at(), affiliation.created_at);
    assert_eq!(affiliation.updated_at(), affiliation.updated_at);
    let institution = affiliation.institution(context).unwrap();
    assert_eq!(institution.institution_id, affiliation.institution_id);
    let contribution = affiliation.contribution(context).unwrap();
    assert_eq!(contribution.contribution_id, affiliation.contribution_id);
}

fn assert_work_relation_resolvers(work_relation: &WorkRelation, context: &Context) {
    assert_eq!(
        work_relation.work_relation_id(),
        &work_relation.work_relation_id
    );
    assert_eq!(
        work_relation.relator_work_id(),
        &work_relation.relator_work_id
    );
    assert_eq!(
        work_relation.related_work_id(),
        &work_relation.related_work_id
    );
    assert_eq!(work_relation.relation_type(), &work_relation.relation_type);
    assert_eq!(
        work_relation.relation_ordinal(),
        &work_relation.relation_ordinal
    );
    assert_eq!(work_relation.created_at(), work_relation.created_at);
    assert_eq!(work_relation.updated_at(), work_relation.updated_at);
    let related = work_relation.related_work(context).unwrap();
    assert_eq!(related.work_id, work_relation.related_work_id);
}

fn assert_reference_resolvers(reference: &Reference, context: &Context) {
    assert_eq!(reference.reference_id(), reference.reference_id);
    assert_eq!(reference.work_id(), reference.work_id);
    assert_eq!(reference.reference_ordinal(), &reference.reference_ordinal);
    assert_eq!(reference.doi(), reference.doi.as_ref());
    assert_eq!(
        reference.unstructured_citation(),
        reference.unstructured_citation.as_ref()
    );
    assert_eq!(reference.issn(), reference.issn.as_ref());
    assert_eq!(reference.isbn(), reference.isbn.as_ref());
    assert_eq!(reference.journal_title(), reference.journal_title.as_ref());
    assert_eq!(reference.article_title(), reference.article_title.as_ref());
    assert_eq!(reference.series_title(), reference.series_title.as_ref());
    assert_eq!(reference.volume_title(), reference.volume_title.as_ref());
    assert_eq!(reference.edition(), reference.edition.as_ref());
    assert_eq!(reference.author(), reference.author.as_ref());
    assert_eq!(reference.volume(), reference.volume.as_ref());
    assert_eq!(reference.issue(), reference.issue.as_ref());
    assert_eq!(reference.first_page(), reference.first_page.as_ref());
    assert_eq!(
        reference.component_number(),
        reference.component_number.as_ref()
    );
    assert_eq!(
        reference.standard_designator(),
        reference.standard_designator.as_ref()
    );
    assert_eq!(
        reference.standards_body_name(),
        reference.standards_body_name.as_ref()
    );
    assert_eq!(
        reference.standards_body_acronym(),
        reference.standards_body_acronym.as_ref()
    );
    assert_eq!(reference.url(), reference.url.as_ref());
    assert_eq!(reference.publication_date(), reference.publication_date);
    assert_eq!(reference.retrieval_date(), reference.retrieval_date);
    assert_eq!(reference.created_at(), reference.created_at);
    assert_eq!(reference.updated_at(), reference.updated_at);
    let work = reference.work(context).unwrap();
    assert_eq!(work.work_id, reference.work_id);
}

fn assert_title_resolvers(title: &Title, context: &Context) {
    assert_eq!(title.title_id(), title.title_id);
    assert_eq!(title.work_id(), title.work_id);
    assert_eq!(title.locale_code(), &title.locale_code);
    assert_eq!(title.full_title(), &title.full_title);
    assert_eq!(title.title(), &title.title);
    assert_eq!(title.subtitle(), title.subtitle.as_ref());
    assert_eq!(title.canonical(), title.canonical);
    let work = title.work(context).unwrap();
    assert_eq!(work.work_id, title.work_id);
}

fn assert_abstract_resolvers(abstract_item: &Abstract, context: &Context) {
    assert_eq!(abstract_item.abstract_id(), abstract_item.abstract_id);
    assert_eq!(abstract_item.work_id(), abstract_item.work_id);
    assert_eq!(abstract_item.locale_code(), &abstract_item.locale_code);
    assert_eq!(abstract_item.content(), &abstract_item.content);
    assert_eq!(abstract_item.canonical(), abstract_item.canonical);
    assert_eq!(abstract_item.abstract_type(), &abstract_item.abstract_type);
    let work = abstract_item.work(context).unwrap();
    assert_eq!(work.work_id, abstract_item.work_id);
}

fn assert_biography_resolvers(biography: &Biography, context: &Context, expected_work_id: Uuid) {
    assert_eq!(biography.biography_id(), biography.biography_id);
    assert_eq!(biography.contribution_id(), biography.contribution_id);
    assert_eq!(biography.locale_code(), &biography.locale_code);
    assert_eq!(biography.content(), &biography.content);
    assert_eq!(biography.canonical(), biography.canonical);
    let work = biography.work(context).unwrap();
    assert_eq!(work.work_id, expected_work_id);
    let contribution = biography.contribution(context).unwrap();
    assert_eq!(contribution.contribution_id, biography.contribution_id);
}

fn assert_contact_resolvers(contact: &Contact, context: &Context) {
    assert_eq!(contact.contact_id(), contact.contact_id);
    assert_eq!(contact.publisher_id(), contact.publisher_id);
    assert_eq!(contact.contact_type(), &contact.contact_type);
    assert_eq!(contact.email(), &contact.email);
    assert_eq!(contact.created_at(), contact.created_at);
    assert_eq!(contact.updated_at(), contact.updated_at);
    let publisher = contact.publisher(context).unwrap();
    assert_eq!(publisher.publisher_id, contact.publisher_id);
}

#[test]
fn graphql_query_and_model_resolvers_cover_all() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-1");
    let context = test_db::test_context_with_user(pool.clone(), superuser);

    let seed = seed_data(&schema, &context);

    let query = r#"
query Root(
  $workId: Uuid!,
  $bookDoi: Doi!,
  $chapterDoi: Doi!,
  $publicationId: Uuid!,
  $publisherId: Uuid!,
  $imprintId: Uuid!,
  $contributorId: Uuid!,
  $contributionId: Uuid!,
  $seriesId: Uuid!,
  $issueId: Uuid!,
  $languageId: Uuid!,
  $locationId: Uuid!,
  $priceId: Uuid!,
  $subjectId: Uuid!,
  $institutionId: Uuid!,
  $fundingId: Uuid!,
  $affiliationId: Uuid!,
  $referenceId: Uuid!,
  $titleId: Uuid!,
  $abstractId: Uuid!,
  $biographyId: Uuid!,
  $contactId: Uuid!,
  $workStatus: WorkStatus!,
  $titleMarkup: MarkupFormat!,
  $abstractMarkup: MarkupFormat!,
  $biographyMarkup: MarkupFormat!
) {
  works(limit: 10, workStatus: $workStatus) { workId }
  work(workId: $workId) {
    workId
    titles(limit: 10, markupFormat: $titleMarkup) { titleId fullTitle title subtitle }
    abstracts(limit: 10, markupFormat: $abstractMarkup) { abstractId content }
  }
  workByDoi(doi: $bookDoi) { workId }
  workCount(workStatus: $workStatus)
  books(limit: 10, workStatus: $workStatus) { workId }
  bookByDoi(doi: $bookDoi) { workId }
  bookCount(workStatus: $workStatus)
  chapters(limit: 10, workStatus: $workStatus) { workId }
  chapterByDoi(doi: $chapterDoi) { workId }
  chapterCount(workStatus: $workStatus)
  publications(limit: 10) { publicationId }
  publication(publicationId: $publicationId) { publicationId }
  publicationCount
  publishers(limit: 10) { publisherId }
  publisher(publisherId: $publisherId) { publisherId }
  publisherCount
  imprints(limit: 10) { imprintId }
  imprint(imprintId: $imprintId) { imprintId }
  imprintCount
  contributors(limit: 10) { contributorId }
  contributor(contributorId: $contributorId) { contributorId }
  contributorCount
  contributions(limit: 10) { contributionId }
  contribution(contributionId: $contributionId) { contributionId }
  contributionCount
  serieses(limit: 10) { seriesId }
  series(seriesId: $seriesId) { seriesId }
  seriesCount
  issues(limit: 10) { issueId }
  issue(issueId: $issueId) { issueId }
  issueCount
  languages(limit: 10) { languageId }
  language(languageId: $languageId) { languageId }
  languageCount
  locations(limit: 10) { locationId }
  location(locationId: $locationId) { locationId }
  locationCount
  prices(limit: 10) { priceId }
  price(priceId: $priceId) { priceId }
  priceCount
  subjects(limit: 10) { subjectId }
  subject(subjectId: $subjectId) { subjectId }
  subjectCount
  institutions(limit: 10) { institutionId }
  institution(institutionId: $institutionId) { institutionId }
  institutionCount
  fundings(limit: 10) { fundingId }
  funding(fundingId: $fundingId) { fundingId }
  fundingCount
  affiliations(limit: 10) { affiliationId }
  affiliation(affiliationId: $affiliationId) { affiliationId }
  affiliationCount
  references(limit: 10) { referenceId }
  reference(referenceId: $referenceId) { referenceId }
  referenceCount
  title(titleId: $titleId, markupFormat: $titleMarkup) { titleId fullTitle title subtitle }
  titles(limit: 10, markupFormat: $titleMarkup) { titleId fullTitle title subtitle }
  abstract(abstractId: $abstractId, markupFormat: $abstractMarkup) { abstractId content }
  abstracts(limit: 10, markupFormat: $abstractMarkup) { abstractId content }
  biography(biographyId: $biographyId, markupFormat: $biographyMarkup) { biographyId content }
  biographies(limit: 10, markupFormat: $biographyMarkup) { biographyId content }
  contacts(limit: 10) { contactId }
  contact(contactId: $contactId) { contactId }
  contactCount
  me {
    userId
    isSuperuser
    publisherContexts {
      publisher { publisherId }
      permissions { publisherAdmin workLifecycle cdnWrite }
    }
  }
}
"#;

    let mut vars = Variables::new();
    insert_var(&mut vars, "workId", seed.book_work_id);
    insert_var(&mut vars, "bookDoi", seed.book_doi.clone());
    insert_var(&mut vars, "chapterDoi", seed.chapter_doi.clone());
    insert_var(&mut vars, "publicationId", seed.publication_id);
    insert_var(&mut vars, "publisherId", seed.publisher_id);
    insert_var(&mut vars, "imprintId", seed.imprint_id);
    insert_var(&mut vars, "contributorId", seed.contributor_id);
    insert_var(&mut vars, "contributionId", seed.contribution_id);
    insert_var(&mut vars, "seriesId", seed.series_id);
    insert_var(&mut vars, "issueId", seed.issue_id);
    insert_var(&mut vars, "languageId", seed.language_id);
    insert_var(&mut vars, "locationId", seed.location_id);
    insert_var(&mut vars, "priceId", seed.price_id);
    insert_var(&mut vars, "subjectId", seed.subject_id);
    insert_var(&mut vars, "institutionId", seed.institution_id);
    insert_var(&mut vars, "fundingId", seed.funding_id);
    insert_var(&mut vars, "affiliationId", seed.affiliation_id);
    insert_var(&mut vars, "referenceId", seed.reference_id);
    insert_var(&mut vars, "titleId", seed.title_id);
    insert_var(&mut vars, "abstractId", seed.abstract_short_id);
    insert_var(&mut vars, "biographyId", seed.biography_id);
    insert_var(&mut vars, "contactId", seed.contact_id);
    insert_var(&mut vars, "workStatus", WorkStatus::Active);
    insert_var(&mut vars, "titleMarkup", MarkupFormat::PlainText);
    insert_var(&mut vars, "abstractMarkup", MarkupFormat::PlainText);
    insert_var(&mut vars, "biographyMarkup", MarkupFormat::PlainText);

    let data = execute_graphql(&schema, &context, query, Some(vars));
    assert!(data.get("workCount").is_some());

    let org_user =
        test_db::test_user_with_role("user-2", Role::PublisherAdmin, &seed.publisher_org);
    let org_context = test_db::test_context_with_user(pool.clone(), org_user);
    let me_data = execute_graphql(
        &schema,
        &org_context,
        "query { me { userId publisherContexts { publisher { publisherId } } } }",
        None,
    );
    assert!(me_data.get("me").is_some());

    let no_role_context = test_db::test_context(pool.clone(), "user-3");
    let me_empty = execute_graphql(
        &schema,
        &no_role_context,
        "query { me { userId publisherContexts { publisher { publisherId } } } }",
        None,
    );
    assert!(me_empty.get("me").is_some());

    let _ = context.db();
    let _ = context.user();

    let work = Work::from_id(pool.as_ref(), &seed.book_work_id).unwrap();
    let title = Title::from_id(pool.as_ref(), &seed.title_id).unwrap();
    let short_abs = Abstract::from_id(pool.as_ref(), &seed.abstract_short_id).unwrap();
    let long_abs = Abstract::from_id(pool.as_ref(), &seed.abstract_long_id).unwrap();
    let biography = Biography::from_id(pool.as_ref(), &seed.biography_id).unwrap();
    assert_work_resolvers(
        &work,
        &context,
        &title,
        &short_abs,
        &long_abs,
        seed.imprint_id,
    );

    let publication = Publication::from_id(pool.as_ref(), &seed.publication_id).unwrap();
    assert_publication_resolvers(&publication, &context);

    let publisher = Publisher::from_id(pool.as_ref(), &seed.publisher_id).unwrap();
    assert_publisher_resolvers(&publisher, &context);

    let imprint = Imprint::from_id(pool.as_ref(), &seed.imprint_id).unwrap();
    assert_imprint_resolvers(&imprint, &context);

    let contributor = Contributor::from_id(pool.as_ref(), &seed.contributor_id).unwrap();
    assert_contributor_resolvers(&contributor, &context);

    let contribution = Contribution::from_id(pool.as_ref(), &seed.contribution_id).unwrap();
    assert_contribution_resolvers(&contribution, &context, &biography.content);

    let series = Series::from_id(pool.as_ref(), &seed.series_id).unwrap();
    assert_series_resolvers(&series, &context);

    let issue = Issue::from_id(pool.as_ref(), &seed.issue_id).unwrap();
    assert_issue_resolvers(&issue, &context);

    let language = Language::from_id(pool.as_ref(), &seed.language_id).unwrap();
    assert_language_resolvers(&language, &context);

    let location = Location::from_id(pool.as_ref(), &seed.location_id).unwrap();
    assert_location_resolvers(&location, &context);

    let price = Price::from_id(pool.as_ref(), &seed.price_id).unwrap();
    assert_price_resolvers(&price, &context);

    let subject = Subject::from_id(pool.as_ref(), &seed.subject_id).unwrap();
    assert_subject_resolvers(&subject, &context);

    let institution = Institution::from_id(pool.as_ref(), &seed.institution_id).unwrap();
    assert_institution_resolvers(&institution, &context);

    let funding = Funding::from_id(pool.as_ref(), &seed.funding_id).unwrap();
    assert_funding_resolvers(&funding, &context);

    let affiliation = Affiliation::from_id(pool.as_ref(), &seed.affiliation_id).unwrap();
    assert_affiliation_resolvers(&affiliation, &context);

    let work_relation = WorkRelation::from_id(pool.as_ref(), &seed.work_relation_id).unwrap();
    assert_work_relation_resolvers(&work_relation, &context);

    let reference = Reference::from_id(pool.as_ref(), &seed.reference_id).unwrap();
    assert_reference_resolvers(&reference, &context);

    let abstract_item = Abstract::from_id(pool.as_ref(), &seed.abstract_short_id).unwrap();
    assert_abstract_resolvers(&abstract_item, &context);

    assert_biography_resolvers(&biography, &context, contribution.work_id);

    let contact = Contact::from_id(pool.as_ref(), &seed.contact_id).unwrap();
    assert_contact_resolvers(&contact, &context);

    assert_title_resolvers(&title, &context);
}

#[test]
fn graphql_books_order_respects_field_and_direction() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-books-order");
    let context = test_db::test_context_with_user(pool.clone(), superuser);

    let publisher = Publisher::create(pool.as_ref(), &make_new_publisher("org-books-order"))
        .expect("Failed to create publisher");
    let imprint = Imprint::create(pool.as_ref(), &make_new_imprint(publisher.publisher_id))
        .expect("Failed to create imprint");

    let first = Work::create(
        pool.as_ref(),
        &make_new_book_work(
            imprint.imprint_id,
            Doi::from_str("https://doi.org/10.1111/BOOK.ORDER.FIRST").unwrap(),
        ),
    )
    .expect("Failed to create first book");
    let second = Work::create(
        pool.as_ref(),
        &make_new_book_work(
            imprint.imprint_id,
            Doi::from_str("https://doi.org/10.1111/BOOK.ORDER.SECOND").unwrap(),
        ),
    )
    .expect("Failed to create second book");

    let mut by_id = [first, second];
    by_id.sort_by_key(|work| work.work_id);

    let mut newer_patch: PatchWork = by_id[0].clone().into();
    newer_patch.publication_date = NaiveDate::from_ymd_opt(2025, 1, 1);
    by_id[0]
        .update(&context, &newer_patch)
        .expect("Failed to update newer book");

    let mut older_patch: PatchWork = by_id[1].clone().into();
    older_patch.publication_date = NaiveDate::from_ymd_opt(2020, 1, 1);
    by_id[1]
        .update(&context, &older_patch)
        .expect("Failed to update older book");

    let query = r#"
{
  asc: books(limit: 10, order: {field: PUBLICATION_DATE, direction: ASC}) { workId }
  desc: books(limit: 10, order: {field: PUBLICATION_DATE, direction: DESC}) { workId }
}
"#;

    let data = execute_graphql(&schema, &context, query, None);
    let asc = data["asc"].as_array().expect("Expected asc books array");
    let desc = data["desc"].as_array().expect("Expected desc books array");

    let asc_first_id = json_uuid(&asc[0]["workId"]);
    let desc_first_id = json_uuid(&desc[0]["workId"]);

    assert_eq!(asc_first_id, by_id[1].work_id);
    assert_eq!(desc_first_id, by_id[0].work_id);
}

#[test]
fn work_additional_resources_applies_markup_format_argument() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-additional-resources-markup");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);

    let resource = AdditionalResource::create(
        pool.as_ref(),
        &NewAdditionalResource {
            work_id: seed.book_work_id,
            title: "<italic>Resource Title</italic>".to_string(),
            description: Some("<p>Description <italic>markup</italic></p>".to_string()),
            attribution: None,
            resource_type: ResourceType::Video,
            doi: None,
            handle: None,
            url: Some("https://example.com/resource.mp4".to_string()),
            date: None,
            resource_ordinal: 1,
        },
    )
    .expect("Failed to create additional resource");

    let work = Work::from_id(pool.as_ref(), &seed.book_work_id).expect("Failed to load work");

    let resources_plain = work
        .additional_resources(&context, Some(10), Some(0), Some(MarkupFormat::PlainText))
        .expect("Failed to fetch additional resources in plain text");
    let plain = resources_plain
        .iter()
        .find(|item| item.additional_resource_id == resource.additional_resource_id)
        .expect("Missing created additional resource in plain text results");
    assert_eq!(plain.title, "Resource Title");
    assert_eq!(plain.description.as_deref(), Some("Description markup"));
    assert!(!plain.title.contains('<'));
    assert!(!plain
        .description
        .as_deref()
        .unwrap_or_default()
        .contains('<'));

    let resources_jats = work
        .additional_resources(&context, Some(10), Some(0), Some(MarkupFormat::JatsXml))
        .expect("Failed to fetch additional resources in JATS");
    let jats = resources_jats
        .iter()
        .find(|item| item.additional_resource_id == resource.additional_resource_id)
        .expect("Missing created additional resource in JATS results");
    assert!(jats.title.contains("<italic>"));
    assert!(jats
        .description
        .as_deref()
        .unwrap_or_default()
        .contains("<italic>"));
}

#[test]
fn graphql_work_additional_resources_uses_parent_markup_when_nested_args_omitted() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-parent-markup");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);

    let resource = AdditionalResource::create(
        pool.as_ref(),
        &NewAdditionalResource {
            work_id: seed.book_work_id,
            title: "<italic>Parent Title</italic>".to_string(),
            description: Some("<p>Parent <italic>Description</italic></p>".to_string()),
            attribution: Some("Attribution".to_string()),
            resource_type: ResourceType::Video,
            doi: None,
            handle: None,
            url: Some("https://example.com/parent-markup.mp4".to_string()),
            date: None,
            resource_ordinal: 1,
        },
    )
    .expect("Failed to create additional resource");

    let query = r#"
query ParentMarkup($id: Uuid!) {
  work(workId: $id) {
    additionalResources(markupFormat: MARKDOWN) {
      workResourceId
      title
      description
    }
  }
}
"#;
    let mut vars = Variables::new();
    insert_var(&mut vars, "id", seed.book_work_id);
    let data = execute_graphql(&schema, &context, query, Some(vars));

    let resources = data["work"]["additionalResources"]
        .as_array()
        .expect("Expected additionalResources array");
    let resource_id = resource.additional_resource_id.to_string();
    let matching = resources
        .iter()
        .find(|item| item["workResourceId"].as_str() == Some(resource_id.as_str()))
        .expect("Missing created additional resource in GraphQL response");

    let title = matching["title"]
        .as_str()
        .expect("Expected title string in GraphQL response");
    let description = matching["description"]
        .as_str()
        .expect("Expected description string in GraphQL response");

    assert_ne!(title, "<italic>Parent Title</italic>");
    assert_ne!(description, "<p>Parent <italic>Description</italic></p>");
    assert!(!title.contains('<'));
    assert!(!description.contains('<'));
}

#[test]
fn graphql_award_supports_role_prize_statement_and_new_fields() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-award-markup");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);

    let award = create_with_data_and_markup(
        &schema,
        &context,
        "createAward",
        "NewAward",
        "awardId role year jury country title(markupFormat: PLAIN_TEXT) prizeStatement(markupFormat: PLAIN_TEXT)",
        NewAward {
            work_id: seed.book_work_id,
            title: "*Award*".to_string(),
            url: Some("https://example.com/award".to_string()),
            category: Some("Prize".to_string()),
            year: Some("2025-2026".to_string()),
            jury: Some("International Jury".to_string()),
            country: Some(CountryCode::Gbr),
            prize_statement: Some("**Prize** statement".to_string()),
            role: Some(AwardRole::JointWinner),
            award_ordinal: 1,
        },
        MarkupFormat::Markdown,
    );

    assert_eq!(award["role"].as_str(), Some("JOINT_WINNER"));
    assert_eq!(award["year"].as_str(), Some("2025-2026"));
    assert_eq!(award["jury"].as_str(), Some("International Jury"));
    assert_eq!(award["country"].as_str(), Some("GBR"));
    assert_eq!(award["title"].as_str(), Some("Award"));
    assert_eq!(award["prizeStatement"].as_str(), Some("Prize statement"));

    let award_id = json_uuid(&award["awardId"]);
    let stored = Award::from_id(pool.as_ref(), &award_id).expect("Failed to fetch stored award");
    assert_eq!(stored.role, Some(AwardRole::JointWinner));
    assert_eq!(stored.year.as_deref(), Some("2025-2026"));
    assert_eq!(stored.jury.as_deref(), Some("International Jury"));
    assert_eq!(stored.country, Some(CountryCode::Gbr));
    assert!(stored.title.contains("<italic>"));
    assert!(stored
        .prize_statement
        .as_deref()
        .unwrap_or_default()
        .contains("<bold>"));
}

#[test]
fn graphql_additional_resource_exposes_date() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-resource-date");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);
    let resource_date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();

    let resource = create_with_data_and_markup(
        &schema,
        &context,
        "createAdditionalResource",
        "NewAdditionalResource",
        "workResourceId date title(markupFormat: PLAIN_TEXT) description(markupFormat: PLAIN_TEXT)",
        NewAdditionalResource {
            work_id: seed.book_work_id,
            title: "*Resource*".to_string(),
            description: Some("**Description**".to_string()),
            attribution: Some("Attribution".to_string()),
            resource_type: ResourceType::Dataset,
            doi: None,
            handle: None,
            url: Some("https://example.com/resource".to_string()),
            date: Some(resource_date),
            resource_ordinal: 1,
        },
        MarkupFormat::Markdown,
    );

    assert_eq!(resource["date"].as_str(), Some("2025-03-01"));
    assert_eq!(resource["title"].as_str(), Some("Resource"));
    assert_eq!(resource["description"].as_str(), Some("Description"));

    let resource_id = json_uuid(&resource["workResourceId"]);
    let stored = AdditionalResource::from_id(pool.as_ref(), &resource_id)
        .expect("Failed to fetch stored resource");
    assert_eq!(stored.date, Some(resource_date));
}

#[test]
fn graphql_book_review_supports_reviewer_fields_and_title_markup() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-book-review-fields");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);
    let institution = Institution::create(
        pool.as_ref(),
        &NewInstitution {
            institution_name: unique("Reviewer Institution"),
            institution_doi: None,
            ror: Some(Ror::from_str("https://ror.org/051z6e826").unwrap()),
            country_code: Some(CountryCode::Gbr),
        },
    )
    .expect("Failed to create reviewer institution");

    let review = create_with_data_and_markup(
        &schema,
        &context,
        "createBookReview",
        "NewBookReview",
        "bookReviewId title(markupFormat: PLAIN_TEXT) reviewerOrcid reviewerInstitutionId reviewerInstitution { institutionId ror } pageRange text(markupFormat: PLAIN_TEXT)",
        NewBookReview {
            work_id: seed.book_work_id,
            title: Some("*Review* Title".to_string()),
            author_name: Some("Reviewer".to_string()),
            reviewer_orcid: Some(Orcid::from_str("https://orcid.org/0000-0002-1234-5678").unwrap()),
            reviewer_institution_id: Some(institution.institution_id),
            url: Some("https://example.com/review".to_string()),
            doi: None,
            review_date: Some(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()),
            journal_name: Some("Journal".to_string()),
            journal_volume: Some("12".to_string()),
            journal_number: Some("3".to_string()),
            journal_issn: Some("1234-5678".to_string()),
            page_range: Some("10-12".to_string()),
            text: Some("**Review** text".to_string()),
            review_ordinal: 1,
        },
        MarkupFormat::Markdown,
    );
    let reviewer_institution_id = institution.institution_id.to_string();

    assert_eq!(review["title"].as_str(), Some("Review Title"));
    assert_eq!(
        review["reviewerOrcid"].as_str(),
        Some("https://orcid.org/0000-0002-1234-5678")
    );
    assert_eq!(
        review["reviewerInstitutionId"].as_str(),
        Some(reviewer_institution_id.as_str())
    );
    assert_eq!(review["pageRange"].as_str(), Some("10-12"));
    assert_eq!(review["text"].as_str(), Some("Review text"));
    assert_eq!(
        review["reviewerInstitution"]["ror"].as_str(),
        Some("https://ror.org/051z6e826")
    );

    let review_id = json_uuid(&review["bookReviewId"]);
    let stored =
        BookReview::from_id(pool.as_ref(), &review_id).expect("Failed to fetch stored review");
    assert!(stored
        .title
        .as_deref()
        .unwrap_or_default()
        .contains("<italic>"));
}

#[test]
fn graphql_endorsement_supports_author_identity_fields() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-endorsement-fields");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);
    let institution = Institution::create(
        pool.as_ref(),
        &NewInstitution {
            institution_name: unique("Author Institution"),
            institution_doi: None,
            ror: Some(Ror::from_str("https://ror.org/03yrm5c26").unwrap()),
            country_code: Some(CountryCode::Gbr),
        },
    )
    .expect("Failed to create author institution");

    let endorsement = create_with_data_and_markup(
        &schema,
        &context,
        "createEndorsement",
        "NewEndorsement",
        "endorsementId authorRole(markupFormat: PLAIN_TEXT) authorOrcid authorInstitutionId authorInstitution { institutionId ror } text(markupFormat: PLAIN_TEXT)",
        NewEndorsement {
            work_id: seed.book_work_id,
            author_name: Some("Author".to_string()),
            author_role: Some("*Visiting Scholar*".to_string()),
            author_orcid: Some(Orcid::from_str("https://orcid.org/0000-0001-2345-6789").unwrap()),
            author_institution_id: Some(institution.institution_id),
            url: Some("https://example.com/endorsement".to_string()),
            text: Some("*Excellent* book".to_string()),
            endorsement_ordinal: 1,
        },
        MarkupFormat::Markdown,
    );
    let author_institution_id = institution.institution_id.to_string();

    assert_eq!(
        endorsement["authorOrcid"].as_str(),
        Some("https://orcid.org/0000-0001-2345-6789")
    );
    assert_eq!(
        endorsement["authorInstitutionId"].as_str(),
        Some(author_institution_id.as_str())
    );
    assert_eq!(endorsement["authorRole"].as_str(), Some("Visiting Scholar"));
    assert_eq!(endorsement["text"].as_str(), Some("Excellent book"));
    assert_eq!(
        endorsement["authorInstitution"]["ror"].as_str(),
        Some("https://ror.org/03yrm5c26")
    );

    let endorsement_id = json_uuid(&endorsement["endorsementId"]);
    let stored = Endorsement::from_id(pool.as_ref(), &endorsement_id)
        .expect("Failed to fetch stored endorsement");
    assert_eq!(
        stored.author_orcid,
        Some(Orcid::from_str("https://orcid.org/0000-0001-2345-6789").unwrap())
    );
    assert!(stored
        .author_role
        .as_deref()
        .unwrap_or_default()
        .contains("<italic>"));
}

#[test]
fn graphql_update_endorsement_supports_author_role_markup() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-endorsement-author-role-markup");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);

    let endorsement = create_with_data_and_markup(
        &schema,
        &context,
        "createEndorsement",
        "NewEndorsement",
        "endorsementId authorRole(markupFormat: PLAIN_TEXT)",
        NewEndorsement {
            work_id: seed.book_work_id,
            author_name: Some("Author".to_string()),
            author_role: Some("Scholar".to_string()),
            author_orcid: None,
            author_institution_id: None,
            url: Some("https://example.com/endorsement".to_string()),
            text: Some("Excellent book".to_string()),
            endorsement_ordinal: 1,
        },
        MarkupFormat::PlainText,
    );

    let endorsement_id = json_uuid(&endorsement["endorsementId"]);
    let updated = update_with_data_and_markup(
        &schema,
        &context,
        "updateEndorsement",
        "PatchEndorsement",
        "endorsementId authorRole(markupFormat: PLAIN_TEXT)",
        PatchEndorsement {
            endorsement_id,
            work_id: seed.book_work_id,
            author_name: Some("Author".to_string()),
            author_role: Some("*Lead Editor*".to_string()),
            author_orcid: None,
            author_institution_id: None,
            url: Some("https://example.com/endorsement".to_string()),
            text: Some("Excellent book".to_string()),
            endorsement_ordinal: 1,
        },
        MarkupFormat::Markdown,
    );

    assert_eq!(updated["authorRole"].as_str(), Some("Lead Editor"));

    let stored = Endorsement::from_id(pool.as_ref(), &endorsement_id)
        .expect("Failed to fetch stored endorsement");
    assert!(stored
        .author_role
        .as_deref()
        .unwrap_or_default()
        .contains("<italic>"));
}

#[test]
fn graphql_review_and_endorsement_relations_null_after_institution_delete() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-institution-null-relations");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);
    let institution = Institution::create(
        pool.as_ref(),
        &NewInstitution {
            institution_name: unique("Linked Institution"),
            institution_doi: None,
            ror: Some(Ror::from_str("https://ror.org/04wxnsj81").unwrap()),
            country_code: Some(CountryCode::Gbr),
        },
    )
    .expect("Failed to create institution");

    let review = BookReview::create(
        pool.as_ref(),
        &NewBookReview {
            work_id: seed.book_work_id,
            title: Some("Review title".to_string()),
            author_name: Some("Reviewer".to_string()),
            reviewer_orcid: Some(Orcid::from_str("https://orcid.org/0000-0002-1234-5678").unwrap()),
            reviewer_institution_id: Some(institution.institution_id),
            url: Some("https://example.com/review".to_string()),
            doi: None,
            review_date: Some(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()),
            journal_name: Some("Journal".to_string()),
            journal_volume: Some("12".to_string()),
            journal_number: Some("3".to_string()),
            journal_issn: Some("1234-5678".to_string()),
            page_range: Some("10-12".to_string()),
            text: Some("Review text".to_string()),
            review_ordinal: 1,
        },
    )
    .expect("Failed to create review");

    let endorsement = Endorsement::create(
        pool.as_ref(),
        &NewEndorsement {
            work_id: seed.book_work_id,
            author_name: Some("Author".to_string()),
            author_role: Some("Scholar".to_string()),
            author_orcid: Some(Orcid::from_str("https://orcid.org/0000-0001-2345-6789").unwrap()),
            author_institution_id: Some(institution.institution_id),
            url: Some("https://example.com/endorsement".to_string()),
            text: Some("Endorsement text".to_string()),
            endorsement_ordinal: 1,
        },
    )
    .expect("Failed to create endorsement");

    institution
        .delete(pool.as_ref())
        .expect("Failed to delete linked institution");

    let query = r#"
query LinkedRelations($reviewId: Uuid!, $endorsementId: Uuid!) {
  bookReview(bookReviewId: $reviewId) {
    reviewerInstitutionId
    reviewerInstitution { institutionId }
  }
  endorsement(endorsementId: $endorsementId) {
    authorInstitutionId
    authorInstitution { institutionId }
  }
}
"#;
    let mut vars = Variables::new();
    insert_var(&mut vars, "reviewId", review.book_review_id);
    insert_var(&mut vars, "endorsementId", endorsement.endorsement_id);
    let data = execute_graphql(&schema, &context, query, Some(vars));

    assert!(data["bookReview"]["reviewerInstitutionId"].is_null());
    assert!(data["bookReview"]["reviewerInstitution"].is_null());
    assert!(data["endorsement"]["authorInstitutionId"].is_null());
    assert!(data["endorsement"]["authorInstitution"].is_null());
}

#[test]
fn graphql_markup_mutations_accept_plain_text_when_markup_is_jats_xml() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-jats-xml-mutations");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);

    let title = Title::from_id(pool.as_ref(), &seed.title_id).unwrap();
    update_with_data_and_markup(
        &schema,
        &context,
        "updateTitle",
        "PatchTitle",
        "titleId",
        PatchTitle {
            title_id: title.title_id,
            work_id: title.work_id,
            locale_code: title.locale_code,
            full_title: "Foundations for Moral <italic>Relativism</italic> Second Expanded Edition"
                .to_string(),
            title: "Foundations for Moral <italic>Relativism</italic>".to_string(),
            subtitle: Some("Second Expanded Edition".to_string()),
            canonical: title.canonical,
        },
        MarkupFormat::JatsXml,
    );

    let stored_title = Title::from_id(pool.as_ref(), &seed.title_id).unwrap();
    assert_eq!(
        stored_title.full_title,
        "Foundations for Moral <italic>Relativism</italic> Second Expanded Edition"
    );
    assert_eq!(
        stored_title.title,
        "Foundations for Moral <italic>Relativism</italic>"
    );
    assert_eq!(
        stored_title.subtitle.as_deref(),
        Some("Second Expanded Edition")
    );

    let abstract_item = Abstract::from_id(pool.as_ref(), &seed.abstract_short_id).unwrap();
    update_with_data_and_markup(
        &schema,
        &context,
        "updateAbstract",
        "PatchAbstract",
        "abstractId",
        PatchAbstract {
            abstract_id: abstract_item.abstract_id,
            work_id: abstract_item.work_id,
            content: "Plain abstract content updated".to_string(),
            locale_code: abstract_item.locale_code,
            abstract_type: abstract_item.abstract_type,
            canonical: abstract_item.canonical,
        },
        MarkupFormat::JatsXml,
    );

    let stored_abstract = Abstract::from_id(pool.as_ref(), &seed.abstract_short_id).unwrap();
    assert_eq!(
        stored_abstract.content,
        "<p>Plain abstract content updated</p>"
    );
}

#[test]
fn graphql_mutations_cover_all() {
    let (_guard, pool) = test_db::setup_test_db();
    let schema = create_schema();
    let superuser = test_db::test_superuser("user-4");
    let context = test_db::test_context_with_user(pool.clone(), superuser);
    let seed = seed_data(&schema, &context);

    let publisher = Publisher::from_id(pool.as_ref(), &seed.publisher_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updatePublisher",
        "PatchPublisher",
        "publisherId",
        patch_publisher(&publisher),
    );

    let imprint = Imprint::from_id(pool.as_ref(), &seed.imprint_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateImprint",
        "PatchImprint",
        "imprintId",
        patch_imprint(&imprint),
    );

    let contributor = Contributor::from_id(pool.as_ref(), &seed.contributor_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateContributor",
        "PatchContributor",
        "contributorId",
        patch_contributor(&contributor),
    );

    let contribution = Contribution::from_id(pool.as_ref(), &seed.contribution_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateContribution",
        "PatchContribution",
        "contributionId",
        patch_contribution(&contribution),
    );

    let publication = Publication::from_id(pool.as_ref(), &seed.publication_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updatePublication",
        "PatchPublication",
        "publicationId",
        patch_publication(&publication),
    );

    let series = Series::from_id(pool.as_ref(), &seed.series_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateSeries",
        "PatchSeries",
        "seriesId",
        patch_series(&series),
    );

    let issue = Issue::from_id(pool.as_ref(), &seed.issue_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateIssue",
        "PatchIssue",
        "issueId",
        patch_issue(&issue),
    );

    let language = Language::from_id(pool.as_ref(), &seed.language_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateLanguage",
        "PatchLanguage",
        "languageId",
        patch_language(&language),
    );

    let institution = Institution::from_id(pool.as_ref(), &seed.institution_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateInstitution",
        "PatchInstitution",
        "institutionId",
        patch_institution(&institution),
    );

    let funding = Funding::from_id(pool.as_ref(), &seed.funding_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateFunding",
        "PatchFunding",
        "fundingId",
        patch_funding(&funding),
    );

    let location = Location::from_id(pool.as_ref(), &seed.location_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateLocation",
        "PatchLocation",
        "locationId",
        patch_location(&location),
    );

    let price = Price::from_id(pool.as_ref(), &seed.price_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updatePrice",
        "PatchPrice",
        "priceId",
        patch_price(&price),
    );

    let subject = Subject::from_id(pool.as_ref(), &seed.subject_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateSubject",
        "PatchSubject",
        "subjectId",
        patch_subject(&subject),
    );

    let affiliation = Affiliation::from_id(pool.as_ref(), &seed.affiliation_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateAffiliation",
        "PatchAffiliation",
        "affiliationId",
        patch_affiliation(&affiliation),
    );

    let work_relation = WorkRelation::from_id(pool.as_ref(), &seed.work_relation_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateWorkRelation",
        "PatchWorkRelation",
        "workRelationId",
        patch_work_relation(&work_relation),
    );

    let reference = Reference::from_id(pool.as_ref(), &seed.reference_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateReference",
        "PatchReference",
        "referenceId",
        patch_reference(&reference),
    );

    let contact = Contact::from_id(pool.as_ref(), &seed.contact_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateContact",
        "PatchContact",
        "contactId",
        patch_contact(&contact),
    );

    let title = Title::from_id(pool.as_ref(), &seed.title_id).unwrap();
    update_with_data_and_markup(
        &schema,
        &context,
        "updateTitle",
        "PatchTitle",
        "titleId",
        patch_title(&title),
        MarkupFormat::PlainText,
    );

    let abstract_item = Abstract::from_id(pool.as_ref(), &seed.abstract_short_id).unwrap();
    update_with_data_and_markup(
        &schema,
        &context,
        "updateAbstract",
        "PatchAbstract",
        "abstractId",
        patch_abstract(&abstract_item),
        MarkupFormat::PlainText,
    );

    let biography = Biography::from_id(pool.as_ref(), &seed.biography_id).unwrap();
    update_with_data_and_markup(
        &schema,
        &context,
        "updateBiography",
        "PatchBiography",
        "biographyId",
        patch_biography(&biography),
        MarkupFormat::PlainText,
    );

    let work = Work::from_id(pool.as_ref(), &seed.book_work_id).unwrap();
    update_with_data(
        &schema,
        &context,
        "updateWork",
        "PatchWork",
        "workId",
        PatchWork::from(work),
    );

    move_with_ordinal(
        &schema,
        &context,
        "moveAffiliation",
        "affiliationId",
        seed.affiliation_id,
        1,
        "affiliationId",
    );
    move_with_ordinal(
        &schema,
        &context,
        "moveAffiliation",
        "affiliationId",
        seed.affiliation_id,
        2,
        "affiliationId",
    );

    move_with_ordinal(
        &schema,
        &context,
        "moveContribution",
        "contributionId",
        seed.contribution_id,
        1,
        "contributionId",
    );
    move_with_ordinal(
        &schema,
        &context,
        "moveContribution",
        "contributionId",
        seed.contribution_id,
        2,
        "contributionId",
    );

    move_with_ordinal(
        &schema,
        &context,
        "moveIssue",
        "issueId",
        seed.issue_id,
        1,
        "issueId",
    );
    move_with_ordinal(
        &schema,
        &context,
        "moveIssue",
        "issueId",
        seed.issue_id,
        2,
        "issueId",
    );

    move_with_ordinal(
        &schema,
        &context,
        "moveReference",
        "referenceId",
        seed.reference_id,
        1,
        "referenceId",
    );
    move_with_ordinal(
        &schema,
        &context,
        "moveReference",
        "referenceId",
        seed.reference_id,
        2,
        "referenceId",
    );

    move_with_ordinal(
        &schema,
        &context,
        "moveSubject",
        "subjectId",
        seed.subject_id,
        1,
        "subjectId",
    );
    move_with_ordinal(
        &schema,
        &context,
        "moveSubject",
        "subjectId",
        seed.subject_id,
        2,
        "subjectId",
    );

    move_with_ordinal(
        &schema,
        &context,
        "moveWorkRelation",
        "workRelationId",
        seed.work_relation_id,
        1,
        "workRelationId",
    );
    move_with_ordinal(
        &schema,
        &context,
        "moveWorkRelation",
        "workRelationId",
        seed.work_relation_id,
        2,
        "workRelationId",
    );

    delete_with_id(
        &schema,
        &context,
        "deleteContact",
        "contactId",
        seed.contact_id,
        "contactId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteBiography",
        "biographyId",
        seed.biography_id,
        "biographyId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteAbstract",
        "abstractId",
        seed.abstract_short_id,
        "abstractId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteAbstract",
        "abstractId",
        seed.abstract_long_id,
        "abstractId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteReference",
        "referenceId",
        seed.reference_id,
        "referenceId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteReference",
        "referenceId",
        seed.reference_id_two,
        "referenceId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteWorkRelation",
        "workRelationId",
        seed.work_relation_id,
        "workRelationId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteWorkRelation",
        "workRelationId",
        seed.work_relation_id_two,
        "workRelationId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteAffiliation",
        "affiliationId",
        seed.affiliation_id,
        "affiliationId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteAffiliation",
        "affiliationId",
        seed.affiliation_id_two,
        "affiliationId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteSubject",
        "subjectId",
        seed.subject_id,
        "subjectId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteSubject",
        "subjectId",
        seed.subject_id_two,
        "subjectId",
    );
    delete_with_id(
        &schema,
        &context,
        "deletePrice",
        "priceId",
        seed.price_id,
        "priceId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteLocation",
        "locationId",
        seed.location_id,
        "locationId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteFunding",
        "fundingId",
        seed.funding_id,
        "fundingId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteInstitution",
        "institutionId",
        seed.institution_id,
        "institutionId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteTitle",
        "titleId",
        seed.title_id,
        "titleId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteLanguage",
        "languageId",
        seed.language_id,
        "languageId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteIssue",
        "issueId",
        seed.issue_id,
        "issueId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteIssue",
        "issueId",
        seed.issue_id_two,
        "issueId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteSeries",
        "seriesId",
        seed.series_id,
        "seriesId",
    );
    delete_with_id(
        &schema,
        &context,
        "deletePublication",
        "publicationId",
        seed.publication_id,
        "publicationId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteContribution",
        "contributionId",
        seed.contribution_id,
        "contributionId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteContribution",
        "contributionId",
        seed.contribution_id_two,
        "contributionId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteContributor",
        "contributorId",
        seed.contributor_id,
        "contributorId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteContributor",
        "contributorId",
        seed.contributor_id_two,
        "contributorId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteWork",
        "workId",
        seed.book_work_id,
        "workId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteWork",
        "workId",
        seed.chapter_work_id,
        "workId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteWork",
        "workId",
        seed.other_chapter_work_id,
        "workId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteWork",
        "workId",
        seed.issue_work_id,
        "workId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteWork",
        "workId",
        seed.issue_work_id_two,
        "workId",
    );
    delete_with_id(
        &schema,
        &context,
        "deleteImprint",
        "imprintId",
        seed.imprint_id,
        "imprintId",
    );
    delete_with_id(
        &schema,
        &context,
        "deletePublisher",
        "publisherId",
        seed.publisher_id,
        "publisherId",
    );
}
