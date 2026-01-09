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
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::InstitutionPolicy;
