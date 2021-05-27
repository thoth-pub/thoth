use strum::{EnumString, ToString};
use thoth_client::{ContributionType, LanguageRelation, SubjectType, WorkStatus};
use xml::writer::{XmlEvent, Result};
use std::io::Write;
use xml::EventWriter;

use crate::onix::write_element_block;

pub trait XmlElement {
    fn name(&self) -> &'static str;

    fn value(&self) -> &'static str;

    fn xml_element<W: Write>(&self, w: &mut EventWriter<W>) -> Result<()> {
        write_element_block(self.name(), None, None, w, |w| {
            w.write(XmlEvent::Characters(self.value())).ok();
        })
    }
}

pub trait Onix3: XmlElement {
    const ELEMENT: &'static str = "";

    fn onix_code(&self) -> &'static str;
}

impl XmlElement for WorkStatus {
    fn name(&self) -> &'static str {
        Self::ELEMENT
    }

    fn value(&self) -> &'static str {
        self.onix_code()
    }
}

impl Onix3 for WorkStatus {
    const ELEMENT: &'static str = "PublishingStatus";

    fn onix_code(&self) -> &'static str {
        match self {
            WorkStatus::UNSPECIFIED => "00",
            WorkStatus::CANCELLED => "01",
            WorkStatus::FORTHCOMING => "02",
            WorkStatus::POSTPONED_INDEFINITELY => "03",
            WorkStatus::ACTIVE => "04",
            WorkStatus::NO_LONGER_OUR_PRODUCT => "05",
            WorkStatus::OUT_OF_STOCK_INDEFINITELY => "06",
            WorkStatus::OUT_OF_PRINT => "07",
            WorkStatus::INACTIVE => "08",
            WorkStatus::UNKNOWN => "09",
            WorkStatus::REMAINDERED => "10",
            WorkStatus::WITHDRAWN_FROM_SALE => "11",
            WorkStatus::RECALLED => "15",
            _ => unreachable!(),
        }
    }
}

#[derive(EnumString, ToString)]
pub(crate) enum OnixSubjectScheme {
    #[strum(serialize = "12")]
    Bic,
    #[strum(serialize = "10")]
    Bisac,
    #[strum(serialize = "20")]
    Keyword,
    #[strum(serialize = "04")]
    Lcc,
    #[strum(serialize = "93")]
    Thema,
    #[strum(serialize = "B2")]
    KeywordNotForDisplay, // B2 Keywords (not for display)
}

impl From<SubjectType> for OnixSubjectScheme {
    fn from(input: SubjectType) -> Self {
        match input {
            SubjectType::BIC => OnixSubjectScheme::Bic,
            SubjectType::BISAC => OnixSubjectScheme::Bisac,
            SubjectType::KEYWORD => OnixSubjectScheme::Keyword,
            SubjectType::LCC => OnixSubjectScheme::Lcc,
            SubjectType::THEMA => OnixSubjectScheme::Thema,
            SubjectType::CUSTOM => OnixSubjectScheme::KeywordNotForDisplay,
            SubjectType::Other(_) => unreachable!(),
        }
    }
}

#[derive(EnumString, ToString)]
pub(crate) enum OnixLanguageRole {
    #[strum(serialize = "01")]
    LanguageOfText,
    #[strum(serialize = "02")]
    OriginalLanguageOfTranslatedText,
}

impl From<LanguageRelation> for OnixLanguageRole {
    fn from(input: LanguageRelation) -> Self {
        match input {
            LanguageRelation::ORIGINAL => OnixLanguageRole::LanguageOfText,
            LanguageRelation::TRANSLATED_FROM => OnixLanguageRole::OriginalLanguageOfTranslatedText,
            LanguageRelation::TRANSLATED_INTO => OnixLanguageRole::LanguageOfText,
            LanguageRelation::Other(_) => unreachable!(),
        }
    }
}

#[derive(EnumString, ToString)]
pub(crate) enum OnixContributorRole {
    #[strum(serialize = "A01")]
    ByAuthor,
    #[strum(serialize = "B01")]
    EditedBy,
    #[strum(serialize = "B06")]
    TranslatedBy,
    #[strum(serialize = "A13")]
    PhotographsBy,
    #[strum(serialize = "A12")]
    IllustratedBy,
    #[strum(serialize = "B25")]
    MusicArrangedBy,
    #[strum(serialize = "A23")]
    ForewordByBy,
    #[strum(serialize = "A24")]
    IntroductionBy,
    #[strum(serialize = "A19")]
    AfterwordBy,
    #[strum(serialize = "A15")]
    PrefaceBy,
}

impl From<ContributionType> for OnixContributorRole {
    fn from(input: ContributionType) -> Self {
        match input {
            ContributionType::AUTHOR => OnixContributorRole::ByAuthor,
            ContributionType::EDITOR => OnixContributorRole::EditedBy,
            ContributionType::TRANSLATOR => OnixContributorRole::TranslatedBy,
            ContributionType::PHOTOGRAPHER => OnixContributorRole::PhotographsBy,
            ContributionType::ILUSTRATOR => OnixContributorRole::IllustratedBy,
            ContributionType::MUSIC_EDITOR => OnixContributorRole::MusicArrangedBy,
            ContributionType::FOREWORD_BY => OnixContributorRole::ForewordByBy,
            ContributionType::INTRODUCTION_BY => OnixContributorRole::IntroductionBy,
            ContributionType::AFTERWORD_BY => OnixContributorRole::AfterwordBy,
            ContributionType::PREFACE_BY => OnixContributorRole::PrefaceBy,
            ContributionType::Other(_) => unreachable!(),
        }
    }
}
