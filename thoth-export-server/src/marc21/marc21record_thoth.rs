use crate::marc21::Marc21Field;
use cc_license::License;
use chrono::Datelike;
use marc::{FieldRepr, Record, RecordBuilder};
use std::collections::HashMap;
use thoth_api::model::contribution::ContributionType;
use thoth_api::model::publication::PublicationType;
use thoth_client::{
    SubjectType, Work, WorkContributions, WorkPublications, WorkSubjects, WorkType,
};
use thoth_errors::{ThothError, ThothResult};

use super::{Marc21Entry, Marc21Specification};

#[derive(Copy, Clone)]
pub(crate) struct Marc21RecordThoth;

impl Marc21Specification for Marc21RecordThoth {
    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()> {
        match works.len() {
            0 => Err(ThothError::IncompleteMetadataRecord(
                "marc21record::thoth".to_string(),
                "Not enough data".to_string(),
            )),
            1 => Marc21Entry::<Marc21RecordThoth>::marc21_record(works.first().unwrap(), w),
            _ => {
                for work in works.iter() {
                    // Do not include Chapters in full publisher metadata record
                    // (assumes that a publisher will always have more than one work)
                    if work.work_type != WorkType::BOOK_CHAPTER {
                        Marc21Entry::<Marc21RecordThoth>::marc21_record(work, w).ok();
                    }
                }
                Ok(())
            }
        }
    }
}

impl Marc21Entry<Marc21RecordThoth> for Work {
    fn to_record(&self) -> ThothResult<Record> {
        let mut builder = RecordBuilder::new();

        // 001 - control number
        builder.add_field((b"001", self.work_id.to_string()))?;

        // 006 - media type
        builder.add_field((b"006", "m        d        "))?;

        // 007 - characteristics
        builder.add_field((b"007", "cr  n         "))?;

        // 010 - LCCN
        if let Some(lccn) = self.lccn.clone() {
            let mut lccn_field: FieldRepr = FieldRepr::from((b"010", "\\\\"));
            lccn_field = lccn_field.add_subfield(b"a", lccn.as_bytes())?;
            builder.add_field(lccn_field)?;
        }

        // 020 - ISBN
        for publication in &self.publications {
            Marc21Field::<Marc21RecordThoth>::to_field(publication, &mut builder)?;
        }

        // 024 - DOI
        if let Some(doi) = &self.doi {
            let mut doi_field: FieldRepr = FieldRepr::from((b"024", "7\\"));
            doi_field = doi_field.add_subfield(b"a", doi.to_string().as_bytes())?;
            doi_field = doi_field.add_subfield(b"2", "doi")?;
            builder.add_field(doi_field)?;
        }

        // 040 - cataloging source field \\$aStSaUL$beng$erda
        let mut cataloguing_field: FieldRepr = FieldRepr::from((b"040", "\\\\"));
        cataloguing_field = cataloguing_field.add_subfield(b"a", "Thoth")?;
        cataloguing_field = cataloguing_field.add_subfield(b"b", "eng")?;
        cataloguing_field = cataloguing_field.add_subfield(b"e", "rda")?;
        builder.add_field(cataloguing_field)?;

        // 050 - LCC
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::LCC)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 072 - BIC
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::BIC)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 072 - BISAC
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::BISAC)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 072 - Thema
        for subject in self
            .subjects
            .iter()
            .filter(|s| s.subject_type == SubjectType::THEMA)
        {
            Marc21Field::<Marc21RecordThoth>::to_field(subject, &mut builder)?;
        }

        // 245 – title
        let mut title_field: FieldRepr = FieldRepr::from((b"245", "00")); // no title added entry
        title_field = title_field.add_subfield(b"a", self.title.clone().into_bytes())?; // main title
        title_field = title_field.add_subfield(b"h", b"[electronic resource] :")?; // general material designation (GMD)
        if let Some(subtitle) = self.subtitle.clone() {
            title_field = title_field.add_subfield(b"b", subtitle.into_bytes())?;
            // subtitle
        }
        title_field =
            title_field.add_subfield(b"c", contributors_string(&self.contributions).as_bytes())?; // statement of responsibility
        builder.add_field(title_field)?;

        // 264 - publication
        let mut publication_field: FieldRepr = FieldRepr::from((b"264", "\\1"));
        if let Some(place) = self.place.clone() {
            publication_field = publication_field.add_subfield(b"a", place.into_bytes())?;
            // place of publication
        }
        publication_field = publication_field.add_subfield(
            b"b",
            self.imprint.publisher.publisher_name.clone().into_bytes(),
        )?; // publisher
        if let Some(publication_date) = self.publication_date {
            // year of publication is used in two 264 fields, let's do both
            let year = publication_date.year().to_string();
            publication_field = publication_field.add_subfield(b"c", year.clone().into_bytes())?;
            let mut copyright_year_field = FieldRepr::from((b"264", "\\4"));
            copyright_year_field =
                copyright_year_field.add_subfield(b"c", format!("©{}", year).into_bytes())?;
            builder.add_field(publication_field)?;
            builder.add_field(copyright_year_field)?;
        } else {
            builder.add_field(publication_field)?;
        }

        // 300 - extent and physical description
        let mut extent_field: FieldRepr = FieldRepr::from((b"300", "\\\\"));
        let (extent_str, resource_count) = description_string(self);
        extent_field = extent_field.add_subfield(b"a", extent_str)?;
        if let Some(resource_count_str) = resource_count {
            extent_field = extent_field.add_subfield(b"b", resource_count_str)?;
        }
        builder.add_field(extent_field)?;

        // 336 - content type
        let mut content_type_field: FieldRepr = FieldRepr::from((b"336", "\\\\"));
        content_type_field = content_type_field.add_subfield(b"a", "text")?;
        content_type_field = content_type_field.add_subfield(b"b", "txt")?;
        content_type_field = content_type_field.add_subfield(b"2", "rdacontent")?;
        builder.add_field(content_type_field)?;

        // 337 - type of material
        let mut material_field: FieldRepr = FieldRepr::from((b"337", "\\\\"));
        material_field = material_field.add_subfield(b"a", "computer")?;
        material_field = material_field.add_subfield(b"b", "c")?;
        material_field = material_field.add_subfield(b"2", "rdamedia")?;
        builder.add_field(material_field)?;

        // 338 - type of media
        let mut media_field: FieldRepr = FieldRepr::from((b"338", "\\\\"));
        media_field = media_field.add_subfield(b"a", "online resource")?;
        media_field = media_field.add_subfield(b"b", "cr")?;
        media_field = media_field.add_subfield(b"2", "rdacarrier")?;
        builder.add_field(media_field)?;

        // 500 - availability
        let mut availability_field: FieldRepr = FieldRepr::from((b"500", "\\\\"));
        availability_field = availability_field.add_subfield(
            b"a",
            format!(
                "Available through {}.",
                self.imprint.publisher.publisher_name.clone()
            )
            .into_bytes(),
        )?;
        builder.add_field(availability_field)?;

        // 504 - general note
        if let Some(general_note) = self.general_note.clone() {
            let mut note_field: FieldRepr = FieldRepr::from((b"504", "\\\\"));
            note_field = note_field.add_subfield(b"a", general_note.into_bytes())?;
            builder.add_field(note_field)?;
        }

        // 505 - contents note
        if let Some(toc) = self.toc.clone() {
            let mut toc_field: FieldRepr = FieldRepr::from((b"505", "0\\"));
            toc_field = toc_field.add_subfield(b"a", toc.into_bytes())?;
            builder.add_field(toc_field)?;
        }

        // 506 - restrictions on access
        let mut restrictions_field: FieldRepr = FieldRepr::from((b"506", "\\\\"));
        restrictions_field =
            restrictions_field.add_subfield(b"a", "Open access resource providing free access.")?;
        builder.add_field(restrictions_field)?;

        // 520 - abstract
        if let Some(long_abstract) = self.long_abstract.clone() {
            let mut abstract_field: FieldRepr = FieldRepr::from((b"520", "\\\\"));
            abstract_field = abstract_field.add_subfield(b"a", long_abstract.into_bytes())?;
            builder.add_field(abstract_field)?;
        }

        // 538 - mode of access
        let mut access_field: FieldRepr = FieldRepr::from((b"538", "\\\\"));
        access_field = access_field.add_subfield(b"a", "Mode of access: World Wide Web.")?;
        builder.add_field(access_field)?;

        // 540 - license
        if let Some(license_url) = self.license.clone() {
            let mut license_field: FieldRepr = FieldRepr::from((b"540", "\\\\"));
            match License::from_url(&license_url) {
                Ok(license) => license_field =
                    license_field.add_subfield(b"a", format!("The text of this book is licensed under a {} For more detailed information consult the publisher's website.", license.to_string()).into_bytes())?,
                Err(_) => license_field =
                    license_field.add_subfield(b"a", "The text of this book is licensed under a custom license. For more detailed information consult the publisher's website.")?,
            }
            license_field = license_field.add_subfield(b"u", license_url.into_bytes())?;
            builder.add_field(license_field)?;
        }

        // 700 - contributors
        let mut contributions_by_name: HashMap<String, Vec<WorkContributions>> = HashMap::new();
        for contribution in &self.contributions {
            let name = contribution.full_name.clone();
            let contributions_for_name = contributions_by_name.entry(name).or_insert(Vec::new());
            contributions_for_name.push(contribution.clone());
        }
        for (name, contributions) in contributions_by_name.iter() {
            let roles = contributions
                .iter()
                .map(|c| ContributionType::from(c.contribution_type.clone()).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let mut contributor_field = FieldRepr::from((b"700", "1\\"));
            contributor_field = contributor_field.add_subfield(b"a", name)?;
            contributor_field = contributor_field.add_subfield(b"e", roles)?;
            builder.add_field(contributor_field)?;
        }

        // 710 - publisher
        let mut publisher_field: FieldRepr = FieldRepr::from((b"710", "2\\"));
        publisher_field = publisher_field.add_subfield(
            b"a",
            format!("{},", self.imprint.publisher.publisher_name.clone()).into_bytes(),
        )?;
        publisher_field = publisher_field.add_subfield(b"e", "publisher.")?;
        builder.add_field(publisher_field)?;

        // 856 - location
        if let Some(doi) = &self.doi {
            let mut cover_field: FieldRepr = FieldRepr::from((b"856", "40")); // version of resource
            cover_field = cover_field.add_subfield(b"u", doi.to_lowercase_string().into_bytes())?;
            cover_field = cover_field.add_subfield(b"z", b"Connect to e-book")?;
            builder.add_field(cover_field)?;
        }
        // 856 - cover
        if let Some(cover_url) = self.cover_url.clone() {
            let mut cover_field: FieldRepr = FieldRepr::from((b"856", "42")); // related resource
            cover_field = cover_field.add_subfield(b"u", cover_url.into_bytes())?;
            cover_field = cover_field.add_subfield(b"z", b"Connect to cover image")?;
            builder.add_field(cover_field)?;
        }

        Ok(builder.get_record()?)
    }
}

impl Marc21Field<Marc21RecordThoth> for WorkPublications {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()> {
        if let Some(isbn) = &self.isbn {
            let mut isbn_field: FieldRepr = FieldRepr::from((b"020", "\\\\")); // 2 backslashes represent that the subfield can appear multiple times within the field
            isbn_field = isbn_field.add_subfield(b"a", isbn.to_hyphenless_string().as_bytes())?;
            let publication_type: PublicationType = self.publication_type.clone().into();
            isbn_field =
                isbn_field.add_subfield(b"q", format!("({})", publication_type).into_bytes())?;

            builder.add_field(isbn_field)?;
        }
        Ok(())
    }
}

type SubjectField<'a, 'b> = (&'a [u8; 3], &'b str, Option<(&'a [u8; 1], &'b str)>);
impl Marc21Field<Marc21RecordThoth> for WorkSubjects {
    fn to_field(&self, builder: &mut RecordBuilder) -> ThothResult<()> {
        let (tag, ind, sub2): SubjectField = match self.subject_type {
            SubjectType::BIC => (b"072", " 7", Some((b"2", "bicssc"))),
            SubjectType::BISAC => (b"072", " 7", Some((b"2", "bisacsh"))),
            SubjectType::THEMA => (b"072", " 7", Some((b"2", "thema"))),
            SubjectType::LCC => (b"050", "00", None),
            _ => {
                return Ok(());
            }
        };
        let mut subject_field: FieldRepr = FieldRepr::from((tag, ind));
        subject_field = subject_field.add_subfield(b"a", self.subject_code.as_bytes())?;
        if let Some((subfield, value)) = sub2 {
            subject_field = subject_field.add_subfield(subfield, value)?;
        }
        builder.add_field(subject_field)?;
        Ok(())
    }
}

fn description_string(work: &Work) -> (String, Option<String>) {
    let description = match (work.page_breakdown.as_ref(), work.page_count) {
        (Some(breakdown), _) => format!("1 online resource ({} pages)", breakdown),
        (_, Some(count)) => format!("1 online resource ({} pages)", count),
        _ => "1 online resource".to_string(),
    };

    // other resource counts
    let counts = [
        (work.image_count, "illustration", "illustrations"),
        (work.table_count, "table", "tables"),
        (work.audio_count, "audio track", "audio tracks"),
        (work.video_count, "video", "videos"),
    ];
    let other_counts = counts
        .iter()
        .filter_map(|(count, singular, plural)| match count {
            Some(c) if *c > 0 => Some(format!("{} {}", c, if *c == 1 { singular } else { plural })),
            _ => None,
        })
        .collect::<Vec<_>>();

    match other_counts.is_empty() {
        true => (description + ".", None),
        false => (
            description + ": ",
            Some(format!("{}.", other_counts.join(", "))),
        ),
    }
}

fn contributors_string(contributions: &[WorkContributions]) -> String {
    // group main contributions by contribution type
    let mut contributions_by_type: std::collections::HashMap<
        ContributionType,
        Vec<&WorkContributions>,
    > = std::collections::HashMap::new();
    for c in contributions.iter().filter(|c| c.main_contribution) {
        let entry = contributions_by_type
            .entry(ContributionType::from(c.contribution_type.clone()))
            .or_insert(vec![]);
        entry.push(c);
    }

    // build string for each contribution type
    let mut type_strings = vec![];
    for (contribution_type, contributions) in contributions_by_type.iter() {
        let names = contributions
            .iter()
            .map(|c| c.full_name.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let type_string = match contribution_type {
            ContributionType::Author => names,
            ContributionType::Editor => format!("edited by {}", names),
            _ => format!("{} ({})", contribution_type, names),
        };
        type_strings.push(type_string);
    }

    // join type strings with appropriate separators
    let mut result = type_strings.join("; ");
    result.push('.');

    result
}
