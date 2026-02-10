use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::price;
#[cfg(feature = "backend")]
use crate::schema::price_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting prices list")
)]
pub enum PriceField {
    PriceId,
    PublicationId,
    CurrencyCode,
    UnitPrice,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price_id: Uuid,
    pub publication_id: Uuid,
    pub currency_code: CurrencyCode,
    pub unit_price: f64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new amount of money that a publication costs"),
    diesel(table_name = price)
)]
pub struct NewPrice {
    pub publication_id: Uuid,
    pub currency_code: CurrencyCode,
    pub unit_price: f64,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing amount of money that a publication costs"),
    diesel(table_name = price, treat_none_as_null = true)
)]
pub struct PatchPrice {
    pub price_id: Uuid,
    pub publication_id: Uuid,
    pub currency_code: CurrencyCode,
    pub unit_price: f64,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Three-letter ISO 4217 code representing a currency"),
    ExistingTypePath = "crate::schema::sql_types::CurrencyCode"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum CurrencyCode {
    #[cfg_attr(feature = "backend", graphql(description = "Andorran peseta"))]
    Adp,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "United Arab Emirates dirham")
    )]
    Aed,
    #[cfg_attr(feature = "backend", graphql(description = "Afghan afghani (first)"))]
    Afa,
    #[cfg_attr(feature = "backend", graphql(description = "Afghan afghani"))]
    Afn,
    #[cfg_attr(feature = "backend", graphql(description = "Old Albanian lek"))]
    Alk,
    #[cfg_attr(feature = "backend", graphql(description = "Albanian lek"))]
    All,
    #[cfg_attr(feature = "backend", graphql(description = "Armenian dram"))]
    Amd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Netherlands Antillean guilder")
    )]
    Ang,
    #[cfg_attr(feature = "backend", graphql(description = "Angolan kwanza"))]
    Aoa,
    #[cfg_attr(feature = "backend", graphql(description = "Angolan kwanza (first)"))]
    Aok,
    #[cfg_attr(feature = "backend", graphql(description = "Angolan novo kwanza"))]
    Aon,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Angolan kwanza reajustado")
    )]
    Aor,
    #[cfg_attr(feature = "backend", graphql(description = "Argentine austral"))]
    Ara,
    #[cfg_attr(feature = "backend", graphql(description = "Argentine peso argentino"))]
    Arp,
    #[cfg_attr(feature = "backend", graphql(description = "Argentine peso"))]
    Ars,
    #[cfg_attr(feature = "backend", graphql(description = "Argentine peso ley"))]
    Ary,
    #[cfg_attr(feature = "backend", graphql(description = "Austrian schilling"))]
    Ats,
    #[cfg_attr(feature = "backend", graphql(description = "Australian dollar"))]
    Aud,
    #[cfg_attr(feature = "backend", graphql(description = "Aruban florin"))]
    Awg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Azerbaijani manat (first)")
    )]
    Aym,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Azerbaijani manat (second)")
    )]
    Azm,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijani manat"))]
    Azn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Bosnia and Herzegovina dinar")
    )]
    Bad,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Bosnia and Herzegovina convertible mark")
    )]
    Bam,
    #[cfg_attr(feature = "backend", graphql(description = "Barbados dollar"))]
    Bbd,
    #[cfg_attr(feature = "backend", graphql(description = "Bangladeshi taka"))]
    Bdt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Belgian convertible franc")
    )]
    Bec,
    #[cfg_attr(feature = "backend", graphql(description = "Belgian franc"))]
    Bef,
    #[cfg_attr(feature = "backend", graphql(description = "Belgian financial franc"))]
    Bel,
    #[cfg_attr(feature = "backend", graphql(description = "Bulgarian lev (first)"))]
    Bgj,
    #[cfg_attr(feature = "backend", graphql(description = "Bulgarian lev (second)"))]
    Bgk,
    #[cfg_attr(feature = "backend", graphql(description = "Bulgarian lev (third)"))]
    Bgl,
    #[cfg_attr(feature = "backend", graphql(description = "Bulgarian lev"))]
    Bgn,
    #[cfg_attr(feature = "backend", graphql(description = "Bahraini dinar"))]
    Bhd,
    #[cfg_attr(feature = "backend", graphql(description = "Burundian franc"))]
    Bif,
    #[cfg_attr(feature = "backend", graphql(description = "Bermudian dollar"))]
    Bmd,
    #[cfg_attr(feature = "backend", graphql(description = "Brunei dollar"))]
    Bnd,
    #[cfg_attr(feature = "backend", graphql(description = "Boliviano"))]
    Bob,
    #[cfg_attr(feature = "backend", graphql(description = "Bolivian peso"))]
    Bop,
    #[cfg_attr(feature = "backend", graphql(description = "Bolivian Mvdol"))]
    Bov,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Brazilian cruzeiro (1967-1986)")
    )]
    Brb,
    #[cfg_attr(feature = "backend", graphql(description = "Brazilian cruzado"))]
    Brc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Brazilian cruzeiro (1990–1993)")
    )]
    Bre,
    #[cfg_attr(feature = "backend", graphql(description = "Brazilian real"))]
    Brl,
    #[cfg_attr(feature = "backend", graphql(description = "Brazilian cruzado novo"))]
    Brn,
    #[cfg_attr(feature = "backend", graphql(description = "Brazilian cruzeiro real"))]
    Brr,
    #[cfg_attr(feature = "backend", graphql(description = "Bahamian dollar"))]
    Bsd,
    #[cfg_attr(feature = "backend", graphql(description = "Bhutanese ngultrum"))]
    Btn,
    #[cfg_attr(feature = "backend", graphql(description = "Burmese kyat"))]
    Buk,
    #[cfg_attr(feature = "backend", graphql(description = "Botswana pula"))]
    Bwp,
    #[cfg_attr(feature = "backend", graphql(description = "Belarusian ruble (first)"))]
    Byb,
    #[cfg_attr(feature = "backend", graphql(description = "Belarusian ruble"))]
    Byn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Belarusian ruble (second)")
    )]
    Byr,
    #[cfg_attr(feature = "backend", graphql(description = "Belize dollar"))]
    Bzd,
    #[cfg_attr(feature = "backend", graphql(description = "Canadian dollar"))]
    Cad,
    #[cfg_attr(feature = "backend", graphql(description = "Congolese franc"))]
    Cdf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "WIR franc (for electronic currency)")
    )]
    Chc,
    #[cfg_attr(feature = "backend", graphql(description = "WIR euro"))]
    Che,
    #[cfg_attr(feature = "backend", graphql(description = "Swiss franc"))]
    Chf,
    #[cfg_attr(feature = "backend", graphql(description = "WIR franc"))]
    Chw,
    #[cfg_attr(feature = "backend", graphql(description = "Unidad de Fomento"))]
    Clf,
    #[cfg_attr(feature = "backend", graphql(description = "Chilean peso"))]
    Clp,
    #[cfg_attr(feature = "backend", graphql(description = "Renminbi"))]
    Cny,
    #[cfg_attr(feature = "backend", graphql(description = "Colombian peso"))]
    Cop,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Unidad de Valor Real (UVR)")
    )]
    Cou,
    #[cfg_attr(feature = "backend", graphql(description = "Costa Rican colon"))]
    Crc,
    #[cfg_attr(feature = "backend", graphql(description = "Serbian dinar"))]
    Csd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Czechoslovak koruna (second)")
    )]
    Csj,
    #[cfg_attr(feature = "backend", graphql(description = "Czechoslovak koruna"))]
    Csk,
    #[cfg_attr(feature = "backend", graphql(description = "Cuban convertible peso"))]
    Cuc,
    #[cfg_attr(feature = "backend", graphql(description = "Cuban peso"))]
    Cup,
    #[cfg_attr(feature = "backend", graphql(description = "Cape Verdean escudo"))]
    Cve,
    #[cfg_attr(feature = "backend", graphql(description = "Cypriot pound"))]
    Cyp,
    #[cfg_attr(feature = "backend", graphql(description = "Czech koruna"))]
    Czk,
    #[cfg_attr(feature = "backend", graphql(description = "East German mark"))]
    Ddm,
    #[cfg_attr(feature = "backend", graphql(description = "German mark"))]
    Dem,
    #[cfg_attr(feature = "backend", graphql(description = "Djiboutian franc"))]
    Djf,
    #[cfg_attr(feature = "backend", graphql(description = "Danish krone"))]
    Dkk,
    #[cfg_attr(feature = "backend", graphql(description = "Dominican peso"))]
    Dop,
    #[cfg_attr(feature = "backend", graphql(description = "Algerian dinar"))]
    Dzd,
    #[cfg_attr(feature = "backend", graphql(description = "Ecuadorian sucre"))]
    Ecs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Ecuador Unidad de Valor Constante")
    )]
    Ecv,
    #[cfg_attr(feature = "backend", graphql(description = "Estonian kroon"))]
    Eek,
    #[cfg_attr(feature = "backend", graphql(description = "Egyptian pound"))]
    Egp,
    #[cfg_attr(feature = "backend", graphql(description = "Eritrean nakfa"))]
    Ern,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish peseta (account A)")
    )]
    Esa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish peseta (account B)")
    )]
    Esb,
    #[cfg_attr(feature = "backend", graphql(description = "Spanish peseta"))]
    Esp,
    #[cfg_attr(feature = "backend", graphql(description = "Ethiopian birr"))]
    Etb,
    #[cfg_attr(feature = "backend", graphql(description = "Euro"))]
    Eur,
    #[cfg_attr(feature = "backend", graphql(description = "Finnish markka"))]
    Fim,
    #[cfg_attr(feature = "backend", graphql(description = "Fiji dollar"))]
    Fjd,
    #[cfg_attr(feature = "backend", graphql(description = "Falkland Islands pound"))]
    Fkp,
    #[cfg_attr(feature = "backend", graphql(description = "French franc"))]
    Frf,
    #[default]
    #[cfg_attr(feature = "backend", graphql(description = "Pound sterling"))]
    Gbp,
    #[cfg_attr(feature = "backend", graphql(description = "Georgian kuponi"))]
    Gek,
    #[cfg_attr(feature = "backend", graphql(description = "Georgian lari"))]
    Gel,
    #[cfg_attr(feature = "backend", graphql(description = "Ghanaian cedi (second)"))]
    Ghc,
    #[cfg_attr(feature = "backend", graphql(description = "Ghanaian cedi (first)"))]
    Ghp,
    #[cfg_attr(feature = "backend", graphql(description = "Ghanaian cedi"))]
    Ghs,
    #[cfg_attr(feature = "backend", graphql(description = "Gibraltar pound"))]
    Gip,
    #[cfg_attr(feature = "backend", graphql(description = "Gambian dalasi"))]
    Gmd,
    #[cfg_attr(feature = "backend", graphql(description = "Guinean syli (first)"))]
    Gne,
    #[cfg_attr(feature = "backend", graphql(description = "Guinean franc"))]
    Gnf,
    #[cfg_attr(feature = "backend", graphql(description = "Guinean syli (second)"))]
    Gns,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Equatorial Guinean ekwele")
    )]
    Gqe,
    #[cfg_attr(feature = "backend", graphql(description = "Greek drachma"))]
    Grd,
    #[cfg_attr(feature = "backend", graphql(description = "Guatemalan quetzal"))]
    Gtq,
    #[cfg_attr(feature = "backend", graphql(description = "Guinean escudo"))]
    Gwe,
    #[cfg_attr(feature = "backend", graphql(description = "Guinea-Bissau peso"))]
    Gwp,
    #[cfg_attr(feature = "backend", graphql(description = "Guyanese dollar"))]
    Gyd,
    #[cfg_attr(feature = "backend", graphql(description = "Hong Kong dollar"))]
    Hkd,
    #[cfg_attr(feature = "backend", graphql(description = "Honduran lempira"))]
    Hnl,
    #[cfg_attr(feature = "backend", graphql(description = "Croatian dinar"))]
    Hrd,
    #[cfg_attr(feature = "backend", graphql(description = "Croatian kuna"))]
    Hrk,
    #[cfg_attr(feature = "backend", graphql(description = "Haitian gourde"))]
    Htg,
    #[cfg_attr(feature = "backend", graphql(description = "Hungarian forint"))]
    Huf,
    #[cfg_attr(feature = "backend", graphql(description = "Indonesian rupiah"))]
    Idr,
    #[cfg_attr(feature = "backend", graphql(description = "Irish pound"))]
    Iep,
    #[cfg_attr(feature = "backend", graphql(description = "Israeli pound"))]
    Ilp,
    #[cfg_attr(feature = "backend", graphql(description = "Israeli shekel"))]
    Ilr,
    #[cfg_attr(feature = "backend", graphql(description = "Israeli new shekel"))]
    Ils,
    #[cfg_attr(feature = "backend", graphql(description = "Indian rupee"))]
    Inr,
    #[cfg_attr(feature = "backend", graphql(description = "Iraqi dinar"))]
    Iqd,
    #[cfg_attr(feature = "backend", graphql(description = "Iranian rial"))]
    Irr,
    #[cfg_attr(feature = "backend", graphql(description = "Icelandic króna (first)"))]
    Isj,
    #[cfg_attr(feature = "backend", graphql(description = "Icelandic króna"))]
    Isk,
    #[cfg_attr(feature = "backend", graphql(description = "Italian lira"))]
    Itl,
    #[cfg_attr(feature = "backend", graphql(description = "Jamaican dollar"))]
    Jmd,
    #[cfg_attr(feature = "backend", graphql(description = "Jordanian dinar"))]
    Jod,
    #[cfg_attr(feature = "backend", graphql(description = "Japanese yen"))]
    Jpy,
    #[cfg_attr(feature = "backend", graphql(description = "Kenyan shilling"))]
    Kes,
    #[cfg_attr(feature = "backend", graphql(description = "Kyrgyzstani som"))]
    Kgs,
    #[cfg_attr(feature = "backend", graphql(description = "Cambodian riel"))]
    Khr,
    #[cfg_attr(feature = "backend", graphql(description = "Comoro franc"))]
    Kmf,
    #[cfg_attr(feature = "backend", graphql(description = "North Korean won"))]
    Kpw,
    #[cfg_attr(feature = "backend", graphql(description = "South Korean won"))]
    Krw,
    #[cfg_attr(feature = "backend", graphql(description = "Kuwaiti dinar"))]
    Kwd,
    #[cfg_attr(feature = "backend", graphql(description = "Cayman Islands dollar"))]
    Kyd,
    #[cfg_attr(feature = "backend", graphql(description = "Kazakhstani tenge"))]
    Kzt,
    #[cfg_attr(feature = "backend", graphql(description = "Pathet Lao kip"))]
    Laj,
    #[cfg_attr(feature = "backend", graphql(description = "Lao kip"))]
    Lak,
    #[cfg_attr(feature = "backend", graphql(description = "Lebanese pound"))]
    Lbp,
    #[cfg_attr(feature = "backend", graphql(description = "Sri Lankan rupee"))]
    Lkr,
    #[cfg_attr(feature = "backend", graphql(description = "Liberian dollar"))]
    Lrd,
    #[cfg_attr(feature = "backend", graphql(description = "Lesotho loti"))]
    Lsl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Lesotho loti (historic code)")
    )]
    Lsm,
    #[cfg_attr(feature = "backend", graphql(description = "Lithuanian litas"))]
    Ltl,
    #[cfg_attr(feature = "backend", graphql(description = "Lithuanian talonas"))]
    Ltt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Luxembourg convertible franc")
    )]
    Luc,
    #[cfg_attr(feature = "backend", graphql(description = "Luxembourg franc"))]
    Luf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Luxembourg financial franc")
    )]
    Lul,
    #[cfg_attr(feature = "backend", graphql(description = "Latvian lats"))]
    Lvl,
    #[cfg_attr(feature = "backend", graphql(description = "Latvian rublis"))]
    Lvr,
    #[cfg_attr(feature = "backend", graphql(description = "Libyan dinar"))]
    Lyd,
    #[cfg_attr(feature = "backend", graphql(description = "Moroccan dirham"))]
    Mad,
    #[cfg_attr(feature = "backend", graphql(description = "Moldovan leu"))]
    Mdl,
    #[cfg_attr(feature = "backend", graphql(description = "Malagasy ariary"))]
    Mga,
    #[cfg_attr(feature = "backend", graphql(description = "Malagasy franc"))]
    Mgf,
    #[cfg_attr(feature = "backend", graphql(description = "Macedonian denar"))]
    Mkd,
    #[cfg_attr(feature = "backend", graphql(description = "Malian franc"))]
    Mlf,
    #[cfg_attr(feature = "backend", graphql(description = "Myanmar kyat"))]
    Mmk,
    #[cfg_attr(feature = "backend", graphql(description = "Mongolian tögrög"))]
    Mnt,
    #[cfg_attr(feature = "backend", graphql(description = "Macanese pataca"))]
    Mop,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Mauritanian ouguiya (first)")
    )]
    Mro,
    #[cfg_attr(feature = "backend", graphql(description = "Mauritanian ouguiya"))]
    Mru,
    #[cfg_attr(feature = "backend", graphql(description = "Maltese lira"))]
    Mtl,
    #[cfg_attr(feature = "backend", graphql(description = "Maltese pound"))]
    Mtp,
    #[cfg_attr(feature = "backend", graphql(description = "Mauritian rupee"))]
    Mur,
    #[cfg_attr(feature = "backend", graphql(description = "Maldivian rupee"))]
    Mvq,
    #[cfg_attr(feature = "backend", graphql(description = "Maldivian rufiyaa"))]
    Mvr,
    #[cfg_attr(feature = "backend", graphql(description = "Malawian kwacha"))]
    Mwk,
    #[cfg_attr(feature = "backend", graphql(description = "Mexican peso"))]
    Mxn,
    #[cfg_attr(feature = "backend", graphql(description = "Mexican peso (first)"))]
    Mxp,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Mexican Unidad de Inversion (UDI)")
    )]
    Mxv,
    #[cfg_attr(feature = "backend", graphql(description = "Malaysian ringgit"))]
    Myr,
    #[cfg_attr(feature = "backend", graphql(description = "Mozambican escudo"))]
    Mze,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Mozambican metical (first)")
    )]
    Mzm,
    #[cfg_attr(feature = "backend", graphql(description = "Mozambican metical"))]
    Mzn,
    #[cfg_attr(feature = "backend", graphql(description = "Namibian dollar"))]
    Nad,
    #[cfg_attr(feature = "backend", graphql(description = "Nigerian naira"))]
    Ngn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Nicaraguan córdoba (second)")
    )]
    Nic,
    #[cfg_attr(feature = "backend", graphql(description = "Nicaraguan córdoba"))]
    Nio,
    #[cfg_attr(feature = "backend", graphql(description = "Dutch guilder"))]
    Nlg,
    #[cfg_attr(feature = "backend", graphql(description = "Norwegian krone"))]
    Nok,
    #[cfg_attr(feature = "backend", graphql(description = "Nepalese rupee"))]
    Npr,
    #[cfg_attr(feature = "backend", graphql(description = "New Zealand dollar"))]
    Nzd,
    #[cfg_attr(feature = "backend", graphql(description = "Omani rial"))]
    Omr,
    #[cfg_attr(feature = "backend", graphql(description = "Panamanian balboa"))]
    Pab,
    #[cfg_attr(feature = "backend", graphql(description = "Peruvian old sol"))]
    Peh,
    #[cfg_attr(feature = "backend", graphql(description = "Peruvian inti"))]
    Pei,
    #[cfg_attr(feature = "backend", graphql(description = "Peruvian sol"))]
    Pen,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Peruvian sol (historic code)")
    )]
    Pes,
    #[cfg_attr(feature = "backend", graphql(description = "Papua New Guinean kina"))]
    Pgk,
    #[cfg_attr(feature = "backend", graphql(description = "Philippine peso"))]
    Php,
    #[cfg_attr(feature = "backend", graphql(description = "Pakistani rupee"))]
    Pkr,
    #[cfg_attr(feature = "backend", graphql(description = "Polish złoty"))]
    Pln,
    #[cfg_attr(feature = "backend", graphql(description = "Polish złoty (third)"))]
    Plz,
    #[cfg_attr(feature = "backend", graphql(description = "Portuguese escudo"))]
    Pte,
    #[cfg_attr(feature = "backend", graphql(description = "Paraguayan guaraní"))]
    Pyg,
    #[cfg_attr(feature = "backend", graphql(description = "Qatari riyal"))]
    Qar,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Rhodesian dollar (historic code)")
    )]
    Rhd,
    #[cfg_attr(feature = "backend", graphql(description = "Romanian leu (second)"))]
    Rok,
    #[cfg_attr(feature = "backend", graphql(description = "Romanian leu (third)"))]
    Rol,
    #[cfg_attr(feature = "backend", graphql(description = "Romanian leu"))]
    Ron,
    #[cfg_attr(feature = "backend", graphql(description = "Serbian dinar"))]
    Rsd,
    #[cfg_attr(feature = "backend", graphql(description = "Russian ruble"))]
    Rub,
    #[cfg_attr(feature = "backend", graphql(description = "Russian ruble (old)"))]
    Rur,
    #[cfg_attr(feature = "backend", graphql(description = "Rwandan franc"))]
    Rwf,
    #[cfg_attr(feature = "backend", graphql(description = "Saudi riyal"))]
    Sar,
    #[cfg_attr(feature = "backend", graphql(description = "Solomon Islands dollar"))]
    Sbd,
    #[cfg_attr(feature = "backend", graphql(description = "Seychelles rupee"))]
    Scr,
    #[cfg_attr(feature = "backend", graphql(description = "Sudanese dinar"))]
    Sdd,
    #[cfg_attr(feature = "backend", graphql(description = "Sudanese pound"))]
    Sdg,
    #[cfg_attr(feature = "backend", graphql(description = "Sudanese old pound"))]
    Sdp,
    #[cfg_attr(feature = "backend", graphql(description = "Swedish krona"))]
    Sek,
    #[cfg_attr(feature = "backend", graphql(description = "Singapore dollar"))]
    Sgd,
    #[cfg_attr(feature = "backend", graphql(description = "Saint Helena pound"))]
    Shp,
    #[cfg_attr(feature = "backend", graphql(description = "Slovenian tolar"))]
    Sit,
    #[cfg_attr(feature = "backend", graphql(description = "Slovak koruna"))]
    Skk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Sierra Leonean leone (old leone)")
    )]
    Sll,
    #[cfg_attr(feature = "backend", graphql(description = "Somalian shilling"))]
    Sos,
    #[cfg_attr(feature = "backend", graphql(description = "Surinamese dollar"))]
    Srd,
    #[cfg_attr(feature = "backend", graphql(description = "Surinamese guilder"))]
    Srg,
    #[cfg_attr(feature = "backend", graphql(description = "South Sudanese pound"))]
    Ssp,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "São Tomé and Príncipe dobra (first)")
    )]
    Std,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "São Tomé and Príncipe dobra")
    )]
    Stn,
    #[cfg_attr(feature = "backend", graphql(description = "Soviet Union ruble"))]
    Sur,
    #[cfg_attr(feature = "backend", graphql(description = "Salvadoran colón"))]
    Svc,
    #[cfg_attr(feature = "backend", graphql(description = "Syrian pound"))]
    Syp,
    #[cfg_attr(feature = "backend", graphql(description = "Swazi lilangeni"))]
    Szl,
    #[cfg_attr(feature = "backend", graphql(description = "Thai baht"))]
    Thb,
    #[cfg_attr(feature = "backend", graphql(description = "Tajikistani ruble"))]
    Tjr,
    #[cfg_attr(feature = "backend", graphql(description = "Tajikistani somoni"))]
    Tjs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Turkmenistani manat (first)")
    )]
    Tmm,
    #[cfg_attr(feature = "backend", graphql(description = "Turkmenistani manat"))]
    Tmt,
    #[cfg_attr(feature = "backend", graphql(description = "Tunisian dinar"))]
    Tnd,
    #[cfg_attr(feature = "backend", graphql(description = "Tongan paʻanga"))]
    Top,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Portuguese Timorese escudo")
    )]
    Tpe,
    #[cfg_attr(feature = "backend", graphql(description = "Turkish lira (first)"))]
    Trl,
    #[cfg_attr(feature = "backend", graphql(description = "Turkish lira"))]
    Try,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Trinidad and Tobago dollar")
    )]
    Ttd,
    #[cfg_attr(feature = "backend", graphql(description = "New Taiwan dollar"))]
    Twd,
    #[cfg_attr(feature = "backend", graphql(description = "Tanzanian shilling"))]
    Tzs,
    #[cfg_attr(feature = "backend", graphql(description = "Ukrainian hryvnia"))]
    Uah,
    #[cfg_attr(feature = "backend", graphql(description = "Ukrainian karbovanets"))]
    Uak,
    #[cfg_attr(feature = "backend", graphql(description = "Ugandan shilling"))]
    Ugs,
    #[cfg_attr(feature = "backend", graphql(description = "Old Shilling"))]
    Ugw,
    #[cfg_attr(feature = "backend", graphql(description = "Ugandan shilling"))]
    Ugx,
    #[cfg_attr(feature = "backend", graphql(description = "United States dollar"))]
    Usd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "United States dollar (next day)")
    )]
    Usn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "United States dollar (same day)")
    )]
    Uss,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Uruguay Peso en Unidades Indexadas (URUIURUI)")
    )]
    Uyi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Uruguayan peso (gold standard)")
    )]
    Uyn,
    #[cfg_attr(feature = "backend", graphql(description = "Uruguayan nuevo peso"))]
    Uyp,
    #[cfg_attr(feature = "backend", graphql(description = "Uruguayan peso"))]
    Uyu,
    #[cfg_attr(feature = "backend", graphql(description = "Unidad previsional"))]
    Uyw,
    #[cfg_attr(feature = "backend", graphql(description = "Uzbekistani sum"))]
    Uzs,
    #[cfg_attr(feature = "backend", graphql(description = "Venezuelan bolívar"))]
    Veb,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Venezuelan bolívar fuerte")
    )]
    Vef,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Venezuelan sovereign bolívar")
    )]
    Ves,
    #[cfg_attr(feature = "backend", graphql(description = "Old Vietnamese dong"))]
    Vnc,
    #[cfg_attr(feature = "backend", graphql(description = "Vietnamese đồng"))]
    Vnd,
    #[cfg_attr(feature = "backend", graphql(description = "Vanuatu vatu"))]
    Vuv,
    #[cfg_attr(feature = "backend", graphql(description = "Samoan tala"))]
    Wst,
    #[cfg_attr(feature = "backend", graphql(description = "CFA franc BEAC"))]
    Xaf,
    #[cfg_attr(feature = "backend", graphql(description = "Silver"))]
    Xag,
    #[cfg_attr(feature = "backend", graphql(description = "Gold"))]
    Xau,
    #[cfg_attr(feature = "backend", graphql(description = "European Composite Unit"))]
    Xba,
    #[cfg_attr(feature = "backend", graphql(description = "European Monetary Unit"))]
    Xbb,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "European Unit of Account 9")
    )]
    Xbc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "European Unit of Account 17")
    )]
    Xbd,
    #[cfg_attr(feature = "backend", graphql(description = "East Caribbean dollar"))]
    Xcd,
    #[cfg_attr(feature = "backend", graphql(description = "Special drawing rights"))]
    Xdr,
    #[cfg_attr(feature = "backend", graphql(description = "European Currency Unit"))]
    Xeu,
    #[cfg_attr(feature = "backend", graphql(description = "Gold franc"))]
    Xfo,
    #[cfg_attr(feature = "backend", graphql(description = "UIC franc"))]
    Xfu,
    #[cfg_attr(feature = "backend", graphql(description = "CFA franc BCEAO"))]
    Xof,
    #[cfg_attr(feature = "backend", graphql(description = "Palladium"))]
    Xpd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "CFP franc (franc Pacifique)")
    )]
    Xpf,
    #[cfg_attr(feature = "backend", graphql(description = "Platinum"))]
    Xpt,
    #[cfg_attr(feature = "backend", graphql(description = "RINET funds code"))]
    Xre,
    #[cfg_attr(feature = "backend", graphql(description = "SUCRE"))]
    Xsu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Code reserved for testing")
    )]
    Xts,
    #[cfg_attr(feature = "backend", graphql(description = "ADB Unit of Account"))]
    Xua,
    #[cfg_attr(feature = "backend", graphql(description = "No currency"))]
    Xxx,
    #[cfg_attr(feature = "backend", graphql(description = "South Yemeni dinar"))]
    Ydd,
    #[cfg_attr(feature = "backend", graphql(description = "Yemeni rial"))]
    Yer,
    #[cfg_attr(feature = "backend", graphql(description = "Yugoslav dinar (hard)"))]
    Yud,
    #[cfg_attr(feature = "backend", graphql(description = "Yugoslav dinar (Novi)"))]
    Yum,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Yugoslav dinar (convertible)")
    )]
    Yun,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "South African financial rand")
    )]
    Zal,
    #[cfg_attr(feature = "backend", graphql(description = "South African rand"))]
    Zar,
    #[cfg_attr(feature = "backend", graphql(description = "Zambian kwacha"))]
    Zmk,
    #[cfg_attr(feature = "backend", graphql(description = "Zambian new kwacha"))]
    Zmw,
    #[cfg_attr(feature = "backend", graphql(description = "Zairean new zaire"))]
    Zrn,
    #[cfg_attr(feature = "backend", graphql(description = "Zairean zaire"))]
    Zrz,
    #[cfg_attr(feature = "backend", graphql(description = "Rhodesian dollar"))]
    Zwc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Zimbabwean dollar (first)")
    )]
    Zwd,
    #[cfg_attr(feature = "backend", graphql(description = "Zimbabwean dollar"))]
    Zwl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Zimbabwean dollar (second)")
    )]
    Zwn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Zimbabwean dollar (third)")
    )]
    Zwr,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct PriceHistory {
    pub price_history_id: Uuid,
    pub price_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(feature = "backend", derive(diesel::Insertable), diesel(table_name = price_history))]
pub struct NewPriceHistory {
    pub price_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::PricePolicy;
#[cfg(test)]
mod tests;
