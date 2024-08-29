use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Doi;
use crate::model::Ror;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::institution;
#[cfg(feature = "backend")]
use crate::schema::institution_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting institutions list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionField {
    #[strum(serialize = "ID")]
    InstitutionId,
    #[strum(serialize = "Institution")]
    #[default]
    InstitutionName,
    #[strum(serialize = "DOI")]
    InstitutionDoi,
    #[strum(serialize = "ROR ID")]
    Ror,
    #[strum(serialize = "Country")]
    CountryCode,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Institution {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new organisation with which contributors may be affiliated or by which works may be funded"),
    diesel(table_name = institution)
)]
pub struct NewInstitution {
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing organisation with which contributors may be affiliated or by which works may be funded"),
    diesel(table_name = institution, treat_none_as_null = true)
)]
pub struct PatchInstitution {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Three-letter ISO 3166-1 code representing a country"),
    ExistingTypePath = "crate::schema::sql_types::CountryCode"
)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CountryCode {
    #[strum(serialize = "Afghanistan")]
    Afg,
    #[strum(serialize = "Åland Islands")]
    Ala,
    #[strum(serialize = "Albania")]
    Alb,
    #[strum(serialize = "Algeria")]
    Dza,
    #[strum(serialize = "American Samoa")]
    Asm,
    #[strum(serialize = "Andorra")]
    And,
    #[strum(serialize = "Angola")]
    Ago,
    #[strum(serialize = "Anguilla")]
    Aia,
    #[strum(serialize = "Antarctica")]
    Ata,
    #[strum(serialize = "Antigua and Barbuda")]
    Atg,
    #[strum(serialize = "Argentina")]
    Arg,
    #[strum(serialize = "Armenia")]
    Arm,
    #[strum(serialize = "Aruba")]
    Abw,
    #[strum(serialize = "Australia")]
    Aus,
    #[strum(serialize = "Austria")]
    Aut,
    #[strum(serialize = "Azerbaijan")]
    Aze,
    #[strum(serialize = "Bahamas")]
    Bhs,
    #[strum(serialize = "Bahrain")]
    Bhr,
    #[strum(serialize = "Bangladesh")]
    Bgd,
    #[strum(serialize = "Barbados")]
    Brb,
    #[strum(serialize = "Belarus")]
    Blr,
    #[strum(serialize = "Belgium")]
    Bel,
    #[strum(serialize = "Belize")]
    Blz,
    #[strum(serialize = "Benin")]
    Ben,
    #[strum(serialize = "Bermuda")]
    Bmu,
    #[strum(serialize = "Bhutan")]
    Btn,
    #[strum(serialize = "Bolivia")]
    Bol,
    #[strum(serialize = "Bonaire, Sint Eustatius and Saba")]
    Bes,
    #[strum(serialize = "Bosnia and Herzegovina")]
    Bih,
    #[strum(serialize = "Botswana")]
    Bwa,
    #[strum(serialize = "Bouvet Island")]
    Bvt,
    #[strum(serialize = "Brazil")]
    Bra,
    #[strum(serialize = "British Indian Ocean Territory")]
    Iot,
    #[strum(serialize = "Brunei")]
    Brn,
    #[strum(serialize = "Bulgaria")]
    Bgr,
    #[strum(serialize = "Burkina Faso")]
    Bfa,
    #[strum(serialize = "Burundi")]
    Bdi,
    #[strum(serialize = "Cabo Verde")]
    Cpv,
    #[strum(serialize = "Cambodia")]
    Khm,
    #[strum(serialize = "Cameroon")]
    Cmr,
    #[strum(serialize = "Canada")]
    Can,
    #[strum(serialize = "Cayman Islands")]
    Cym,
    #[strum(serialize = "Central African Republic")]
    Caf,
    #[strum(serialize = "Chad")]
    Tcd,
    #[strum(serialize = "Chile")]
    Chl,
    #[strum(serialize = "China")]
    Chn,
    #[strum(serialize = "Christmas Island")]
    Cxr,
    #[strum(serialize = "Cocos (Keeling) Islands")]
    Cck,
    #[strum(serialize = "Colombia")]
    Col,
    #[strum(serialize = "Comoros")]
    Com,
    #[strum(serialize = "Cook Islands")]
    Cok,
    #[strum(serialize = "Costa Rica")]
    Cri,
    #[strum(serialize = "Côte d'Ivoire")]
    Civ,
    #[strum(serialize = "Croatia")]
    Hrv,
    #[strum(serialize = "Cuba")]
    Cub,
    #[strum(serialize = "Curaçao")]
    Cuw,
    #[strum(serialize = "Cyprus")]
    Cyp,
    #[strum(serialize = "Czechia")]
    Cze,
    #[strum(serialize = "Democratic Republic of the Congo")]
    Cod,
    #[strum(serialize = "Denmark")]
    Dnk,
    #[strum(serialize = "Djibouti")]
    Dji,
    #[strum(serialize = "Dominica")]
    Dma,
    #[strum(serialize = "Dominican Republic")]
    Dom,
    #[strum(serialize = "Ecuador")]
    Ecu,
    #[strum(serialize = "Egypt")]
    Egy,
    #[strum(serialize = "El Salvador")]
    Slv,
    #[strum(serialize = "Equatorial Guinea")]
    Gnq,
    #[strum(serialize = "Eritrea")]
    Eri,
    #[strum(serialize = "Estonia")]
    Est,
    #[strum(serialize = "Eswatini")]
    Swz,
    #[strum(serialize = "Ethiopia")]
    Eth,
    #[strum(serialize = "Falkland Islands")]
    Flk,
    #[strum(serialize = "Faroe Islands")]
    Fro,
    #[strum(serialize = "Fiji")]
    Fji,
    #[strum(serialize = "Finland")]
    Fin,
    #[strum(serialize = "France")]
    Fra,
    #[strum(serialize = "French Guiana")]
    Guf,
    #[strum(serialize = "French Polynesia")]
    Pyf,
    #[strum(serialize = "French Southern Territories")]
    Atf,
    #[strum(serialize = "Gabon")]
    Gab,
    #[strum(serialize = "Gambia")]
    Gmb,
    #[strum(serialize = "Georgia")]
    Geo,
    #[strum(serialize = "Germany")]
    Deu,
    #[strum(serialize = "Ghana")]
    Gha,
    #[strum(serialize = "Gibraltar")]
    Gib,
    #[strum(serialize = "Greece")]
    Grc,
    #[strum(serialize = "Greenland")]
    Grl,
    #[strum(serialize = "Grenada")]
    Grd,
    #[strum(serialize = "Guadeloupe")]
    Glp,
    #[strum(serialize = "Guam")]
    Gum,
    #[strum(serialize = "Guatemala")]
    Gtm,
    #[strum(serialize = "Guernsey")]
    Ggy,
    #[strum(serialize = "Guinea")]
    Gin,
    #[strum(serialize = "Guinea-Bissau")]
    Gnb,
    #[strum(serialize = "Guyana")]
    Guy,
    #[strum(serialize = "Haiti")]
    Hti,
    #[strum(serialize = "Heard Island and McDonald Islands")]
    Hmd,
    #[strum(serialize = "Honduras")]
    Hnd,
    #[strum(serialize = "Hong Kong")]
    Hkg,
    #[strum(serialize = "Hungary")]
    Hun,
    #[strum(serialize = "Iceland")]
    Isl,
    #[strum(serialize = "India")]
    Ind,
    #[strum(serialize = "Indonesia")]
    Idn,
    #[strum(serialize = "Iran")]
    Irn,
    #[strum(serialize = "Iraq")]
    Irq,
    #[strum(serialize = "Ireland")]
    Irl,
    #[strum(serialize = "Isle of Man")]
    Imn,
    #[strum(serialize = "Israel")]
    Isr,
    #[strum(serialize = "Italy")]
    Ita,
    #[strum(serialize = "Jamaica")]
    Jam,
    #[strum(serialize = "Japan")]
    Jpn,
    #[strum(serialize = "Jersey")]
    Jey,
    #[strum(serialize = "Jordan")]
    Jor,
    #[strum(serialize = "Kazakhstan")]
    Kaz,
    #[strum(serialize = "Kenya")]
    Ken,
    #[strum(serialize = "Kiribati")]
    Kir,
    #[strum(serialize = "Kuwait")]
    Kwt,
    #[strum(serialize = "Kyrgyzstan")]
    Kgz,
    #[strum(serialize = "Laos")]
    Lao,
    #[strum(serialize = "Latvia")]
    Lva,
    #[strum(serialize = "Lebanon")]
    Lbn,
    #[strum(serialize = "Lesotho")]
    Lso,
    #[strum(serialize = "Liberia")]
    Lbr,
    #[strum(serialize = "Libya")]
    Lby,
    #[strum(serialize = "Liechtenstein")]
    Lie,
    #[strum(serialize = "Lithuania")]
    Ltu,
    #[strum(serialize = "Luxembourg")]
    Lux,
    #[strum(serialize = "Macao")]
    Mac,
    #[strum(serialize = "Madagascar")]
    Mdg,
    #[strum(serialize = "Malawi")]
    Mwi,
    #[strum(serialize = "Malaysia")]
    Mys,
    #[strum(serialize = "Maldives")]
    Mdv,
    #[strum(serialize = "Mali")]
    Mli,
    #[strum(serialize = "Malta")]
    Mlt,
    #[strum(serialize = "Marshall Islands")]
    Mhl,
    #[strum(serialize = "Martinique")]
    Mtq,
    #[strum(serialize = "Mauritania")]
    Mrt,
    #[strum(serialize = "Mauritius")]
    Mus,
    #[strum(serialize = "Mayotte")]
    Myt,
    #[strum(serialize = "Mexico")]
    Mex,
    #[strum(serialize = "Micronesia")]
    Fsm,
    #[strum(serialize = "Moldova")]
    Mda,
    #[strum(serialize = "Monaco")]
    Mco,
    #[strum(serialize = "Mongolia")]
    Mng,
    #[strum(serialize = "Montenegro")]
    Mne,
    #[strum(serialize = "Montserrat")]
    Msr,
    #[strum(serialize = "Morocco")]
    Mar,
    #[strum(serialize = "Mozambique")]
    Moz,
    #[strum(serialize = "Myanmar")]
    Mmr,
    #[strum(serialize = "Namibia")]
    Nam,
    #[strum(serialize = "Nauru")]
    Nru,
    #[strum(serialize = "Nepal")]
    Npl,
    #[strum(serialize = "Netherlands")]
    Nld,
    #[strum(serialize = "New Caledonia")]
    Ncl,
    #[strum(serialize = "New Zealand")]
    Nzl,
    #[strum(serialize = "Nicaragua")]
    Nic,
    #[strum(serialize = "Niger")]
    Ner,
    #[strum(serialize = "Nigeria")]
    Nga,
    #[strum(serialize = "Niue")]
    Niu,
    #[strum(serialize = "Norfolk Island")]
    Nfk,
    #[strum(serialize = "North Korea")]
    Prk,
    #[strum(serialize = "North Macedonia")]
    Mkd,
    #[strum(serialize = "Northern Mariana Islands")]
    Mnp,
    #[strum(serialize = "Norway")]
    Nor,
    #[strum(serialize = "Oman")]
    Omn,
    #[strum(serialize = "Pakistan")]
    Pak,
    #[strum(serialize = "Palau")]
    Plw,
    #[strum(serialize = "Palestine")]
    Pse,
    #[strum(serialize = "Panama")]
    Pan,
    #[strum(serialize = "Papua New Guinea")]
    Png,
    #[strum(serialize = "Paraguay")]
    Pry,
    #[strum(serialize = "Peru")]
    Per,
    #[strum(serialize = "Philippines")]
    Phl,
    #[strum(serialize = "Pitcairn")]
    Pcn,
    #[strum(serialize = "Poland")]
    Pol,
    #[strum(serialize = "Portugal")]
    Prt,
    #[strum(serialize = "Puerto Rico")]
    Pri,
    #[strum(serialize = "Qatar")]
    Qat,
    #[strum(serialize = "Republic of the Congo")]
    Cog,
    #[strum(serialize = "Réunion")]
    Reu,
    #[strum(serialize = "Romania")]
    Rou,
    #[strum(serialize = "Russia")]
    Rus,
    #[strum(serialize = "Rwanda")]
    Rwa,
    #[strum(serialize = "Saint Barthélemy")]
    Blm,
    #[strum(serialize = "Saint Helena, Ascension and Tristan da Cunha")]
    Shn,
    #[strum(serialize = "Saint Kitts and Nevis")]
    Kna,
    #[strum(serialize = "Saint Lucia")]
    Lca,
    #[strum(serialize = "Saint Martin")]
    Maf,
    #[strum(serialize = "Saint Pierre and Miquelon")]
    Spm,
    #[strum(serialize = "Saint Vincent and the Grenadines")]
    Vct,
    #[strum(serialize = "Samoa")]
    Wsm,
    #[strum(serialize = "San Marino")]
    Smr,
    #[strum(serialize = "Sao Tome and Principe")]
    Stp,
    #[strum(serialize = "Saudi Arabia")]
    Sau,
    #[strum(serialize = "Senegal")]
    Sen,
    #[strum(serialize = "Serbia")]
    Srb,
    #[strum(serialize = "Seychelles")]
    Syc,
    #[strum(serialize = "Sierra Leone")]
    Sle,
    #[strum(serialize = "Singapore")]
    Sgp,
    #[strum(serialize = "Sint Maarten")]
    Sxm,
    #[strum(serialize = "Slovakia")]
    Svk,
    #[strum(serialize = "Slovenia")]
    Svn,
    #[strum(serialize = "Solomon Islands")]
    Slb,
    #[strum(serialize = "Somalia")]
    Som,
    #[strum(serialize = "South Africa")]
    Zaf,
    #[strum(serialize = "South Georgia and the South Sandwich Islands")]
    Sgs,
    #[strum(serialize = "South Korea")]
    Kor,
    #[strum(serialize = "South Sudan")]
    Ssd,
    #[strum(serialize = "Spain")]
    Esp,
    #[strum(serialize = "Sri Lanka")]
    Lka,
    #[strum(serialize = "Sudan")]
    Sdn,
    #[strum(serialize = "Suriname")]
    Sur,
    #[strum(serialize = "Svalbard and Jan Mayen")]
    Sjm,
    #[strum(serialize = "Sweden")]
    Swe,
    #[strum(serialize = "Switzerland")]
    Che,
    #[strum(serialize = "Syria")]
    Syr,
    #[strum(serialize = "Taiwan")]
    Twn,
    #[strum(serialize = "Tajikistan")]
    Tjk,
    #[strum(serialize = "Tanzania")]
    Tza,
    #[strum(serialize = "Thailand")]
    Tha,
    #[strum(serialize = "Timor-Leste")]
    Tls,
    #[strum(serialize = "Togo")]
    Tgo,
    #[strum(serialize = "Tokelau")]
    Tkl,
    #[strum(serialize = "Tonga")]
    Ton,
    #[strum(serialize = "Trinidad and Tobago")]
    Tto,
    #[strum(serialize = "Tunisia")]
    Tun,
    #[strum(serialize = "Turkey")]
    Tur,
    #[strum(serialize = "Turkmenistan")]
    Tkm,
    #[strum(serialize = "Turks and Caicos Islands")]
    Tca,
    #[strum(serialize = "Tuvalu")]
    Tuv,
    #[strum(serialize = "Uganda")]
    Uga,
    #[strum(serialize = "Ukraine")]
    Ukr,
    #[strum(serialize = "United Arab Emirates")]
    Are,
    #[strum(serialize = "United Kingdom")]
    Gbr,
    #[strum(serialize = "United States Minor Outlying Islands")]
    Umi,
    #[strum(serialize = "United States of America")]
    Usa,
    #[strum(serialize = "Uruguay")]
    Ury,
    #[strum(serialize = "Uzbekistan")]
    Uzb,
    #[strum(serialize = "Vanuatu")]
    Vut,
    #[strum(serialize = "Vatican City")]
    Vat,
    #[strum(serialize = "Venezuela")]
    Ven,
    #[strum(serialize = "Viet Nam")]
    Vnm,
    #[strum(serialize = "Virgin Islands (British)")]
    Vgb,
    #[strum(serialize = "Virgin Islands (U.S.)")]
    Vir,
    #[strum(serialize = "Wallis and Futuna")]
    Wlf,
    #[strum(serialize = "Western Sahara")]
    Esh,
    #[strum(serialize = "Yemen")]
    Yem,
    #[strum(serialize = "Zambia")]
    Zmb,
    #[strum(serialize = "Zimbabwe")]
    Zwe,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct InstitutionHistory {
    pub institution_history_id: Uuid,
    pub institution_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = institution_history)
)]
pub struct NewInstitutionHistory {
    pub institution_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting institutions list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionOrderBy {
    pub field: InstitutionField,
    pub direction: Direction,
}

impl fmt::Display for Institution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ror) = &self.ror {
            write!(f, "{} - {}", &self.institution_name, ror)
        } else if let Some(doi) = &self.institution_doi {
            write!(f, "{} - {}", &self.institution_name, doi)
        } else {
            write!(f, "{}", &self.institution_name)
        }
    }
}

#[test]
fn test_institutionfield_default() {
    let fundfield: InstitutionField = Default::default();
    assert_eq!(fundfield, InstitutionField::InstitutionName);
}

#[test]
fn test_institutionfield_display() {
    assert_eq!(format!("{}", InstitutionField::InstitutionId), "ID");
    assert_eq!(
        format!("{}", InstitutionField::InstitutionName),
        "Institution"
    );
    assert_eq!(format!("{}", InstitutionField::InstitutionDoi), "DOI");
    assert_eq!(format!("{}", InstitutionField::Ror), "ROR ID");
    assert_eq!(format!("{}", InstitutionField::CountryCode), "Country");
    assert_eq!(format!("{}", InstitutionField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", InstitutionField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_institutionfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        InstitutionField::from_str("ID").unwrap(),
        InstitutionField::InstitutionId
    );
    assert_eq!(
        InstitutionField::from_str("Institution").unwrap(),
        InstitutionField::InstitutionName
    );
    assert_eq!(
        InstitutionField::from_str("DOI").unwrap(),
        InstitutionField::InstitutionDoi
    );
    assert_eq!(
        InstitutionField::from_str("ROR ID").unwrap(),
        InstitutionField::Ror
    );
    assert_eq!(
        InstitutionField::from_str("Country").unwrap(),
        InstitutionField::CountryCode
    );
    assert_eq!(
        InstitutionField::from_str("CreatedAt").unwrap(),
        InstitutionField::CreatedAt
    );
    assert_eq!(
        InstitutionField::from_str("UpdatedAt").unwrap(),
        InstitutionField::UpdatedAt
    );
    assert!(InstitutionField::from_str("InstitutionID").is_err());
    assert!(InstitutionField::from_str("Website").is_err());
    assert!(InstitutionField::from_str("Fundings").is_err());
}

#[test]
fn test_countrycode_display() {
    assert_eq!(format!("{}", CountryCode::Afg), "Afghanistan");
    assert_eq!(format!("{}", CountryCode::Ala), "Åland Islands");
    assert_eq!(format!("{}", CountryCode::Alb), "Albania");
    assert_eq!(format!("{}", CountryCode::Dza), "Algeria");
    assert_eq!(format!("{}", CountryCode::Asm), "American Samoa");
    assert_eq!(format!("{}", CountryCode::And), "Andorra");
    assert_eq!(format!("{}", CountryCode::Ago), "Angola");
    assert_eq!(format!("{}", CountryCode::Aia), "Anguilla");
    assert_eq!(format!("{}", CountryCode::Ata), "Antarctica");
    assert_eq!(format!("{}", CountryCode::Atg), "Antigua and Barbuda");
    assert_eq!(format!("{}", CountryCode::Arg), "Argentina");
    assert_eq!(format!("{}", CountryCode::Arm), "Armenia");
    assert_eq!(format!("{}", CountryCode::Abw), "Aruba");
    assert_eq!(format!("{}", CountryCode::Aus), "Australia");
    assert_eq!(format!("{}", CountryCode::Aut), "Austria");
    assert_eq!(format!("{}", CountryCode::Aze), "Azerbaijan");
    assert_eq!(format!("{}", CountryCode::Bhs), "Bahamas");
    assert_eq!(format!("{}", CountryCode::Bhr), "Bahrain");
    assert_eq!(format!("{}", CountryCode::Bgd), "Bangladesh");
    assert_eq!(format!("{}", CountryCode::Brb), "Barbados");
    assert_eq!(format!("{}", CountryCode::Blr), "Belarus");
    assert_eq!(format!("{}", CountryCode::Bel), "Belgium");
    assert_eq!(format!("{}", CountryCode::Blz), "Belize");
    assert_eq!(format!("{}", CountryCode::Ben), "Benin");
    assert_eq!(format!("{}", CountryCode::Bmu), "Bermuda");
    assert_eq!(format!("{}", CountryCode::Btn), "Bhutan");
    assert_eq!(format!("{}", CountryCode::Bol), "Bolivia");
    assert_eq!(
        format!("{}", CountryCode::Bes),
        "Bonaire, Sint Eustatius and Saba"
    );
    assert_eq!(format!("{}", CountryCode::Bih), "Bosnia and Herzegovina");
    assert_eq!(format!("{}", CountryCode::Bwa), "Botswana");
    assert_eq!(format!("{}", CountryCode::Bvt), "Bouvet Island");
    assert_eq!(format!("{}", CountryCode::Bra), "Brazil");
    assert_eq!(
        format!("{}", CountryCode::Iot),
        "British Indian Ocean Territory"
    );
    assert_eq!(format!("{}", CountryCode::Brn), "Brunei");
    assert_eq!(format!("{}", CountryCode::Bgr), "Bulgaria");
    assert_eq!(format!("{}", CountryCode::Bfa), "Burkina Faso");
    assert_eq!(format!("{}", CountryCode::Bdi), "Burundi");
    assert_eq!(format!("{}", CountryCode::Cpv), "Cabo Verde");
    assert_eq!(format!("{}", CountryCode::Khm), "Cambodia");
    assert_eq!(format!("{}", CountryCode::Cmr), "Cameroon");
    assert_eq!(format!("{}", CountryCode::Can), "Canada");
    assert_eq!(format!("{}", CountryCode::Cym), "Cayman Islands");
    assert_eq!(format!("{}", CountryCode::Caf), "Central African Republic");
    assert_eq!(format!("{}", CountryCode::Tcd), "Chad");
    assert_eq!(format!("{}", CountryCode::Chl), "Chile");
    assert_eq!(format!("{}", CountryCode::Chn), "China");
    assert_eq!(format!("{}", CountryCode::Cxr), "Christmas Island");
    assert_eq!(format!("{}", CountryCode::Cck), "Cocos (Keeling) Islands");
    assert_eq!(format!("{}", CountryCode::Col), "Colombia");
    assert_eq!(format!("{}", CountryCode::Com), "Comoros");
    assert_eq!(format!("{}", CountryCode::Cok), "Cook Islands");
    assert_eq!(format!("{}", CountryCode::Cri), "Costa Rica");
    assert_eq!(format!("{}", CountryCode::Civ), "Côte d'Ivoire");
    assert_eq!(format!("{}", CountryCode::Hrv), "Croatia");
    assert_eq!(format!("{}", CountryCode::Cub), "Cuba");
    assert_eq!(format!("{}", CountryCode::Cuw), "Curaçao");
    assert_eq!(format!("{}", CountryCode::Cyp), "Cyprus");
    assert_eq!(format!("{}", CountryCode::Cze), "Czechia");
    assert_eq!(
        format!("{}", CountryCode::Cod),
        "Democratic Republic of the Congo"
    );
    assert_eq!(format!("{}", CountryCode::Dnk), "Denmark");
    assert_eq!(format!("{}", CountryCode::Dji), "Djibouti");
    assert_eq!(format!("{}", CountryCode::Dma), "Dominica");
    assert_eq!(format!("{}", CountryCode::Dom), "Dominican Republic");
    assert_eq!(format!("{}", CountryCode::Ecu), "Ecuador");
    assert_eq!(format!("{}", CountryCode::Egy), "Egypt");
    assert_eq!(format!("{}", CountryCode::Slv), "El Salvador");
    assert_eq!(format!("{}", CountryCode::Gnq), "Equatorial Guinea");
    assert_eq!(format!("{}", CountryCode::Eri), "Eritrea");
    assert_eq!(format!("{}", CountryCode::Est), "Estonia");
    assert_eq!(format!("{}", CountryCode::Swz), "Eswatini");
    assert_eq!(format!("{}", CountryCode::Eth), "Ethiopia");
    assert_eq!(format!("{}", CountryCode::Flk), "Falkland Islands");
    assert_eq!(format!("{}", CountryCode::Fro), "Faroe Islands");
    assert_eq!(format!("{}", CountryCode::Fji), "Fiji");
    assert_eq!(format!("{}", CountryCode::Fin), "Finland");
    assert_eq!(format!("{}", CountryCode::Fra), "France");
    assert_eq!(format!("{}", CountryCode::Guf), "French Guiana");
    assert_eq!(format!("{}", CountryCode::Pyf), "French Polynesia");
    assert_eq!(
        format!("{}", CountryCode::Atf),
        "French Southern Territories"
    );
    assert_eq!(format!("{}", CountryCode::Gab), "Gabon");
    assert_eq!(format!("{}", CountryCode::Gmb), "Gambia");
    assert_eq!(format!("{}", CountryCode::Geo), "Georgia");
    assert_eq!(format!("{}", CountryCode::Deu), "Germany");
    assert_eq!(format!("{}", CountryCode::Gha), "Ghana");
    assert_eq!(format!("{}", CountryCode::Gib), "Gibraltar");
    assert_eq!(format!("{}", CountryCode::Grc), "Greece");
    assert_eq!(format!("{}", CountryCode::Grl), "Greenland");
    assert_eq!(format!("{}", CountryCode::Grd), "Grenada");
    assert_eq!(format!("{}", CountryCode::Glp), "Guadeloupe");
    assert_eq!(format!("{}", CountryCode::Gum), "Guam");
    assert_eq!(format!("{}", CountryCode::Gtm), "Guatemala");
    assert_eq!(format!("{}", CountryCode::Ggy), "Guernsey");
    assert_eq!(format!("{}", CountryCode::Gin), "Guinea");
    assert_eq!(format!("{}", CountryCode::Gnb), "Guinea-Bissau");
    assert_eq!(format!("{}", CountryCode::Guy), "Guyana");
    assert_eq!(format!("{}", CountryCode::Hti), "Haiti");
    assert_eq!(
        format!("{}", CountryCode::Hmd),
        "Heard Island and McDonald Islands"
    );
    assert_eq!(format!("{}", CountryCode::Hnd), "Honduras");
    assert_eq!(format!("{}", CountryCode::Hkg), "Hong Kong");
    assert_eq!(format!("{}", CountryCode::Hun), "Hungary");
    assert_eq!(format!("{}", CountryCode::Isl), "Iceland");
    assert_eq!(format!("{}", CountryCode::Ind), "India");
    assert_eq!(format!("{}", CountryCode::Idn), "Indonesia");
    assert_eq!(format!("{}", CountryCode::Irn), "Iran");
    assert_eq!(format!("{}", CountryCode::Irq), "Iraq");
    assert_eq!(format!("{}", CountryCode::Irl), "Ireland");
    assert_eq!(format!("{}", CountryCode::Imn), "Isle of Man");
    assert_eq!(format!("{}", CountryCode::Isr), "Israel");
    assert_eq!(format!("{}", CountryCode::Ita), "Italy");
    assert_eq!(format!("{}", CountryCode::Jam), "Jamaica");
    assert_eq!(format!("{}", CountryCode::Jpn), "Japan");
    assert_eq!(format!("{}", CountryCode::Jey), "Jersey");
    assert_eq!(format!("{}", CountryCode::Jor), "Jordan");
    assert_eq!(format!("{}", CountryCode::Kaz), "Kazakhstan");
    assert_eq!(format!("{}", CountryCode::Ken), "Kenya");
    assert_eq!(format!("{}", CountryCode::Kir), "Kiribati");
    assert_eq!(format!("{}", CountryCode::Kwt), "Kuwait");
    assert_eq!(format!("{}", CountryCode::Kgz), "Kyrgyzstan");
    assert_eq!(format!("{}", CountryCode::Lao), "Laos");
    assert_eq!(format!("{}", CountryCode::Lva), "Latvia");
    assert_eq!(format!("{}", CountryCode::Lbn), "Lebanon");
    assert_eq!(format!("{}", CountryCode::Lso), "Lesotho");
    assert_eq!(format!("{}", CountryCode::Lbr), "Liberia");
    assert_eq!(format!("{}", CountryCode::Lby), "Libya");
    assert_eq!(format!("{}", CountryCode::Lie), "Liechtenstein");
    assert_eq!(format!("{}", CountryCode::Ltu), "Lithuania");
    assert_eq!(format!("{}", CountryCode::Lux), "Luxembourg");
    assert_eq!(format!("{}", CountryCode::Mac), "Macao");
    assert_eq!(format!("{}", CountryCode::Mdg), "Madagascar");
    assert_eq!(format!("{}", CountryCode::Mwi), "Malawi");
    assert_eq!(format!("{}", CountryCode::Mys), "Malaysia");
    assert_eq!(format!("{}", CountryCode::Mdv), "Maldives");
    assert_eq!(format!("{}", CountryCode::Mli), "Mali");
    assert_eq!(format!("{}", CountryCode::Mlt), "Malta");
    assert_eq!(format!("{}", CountryCode::Mhl), "Marshall Islands");
    assert_eq!(format!("{}", CountryCode::Mtq), "Martinique");
    assert_eq!(format!("{}", CountryCode::Mrt), "Mauritania");
    assert_eq!(format!("{}", CountryCode::Mus), "Mauritius");
    assert_eq!(format!("{}", CountryCode::Myt), "Mayotte");
    assert_eq!(format!("{}", CountryCode::Mex), "Mexico");
    assert_eq!(format!("{}", CountryCode::Fsm), "Micronesia");
    assert_eq!(format!("{}", CountryCode::Mda), "Moldova");
    assert_eq!(format!("{}", CountryCode::Mco), "Monaco");
    assert_eq!(format!("{}", CountryCode::Mng), "Mongolia");
    assert_eq!(format!("{}", CountryCode::Mne), "Montenegro");
    assert_eq!(format!("{}", CountryCode::Msr), "Montserrat");
    assert_eq!(format!("{}", CountryCode::Mar), "Morocco");
    assert_eq!(format!("{}", CountryCode::Moz), "Mozambique");
    assert_eq!(format!("{}", CountryCode::Mmr), "Myanmar");
    assert_eq!(format!("{}", CountryCode::Nam), "Namibia");
    assert_eq!(format!("{}", CountryCode::Nru), "Nauru");
    assert_eq!(format!("{}", CountryCode::Npl), "Nepal");
    assert_eq!(format!("{}", CountryCode::Nld), "Netherlands");
    assert_eq!(format!("{}", CountryCode::Ncl), "New Caledonia");
    assert_eq!(format!("{}", CountryCode::Nzl), "New Zealand");
    assert_eq!(format!("{}", CountryCode::Nic), "Nicaragua");
    assert_eq!(format!("{}", CountryCode::Ner), "Niger");
    assert_eq!(format!("{}", CountryCode::Nga), "Nigeria");
    assert_eq!(format!("{}", CountryCode::Niu), "Niue");
    assert_eq!(format!("{}", CountryCode::Nfk), "Norfolk Island");
    assert_eq!(format!("{}", CountryCode::Prk), "North Korea");
    assert_eq!(format!("{}", CountryCode::Mkd), "North Macedonia");
    assert_eq!(format!("{}", CountryCode::Mnp), "Northern Mariana Islands");
    assert_eq!(format!("{}", CountryCode::Nor), "Norway");
    assert_eq!(format!("{}", CountryCode::Omn), "Oman");
    assert_eq!(format!("{}", CountryCode::Pak), "Pakistan");
    assert_eq!(format!("{}", CountryCode::Plw), "Palau");
    assert_eq!(format!("{}", CountryCode::Pse), "Palestine");
    assert_eq!(format!("{}", CountryCode::Pan), "Panama");
    assert_eq!(format!("{}", CountryCode::Png), "Papua New Guinea");
    assert_eq!(format!("{}", CountryCode::Pry), "Paraguay");
    assert_eq!(format!("{}", CountryCode::Per), "Peru");
    assert_eq!(format!("{}", CountryCode::Phl), "Philippines");
    assert_eq!(format!("{}", CountryCode::Pcn), "Pitcairn");
    assert_eq!(format!("{}", CountryCode::Pol), "Poland");
    assert_eq!(format!("{}", CountryCode::Prt), "Portugal");
    assert_eq!(format!("{}", CountryCode::Pri), "Puerto Rico");
    assert_eq!(format!("{}", CountryCode::Qat), "Qatar");
    assert_eq!(format!("{}", CountryCode::Cog), "Republic of the Congo");
    assert_eq!(format!("{}", CountryCode::Reu), "Réunion");
    assert_eq!(format!("{}", CountryCode::Rou), "Romania");
    assert_eq!(format!("{}", CountryCode::Rus), "Russia");
    assert_eq!(format!("{}", CountryCode::Rwa), "Rwanda");
    assert_eq!(format!("{}", CountryCode::Blm), "Saint Barthélemy");
    assert_eq!(
        format!("{}", CountryCode::Shn),
        "Saint Helena, Ascension and Tristan da Cunha"
    );
    assert_eq!(format!("{}", CountryCode::Kna), "Saint Kitts and Nevis");
    assert_eq!(format!("{}", CountryCode::Lca), "Saint Lucia");
    assert_eq!(format!("{}", CountryCode::Maf), "Saint Martin");
    assert_eq!(format!("{}", CountryCode::Spm), "Saint Pierre and Miquelon");
    assert_eq!(
        format!("{}", CountryCode::Vct),
        "Saint Vincent and the Grenadines"
    );
    assert_eq!(format!("{}", CountryCode::Wsm), "Samoa");
    assert_eq!(format!("{}", CountryCode::Smr), "San Marino");
    assert_eq!(format!("{}", CountryCode::Stp), "Sao Tome and Principe");
    assert_eq!(format!("{}", CountryCode::Sau), "Saudi Arabia");
    assert_eq!(format!("{}", CountryCode::Sen), "Senegal");
    assert_eq!(format!("{}", CountryCode::Srb), "Serbia");
    assert_eq!(format!("{}", CountryCode::Syc), "Seychelles");
    assert_eq!(format!("{}", CountryCode::Sle), "Sierra Leone");
    assert_eq!(format!("{}", CountryCode::Sgp), "Singapore");
    assert_eq!(format!("{}", CountryCode::Sxm), "Sint Maarten");
    assert_eq!(format!("{}", CountryCode::Svk), "Slovakia");
    assert_eq!(format!("{}", CountryCode::Svn), "Slovenia");
    assert_eq!(format!("{}", CountryCode::Slb), "Solomon Islands");
    assert_eq!(format!("{}", CountryCode::Som), "Somalia");
    assert_eq!(format!("{}", CountryCode::Zaf), "South Africa");
    assert_eq!(
        format!("{}", CountryCode::Sgs),
        "South Georgia and the South Sandwich Islands"
    );
    assert_eq!(format!("{}", CountryCode::Kor), "South Korea");
    assert_eq!(format!("{}", CountryCode::Ssd), "South Sudan");
    assert_eq!(format!("{}", CountryCode::Esp), "Spain");
    assert_eq!(format!("{}", CountryCode::Lka), "Sri Lanka");
    assert_eq!(format!("{}", CountryCode::Sdn), "Sudan");
    assert_eq!(format!("{}", CountryCode::Sur), "Suriname");
    assert_eq!(format!("{}", CountryCode::Sjm), "Svalbard and Jan Mayen");
    assert_eq!(format!("{}", CountryCode::Swe), "Sweden");
    assert_eq!(format!("{}", CountryCode::Che), "Switzerland");
    assert_eq!(format!("{}", CountryCode::Syr), "Syria");
    assert_eq!(format!("{}", CountryCode::Twn), "Taiwan");
    assert_eq!(format!("{}", CountryCode::Tjk), "Tajikistan");
    assert_eq!(format!("{}", CountryCode::Tza), "Tanzania");
    assert_eq!(format!("{}", CountryCode::Tha), "Thailand");
    assert_eq!(format!("{}", CountryCode::Tls), "Timor-Leste");
    assert_eq!(format!("{}", CountryCode::Tgo), "Togo");
    assert_eq!(format!("{}", CountryCode::Tkl), "Tokelau");
    assert_eq!(format!("{}", CountryCode::Ton), "Tonga");
    assert_eq!(format!("{}", CountryCode::Tto), "Trinidad and Tobago");
    assert_eq!(format!("{}", CountryCode::Tun), "Tunisia");
    assert_eq!(format!("{}", CountryCode::Tur), "Turkey");
    assert_eq!(format!("{}", CountryCode::Tkm), "Turkmenistan");
    assert_eq!(format!("{}", CountryCode::Tca), "Turks and Caicos Islands");
    assert_eq!(format!("{}", CountryCode::Tuv), "Tuvalu");
    assert_eq!(format!("{}", CountryCode::Uga), "Uganda");
    assert_eq!(format!("{}", CountryCode::Ukr), "Ukraine");
    assert_eq!(format!("{}", CountryCode::Are), "United Arab Emirates");
    assert_eq!(format!("{}", CountryCode::Gbr), "United Kingdom");
    assert_eq!(
        format!("{}", CountryCode::Umi),
        "United States Minor Outlying Islands"
    );
    assert_eq!(format!("{}", CountryCode::Usa), "United States of America");
    assert_eq!(format!("{}", CountryCode::Ury), "Uruguay");
    assert_eq!(format!("{}", CountryCode::Uzb), "Uzbekistan");
    assert_eq!(format!("{}", CountryCode::Vut), "Vanuatu");
    assert_eq!(format!("{}", CountryCode::Vat), "Vatican City");
    assert_eq!(format!("{}", CountryCode::Ven), "Venezuela");
    assert_eq!(format!("{}", CountryCode::Vnm), "Viet Nam");
    assert_eq!(format!("{}", CountryCode::Vgb), "Virgin Islands (British)");
    assert_eq!(format!("{}", CountryCode::Vir), "Virgin Islands (U.S.)");
    assert_eq!(format!("{}", CountryCode::Wlf), "Wallis and Futuna");
    assert_eq!(format!("{}", CountryCode::Esh), "Western Sahara");
    assert_eq!(format!("{}", CountryCode::Yem), "Yemen");
    assert_eq!(format!("{}", CountryCode::Zmb), "Zambia");
    assert_eq!(format!("{}", CountryCode::Zwe), "Zimbabwe");
}

#[test]
fn test_countrycode_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        CountryCode::from_str("Afghanistan").unwrap(),
        CountryCode::Afg
    );
    assert_eq!(
        CountryCode::from_str("Åland Islands").unwrap(),
        CountryCode::Ala
    );
    assert_eq!(CountryCode::from_str("Albania").unwrap(), CountryCode::Alb);
    assert_eq!(CountryCode::from_str("Algeria").unwrap(), CountryCode::Dza);
    assert_eq!(
        CountryCode::from_str("American Samoa").unwrap(),
        CountryCode::Asm
    );
    assert_eq!(CountryCode::from_str("Andorra").unwrap(), CountryCode::And);
    assert_eq!(CountryCode::from_str("Angola").unwrap(), CountryCode::Ago);
    assert_eq!(CountryCode::from_str("Anguilla").unwrap(), CountryCode::Aia);
    assert_eq!(
        CountryCode::from_str("Antarctica").unwrap(),
        CountryCode::Ata
    );
    assert_eq!(
        CountryCode::from_str("Antigua and Barbuda").unwrap(),
        CountryCode::Atg
    );
    assert_eq!(
        CountryCode::from_str("Argentina").unwrap(),
        CountryCode::Arg
    );
    assert_eq!(CountryCode::from_str("Armenia").unwrap(), CountryCode::Arm);
    assert_eq!(CountryCode::from_str("Aruba").unwrap(), CountryCode::Abw);
    assert_eq!(
        CountryCode::from_str("Australia").unwrap(),
        CountryCode::Aus
    );
    assert_eq!(CountryCode::from_str("Austria").unwrap(), CountryCode::Aut);
    assert_eq!(
        CountryCode::from_str("Azerbaijan").unwrap(),
        CountryCode::Aze
    );
    assert_eq!(CountryCode::from_str("Bahamas").unwrap(), CountryCode::Bhs);
    assert_eq!(CountryCode::from_str("Bahrain").unwrap(), CountryCode::Bhr);
    assert_eq!(
        CountryCode::from_str("Bangladesh").unwrap(),
        CountryCode::Bgd
    );
    assert_eq!(CountryCode::from_str("Barbados").unwrap(), CountryCode::Brb);
    assert_eq!(CountryCode::from_str("Belarus").unwrap(), CountryCode::Blr);
    assert_eq!(CountryCode::from_str("Belgium").unwrap(), CountryCode::Bel);
    assert_eq!(CountryCode::from_str("Belize").unwrap(), CountryCode::Blz);
    assert_eq!(CountryCode::from_str("Benin").unwrap(), CountryCode::Ben);
    assert_eq!(CountryCode::from_str("Bermuda").unwrap(), CountryCode::Bmu);
    assert_eq!(CountryCode::from_str("Bhutan").unwrap(), CountryCode::Btn);
    assert_eq!(CountryCode::from_str("Bolivia").unwrap(), CountryCode::Bol);
    assert_eq!(
        CountryCode::from_str("Bonaire, Sint Eustatius and Saba").unwrap(),
        CountryCode::Bes
    );
    assert_eq!(
        CountryCode::from_str("Bosnia and Herzegovina").unwrap(),
        CountryCode::Bih
    );
    assert_eq!(CountryCode::from_str("Botswana").unwrap(), CountryCode::Bwa);
    assert_eq!(
        CountryCode::from_str("Bouvet Island").unwrap(),
        CountryCode::Bvt
    );
    assert_eq!(CountryCode::from_str("Brazil").unwrap(), CountryCode::Bra);
    assert_eq!(
        CountryCode::from_str("British Indian Ocean Territory").unwrap(),
        CountryCode::Iot
    );
    assert_eq!(CountryCode::from_str("Brunei").unwrap(), CountryCode::Brn);
    assert_eq!(CountryCode::from_str("Bulgaria").unwrap(), CountryCode::Bgr);
    assert_eq!(
        CountryCode::from_str("Burkina Faso").unwrap(),
        CountryCode::Bfa
    );
    assert_eq!(CountryCode::from_str("Burundi").unwrap(), CountryCode::Bdi);
    assert_eq!(
        CountryCode::from_str("Cabo Verde").unwrap(),
        CountryCode::Cpv
    );
    assert_eq!(CountryCode::from_str("Cambodia").unwrap(), CountryCode::Khm);
    assert_eq!(CountryCode::from_str("Cameroon").unwrap(), CountryCode::Cmr);
    assert_eq!(CountryCode::from_str("Canada").unwrap(), CountryCode::Can);
    assert_eq!(
        CountryCode::from_str("Cayman Islands").unwrap(),
        CountryCode::Cym
    );
    assert_eq!(
        CountryCode::from_str("Central African Republic").unwrap(),
        CountryCode::Caf
    );
    assert_eq!(CountryCode::from_str("Chad").unwrap(), CountryCode::Tcd);
    assert_eq!(CountryCode::from_str("Chile").unwrap(), CountryCode::Chl);
    assert_eq!(CountryCode::from_str("China").unwrap(), CountryCode::Chn);
    assert_eq!(
        CountryCode::from_str("Christmas Island").unwrap(),
        CountryCode::Cxr
    );
    assert_eq!(
        CountryCode::from_str("Cocos (Keeling) Islands").unwrap(),
        CountryCode::Cck
    );
    assert_eq!(CountryCode::from_str("Colombia").unwrap(), CountryCode::Col);
    assert_eq!(CountryCode::from_str("Comoros").unwrap(), CountryCode::Com);
    assert_eq!(
        CountryCode::from_str("Cook Islands").unwrap(),
        CountryCode::Cok
    );
    assert_eq!(
        CountryCode::from_str("Costa Rica").unwrap(),
        CountryCode::Cri
    );
    assert_eq!(
        CountryCode::from_str("Côte d'Ivoire").unwrap(),
        CountryCode::Civ
    );
    assert_eq!(CountryCode::from_str("Croatia").unwrap(), CountryCode::Hrv);
    assert_eq!(CountryCode::from_str("Cuba").unwrap(), CountryCode::Cub);
    assert_eq!(CountryCode::from_str("Curaçao").unwrap(), CountryCode::Cuw);
    assert_eq!(CountryCode::from_str("Cyprus").unwrap(), CountryCode::Cyp);
    assert_eq!(CountryCode::from_str("Czechia").unwrap(), CountryCode::Cze);
    assert_eq!(
        CountryCode::from_str("Democratic Republic of the Congo").unwrap(),
        CountryCode::Cod
    );
    assert_eq!(CountryCode::from_str("Denmark").unwrap(), CountryCode::Dnk);
    assert_eq!(CountryCode::from_str("Djibouti").unwrap(), CountryCode::Dji);
    assert_eq!(CountryCode::from_str("Dominica").unwrap(), CountryCode::Dma);
    assert_eq!(
        CountryCode::from_str("Dominican Republic").unwrap(),
        CountryCode::Dom
    );
    assert_eq!(CountryCode::from_str("Ecuador").unwrap(), CountryCode::Ecu);
    assert_eq!(CountryCode::from_str("Egypt").unwrap(), CountryCode::Egy);
    assert_eq!(
        CountryCode::from_str("El Salvador").unwrap(),
        CountryCode::Slv
    );
    assert_eq!(
        CountryCode::from_str("Equatorial Guinea").unwrap(),
        CountryCode::Gnq
    );
    assert_eq!(CountryCode::from_str("Eritrea").unwrap(), CountryCode::Eri);
    assert_eq!(CountryCode::from_str("Estonia").unwrap(), CountryCode::Est);
    assert_eq!(CountryCode::from_str("Eswatini").unwrap(), CountryCode::Swz);
    assert_eq!(CountryCode::from_str("Ethiopia").unwrap(), CountryCode::Eth);
    assert_eq!(
        CountryCode::from_str("Falkland Islands").unwrap(),
        CountryCode::Flk
    );
    assert_eq!(
        CountryCode::from_str("Faroe Islands").unwrap(),
        CountryCode::Fro
    );
    assert_eq!(CountryCode::from_str("Fiji").unwrap(), CountryCode::Fji);
    assert_eq!(CountryCode::from_str("Finland").unwrap(), CountryCode::Fin);
    assert_eq!(CountryCode::from_str("France").unwrap(), CountryCode::Fra);
    assert_eq!(
        CountryCode::from_str("French Guiana").unwrap(),
        CountryCode::Guf
    );
    assert_eq!(
        CountryCode::from_str("French Polynesia").unwrap(),
        CountryCode::Pyf
    );
    assert_eq!(
        CountryCode::from_str("French Southern Territories").unwrap(),
        CountryCode::Atf
    );
    assert_eq!(CountryCode::from_str("Gabon").unwrap(), CountryCode::Gab);
    assert_eq!(CountryCode::from_str("Gambia").unwrap(), CountryCode::Gmb);
    assert_eq!(CountryCode::from_str("Georgia").unwrap(), CountryCode::Geo);
    assert_eq!(CountryCode::from_str("Germany").unwrap(), CountryCode::Deu);
    assert_eq!(CountryCode::from_str("Ghana").unwrap(), CountryCode::Gha);
    assert_eq!(
        CountryCode::from_str("Gibraltar").unwrap(),
        CountryCode::Gib
    );
    assert_eq!(CountryCode::from_str("Greece").unwrap(), CountryCode::Grc);
    assert_eq!(
        CountryCode::from_str("Greenland").unwrap(),
        CountryCode::Grl
    );
    assert_eq!(CountryCode::from_str("Grenada").unwrap(), CountryCode::Grd);
    assert_eq!(
        CountryCode::from_str("Guadeloupe").unwrap(),
        CountryCode::Glp
    );
    assert_eq!(CountryCode::from_str("Guam").unwrap(), CountryCode::Gum);
    assert_eq!(
        CountryCode::from_str("Guatemala").unwrap(),
        CountryCode::Gtm
    );
    assert_eq!(CountryCode::from_str("Guernsey").unwrap(), CountryCode::Ggy);
    assert_eq!(CountryCode::from_str("Guinea").unwrap(), CountryCode::Gin);
    assert_eq!(
        CountryCode::from_str("Guinea-Bissau").unwrap(),
        CountryCode::Gnb
    );
    assert_eq!(CountryCode::from_str("Guyana").unwrap(), CountryCode::Guy);
    assert_eq!(CountryCode::from_str("Haiti").unwrap(), CountryCode::Hti);
    assert_eq!(
        CountryCode::from_str("Heard Island and McDonald Islands").unwrap(),
        CountryCode::Hmd
    );
    assert_eq!(CountryCode::from_str("Honduras").unwrap(), CountryCode::Hnd);
    assert_eq!(
        CountryCode::from_str("Hong Kong").unwrap(),
        CountryCode::Hkg
    );
    assert_eq!(CountryCode::from_str("Hungary").unwrap(), CountryCode::Hun);
    assert_eq!(CountryCode::from_str("Iceland").unwrap(), CountryCode::Isl);
    assert_eq!(CountryCode::from_str("India").unwrap(), CountryCode::Ind);
    assert_eq!(
        CountryCode::from_str("Indonesia").unwrap(),
        CountryCode::Idn
    );
    assert_eq!(CountryCode::from_str("Iran").unwrap(), CountryCode::Irn);
    assert_eq!(CountryCode::from_str("Iraq").unwrap(), CountryCode::Irq);
    assert_eq!(CountryCode::from_str("Ireland").unwrap(), CountryCode::Irl);
    assert_eq!(
        CountryCode::from_str("Isle of Man").unwrap(),
        CountryCode::Imn
    );
    assert_eq!(CountryCode::from_str("Israel").unwrap(), CountryCode::Isr);
    assert_eq!(CountryCode::from_str("Italy").unwrap(), CountryCode::Ita);
    assert_eq!(CountryCode::from_str("Jamaica").unwrap(), CountryCode::Jam);
    assert_eq!(CountryCode::from_str("Japan").unwrap(), CountryCode::Jpn);
    assert_eq!(CountryCode::from_str("Jersey").unwrap(), CountryCode::Jey);
    assert_eq!(CountryCode::from_str("Jordan").unwrap(), CountryCode::Jor);
    assert_eq!(
        CountryCode::from_str("Kazakhstan").unwrap(),
        CountryCode::Kaz
    );
    assert_eq!(CountryCode::from_str("Kenya").unwrap(), CountryCode::Ken);
    assert_eq!(CountryCode::from_str("Kiribati").unwrap(), CountryCode::Kir);
    assert_eq!(CountryCode::from_str("Kuwait").unwrap(), CountryCode::Kwt);
    assert_eq!(
        CountryCode::from_str("Kyrgyzstan").unwrap(),
        CountryCode::Kgz
    );
    assert_eq!(CountryCode::from_str("Laos").unwrap(), CountryCode::Lao);
    assert_eq!(CountryCode::from_str("Latvia").unwrap(), CountryCode::Lva);
    assert_eq!(CountryCode::from_str("Lebanon").unwrap(), CountryCode::Lbn);
    assert_eq!(CountryCode::from_str("Lesotho").unwrap(), CountryCode::Lso);
    assert_eq!(CountryCode::from_str("Liberia").unwrap(), CountryCode::Lbr);
    assert_eq!(CountryCode::from_str("Libya").unwrap(), CountryCode::Lby);
    assert_eq!(
        CountryCode::from_str("Liechtenstein").unwrap(),
        CountryCode::Lie
    );
    assert_eq!(
        CountryCode::from_str("Lithuania").unwrap(),
        CountryCode::Ltu
    );
    assert_eq!(
        CountryCode::from_str("Luxembourg").unwrap(),
        CountryCode::Lux
    );
    assert_eq!(CountryCode::from_str("Macao").unwrap(), CountryCode::Mac);
    assert_eq!(
        CountryCode::from_str("Madagascar").unwrap(),
        CountryCode::Mdg
    );
    assert_eq!(CountryCode::from_str("Malawi").unwrap(), CountryCode::Mwi);
    assert_eq!(CountryCode::from_str("Malaysia").unwrap(), CountryCode::Mys);
    assert_eq!(CountryCode::from_str("Maldives").unwrap(), CountryCode::Mdv);
    assert_eq!(CountryCode::from_str("Mali").unwrap(), CountryCode::Mli);
    assert_eq!(CountryCode::from_str("Malta").unwrap(), CountryCode::Mlt);
    assert_eq!(
        CountryCode::from_str("Marshall Islands").unwrap(),
        CountryCode::Mhl
    );
    assert_eq!(
        CountryCode::from_str("Martinique").unwrap(),
        CountryCode::Mtq
    );
    assert_eq!(
        CountryCode::from_str("Mauritania").unwrap(),
        CountryCode::Mrt
    );
    assert_eq!(
        CountryCode::from_str("Mauritius").unwrap(),
        CountryCode::Mus
    );
    assert_eq!(CountryCode::from_str("Mayotte").unwrap(), CountryCode::Myt);
    assert_eq!(CountryCode::from_str("Mexico").unwrap(), CountryCode::Mex);
    assert_eq!(
        CountryCode::from_str("Micronesia").unwrap(),
        CountryCode::Fsm
    );
    assert_eq!(CountryCode::from_str("Moldova").unwrap(), CountryCode::Mda);
    assert_eq!(CountryCode::from_str("Monaco").unwrap(), CountryCode::Mco);
    assert_eq!(CountryCode::from_str("Mongolia").unwrap(), CountryCode::Mng);
    assert_eq!(
        CountryCode::from_str("Montenegro").unwrap(),
        CountryCode::Mne
    );
    assert_eq!(
        CountryCode::from_str("Montserrat").unwrap(),
        CountryCode::Msr
    );
    assert_eq!(CountryCode::from_str("Morocco").unwrap(), CountryCode::Mar);
    assert_eq!(
        CountryCode::from_str("Mozambique").unwrap(),
        CountryCode::Moz
    );
    assert_eq!(CountryCode::from_str("Myanmar").unwrap(), CountryCode::Mmr);
    assert_eq!(CountryCode::from_str("Namibia").unwrap(), CountryCode::Nam);
    assert_eq!(CountryCode::from_str("Nauru").unwrap(), CountryCode::Nru);
    assert_eq!(CountryCode::from_str("Nepal").unwrap(), CountryCode::Npl);
    assert_eq!(
        CountryCode::from_str("Netherlands").unwrap(),
        CountryCode::Nld
    );
    assert_eq!(
        CountryCode::from_str("New Caledonia").unwrap(),
        CountryCode::Ncl
    );
    assert_eq!(
        CountryCode::from_str("New Zealand").unwrap(),
        CountryCode::Nzl
    );
    assert_eq!(
        CountryCode::from_str("Nicaragua").unwrap(),
        CountryCode::Nic
    );
    assert_eq!(CountryCode::from_str("Niger").unwrap(), CountryCode::Ner);
    assert_eq!(CountryCode::from_str("Nigeria").unwrap(), CountryCode::Nga);
    assert_eq!(CountryCode::from_str("Niue").unwrap(), CountryCode::Niu);
    assert_eq!(
        CountryCode::from_str("Norfolk Island").unwrap(),
        CountryCode::Nfk
    );
    assert_eq!(
        CountryCode::from_str("North Korea").unwrap(),
        CountryCode::Prk
    );
    assert_eq!(
        CountryCode::from_str("North Macedonia").unwrap(),
        CountryCode::Mkd
    );
    assert_eq!(
        CountryCode::from_str("Northern Mariana Islands").unwrap(),
        CountryCode::Mnp
    );
    assert_eq!(CountryCode::from_str("Norway").unwrap(), CountryCode::Nor);
    assert_eq!(CountryCode::from_str("Oman").unwrap(), CountryCode::Omn);
    assert_eq!(CountryCode::from_str("Pakistan").unwrap(), CountryCode::Pak);
    assert_eq!(CountryCode::from_str("Palau").unwrap(), CountryCode::Plw);
    assert_eq!(
        CountryCode::from_str("Palestine").unwrap(),
        CountryCode::Pse
    );
    assert_eq!(CountryCode::from_str("Panama").unwrap(), CountryCode::Pan);
    assert_eq!(
        CountryCode::from_str("Papua New Guinea").unwrap(),
        CountryCode::Png
    );
    assert_eq!(CountryCode::from_str("Paraguay").unwrap(), CountryCode::Pry);
    assert_eq!(CountryCode::from_str("Peru").unwrap(), CountryCode::Per);
    assert_eq!(
        CountryCode::from_str("Philippines").unwrap(),
        CountryCode::Phl
    );
    assert_eq!(CountryCode::from_str("Pitcairn").unwrap(), CountryCode::Pcn);
    assert_eq!(CountryCode::from_str("Poland").unwrap(), CountryCode::Pol);
    assert_eq!(CountryCode::from_str("Portugal").unwrap(), CountryCode::Prt);
    assert_eq!(
        CountryCode::from_str("Puerto Rico").unwrap(),
        CountryCode::Pri
    );
    assert_eq!(CountryCode::from_str("Qatar").unwrap(), CountryCode::Qat);
    assert_eq!(
        CountryCode::from_str("Republic of the Congo").unwrap(),
        CountryCode::Cog
    );
    assert_eq!(CountryCode::from_str("Réunion").unwrap(), CountryCode::Reu);
    assert_eq!(CountryCode::from_str("Romania").unwrap(), CountryCode::Rou);
    assert_eq!(CountryCode::from_str("Russia").unwrap(), CountryCode::Rus);
    assert_eq!(CountryCode::from_str("Rwanda").unwrap(), CountryCode::Rwa);
    assert_eq!(
        CountryCode::from_str("Saint Barthélemy").unwrap(),
        CountryCode::Blm
    );
    assert_eq!(
        CountryCode::from_str("Saint Helena, Ascension and Tristan da Cunha").unwrap(),
        CountryCode::Shn
    );
    assert_eq!(
        CountryCode::from_str("Saint Kitts and Nevis").unwrap(),
        CountryCode::Kna
    );
    assert_eq!(
        CountryCode::from_str("Saint Lucia").unwrap(),
        CountryCode::Lca
    );
    assert_eq!(
        CountryCode::from_str("Saint Martin").unwrap(),
        CountryCode::Maf
    );
    assert_eq!(
        CountryCode::from_str("Saint Pierre and Miquelon").unwrap(),
        CountryCode::Spm
    );
    assert_eq!(
        CountryCode::from_str("Saint Vincent and the Grenadines").unwrap(),
        CountryCode::Vct
    );
    assert_eq!(CountryCode::from_str("Samoa").unwrap(), CountryCode::Wsm);
    assert_eq!(
        CountryCode::from_str("San Marino").unwrap(),
        CountryCode::Smr
    );
    assert_eq!(
        CountryCode::from_str("Sao Tome and Principe").unwrap(),
        CountryCode::Stp
    );
    assert_eq!(
        CountryCode::from_str("Saudi Arabia").unwrap(),
        CountryCode::Sau
    );
    assert_eq!(CountryCode::from_str("Senegal").unwrap(), CountryCode::Sen);
    assert_eq!(CountryCode::from_str("Serbia").unwrap(), CountryCode::Srb);
    assert_eq!(
        CountryCode::from_str("Seychelles").unwrap(),
        CountryCode::Syc
    );
    assert_eq!(
        CountryCode::from_str("Sierra Leone").unwrap(),
        CountryCode::Sle
    );
    assert_eq!(
        CountryCode::from_str("Singapore").unwrap(),
        CountryCode::Sgp
    );
    assert_eq!(
        CountryCode::from_str("Sint Maarten").unwrap(),
        CountryCode::Sxm
    );
    assert_eq!(CountryCode::from_str("Slovakia").unwrap(), CountryCode::Svk);
    assert_eq!(CountryCode::from_str("Slovenia").unwrap(), CountryCode::Svn);
    assert_eq!(
        CountryCode::from_str("Solomon Islands").unwrap(),
        CountryCode::Slb
    );
    assert_eq!(CountryCode::from_str("Somalia").unwrap(), CountryCode::Som);
    assert_eq!(
        CountryCode::from_str("South Africa").unwrap(),
        CountryCode::Zaf
    );
    assert_eq!(
        CountryCode::from_str("South Georgia and the South Sandwich Islands").unwrap(),
        CountryCode::Sgs
    );
    assert_eq!(
        CountryCode::from_str("South Korea").unwrap(),
        CountryCode::Kor
    );
    assert_eq!(
        CountryCode::from_str("South Sudan").unwrap(),
        CountryCode::Ssd
    );
    assert_eq!(CountryCode::from_str("Spain").unwrap(), CountryCode::Esp);
    assert_eq!(
        CountryCode::from_str("Sri Lanka").unwrap(),
        CountryCode::Lka
    );
    assert_eq!(CountryCode::from_str("Sudan").unwrap(), CountryCode::Sdn);
    assert_eq!(CountryCode::from_str("Suriname").unwrap(), CountryCode::Sur);
    assert_eq!(
        CountryCode::from_str("Svalbard and Jan Mayen").unwrap(),
        CountryCode::Sjm
    );
    assert_eq!(CountryCode::from_str("Sweden").unwrap(), CountryCode::Swe);
    assert_eq!(
        CountryCode::from_str("Switzerland").unwrap(),
        CountryCode::Che
    );
    assert_eq!(CountryCode::from_str("Syria").unwrap(), CountryCode::Syr);
    assert_eq!(CountryCode::from_str("Taiwan").unwrap(), CountryCode::Twn);
    assert_eq!(
        CountryCode::from_str("Tajikistan").unwrap(),
        CountryCode::Tjk
    );
    assert_eq!(CountryCode::from_str("Tanzania").unwrap(), CountryCode::Tza);
    assert_eq!(CountryCode::from_str("Thailand").unwrap(), CountryCode::Tha);
    assert_eq!(
        CountryCode::from_str("Timor-Leste").unwrap(),
        CountryCode::Tls
    );
    assert_eq!(CountryCode::from_str("Togo").unwrap(), CountryCode::Tgo);
    assert_eq!(CountryCode::from_str("Tokelau").unwrap(), CountryCode::Tkl);
    assert_eq!(CountryCode::from_str("Tonga").unwrap(), CountryCode::Ton);
    assert_eq!(
        CountryCode::from_str("Trinidad and Tobago").unwrap(),
        CountryCode::Tto
    );
    assert_eq!(CountryCode::from_str("Tunisia").unwrap(), CountryCode::Tun);
    assert_eq!(CountryCode::from_str("Turkey").unwrap(), CountryCode::Tur);
    assert_eq!(
        CountryCode::from_str("Turkmenistan").unwrap(),
        CountryCode::Tkm
    );
    assert_eq!(
        CountryCode::from_str("Turks and Caicos Islands").unwrap(),
        CountryCode::Tca
    );
    assert_eq!(CountryCode::from_str("Tuvalu").unwrap(), CountryCode::Tuv);
    assert_eq!(CountryCode::from_str("Uganda").unwrap(), CountryCode::Uga);
    assert_eq!(CountryCode::from_str("Ukraine").unwrap(), CountryCode::Ukr);
    assert_eq!(
        CountryCode::from_str("United Arab Emirates").unwrap(),
        CountryCode::Are
    );
    assert_eq!(
        CountryCode::from_str("United Kingdom").unwrap(),
        CountryCode::Gbr
    );
    assert_eq!(
        CountryCode::from_str("United States Minor Outlying Islands").unwrap(),
        CountryCode::Umi
    );
    assert_eq!(
        CountryCode::from_str("United States of America").unwrap(),
        CountryCode::Usa
    );
    assert_eq!(CountryCode::from_str("Uruguay").unwrap(), CountryCode::Ury);
    assert_eq!(
        CountryCode::from_str("Uzbekistan").unwrap(),
        CountryCode::Uzb
    );
    assert_eq!(CountryCode::from_str("Vanuatu").unwrap(), CountryCode::Vut);
    assert_eq!(
        CountryCode::from_str("Vatican City").unwrap(),
        CountryCode::Vat
    );
    assert_eq!(
        CountryCode::from_str("Venezuela").unwrap(),
        CountryCode::Ven
    );
    assert_eq!(CountryCode::from_str("Viet Nam").unwrap(), CountryCode::Vnm);
    assert_eq!(
        CountryCode::from_str("Virgin Islands (British)").unwrap(),
        CountryCode::Vgb
    );
    assert_eq!(
        CountryCode::from_str("Virgin Islands (U.S.)").unwrap(),
        CountryCode::Vir
    );
    assert_eq!(
        CountryCode::from_str("Wallis and Futuna").unwrap(),
        CountryCode::Wlf
    );
    assert_eq!(
        CountryCode::from_str("Western Sahara").unwrap(),
        CountryCode::Esh
    );
    assert_eq!(CountryCode::from_str("Yemen").unwrap(), CountryCode::Yem);
    assert_eq!(CountryCode::from_str("Zambia").unwrap(), CountryCode::Zmb);
    assert_eq!(CountryCode::from_str("Zimbabwe").unwrap(), CountryCode::Zwe);
    assert!(CountryCode::from_str("Narnia").is_err());
    assert!(CountryCode::from_str("Mesopotamia").is_err());
    assert!(CountryCode::from_str("Czechoslovakia").is_err());
}

#[cfg(feature = "backend")]
pub mod crud;
