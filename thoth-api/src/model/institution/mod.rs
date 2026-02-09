use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CountryCode {
    #[cfg_attr(feature = "backend", graphql(description = "Afghanistan"))]
    #[strum(serialize = "Afghanistan")]
    Afg,
    #[cfg_attr(feature = "backend", graphql(description = "Åland Islands"))]
    #[strum(serialize = "Åland Islands")]
    Ala,
    #[cfg_attr(feature = "backend", graphql(description = "Albania"))]
    #[strum(serialize = "Albania")]
    Alb,
    #[cfg_attr(feature = "backend", graphql(description = "Algeria"))]
    #[strum(serialize = "Algeria")]
    Dza,
    #[cfg_attr(feature = "backend", graphql(description = "American Samoa"))]
    #[strum(serialize = "American Samoa")]
    Asm,
    #[cfg_attr(feature = "backend", graphql(description = "Andorra"))]
    #[strum(serialize = "Andorra")]
    And,
    #[cfg_attr(feature = "backend", graphql(description = "Angola"))]
    #[strum(serialize = "Angola")]
    Ago,
    #[cfg_attr(feature = "backend", graphql(description = "Anguilla"))]
    #[strum(serialize = "Anguilla")]
    Aia,
    #[cfg_attr(feature = "backend", graphql(description = "Antarctica"))]
    #[strum(serialize = "Antarctica")]
    Ata,
    #[cfg_attr(feature = "backend", graphql(description = "Antigua and Barbuda"))]
    #[strum(serialize = "Antigua and Barbuda")]
    Atg,
    #[cfg_attr(feature = "backend", graphql(description = "Argentina"))]
    #[strum(serialize = "Argentina")]
    Arg,
    #[cfg_attr(feature = "backend", graphql(description = "Armenia"))]
    #[strum(serialize = "Armenia")]
    Arm,
    #[cfg_attr(feature = "backend", graphql(description = "Aruba"))]
    #[strum(serialize = "Aruba")]
    Abw,
    #[cfg_attr(feature = "backend", graphql(description = "Australia"))]
    #[strum(serialize = "Australia")]
    Aus,
    #[cfg_attr(feature = "backend", graphql(description = "Austria"))]
    #[strum(serialize = "Austria")]
    Aut,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijan"))]
    #[strum(serialize = "Azerbaijan")]
    Aze,
    #[cfg_attr(feature = "backend", graphql(description = "Bahamas"))]
    #[strum(serialize = "Bahamas")]
    Bhs,
    #[cfg_attr(feature = "backend", graphql(description = "Bahrain"))]
    #[strum(serialize = "Bahrain")]
    Bhr,
    #[cfg_attr(feature = "backend", graphql(description = "Bangladesh"))]
    #[strum(serialize = "Bangladesh")]
    Bgd,
    #[cfg_attr(feature = "backend", graphql(description = "Barbados"))]
    #[strum(serialize = "Barbados")]
    Brb,
    #[cfg_attr(feature = "backend", graphql(description = "Belarus"))]
    #[strum(serialize = "Belarus")]
    Blr,
    #[cfg_attr(feature = "backend", graphql(description = "Belgium"))]
    #[strum(serialize = "Belgium")]
    Bel,
    #[cfg_attr(feature = "backend", graphql(description = "Belize"))]
    #[strum(serialize = "Belize")]
    Blz,
    #[cfg_attr(feature = "backend", graphql(description = "Benin"))]
    #[strum(serialize = "Benin")]
    Ben,
    #[cfg_attr(feature = "backend", graphql(description = "Bermuda"))]
    #[strum(serialize = "Bermuda")]
    Bmu,
    #[cfg_attr(feature = "backend", graphql(description = "Bhutan"))]
    #[strum(serialize = "Bhutan")]
    Btn,
    #[cfg_attr(feature = "backend", graphql(description = "Bolivia"))]
    #[strum(serialize = "Bolivia")]
    Bol,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Bonaire, Sint Eustatius and Saba")
    )]
    #[strum(serialize = "Bonaire, Sint Eustatius and Saba")]
    Bes,
    #[cfg_attr(feature = "backend", graphql(description = "Bosnia and Herzegovina"))]
    #[strum(serialize = "Bosnia and Herzegovina")]
    Bih,
    #[cfg_attr(feature = "backend", graphql(description = "Botswana"))]
    #[strum(serialize = "Botswana")]
    Bwa,
    #[cfg_attr(feature = "backend", graphql(description = "Bouvet Island"))]
    #[strum(serialize = "Bouvet Island")]
    Bvt,
    #[cfg_attr(feature = "backend", graphql(description = "Brazil"))]
    #[strum(serialize = "Brazil")]
    Bra,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "British Indian Ocean Territory")
    )]
    #[strum(serialize = "British Indian Ocean Territory")]
    Iot,
    #[cfg_attr(feature = "backend", graphql(description = "Brunei"))]
    #[strum(serialize = "Brunei")]
    Brn,
    #[cfg_attr(feature = "backend", graphql(description = "Bulgaria"))]
    #[strum(serialize = "Bulgaria")]
    Bgr,
    #[cfg_attr(feature = "backend", graphql(description = "Burkina Faso"))]
    #[strum(serialize = "Burkina Faso")]
    Bfa,
    #[cfg_attr(feature = "backend", graphql(description = "Burundi"))]
    #[strum(serialize = "Burundi")]
    Bdi,
    #[cfg_attr(feature = "backend", graphql(description = "Cabo Verde"))]
    #[strum(serialize = "Cabo Verde")]
    Cpv,
    #[cfg_attr(feature = "backend", graphql(description = "Cambodia"))]
    #[strum(serialize = "Cambodia")]
    Khm,
    #[cfg_attr(feature = "backend", graphql(description = "Cameroon"))]
    #[strum(serialize = "Cameroon")]
    Cmr,
    #[cfg_attr(feature = "backend", graphql(description = "Canada"))]
    #[strum(serialize = "Canada")]
    Can,
    #[cfg_attr(feature = "backend", graphql(description = "Cayman Islands"))]
    #[strum(serialize = "Cayman Islands")]
    Cym,
    #[cfg_attr(feature = "backend", graphql(description = "Central African Republic"))]
    #[strum(serialize = "Central African Republic")]
    Caf,
    #[cfg_attr(feature = "backend", graphql(description = "Chad"))]
    #[strum(serialize = "Chad")]
    Tcd,
    #[cfg_attr(feature = "backend", graphql(description = "Chile"))]
    #[strum(serialize = "Chile")]
    Chl,
    #[cfg_attr(feature = "backend", graphql(description = "China"))]
    #[strum(serialize = "China")]
    Chn,
    #[cfg_attr(feature = "backend", graphql(description = "Christmas Island"))]
    #[strum(serialize = "Christmas Island")]
    Cxr,
    #[cfg_attr(feature = "backend", graphql(description = "Cocos (Keeling) Islands"))]
    #[strum(serialize = "Cocos (Keeling) Islands")]
    Cck,
    #[cfg_attr(feature = "backend", graphql(description = "Colombia"))]
    #[strum(serialize = "Colombia")]
    Col,
    #[cfg_attr(feature = "backend", graphql(description = "Comoros"))]
    #[strum(serialize = "Comoros")]
    Com,
    #[cfg_attr(feature = "backend", graphql(description = "Cook Islands"))]
    #[strum(serialize = "Cook Islands")]
    Cok,
    #[cfg_attr(feature = "backend", graphql(description = "Costa Rica"))]
    #[strum(serialize = "Costa Rica")]
    Cri,
    #[cfg_attr(feature = "backend", graphql(description = "Côte d'Ivoire"))]
    #[strum(serialize = "Côte d'Ivoire")]
    Civ,
    #[cfg_attr(feature = "backend", graphql(description = "Croatia"))]
    #[strum(serialize = "Croatia")]
    Hrv,
    #[cfg_attr(feature = "backend", graphql(description = "Cuba"))]
    #[strum(serialize = "Cuba")]
    Cub,
    #[cfg_attr(feature = "backend", graphql(description = "Curaçao"))]
    #[strum(serialize = "Curaçao")]
    Cuw,
    #[cfg_attr(feature = "backend", graphql(description = "Cyprus"))]
    #[strum(serialize = "Cyprus")]
    Cyp,
    #[cfg_attr(feature = "backend", graphql(description = "Czechia"))]
    #[strum(serialize = "Czechia")]
    Cze,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Democratic Republic of the Congo")
    )]
    #[strum(serialize = "Democratic Republic of the Congo")]
    Cod,
    #[cfg_attr(feature = "backend", graphql(description = "Denmark"))]
    #[strum(serialize = "Denmark")]
    Dnk,
    #[cfg_attr(feature = "backend", graphql(description = "Djibouti"))]
    #[strum(serialize = "Djibouti")]
    Dji,
    #[cfg_attr(feature = "backend", graphql(description = "Dominica"))]
    #[strum(serialize = "Dominica")]
    Dma,
    #[cfg_attr(feature = "backend", graphql(description = "Dominican Republic"))]
    #[strum(serialize = "Dominican Republic")]
    Dom,
    #[cfg_attr(feature = "backend", graphql(description = "Ecuador"))]
    #[strum(serialize = "Ecuador")]
    Ecu,
    #[cfg_attr(feature = "backend", graphql(description = "Egypt"))]
    #[strum(serialize = "Egypt")]
    Egy,
    #[cfg_attr(feature = "backend", graphql(description = "El Salvador"))]
    #[strum(serialize = "El Salvador")]
    Slv,
    #[cfg_attr(feature = "backend", graphql(description = "Equatorial Guinea"))]
    #[strum(serialize = "Equatorial Guinea")]
    Gnq,
    #[cfg_attr(feature = "backend", graphql(description = "Eritrea"))]
    #[strum(serialize = "Eritrea")]
    Eri,
    #[cfg_attr(feature = "backend", graphql(description = "Estonia"))]
    #[strum(serialize = "Estonia")]
    Est,
    #[cfg_attr(feature = "backend", graphql(description = "Eswatini"))]
    #[strum(serialize = "Eswatini")]
    Swz,
    #[cfg_attr(feature = "backend", graphql(description = "Ethiopia"))]
    #[strum(serialize = "Ethiopia")]
    Eth,
    #[cfg_attr(feature = "backend", graphql(description = "Falkland Islands"))]
    #[strum(serialize = "Falkland Islands")]
    Flk,
    #[cfg_attr(feature = "backend", graphql(description = "Faroe Islands"))]
    #[strum(serialize = "Faroe Islands")]
    Fro,
    #[cfg_attr(feature = "backend", graphql(description = "Fiji"))]
    #[strum(serialize = "Fiji")]
    Fji,
    #[cfg_attr(feature = "backend", graphql(description = "Finland"))]
    #[strum(serialize = "Finland")]
    Fin,
    #[cfg_attr(feature = "backend", graphql(description = "France"))]
    #[strum(serialize = "France")]
    Fra,
    #[cfg_attr(feature = "backend", graphql(description = "French Guiana"))]
    #[strum(serialize = "French Guiana")]
    Guf,
    #[cfg_attr(feature = "backend", graphql(description = "French Polynesia"))]
    #[strum(serialize = "French Polynesia")]
    Pyf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French Southern Territories")
    )]
    #[strum(serialize = "French Southern Territories")]
    Atf,
    #[cfg_attr(feature = "backend", graphql(description = "Gabon"))]
    #[strum(serialize = "Gabon")]
    Gab,
    #[cfg_attr(feature = "backend", graphql(description = "Gambia"))]
    #[strum(serialize = "Gambia")]
    Gmb,
    #[cfg_attr(feature = "backend", graphql(description = "Georgia"))]
    #[strum(serialize = "Georgia")]
    Geo,
    #[cfg_attr(feature = "backend", graphql(description = "Germany"))]
    #[strum(serialize = "Germany")]
    Deu,
    #[cfg_attr(feature = "backend", graphql(description = "Ghana"))]
    #[strum(serialize = "Ghana")]
    Gha,
    #[cfg_attr(feature = "backend", graphql(description = "Gibraltar"))]
    #[strum(serialize = "Gibraltar")]
    Gib,
    #[cfg_attr(feature = "backend", graphql(description = "Greece"))]
    #[strum(serialize = "Greece")]
    Grc,
    #[cfg_attr(feature = "backend", graphql(description = "Greenland"))]
    #[strum(serialize = "Greenland")]
    Grl,
    #[cfg_attr(feature = "backend", graphql(description = "Grenada"))]
    #[strum(serialize = "Grenada")]
    Grd,
    #[cfg_attr(feature = "backend", graphql(description = "Guadeloupe"))]
    #[strum(serialize = "Guadeloupe")]
    Glp,
    #[cfg_attr(feature = "backend", graphql(description = "Guam"))]
    #[strum(serialize = "Guam")]
    Gum,
    #[cfg_attr(feature = "backend", graphql(description = "Guatemala"))]
    #[strum(serialize = "Guatemala")]
    Gtm,
    #[cfg_attr(feature = "backend", graphql(description = "Guernsey"))]
    #[strum(serialize = "Guernsey")]
    Ggy,
    #[cfg_attr(feature = "backend", graphql(description = "Guinea"))]
    #[strum(serialize = "Guinea")]
    Gin,
    #[cfg_attr(feature = "backend", graphql(description = "Guinea-Bissau"))]
    #[strum(serialize = "Guinea-Bissau")]
    Gnb,
    #[cfg_attr(feature = "backend", graphql(description = "Guyana"))]
    #[strum(serialize = "Guyana")]
    Guy,
    #[cfg_attr(feature = "backend", graphql(description = "Haiti"))]
    #[strum(serialize = "Haiti")]
    Hti,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Heard Island and McDonald Islands")
    )]
    #[strum(serialize = "Heard Island and McDonald Islands")]
    Hmd,
    #[cfg_attr(feature = "backend", graphql(description = "Honduras"))]
    #[strum(serialize = "Honduras")]
    Hnd,
    #[cfg_attr(feature = "backend", graphql(description = "Hong Kong"))]
    #[strum(serialize = "Hong Kong")]
    Hkg,
    #[cfg_attr(feature = "backend", graphql(description = "Hungary"))]
    #[strum(serialize = "Hungary")]
    Hun,
    #[cfg_attr(feature = "backend", graphql(description = "Iceland"))]
    #[strum(serialize = "Iceland")]
    Isl,
    #[cfg_attr(feature = "backend", graphql(description = "India"))]
    #[strum(serialize = "India")]
    Ind,
    #[cfg_attr(feature = "backend", graphql(description = "Indonesia"))]
    #[strum(serialize = "Indonesia")]
    Idn,
    #[cfg_attr(feature = "backend", graphql(description = "Iran"))]
    #[strum(serialize = "Iran")]
    Irn,
    #[cfg_attr(feature = "backend", graphql(description = "Iraq"))]
    #[strum(serialize = "Iraq")]
    Irq,
    #[cfg_attr(feature = "backend", graphql(description = "Ireland"))]
    #[strum(serialize = "Ireland")]
    Irl,
    #[cfg_attr(feature = "backend", graphql(description = "Isle of Man"))]
    #[strum(serialize = "Isle of Man")]
    Imn,
    #[cfg_attr(feature = "backend", graphql(description = "Israel"))]
    #[strum(serialize = "Israel")]
    Isr,
    #[cfg_attr(feature = "backend", graphql(description = "Italy"))]
    #[strum(serialize = "Italy")]
    Ita,
    #[cfg_attr(feature = "backend", graphql(description = "Jamaica"))]
    #[strum(serialize = "Jamaica")]
    Jam,
    #[cfg_attr(feature = "backend", graphql(description = "Japan"))]
    #[strum(serialize = "Japan")]
    Jpn,
    #[cfg_attr(feature = "backend", graphql(description = "Jersey"))]
    #[strum(serialize = "Jersey")]
    Jey,
    #[cfg_attr(feature = "backend", graphql(description = "Jordan"))]
    #[strum(serialize = "Jordan")]
    Jor,
    #[cfg_attr(feature = "backend", graphql(description = "Kazakhstan"))]
    #[strum(serialize = "Kazakhstan")]
    Kaz,
    #[cfg_attr(feature = "backend", graphql(description = "Kenya"))]
    #[strum(serialize = "Kenya")]
    Ken,
    #[cfg_attr(feature = "backend", graphql(description = "Kiribati"))]
    #[strum(serialize = "Kiribati")]
    Kir,
    #[cfg_attr(feature = "backend", graphql(description = "Kuwait"))]
    #[strum(serialize = "Kuwait")]
    Kwt,
    #[cfg_attr(feature = "backend", graphql(description = "Kyrgyzstan"))]
    #[strum(serialize = "Kyrgyzstan")]
    Kgz,
    #[cfg_attr(feature = "backend", graphql(description = "Laos"))]
    #[strum(serialize = "Laos")]
    Lao,
    #[cfg_attr(feature = "backend", graphql(description = "Latvia"))]
    #[strum(serialize = "Latvia")]
    Lva,
    #[cfg_attr(feature = "backend", graphql(description = "Lebanon"))]
    #[strum(serialize = "Lebanon")]
    Lbn,
    #[cfg_attr(feature = "backend", graphql(description = "Lesotho"))]
    #[strum(serialize = "Lesotho")]
    Lso,
    #[cfg_attr(feature = "backend", graphql(description = "Liberia"))]
    #[strum(serialize = "Liberia")]
    Lbr,
    #[cfg_attr(feature = "backend", graphql(description = "Libya"))]
    #[strum(serialize = "Libya")]
    Lby,
    #[cfg_attr(feature = "backend", graphql(description = "Liechtenstein"))]
    #[strum(serialize = "Liechtenstein")]
    Lie,
    #[cfg_attr(feature = "backend", graphql(description = "Lithuania"))]
    #[strum(serialize = "Lithuania")]
    Ltu,
    #[cfg_attr(feature = "backend", graphql(description = "Luxembourg"))]
    #[strum(serialize = "Luxembourg")]
    Lux,
    #[cfg_attr(feature = "backend", graphql(description = "Macao"))]
    #[strum(serialize = "Macao")]
    Mac,
    #[cfg_attr(feature = "backend", graphql(description = "Madagascar"))]
    #[strum(serialize = "Madagascar")]
    Mdg,
    #[cfg_attr(feature = "backend", graphql(description = "Malawi"))]
    #[strum(serialize = "Malawi")]
    Mwi,
    #[cfg_attr(feature = "backend", graphql(description = "Malaysia"))]
    #[strum(serialize = "Malaysia")]
    Mys,
    #[cfg_attr(feature = "backend", graphql(description = "Maldives"))]
    #[strum(serialize = "Maldives")]
    Mdv,
    #[cfg_attr(feature = "backend", graphql(description = "Mali"))]
    #[strum(serialize = "Mali")]
    Mli,
    #[cfg_attr(feature = "backend", graphql(description = "Malta"))]
    #[strum(serialize = "Malta")]
    Mlt,
    #[cfg_attr(feature = "backend", graphql(description = "Marshall Islands"))]
    #[strum(serialize = "Marshall Islands")]
    Mhl,
    #[cfg_attr(feature = "backend", graphql(description = "Martinique"))]
    #[strum(serialize = "Martinique")]
    Mtq,
    #[cfg_attr(feature = "backend", graphql(description = "Mauritania"))]
    #[strum(serialize = "Mauritania")]
    Mrt,
    #[cfg_attr(feature = "backend", graphql(description = "Mauritius"))]
    #[strum(serialize = "Mauritius")]
    Mus,
    #[cfg_attr(feature = "backend", graphql(description = "Mayotte"))]
    #[strum(serialize = "Mayotte")]
    Myt,
    #[cfg_attr(feature = "backend", graphql(description = "Mexico"))]
    #[strum(serialize = "Mexico")]
    Mex,
    #[cfg_attr(feature = "backend", graphql(description = "Micronesia"))]
    #[strum(serialize = "Micronesia")]
    Fsm,
    #[cfg_attr(feature = "backend", graphql(description = "Moldova"))]
    #[strum(serialize = "Moldova")]
    Mda,
    #[cfg_attr(feature = "backend", graphql(description = "Monaco"))]
    #[strum(serialize = "Monaco")]
    Mco,
    #[cfg_attr(feature = "backend", graphql(description = "Mongolia"))]
    #[strum(serialize = "Mongolia")]
    Mng,
    #[cfg_attr(feature = "backend", graphql(description = "Montenegro"))]
    #[strum(serialize = "Montenegro")]
    Mne,
    #[cfg_attr(feature = "backend", graphql(description = "Montserrat"))]
    #[strum(serialize = "Montserrat")]
    Msr,
    #[cfg_attr(feature = "backend", graphql(description = "Morocco"))]
    #[strum(serialize = "Morocco")]
    Mar,
    #[cfg_attr(feature = "backend", graphql(description = "Mozambique"))]
    #[strum(serialize = "Mozambique")]
    Moz,
    #[cfg_attr(feature = "backend", graphql(description = "Myanmar"))]
    #[strum(serialize = "Myanmar")]
    Mmr,
    #[cfg_attr(feature = "backend", graphql(description = "Namibia"))]
    #[strum(serialize = "Namibia")]
    Nam,
    #[cfg_attr(feature = "backend", graphql(description = "Nauru"))]
    #[strum(serialize = "Nauru")]
    Nru,
    #[cfg_attr(feature = "backend", graphql(description = "Nepal"))]
    #[strum(serialize = "Nepal")]
    Npl,
    #[cfg_attr(feature = "backend", graphql(description = "Netherlands"))]
    #[strum(serialize = "Netherlands")]
    Nld,
    #[cfg_attr(feature = "backend", graphql(description = "New Caledonia"))]
    #[strum(serialize = "New Caledonia")]
    Ncl,
    #[cfg_attr(feature = "backend", graphql(description = "New Zealand"))]
    #[strum(serialize = "New Zealand")]
    Nzl,
    #[cfg_attr(feature = "backend", graphql(description = "Nicaragua"))]
    #[strum(serialize = "Nicaragua")]
    Nic,
    #[cfg_attr(feature = "backend", graphql(description = "Niger"))]
    #[strum(serialize = "Niger")]
    Ner,
    #[cfg_attr(feature = "backend", graphql(description = "Nigeria"))]
    #[strum(serialize = "Nigeria")]
    Nga,
    #[cfg_attr(feature = "backend", graphql(description = "Niue"))]
    #[strum(serialize = "Niue")]
    Niu,
    #[cfg_attr(feature = "backend", graphql(description = "Norfolk Island"))]
    #[strum(serialize = "Norfolk Island")]
    Nfk,
    #[cfg_attr(feature = "backend", graphql(description = "North Korea"))]
    #[strum(serialize = "North Korea")]
    Prk,
    #[cfg_attr(feature = "backend", graphql(description = "North Macedonia"))]
    #[strum(serialize = "North Macedonia")]
    Mkd,
    #[cfg_attr(feature = "backend", graphql(description = "Northern Mariana Islands"))]
    #[strum(serialize = "Northern Mariana Islands")]
    Mnp,
    #[cfg_attr(feature = "backend", graphql(description = "Norway"))]
    #[strum(serialize = "Norway")]
    Nor,
    #[cfg_attr(feature = "backend", graphql(description = "Oman"))]
    #[strum(serialize = "Oman")]
    Omn,
    #[cfg_attr(feature = "backend", graphql(description = "Pakistan"))]
    #[strum(serialize = "Pakistan")]
    Pak,
    #[cfg_attr(feature = "backend", graphql(description = "Palau"))]
    #[strum(serialize = "Palau")]
    Plw,
    #[cfg_attr(feature = "backend", graphql(description = "Palestine"))]
    #[strum(serialize = "Palestine")]
    Pse,
    #[cfg_attr(feature = "backend", graphql(description = "Panama"))]
    #[strum(serialize = "Panama")]
    Pan,
    #[cfg_attr(feature = "backend", graphql(description = "Papua New Guinea"))]
    #[strum(serialize = "Papua New Guinea")]
    Png,
    #[cfg_attr(feature = "backend", graphql(description = "Paraguay"))]
    #[strum(serialize = "Paraguay")]
    Pry,
    #[cfg_attr(feature = "backend", graphql(description = "Peru"))]
    #[strum(serialize = "Peru")]
    Per,
    #[cfg_attr(feature = "backend", graphql(description = "Philippines"))]
    #[strum(serialize = "Philippines")]
    Phl,
    #[cfg_attr(feature = "backend", graphql(description = "Pitcairn"))]
    #[strum(serialize = "Pitcairn")]
    Pcn,
    #[cfg_attr(feature = "backend", graphql(description = "Poland"))]
    #[strum(serialize = "Poland")]
    Pol,
    #[cfg_attr(feature = "backend", graphql(description = "Portugal"))]
    #[strum(serialize = "Portugal")]
    Prt,
    #[cfg_attr(feature = "backend", graphql(description = "Puerto Rico"))]
    #[strum(serialize = "Puerto Rico")]
    Pri,
    #[cfg_attr(feature = "backend", graphql(description = "Qatar"))]
    #[strum(serialize = "Qatar")]
    Qat,
    #[cfg_attr(feature = "backend", graphql(description = "Republic of the Congo"))]
    #[strum(serialize = "Republic of the Congo")]
    Cog,
    #[cfg_attr(feature = "backend", graphql(description = "Réunion"))]
    #[strum(serialize = "Réunion")]
    Reu,
    #[cfg_attr(feature = "backend", graphql(description = "Romania"))]
    #[strum(serialize = "Romania")]
    Rou,
    #[cfg_attr(feature = "backend", graphql(description = "Russia"))]
    #[strum(serialize = "Russia")]
    Rus,
    #[cfg_attr(feature = "backend", graphql(description = "Rwanda"))]
    #[strum(serialize = "Rwanda")]
    Rwa,
    #[cfg_attr(feature = "backend", graphql(description = "Saint Barthélemy"))]
    #[strum(serialize = "Saint Barthélemy")]
    Blm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Saint Helena, Ascension and Tristan da Cunha")
    )]
    #[strum(serialize = "Saint Helena, Ascension and Tristan da Cunha")]
    Shn,
    #[cfg_attr(feature = "backend", graphql(description = "Saint Kitts and Nevis"))]
    #[strum(serialize = "Saint Kitts and Nevis")]
    Kna,
    #[cfg_attr(feature = "backend", graphql(description = "Saint Lucia"))]
    #[strum(serialize = "Saint Lucia")]
    Lca,
    #[cfg_attr(feature = "backend", graphql(description = "Saint Martin"))]
    #[strum(serialize = "Saint Martin")]
    Maf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Saint Pierre and Miquelon")
    )]
    #[strum(serialize = "Saint Pierre and Miquelon")]
    Spm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Saint Vincent and the Grenadines")
    )]
    #[strum(serialize = "Saint Vincent and the Grenadines")]
    Vct,
    #[cfg_attr(feature = "backend", graphql(description = "Samoa"))]
    #[strum(serialize = "Samoa")]
    Wsm,
    #[cfg_attr(feature = "backend", graphql(description = "San Marino"))]
    #[strum(serialize = "San Marino")]
    Smr,
    #[cfg_attr(feature = "backend", graphql(description = "Sao Tome and Principe"))]
    #[strum(serialize = "Sao Tome and Principe")]
    Stp,
    #[cfg_attr(feature = "backend", graphql(description = "Saudi Arabia"))]
    #[strum(serialize = "Saudi Arabia")]
    Sau,
    #[cfg_attr(feature = "backend", graphql(description = "Senegal"))]
    #[strum(serialize = "Senegal")]
    Sen,
    #[cfg_attr(feature = "backend", graphql(description = "Serbia"))]
    #[strum(serialize = "Serbia")]
    Srb,
    #[cfg_attr(feature = "backend", graphql(description = "Seychelles"))]
    #[strum(serialize = "Seychelles")]
    Syc,
    #[cfg_attr(feature = "backend", graphql(description = "Sierra Leone"))]
    #[strum(serialize = "Sierra Leone")]
    Sle,
    #[cfg_attr(feature = "backend", graphql(description = "Singapore"))]
    #[strum(serialize = "Singapore")]
    Sgp,
    #[cfg_attr(feature = "backend", graphql(description = "Sint Maarten"))]
    #[strum(serialize = "Sint Maarten")]
    Sxm,
    #[cfg_attr(feature = "backend", graphql(description = "Slovakia"))]
    #[strum(serialize = "Slovakia")]
    Svk,
    #[cfg_attr(feature = "backend", graphql(description = "Slovenia"))]
    #[strum(serialize = "Slovenia")]
    Svn,
    #[cfg_attr(feature = "backend", graphql(description = "Solomon Islands"))]
    #[strum(serialize = "Solomon Islands")]
    Slb,
    #[cfg_attr(feature = "backend", graphql(description = "Somalia"))]
    #[strum(serialize = "Somalia")]
    Som,
    #[cfg_attr(feature = "backend", graphql(description = "South Africa"))]
    #[strum(serialize = "South Africa")]
    Zaf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "South Georgia and the South Sandwich Islands")
    )]
    #[strum(serialize = "South Georgia and the South Sandwich Islands")]
    Sgs,
    #[cfg_attr(feature = "backend", graphql(description = "South Korea"))]
    #[strum(serialize = "South Korea")]
    Kor,
    #[cfg_attr(feature = "backend", graphql(description = "South Sudan"))]
    #[strum(serialize = "South Sudan")]
    Ssd,
    #[cfg_attr(feature = "backend", graphql(description = "Spain"))]
    #[strum(serialize = "Spain")]
    Esp,
    #[cfg_attr(feature = "backend", graphql(description = "Sri Lanka"))]
    #[strum(serialize = "Sri Lanka")]
    Lka,
    #[cfg_attr(feature = "backend", graphql(description = "Sudan"))]
    #[strum(serialize = "Sudan")]
    Sdn,
    #[cfg_attr(feature = "backend", graphql(description = "Suriname"))]
    #[strum(serialize = "Suriname")]
    Sur,
    #[cfg_attr(feature = "backend", graphql(description = "Svalbard and Jan Mayen"))]
    #[strum(serialize = "Svalbard and Jan Mayen")]
    Sjm,
    #[cfg_attr(feature = "backend", graphql(description = "Sweden"))]
    #[strum(serialize = "Sweden")]
    Swe,
    #[cfg_attr(feature = "backend", graphql(description = "Switzerland"))]
    #[strum(serialize = "Switzerland")]
    Che,
    #[cfg_attr(feature = "backend", graphql(description = "Syria"))]
    #[strum(serialize = "Syria")]
    Syr,
    #[cfg_attr(feature = "backend", graphql(description = "Taiwan"))]
    #[strum(serialize = "Taiwan")]
    Twn,
    #[cfg_attr(feature = "backend", graphql(description = "Tajikistan"))]
    #[strum(serialize = "Tajikistan")]
    Tjk,
    #[cfg_attr(feature = "backend", graphql(description = "Tanzania"))]
    #[strum(serialize = "Tanzania")]
    Tza,
    #[cfg_attr(feature = "backend", graphql(description = "Thailand"))]
    #[strum(serialize = "Thailand")]
    Tha,
    #[cfg_attr(feature = "backend", graphql(description = "Timor-Leste"))]
    #[strum(serialize = "Timor-Leste")]
    Tls,
    #[cfg_attr(feature = "backend", graphql(description = "Togo"))]
    #[strum(serialize = "Togo")]
    Tgo,
    #[cfg_attr(feature = "backend", graphql(description = "Tokelau"))]
    #[strum(serialize = "Tokelau")]
    Tkl,
    #[cfg_attr(feature = "backend", graphql(description = "Tonga"))]
    #[strum(serialize = "Tonga")]
    Ton,
    #[cfg_attr(feature = "backend", graphql(description = "Trinidad and Tobago"))]
    #[strum(serialize = "Trinidad and Tobago")]
    Tto,
    #[cfg_attr(feature = "backend", graphql(description = "Tunisia"))]
    #[strum(serialize = "Tunisia")]
    Tun,
    #[cfg_attr(feature = "backend", graphql(description = "Turkey"))]
    #[strum(serialize = "Turkey")]
    Tur,
    #[cfg_attr(feature = "backend", graphql(description = "Turkmenistan"))]
    #[strum(serialize = "Turkmenistan")]
    Tkm,
    #[cfg_attr(feature = "backend", graphql(description = "Turks and Caicos Islands"))]
    #[strum(serialize = "Turks and Caicos Islands")]
    Tca,
    #[cfg_attr(feature = "backend", graphql(description = "Tuvalu"))]
    #[strum(serialize = "Tuvalu")]
    Tuv,
    #[cfg_attr(feature = "backend", graphql(description = "Uganda"))]
    #[strum(serialize = "Uganda")]
    Uga,
    #[cfg_attr(feature = "backend", graphql(description = "Ukraine"))]
    #[strum(serialize = "Ukraine")]
    Ukr,
    #[cfg_attr(feature = "backend", graphql(description = "United Arab Emirates"))]
    #[strum(serialize = "United Arab Emirates")]
    Are,
    #[cfg_attr(feature = "backend", graphql(description = "United Kingdom"))]
    #[strum(serialize = "United Kingdom")]
    Gbr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "United States Minor Outlying Islands")
    )]
    #[strum(serialize = "United States Minor Outlying Islands")]
    Umi,
    #[cfg_attr(feature = "backend", graphql(description = "United States of America"))]
    #[strum(serialize = "United States of America")]
    Usa,
    #[cfg_attr(feature = "backend", graphql(description = "Uruguay"))]
    #[strum(serialize = "Uruguay")]
    Ury,
    #[cfg_attr(feature = "backend", graphql(description = "Uzbekistan"))]
    #[strum(serialize = "Uzbekistan")]
    Uzb,
    #[cfg_attr(feature = "backend", graphql(description = "Vanuatu"))]
    #[strum(serialize = "Vanuatu")]
    Vut,
    #[cfg_attr(feature = "backend", graphql(description = "Vatican City"))]
    #[strum(serialize = "Vatican City")]
    Vat,
    #[cfg_attr(feature = "backend", graphql(description = "Venezuela"))]
    #[strum(serialize = "Venezuela")]
    Ven,
    #[cfg_attr(feature = "backend", graphql(description = "Viet Nam"))]
    #[strum(serialize = "Viet Nam")]
    Vnm,
    #[cfg_attr(feature = "backend", graphql(description = "Virgin Islands (British)"))]
    #[strum(serialize = "Virgin Islands (British)")]
    Vgb,
    #[cfg_attr(feature = "backend", graphql(description = "Virgin Islands (U.S.)"))]
    #[strum(serialize = "Virgin Islands (U.S.)")]
    Vir,
    #[cfg_attr(feature = "backend", graphql(description = "Wallis and Futuna"))]
    #[strum(serialize = "Wallis and Futuna")]
    Wlf,
    #[cfg_attr(feature = "backend", graphql(description = "Western Sahara"))]
    #[strum(serialize = "Western Sahara")]
    Esh,
    #[cfg_attr(feature = "backend", graphql(description = "Yemen"))]
    #[strum(serialize = "Yemen")]
    Yem,
    #[cfg_attr(feature = "backend", graphql(description = "Zambia"))]
    #[strum(serialize = "Zambia")]
    Zmb,
    #[cfg_attr(feature = "backend", graphql(description = "Zimbabwe"))]
    #[strum(serialize = "Zimbabwe")]
    Zwe,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct InstitutionHistory {
    pub institution_history_id: Uuid,
    pub institution_id: Uuid,
    pub user_id: String,
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
    pub user_id: String,
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

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::InstitutionPolicy;
#[cfg(test)]
mod tests;
