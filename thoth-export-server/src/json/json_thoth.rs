use chrono::SecondsFormat;
use serde::Serialize;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

use super::JsonSpecification;

#[derive(Copy, Clone)]
pub(crate) struct JsonThoth;

const JSON_ERROR: &str = "json::thoth";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonWrapper<T: Serialize> {
    json_generated_at: String,
    #[serde(flatten)]
    wrapped_struct: T,
}

impl JsonSpecification for JsonThoth {
    fn handle_event(works: &[Work]) -> ThothResult<String> {
        match works {
            [] => Err(ThothError::IncompleteMetadataRecord(
                JSON_ERROR.to_string(),
                "Not enough data".to_string(),
            )),
            [work] => {
                let wrapper = JsonWrapper {
                    json_generated_at: chrono::Utc::now()
                        .to_rfc3339_opts(SecondsFormat::Secs, true),
                    wrapped_struct: {
                        // Sort subjects alphabetically by type. Like all other child objects
                        // which have ordinals, they were already sorted by ordinal by WorkQuery.
                        let mut ordered_work = work.clone();
                        ordered_work.subjects.sort_by(|a, b| {
                            a.subject_type.to_string().cmp(&b.subject_type.to_string())
                        });
                        ordered_work
                    },
                };
                serde_json::to_string_pretty(&wrapper)
                    .map_err(|e| ThothError::InternalError(e.to_string()))
            }
            // handler::by_publisher() prevents generation of output for multiple records
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::str::FromStr;
    use thoth_api::model::Doi;
    use thoth_api::model::Isbn;
    use thoth_api::model::Orcid;
    use thoth_api::model::Ror;
    use thoth_client::{
        ContributionType, CountryCode, CurrencyCode, FundingInstitution, LanguageCode,
        LanguageRelation, LocationPlatform, PublicationType, RelationType, SeriesType, SubjectType,
        Work, WorkContributions, WorkContributionsAffiliations,
        WorkContributionsAffiliationsInstitution, WorkContributionsContributor, WorkFundings,
        WorkImprint, WorkImprintPublisher, WorkIssues, WorkIssuesSeries, WorkLanguages,
        WorkPublications, WorkPublicationsLocations, WorkPublicationsPrices, WorkReferences,
        WorkRelations, WorkRelationsRelatedWork, WorkRelationsRelatedWorkImprint,
        WorkRelationsRelatedWorkImprintPublisher, WorkStatus, WorkSubjects, WorkType,
    };
    use uuid::Uuid;

    // This tests the sorting of subjects by subject type, but cannot test the sorting
    // of child objects by ordinal which occurs in the WorkQuery itself. This is because
    // TEST_WORK mocks the output of WorkQuery. All child objects with ordinals have been
    // listed within TEST_WORK in correct ordinal order.
    lazy_static! {
        static ref TEST_WORK: Work = Work {
            work_id: Uuid::from_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_status: WorkStatus::ACTIVE,
            full_title: "Book Title: Book Subtitle".to_string(),
            title: "Book Title".to_string(),
            subtitle: Some("Book Subtitle".to_string()),
            work_type: WorkType::MONOGRAPH,
            reference: None,
            edition: Some(1),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: chrono::NaiveDate::from_ymd_opt(1999, 12, 31),
            license: Some("http://creativecommons.org/licenses/by/4.0/".to_string()),
            copyright_holder: Some("Author 1; Author 2".to_string()),
            short_abstract: Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.".to_string()),
            long_abstract: Some("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.".to_string()),
            general_note: Some("This is a general note".to_string()),
            bibliography_note: Some("This is a bibliography note".to_string()),
            place: Some("León, Spain".to_string()),
            page_count: Some(334),
            page_breakdown: Some("x+334".to_string()),
            first_page: None,
            last_page: None,
            page_interval: None,
            image_count: Some(15),
            table_count: Some(20),
            audio_count: Some(25),
            video_count: Some(30),
            landing_page: Some("https://www.book.com".to_string()),
            toc: Some("1. Chapter 1".to_string()),
            lccn: Some("123456789".to_string()),
            oclc: Some("987654321".to_string()),
            cover_url: Some("https://www.book.com/cover".to_string()),
            cover_caption: Some("This is a cover caption".to_string()),
            imprint: WorkImprint {
                imprint_name: "OA Editions Imprint".to_string(),
                imprint_url: None,
                publisher: WorkImprintPublisher {
                    publisher_name: "OA Editions".to_string(),
                    publisher_shortname: Some("OAE".to_string()),
                    publisher_url: None,
                },
            },
            issues: vec![WorkIssues {
                issue_ordinal: 1,
                series: WorkIssuesSeries {
                    series_type: SeriesType::JOURNAL,
                    series_name: "Name of series".to_string(),
                    issn_print: "1234-5678".to_string(),
                    issn_digital: "8765-4321".to_string(),
                    series_url: Some("https://www.series.com".to_string()),
                    series_description: Some("Description of series".to_string()),
                    series_cfp_url: Some("https://www.series.com/cfp".to_string()),
                },
            }],
            contributions: vec![
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "1".to_string(),
                    full_name: "Author 1".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 1,
                    contributor: WorkContributionsContributor {
                        orcid: Some(Orcid::from_str("https://orcid.org/0000-0002-0000-0001").unwrap()),
                        website: None,
                    },
                    affiliations: vec![
                        WorkContributionsAffiliations {
                            position: Some("Manager".to_string()),
                            affiliation_ordinal: 1,
                            institution: WorkContributionsAffiliationsInstitution {
                                institution_name: "University of Life".to_string(),
                                institution_doi: None,
                                ror: None,
                                country_code: None,
                            },
                        },
                    ],
                },
                WorkContributions {
                    contribution_type: ContributionType::AUTHOR,
                    first_name: Some("Author".to_string()),
                    last_name: "2".to_string(),
                    full_name: "Author 2".to_string(),
                    main_contribution: true,
                    biography: None,
                    contribution_ordinal: 2,
                    contributor: WorkContributionsContributor {
                        orcid: None,
                        website: None,
                    },
                    affiliations: vec![],
                },
            ],
            languages: vec![
                WorkLanguages {
                    language_code: LanguageCode::SPA,
                    language_relation: LanguageRelation::ORIGINAL,
                    main_language: true,
                },
            ],
            publications: vec![
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
                    publication_type: PublicationType::PAPERBACK,
                    width_mm: Some(156.0),
                    width_cm: Some(15.6),
                    width_in: Some(6.14),
                    height_mm: Some(234.0),
                    height_cm: Some(23.4),
                    height_in: Some(9.21),
                    depth_mm: Some(25.0),
                    depth_cm: Some(2.5),
                    depth_in: Some(1.0),
                    weight_g: Some(152.0),
                    weight_oz: Some(5.3616),
                    isbn: Some(Isbn::from_str("978-3-16-148410-0").unwrap()),
                    prices: vec![
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::EUR,
                            unit_price: 25.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::GBP,
                            unit_price: 22.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::USD,
                            unit_price: 31.95,
                        },
                    ],
                    locations: vec![
                        WorkPublicationsLocations {
                            landing_page: Some("https://www.book.com/paperback".to_string()),
                            full_text_url: None,
                            location_platform: LocationPlatform::OTHER,
                            canonical: true,
                        },
                        WorkPublicationsLocations {
                            landing_page: Some("https://www.jstor.com/paperback".to_string()),
                            full_text_url: None,
                            location_platform: LocationPlatform::JSTOR,
                            canonical: false,
                        },
                    ],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-CCCC-000000000003").unwrap(),
                    publication_type: PublicationType::HARDBACK,
                    isbn: Some(Isbn::from_str("978-1-4028-9462-6").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::EUR,
                            unit_price: 36.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::GBP,
                            unit_price: 32.95,
                        },
                        WorkPublicationsPrices {
                            currency_code: CurrencyCode::USD,
                            unit_price: 40.95,
                        },
                    ],
                    locations: vec![],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-DDDD-000000000004").unwrap(),
                    publication_type: PublicationType::PDF,
                    isbn: Some(Isbn::from_str("978-1-56619-909-4").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![WorkPublicationsLocations {
                        landing_page: Some("https://www.book.com/pdf_landing".to_string()),
                        full_text_url: Some("https://www.book.com/pdf_fulltext".to_string()),
                        location_platform: LocationPlatform::OTHER,
                        canonical: true,
                    }],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-EEEE-000000000005").unwrap(),
                    publication_type: PublicationType::HTML,
                    isbn: None,
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![WorkPublicationsLocations {
                        landing_page: Some("https://www.book.com/html_landing".to_string()),
                        full_text_url: Some("https://www.book.com/html_fulltext".to_string()),
                        location_platform: LocationPlatform::OTHER,
                        canonical: true,
                    }],
                },
                WorkPublications {
                    publication_id: Uuid::from_str("00000000-0000-0000-FFFF-000000000006").unwrap(),
                    publication_type: PublicationType::XML,
                    isbn: Some(Isbn::from_str("978-92-95055-02-5").unwrap()),
                    width_mm: None,
                    width_cm: None,
                    width_in: None,
                    height_mm: None,
                    height_cm: None,
                    height_in: None,
                    depth_mm: None,
                    depth_cm: None,
                    depth_in: None,
                    weight_g: None,
                    weight_oz: None,
                    prices: vec![],
                    locations: vec![],
                },
            ],
            subjects: vec![
                WorkSubjects {
                    subject_code: "AAA".to_string(),
                    subject_type: SubjectType::BIC,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "AAA000000".to_string(),
                    subject_type: SubjectType::BISAC,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "AAB".to_string(),
                    subject_type: SubjectType::BIC,
                    subject_ordinal: 2,
                },
                WorkSubjects {
                    subject_code: "AAA000001".to_string(),
                    subject_type: SubjectType::BISAC,
                    subject_ordinal: 2,
                },
                WorkSubjects {
                    subject_code: "keyword1".to_string(),
                    subject_type: SubjectType::KEYWORD,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "Category1".to_string(),
                    subject_type: SubjectType::CUSTOM,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "keyword2".to_string(),
                    subject_type: SubjectType::KEYWORD,
                    subject_ordinal: 2,
                },
                WorkSubjects {
                    subject_code: "JA85".to_string(),
                    subject_type: SubjectType::LCC,
                    subject_ordinal: 1,
                },
                WorkSubjects {
                    subject_code: "JWA".to_string(),
                    subject_type: SubjectType::THEMA,
                    subject_ordinal: 1,
                },
            ],
            fundings: vec![WorkFundings {
                program: Some("Name of program".to_string()),
                project_name: Some("Name of project".to_string()),
                project_shortname: None,
                grant_number: Some("Number of grant".to_string()),
                jurisdiction: Some("Funding jurisdiction".to_string()),
                institution: FundingInstitution {
                    institution_name: "Name of institution".to_string(),
                    institution_doi: Some(Doi::from_str("https://doi.org/10.00001/INSTITUTION.0001").unwrap()),
                    ror: Some(Ror::from_str("https://ror.org/0aaaaaa00").unwrap()),
                    country_code: Some(CountryCode::MDA),
                },
            }],
            relations: vec![WorkRelations {
                relation_type: RelationType::HAS_CHILD,
                relation_ordinal: 1,
                related_work: WorkRelationsRelatedWork {
                    full_title: "Related work title".to_string(),
                    title: "N/A".to_string(),
                    subtitle: None,
                    edition: None,
                    doi: None,
                    publication_date: None,
                    license: None,
                    short_abstract: None,
                    long_abstract: None,
                    place: None,
                    first_page: None,
                    last_page: None,
                    landing_page: None,
                    imprint: WorkRelationsRelatedWorkImprint {
                        publisher: WorkRelationsRelatedWorkImprintPublisher {
                            publisher_name: "N/A".to_string(),
                        },
                    },
                    contributions: vec![],
                    publications: vec![],
                    references: vec![],
                    fundings: vec![],
                },
            }],
            references: vec![WorkReferences {
                reference_ordinal: 1,
                doi: Some(Doi::from_str("https://doi.org/10.00001/reference").unwrap()),
                unstructured_citation: Some("Author, A. (2022) Article, Journal.".to_string()),
                issn: Some("1111-2222".to_string()),
                isbn: None,
                journal_title: Some("Journal".to_string()),
                article_title: Some("Article".to_string()),
                series_title: None,
                volume_title: None,
                edition: None,
                author: Some("Author, A".to_string()),
                volume: None,
                issue: None,
                first_page: Some("3".to_string()),
                component_number: None,
                standard_designator: None,
                standards_body_name: None,
                standards_body_acronym: None,
                publication_date: chrono::NaiveDate::from_ymd_opt(2022, 1, 1),
                retrieval_date: None,
            }],
        };
    }

    const TEST_RESULT: &str = r#"
  "workId": "00000000-0000-0000-aaaa-000000000001",
  "workStatus": "ACTIVE",
  "fullTitle": "Book Title: Book Subtitle",
  "title": "Book Title",
  "subtitle": "Book Subtitle",
  "workType": "MONOGRAPH",
  "reference": null,
  "edition": 1,
  "doi": "https://doi.org/10.00001/BOOK.0001",
  "publicationDate": "1999-12-31",
  "license": "http://creativecommons.org/licenses/by/4.0/",
  "copyrightHolder": "Author 1; Author 2",
  "shortAbstract": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.",
  "longAbstract": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum vel libero eleifend, ultrices purus vitae, suscipit ligula. Aliquam ornare quam et nulla vestibulum, id euismod tellus malesuada. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam ornare bibendum ex nec dapibus. Proin porta risus elementum odio feugiat tempus. Etiam eu felis ac metus viverra ornare. In consectetur neque sed feugiat ornare. Mauris at purus fringilla orci tincidunt pulvinar sed a massa. Nullam vestibulum posuere augue, sit amet tincidunt nisl pulvinar ac.",
  "generalNote": "This is a general note",
  "bibliographyNote": "This is a bibliography note",
  "place": "León, Spain",
  "pageCount": 334,
  "pageBreakdown": "x+334",
  "firstPage": null,
  "lastPage": null,
  "pageInterval": null,
  "imageCount": 15,
  "tableCount": 20,
  "audioCount": 25,
  "videoCount": 30,
  "landingPage": "https://www.book.com",
  "toc": "1. Chapter 1",
  "lccn": "123456789",
  "oclc": "987654321",
  "coverUrl": "https://www.book.com/cover",
  "coverCaption": "This is a cover caption",
  "imprint": {
    "imprintName": "OA Editions Imprint",
    "imprintUrl": null,
    "publisher": {
      "publisherName": "OA Editions",
      "publisherShortname": "OAE",
      "publisherUrl": null
    }
  },
  "issues": [
    {
      "issueOrdinal": 1,
      "series": {
        "seriesType": "JOURNAL",
        "seriesName": "Name of series",
        "issnPrint": "1234-5678",
        "issnDigital": "8765-4321",
        "seriesUrl": "https://www.series.com",
        "seriesDescription": "Description of series",
        "seriesCfpUrl": "https://www.series.com/cfp"
      }
    }
  ],
  "contributions": [
    {
      "contributionType": "AUTHOR",
      "firstName": "Author",
      "lastName": "1",
      "fullName": "Author 1",
      "mainContribution": true,
      "biography": null,
      "contributionOrdinal": 1,
      "contributor": {
        "orcid": "https://orcid.org/0000-0002-0000-0001",
        "website": null
      },
      "affiliations": [
        {
          "position": "Manager",
          "affiliationOrdinal": 1,
          "institution": {
            "institutionName": "University of Life",
            "institutionDoi": null,
            "ror": null,
            "countryCode": null
          }
        }
      ]
    },
    {
      "contributionType": "AUTHOR",
      "firstName": "Author",
      "lastName": "2",
      "fullName": "Author 2",
      "mainContribution": true,
      "biography": null,
      "contributionOrdinal": 2,
      "contributor": {
        "orcid": null,
        "website": null
      },
      "affiliations": []
    }
  ],
  "languages": [
    {
      "languageCode": "SPA",
      "languageRelation": "ORIGINAL",
      "mainLanguage": true
    }
  ],
  "publications": [
    {
      "publicationId": "00000000-0000-0000-bbbb-000000000002",
      "publicationType": "PAPERBACK",
      "isbn": "978-3-16-148410-0",
      "weightG": 152.0,
      "weightOz": 5.3616,
      "widthMm": 156.0,
      "widthCm": 15.6,
      "widthIn": 6.14,
      "heightMm": 234.0,
      "heightCm": 23.4,
      "heightIn": 9.21,
      "depthMm": 25.0,
      "depthCm": 2.5,
      "depthIn": 1.0,
      "prices": [
        {
          "currencyCode": "EUR",
          "unitPrice": 25.95
        },
        {
          "currencyCode": "GBP",
          "unitPrice": 22.95
        },
        {
          "currencyCode": "USD",
          "unitPrice": 31.95
        }
      ],
      "locations": [
        {
          "landingPage": "https://www.book.com/paperback",
          "fullTextUrl": null,
          "locationPlatform": "OTHER",
          "canonical": true
        },
        {
          "landingPage": "https://www.jstor.com/paperback",
          "fullTextUrl": null,
          "locationPlatform": "JSTOR",
          "canonical": false
        }
      ]
    },
    {
      "publicationId": "00000000-0000-0000-cccc-000000000003",
      "publicationType": "HARDBACK",
      "isbn": "978-1-4028-9462-6",
      "weightG": null,
      "weightOz": null,
      "widthMm": null,
      "widthCm": null,
      "widthIn": null,
      "heightMm": null,
      "heightCm": null,
      "heightIn": null,
      "depthMm": null,
      "depthCm": null,
      "depthIn": null,
      "prices": [
        {
          "currencyCode": "EUR",
          "unitPrice": 36.95
        },
        {
          "currencyCode": "GBP",
          "unitPrice": 32.95
        },
        {
          "currencyCode": "USD",
          "unitPrice": 40.95
        }
      ],
      "locations": []
    },
    {
      "publicationId": "00000000-0000-0000-dddd-000000000004",
      "publicationType": "PDF",
      "isbn": "978-1-56619-909-4",
      "weightG": null,
      "weightOz": null,
      "widthMm": null,
      "widthCm": null,
      "widthIn": null,
      "heightMm": null,
      "heightCm": null,
      "heightIn": null,
      "depthMm": null,
      "depthCm": null,
      "depthIn": null,
      "prices": [],
      "locations": [
        {
          "landingPage": "https://www.book.com/pdf_landing",
          "fullTextUrl": "https://www.book.com/pdf_fulltext",
          "locationPlatform": "OTHER",
          "canonical": true
        }
      ]
    },
    {
      "publicationId": "00000000-0000-0000-eeee-000000000005",
      "publicationType": "HTML",
      "isbn": null,
      "weightG": null,
      "weightOz": null,
      "widthMm": null,
      "widthCm": null,
      "widthIn": null,
      "heightMm": null,
      "heightCm": null,
      "heightIn": null,
      "depthMm": null,
      "depthCm": null,
      "depthIn": null,
      "prices": [],
      "locations": [
        {
          "landingPage": "https://www.book.com/html_landing",
          "fullTextUrl": "https://www.book.com/html_fulltext",
          "locationPlatform": "OTHER",
          "canonical": true
        }
      ]
    },
    {
      "publicationId": "00000000-0000-0000-ffff-000000000006",
      "publicationType": "XML",
      "isbn": "978-92-95055-02-5",
      "weightG": null,
      "weightOz": null,
      "widthMm": null,
      "widthCm": null,
      "widthIn": null,
      "heightMm": null,
      "heightCm": null,
      "heightIn": null,
      "depthMm": null,
      "depthCm": null,
      "depthIn": null,
      "prices": [],
      "locations": []
    }
  ],
  "subjects": [
    {
      "subjectCode": "AAA",
      "subjectType": "BIC",
      "subjectOrdinal": 1
    },
    {
      "subjectCode": "AAB",
      "subjectType": "BIC",
      "subjectOrdinal": 2
    },
    {
      "subjectCode": "AAA000000",
      "subjectType": "BISAC",
      "subjectOrdinal": 1
    },
    {
      "subjectCode": "AAA000001",
      "subjectType": "BISAC",
      "subjectOrdinal": 2
    },
    {
      "subjectCode": "Category1",
      "subjectType": "CUSTOM",
      "subjectOrdinal": 1
    },
    {
      "subjectCode": "keyword1",
      "subjectType": "KEYWORD",
      "subjectOrdinal": 1
    },
    {
      "subjectCode": "keyword2",
      "subjectType": "KEYWORD",
      "subjectOrdinal": 2
    },
    {
      "subjectCode": "JA85",
      "subjectType": "LCC",
      "subjectOrdinal": 1
    },
    {
      "subjectCode": "JWA",
      "subjectType": "THEMA",
      "subjectOrdinal": 1
    }
  ],
  "fundings": [
    {
      "program": "Name of program",
      "projectName": "Name of project",
      "projectShortname": null,
      "grantNumber": "Number of grant",
      "jurisdiction": "Funding jurisdiction",
      "institution": {
        "institutionName": "Name of institution",
        "institutionDoi": "https://doi.org/10.00001/INSTITUTION.0001",
        "ror": "https://ror.org/0aaaaaa00",
        "countryCode": "MDA"
      }
    }
  ],
  "relations": [
    {
      "relationType": "HAS_CHILD",
      "relationOrdinal": 1,
      "relatedWork": {
        "fullTitle": "Related work title",
        "title": "N/A",
        "subtitle": null,
        "edition": null,
        "doi": null,
        "publicationDate": null,
        "license": null,
        "shortAbstract": null,
        "longAbstract": null,
        "place": null,
        "firstPage": null,
        "lastPage": null,
        "landingPage": null,
        "imprint": {
          "publisher": {
            "publisherName": "N/A"
          }
        },
        "contributions": [],
        "publications": [],
        "fundings": [],
        "references": []
      }
    }
  ],
  "references": [
    {
      "referenceOrdinal": 1,
      "doi": "https://doi.org/10.00001/reference",
      "unstructuredCitation": "Author, A. (2022) Article, Journal.",
      "issn": "1111-2222",
      "isbn": null,
      "journalTitle": "Journal",
      "articleTitle": "Article",
      "seriesTitle": null,
      "volumeTitle": null,
      "edition": null,
      "author": "Author, A",
      "volume": null,
      "issue": null,
      "firstPage": "3",
      "componentNumber": null,
      "standardDesignator": null,
      "standardsBodyName": null,
      "standardsBodyAcronym": null,
      "publicationDate": "2022-01-01",
      "retrievalDate": null
    }
  ]
}"#;

    #[test]
    fn test_json_thoth() {
        let to_test = JsonThoth.generate(&[TEST_WORK.clone()]);
        assert!(to_test.is_ok());
        let to_test_string = to_test.unwrap();
        let to_test_split = to_test_string.split_once(',');
        assert!(to_test_split.is_some());
        let to_test_tuple = to_test_split.unwrap();
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r#"\{\n  "jsonGeneratedAt": "\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z"#)
                    .unwrap();
        }
        let matches = RE.captures(to_test_tuple.0);
        assert!(matches.is_some());
        assert_eq!(to_test_tuple.1, TEST_RESULT);
    }
}
