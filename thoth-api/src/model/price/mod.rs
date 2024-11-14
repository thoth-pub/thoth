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

#[cfg_attr(feature = "backend", derive(Queryable))]
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
    derive(juniper::GraphQLInputObject, Insertable),
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
    derive(juniper::GraphQLInputObject, AsChangeset),
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
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Three-letter ISO 4217 code representing a currency"),
    ExistingTypePath = "crate::schema::sql_types::CurrencyCode"
)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
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

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct PriceHistory {
    pub price_history_id: Uuid,
    pub price_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(feature = "backend", derive(Insertable), diesel(table_name = price_history))]
pub struct NewPriceHistory {
    pub price_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[test]
fn test_currencycode_default() {
    let currencycode: CurrencyCode = Default::default();
    assert_eq!(currencycode, CurrencyCode::Gbp);
}

#[test]
fn test_currencycode_display() {
    assert_eq!(format!("{}", CurrencyCode::Adp), "ADP");
    assert_eq!(format!("{}", CurrencyCode::Aed), "AED");
    assert_eq!(format!("{}", CurrencyCode::Afa), "AFA");
    assert_eq!(format!("{}", CurrencyCode::Afn), "AFN");
    assert_eq!(format!("{}", CurrencyCode::Alk), "ALK");
    assert_eq!(format!("{}", CurrencyCode::All), "ALL");
    assert_eq!(format!("{}", CurrencyCode::Amd), "AMD");
    assert_eq!(format!("{}", CurrencyCode::Ang), "ANG");
    assert_eq!(format!("{}", CurrencyCode::Aoa), "AOA");
    assert_eq!(format!("{}", CurrencyCode::Aok), "AOK");
    assert_eq!(format!("{}", CurrencyCode::Aon), "AON");
    assert_eq!(format!("{}", CurrencyCode::Aor), "AOR");
    assert_eq!(format!("{}", CurrencyCode::Ara), "ARA");
    assert_eq!(format!("{}", CurrencyCode::Arp), "ARP");
    assert_eq!(format!("{}", CurrencyCode::Ars), "ARS");
    assert_eq!(format!("{}", CurrencyCode::Ary), "ARY");
    assert_eq!(format!("{}", CurrencyCode::Ats), "ATS");
    assert_eq!(format!("{}", CurrencyCode::Aud), "AUD");
    assert_eq!(format!("{}", CurrencyCode::Awg), "AWG");
    assert_eq!(format!("{}", CurrencyCode::Aym), "AYM");
    assert_eq!(format!("{}", CurrencyCode::Azm), "AZM");
    assert_eq!(format!("{}", CurrencyCode::Azn), "AZN");
    assert_eq!(format!("{}", CurrencyCode::Bad), "BAD");
    assert_eq!(format!("{}", CurrencyCode::Bam), "BAM");
    assert_eq!(format!("{}", CurrencyCode::Bbd), "BBD");
    assert_eq!(format!("{}", CurrencyCode::Bdt), "BDT");
    assert_eq!(format!("{}", CurrencyCode::Bec), "BEC");
    assert_eq!(format!("{}", CurrencyCode::Bef), "BEF");
    assert_eq!(format!("{}", CurrencyCode::Bel), "BEL");
    assert_eq!(format!("{}", CurrencyCode::Bgj), "BGJ");
    assert_eq!(format!("{}", CurrencyCode::Bgk), "BGK");
    assert_eq!(format!("{}", CurrencyCode::Bgl), "BGL");
    assert_eq!(format!("{}", CurrencyCode::Bgn), "BGN");
    assert_eq!(format!("{}", CurrencyCode::Bhd), "BHD");
    assert_eq!(format!("{}", CurrencyCode::Bif), "BIF");
    assert_eq!(format!("{}", CurrencyCode::Bmd), "BMD");
    assert_eq!(format!("{}", CurrencyCode::Bnd), "BND");
    assert_eq!(format!("{}", CurrencyCode::Bob), "BOB");
    assert_eq!(format!("{}", CurrencyCode::Bop), "BOP");
    assert_eq!(format!("{}", CurrencyCode::Bov), "BOV");
    assert_eq!(format!("{}", CurrencyCode::Brb), "BRB");
    assert_eq!(format!("{}", CurrencyCode::Brc), "BRC");
    assert_eq!(format!("{}", CurrencyCode::Bre), "BRE");
    assert_eq!(format!("{}", CurrencyCode::Brl), "BRL");
    assert_eq!(format!("{}", CurrencyCode::Brn), "BRN");
    assert_eq!(format!("{}", CurrencyCode::Brr), "BRR");
    assert_eq!(format!("{}", CurrencyCode::Bsd), "BSD");
    assert_eq!(format!("{}", CurrencyCode::Btn), "BTN");
    assert_eq!(format!("{}", CurrencyCode::Buk), "BUK");
    assert_eq!(format!("{}", CurrencyCode::Bwp), "BWP");
    assert_eq!(format!("{}", CurrencyCode::Byb), "BYB");
    assert_eq!(format!("{}", CurrencyCode::Byn), "BYN");
    assert_eq!(format!("{}", CurrencyCode::Byr), "BYR");
    assert_eq!(format!("{}", CurrencyCode::Bzd), "BZD");
    assert_eq!(format!("{}", CurrencyCode::Cad), "CAD");
    assert_eq!(format!("{}", CurrencyCode::Cdf), "CDF");
    assert_eq!(format!("{}", CurrencyCode::Chc), "CHC");
    assert_eq!(format!("{}", CurrencyCode::Che), "CHE");
    assert_eq!(format!("{}", CurrencyCode::Chf), "CHF");
    assert_eq!(format!("{}", CurrencyCode::Chw), "CHW");
    assert_eq!(format!("{}", CurrencyCode::Clf), "CLF");
    assert_eq!(format!("{}", CurrencyCode::Clp), "CLP");
    assert_eq!(format!("{}", CurrencyCode::Cny), "CNY");
    assert_eq!(format!("{}", CurrencyCode::Cop), "COP");
    assert_eq!(format!("{}", CurrencyCode::Cou), "COU");
    assert_eq!(format!("{}", CurrencyCode::Crc), "CRC");
    assert_eq!(format!("{}", CurrencyCode::Csd), "CSD");
    assert_eq!(format!("{}", CurrencyCode::Csj), "CSJ");
    assert_eq!(format!("{}", CurrencyCode::Csk), "CSK");
    assert_eq!(format!("{}", CurrencyCode::Cuc), "CUC");
    assert_eq!(format!("{}", CurrencyCode::Cup), "CUP");
    assert_eq!(format!("{}", CurrencyCode::Cve), "CVE");
    assert_eq!(format!("{}", CurrencyCode::Cyp), "CYP");
    assert_eq!(format!("{}", CurrencyCode::Czk), "CZK");
    assert_eq!(format!("{}", CurrencyCode::Ddm), "DDM");
    assert_eq!(format!("{}", CurrencyCode::Dem), "DEM");
    assert_eq!(format!("{}", CurrencyCode::Djf), "DJF");
    assert_eq!(format!("{}", CurrencyCode::Dkk), "DKK");
    assert_eq!(format!("{}", CurrencyCode::Dop), "DOP");
    assert_eq!(format!("{}", CurrencyCode::Dzd), "DZD");
    assert_eq!(format!("{}", CurrencyCode::Ecs), "ECS");
    assert_eq!(format!("{}", CurrencyCode::Ecv), "ECV");
    assert_eq!(format!("{}", CurrencyCode::Eek), "EEK");
    assert_eq!(format!("{}", CurrencyCode::Egp), "EGP");
    assert_eq!(format!("{}", CurrencyCode::Ern), "ERN");
    assert_eq!(format!("{}", CurrencyCode::Esa), "ESA");
    assert_eq!(format!("{}", CurrencyCode::Esb), "ESB");
    assert_eq!(format!("{}", CurrencyCode::Esp), "ESP");
    assert_eq!(format!("{}", CurrencyCode::Etb), "ETB");
    assert_eq!(format!("{}", CurrencyCode::Eur), "EUR");
    assert_eq!(format!("{}", CurrencyCode::Fim), "FIM");
    assert_eq!(format!("{}", CurrencyCode::Fjd), "FJD");
    assert_eq!(format!("{}", CurrencyCode::Fkp), "FKP");
    assert_eq!(format!("{}", CurrencyCode::Frf), "FRF");
    assert_eq!(format!("{}", CurrencyCode::Gbp), "GBP");
    assert_eq!(format!("{}", CurrencyCode::Gek), "GEK");
    assert_eq!(format!("{}", CurrencyCode::Gel), "GEL");
    assert_eq!(format!("{}", CurrencyCode::Ghc), "GHC");
    assert_eq!(format!("{}", CurrencyCode::Ghp), "GHP");
    assert_eq!(format!("{}", CurrencyCode::Ghs), "GHS");
    assert_eq!(format!("{}", CurrencyCode::Gip), "GIP");
    assert_eq!(format!("{}", CurrencyCode::Gmd), "GMD");
    assert_eq!(format!("{}", CurrencyCode::Gne), "GNE");
    assert_eq!(format!("{}", CurrencyCode::Gnf), "GNF");
    assert_eq!(format!("{}", CurrencyCode::Gns), "GNS");
    assert_eq!(format!("{}", CurrencyCode::Gqe), "GQE");
    assert_eq!(format!("{}", CurrencyCode::Grd), "GRD");
    assert_eq!(format!("{}", CurrencyCode::Gtq), "GTQ");
    assert_eq!(format!("{}", CurrencyCode::Gwe), "GWE");
    assert_eq!(format!("{}", CurrencyCode::Gwp), "GWP");
    assert_eq!(format!("{}", CurrencyCode::Gyd), "GYD");
    assert_eq!(format!("{}", CurrencyCode::Hkd), "HKD");
    assert_eq!(format!("{}", CurrencyCode::Hnl), "HNL");
    assert_eq!(format!("{}", CurrencyCode::Hrd), "HRD");
    assert_eq!(format!("{}", CurrencyCode::Hrk), "HRK");
    assert_eq!(format!("{}", CurrencyCode::Htg), "HTG");
    assert_eq!(format!("{}", CurrencyCode::Huf), "HUF");
    assert_eq!(format!("{}", CurrencyCode::Idr), "IDR");
    assert_eq!(format!("{}", CurrencyCode::Iep), "IEP");
    assert_eq!(format!("{}", CurrencyCode::Ilp), "ILP");
    assert_eq!(format!("{}", CurrencyCode::Ilr), "ILR");
    assert_eq!(format!("{}", CurrencyCode::Ils), "ILS");
    assert_eq!(format!("{}", CurrencyCode::Inr), "INR");
    assert_eq!(format!("{}", CurrencyCode::Iqd), "IQD");
    assert_eq!(format!("{}", CurrencyCode::Irr), "IRR");
    assert_eq!(format!("{}", CurrencyCode::Isj), "ISJ");
    assert_eq!(format!("{}", CurrencyCode::Isk), "ISK");
    assert_eq!(format!("{}", CurrencyCode::Itl), "ITL");
    assert_eq!(format!("{}", CurrencyCode::Jmd), "JMD");
    assert_eq!(format!("{}", CurrencyCode::Jod), "JOD");
    assert_eq!(format!("{}", CurrencyCode::Jpy), "JPY");
    assert_eq!(format!("{}", CurrencyCode::Kes), "KES");
    assert_eq!(format!("{}", CurrencyCode::Kgs), "KGS");
    assert_eq!(format!("{}", CurrencyCode::Khr), "KHR");
    assert_eq!(format!("{}", CurrencyCode::Kmf), "KMF");
    assert_eq!(format!("{}", CurrencyCode::Kpw), "KPW");
    assert_eq!(format!("{}", CurrencyCode::Krw), "KRW");
    assert_eq!(format!("{}", CurrencyCode::Kwd), "KWD");
    assert_eq!(format!("{}", CurrencyCode::Kyd), "KYD");
    assert_eq!(format!("{}", CurrencyCode::Kzt), "KZT");
    assert_eq!(format!("{}", CurrencyCode::Laj), "LAJ");
    assert_eq!(format!("{}", CurrencyCode::Lak), "LAK");
    assert_eq!(format!("{}", CurrencyCode::Lbp), "LBP");
    assert_eq!(format!("{}", CurrencyCode::Lkr), "LKR");
    assert_eq!(format!("{}", CurrencyCode::Lrd), "LRD");
    assert_eq!(format!("{}", CurrencyCode::Lsl), "LSL");
    assert_eq!(format!("{}", CurrencyCode::Lsm), "LSM");
    assert_eq!(format!("{}", CurrencyCode::Ltl), "LTL");
    assert_eq!(format!("{}", CurrencyCode::Ltt), "LTT");
    assert_eq!(format!("{}", CurrencyCode::Luc), "LUC");
    assert_eq!(format!("{}", CurrencyCode::Luf), "LUF");
    assert_eq!(format!("{}", CurrencyCode::Lul), "LUL");
    assert_eq!(format!("{}", CurrencyCode::Lvl), "LVL");
    assert_eq!(format!("{}", CurrencyCode::Lvr), "LVR");
    assert_eq!(format!("{}", CurrencyCode::Lyd), "LYD");
    assert_eq!(format!("{}", CurrencyCode::Mad), "MAD");
    assert_eq!(format!("{}", CurrencyCode::Mdl), "MDL");
    assert_eq!(format!("{}", CurrencyCode::Mga), "MGA");
    assert_eq!(format!("{}", CurrencyCode::Mgf), "MGF");
    assert_eq!(format!("{}", CurrencyCode::Mkd), "MKD");
    assert_eq!(format!("{}", CurrencyCode::Mlf), "MLF");
    assert_eq!(format!("{}", CurrencyCode::Mmk), "MMK");
    assert_eq!(format!("{}", CurrencyCode::Mnt), "MNT");
    assert_eq!(format!("{}", CurrencyCode::Mop), "MOP");
    assert_eq!(format!("{}", CurrencyCode::Mro), "MRO");
    assert_eq!(format!("{}", CurrencyCode::Mru), "MRU");
    assert_eq!(format!("{}", CurrencyCode::Mtl), "MTL");
    assert_eq!(format!("{}", CurrencyCode::Mtp), "MTP");
    assert_eq!(format!("{}", CurrencyCode::Mur), "MUR");
    assert_eq!(format!("{}", CurrencyCode::Mvq), "MVQ");
    assert_eq!(format!("{}", CurrencyCode::Mvr), "MVR");
    assert_eq!(format!("{}", CurrencyCode::Mwk), "MWK");
    assert_eq!(format!("{}", CurrencyCode::Mxn), "MXN");
    assert_eq!(format!("{}", CurrencyCode::Mxp), "MXP");
    assert_eq!(format!("{}", CurrencyCode::Mxv), "MXV");
    assert_eq!(format!("{}", CurrencyCode::Myr), "MYR");
    assert_eq!(format!("{}", CurrencyCode::Mze), "MZE");
    assert_eq!(format!("{}", CurrencyCode::Mzm), "MZM");
    assert_eq!(format!("{}", CurrencyCode::Mzn), "MZN");
    assert_eq!(format!("{}", CurrencyCode::Nad), "NAD");
    assert_eq!(format!("{}", CurrencyCode::Ngn), "NGN");
    assert_eq!(format!("{}", CurrencyCode::Nic), "NIC");
    assert_eq!(format!("{}", CurrencyCode::Nio), "NIO");
    assert_eq!(format!("{}", CurrencyCode::Nlg), "NLG");
    assert_eq!(format!("{}", CurrencyCode::Nok), "NOK");
    assert_eq!(format!("{}", CurrencyCode::Npr), "NPR");
    assert_eq!(format!("{}", CurrencyCode::Nzd), "NZD");
    assert_eq!(format!("{}", CurrencyCode::Omr), "OMR");
    assert_eq!(format!("{}", CurrencyCode::Pab), "PAB");
    assert_eq!(format!("{}", CurrencyCode::Peh), "PEH");
    assert_eq!(format!("{}", CurrencyCode::Pei), "PEI");
    assert_eq!(format!("{}", CurrencyCode::Pen), "PEN");
    assert_eq!(format!("{}", CurrencyCode::Pes), "PES");
    assert_eq!(format!("{}", CurrencyCode::Pgk), "PGK");
    assert_eq!(format!("{}", CurrencyCode::Php), "PHP");
    assert_eq!(format!("{}", CurrencyCode::Pkr), "PKR");
    assert_eq!(format!("{}", CurrencyCode::Pln), "PLN");
    assert_eq!(format!("{}", CurrencyCode::Plz), "PLZ");
    assert_eq!(format!("{}", CurrencyCode::Pte), "PTE");
    assert_eq!(format!("{}", CurrencyCode::Pyg), "PYG");
    assert_eq!(format!("{}", CurrencyCode::Qar), "QAR");
    assert_eq!(format!("{}", CurrencyCode::Rhd), "RHD");
    assert_eq!(format!("{}", CurrencyCode::Rok), "ROK");
    assert_eq!(format!("{}", CurrencyCode::Rol), "ROL");
    assert_eq!(format!("{}", CurrencyCode::Ron), "RON");
    assert_eq!(format!("{}", CurrencyCode::Rsd), "RSD");
    assert_eq!(format!("{}", CurrencyCode::Rub), "RUB");
    assert_eq!(format!("{}", CurrencyCode::Rur), "RUR");
    assert_eq!(format!("{}", CurrencyCode::Rwf), "RWF");
    assert_eq!(format!("{}", CurrencyCode::Sar), "SAR");
    assert_eq!(format!("{}", CurrencyCode::Sbd), "SBD");
    assert_eq!(format!("{}", CurrencyCode::Scr), "SCR");
    assert_eq!(format!("{}", CurrencyCode::Sdd), "SDD");
    assert_eq!(format!("{}", CurrencyCode::Sdg), "SDG");
    assert_eq!(format!("{}", CurrencyCode::Sdp), "SDP");
    assert_eq!(format!("{}", CurrencyCode::Sek), "SEK");
    assert_eq!(format!("{}", CurrencyCode::Sgd), "SGD");
    assert_eq!(format!("{}", CurrencyCode::Shp), "SHP");
    assert_eq!(format!("{}", CurrencyCode::Sit), "SIT");
    assert_eq!(format!("{}", CurrencyCode::Skk), "SKK");
    assert_eq!(format!("{}", CurrencyCode::Sll), "SLL");
    assert_eq!(format!("{}", CurrencyCode::Sos), "SOS");
    assert_eq!(format!("{}", CurrencyCode::Srd), "SRD");
    assert_eq!(format!("{}", CurrencyCode::Srg), "SRG");
    assert_eq!(format!("{}", CurrencyCode::Ssp), "SSP");
    assert_eq!(format!("{}", CurrencyCode::Std), "STD");
    assert_eq!(format!("{}", CurrencyCode::Stn), "STN");
    assert_eq!(format!("{}", CurrencyCode::Sur), "SUR");
    assert_eq!(format!("{}", CurrencyCode::Svc), "SVC");
    assert_eq!(format!("{}", CurrencyCode::Syp), "SYP");
    assert_eq!(format!("{}", CurrencyCode::Szl), "SZL");
    assert_eq!(format!("{}", CurrencyCode::Thb), "THB");
    assert_eq!(format!("{}", CurrencyCode::Tjr), "TJR");
    assert_eq!(format!("{}", CurrencyCode::Tjs), "TJS");
    assert_eq!(format!("{}", CurrencyCode::Tmm), "TMM");
    assert_eq!(format!("{}", CurrencyCode::Tmt), "TMT");
    assert_eq!(format!("{}", CurrencyCode::Tnd), "TND");
    assert_eq!(format!("{}", CurrencyCode::Top), "TOP");
    assert_eq!(format!("{}", CurrencyCode::Tpe), "TPE");
    assert_eq!(format!("{}", CurrencyCode::Trl), "TRL");
    assert_eq!(format!("{}", CurrencyCode::Try), "TRY");
    assert_eq!(format!("{}", CurrencyCode::Ttd), "TTD");
    assert_eq!(format!("{}", CurrencyCode::Twd), "TWD");
    assert_eq!(format!("{}", CurrencyCode::Tzs), "TZS");
    assert_eq!(format!("{}", CurrencyCode::Uah), "UAH");
    assert_eq!(format!("{}", CurrencyCode::Uak), "UAK");
    assert_eq!(format!("{}", CurrencyCode::Ugs), "UGS");
    assert_eq!(format!("{}", CurrencyCode::Ugw), "UGW");
    assert_eq!(format!("{}", CurrencyCode::Ugx), "UGX");
    assert_eq!(format!("{}", CurrencyCode::Usd), "USD");
    assert_eq!(format!("{}", CurrencyCode::Usn), "USN");
    assert_eq!(format!("{}", CurrencyCode::Uss), "USS");
    assert_eq!(format!("{}", CurrencyCode::Uyi), "UYI");
    assert_eq!(format!("{}", CurrencyCode::Uyn), "UYN");
    assert_eq!(format!("{}", CurrencyCode::Uyp), "UYP");
    assert_eq!(format!("{}", CurrencyCode::Uyu), "UYU");
    assert_eq!(format!("{}", CurrencyCode::Uyw), "UYW");
    assert_eq!(format!("{}", CurrencyCode::Uzs), "UZS");
    assert_eq!(format!("{}", CurrencyCode::Veb), "VEB");
    assert_eq!(format!("{}", CurrencyCode::Vef), "VEF");
    assert_eq!(format!("{}", CurrencyCode::Ves), "VES");
    assert_eq!(format!("{}", CurrencyCode::Vnc), "VNC");
    assert_eq!(format!("{}", CurrencyCode::Vnd), "VND");
    assert_eq!(format!("{}", CurrencyCode::Vuv), "VUV");
    assert_eq!(format!("{}", CurrencyCode::Wst), "WST");
    assert_eq!(format!("{}", CurrencyCode::Xaf), "XAF");
    assert_eq!(format!("{}", CurrencyCode::Xag), "XAG");
    assert_eq!(format!("{}", CurrencyCode::Xau), "XAU");
    assert_eq!(format!("{}", CurrencyCode::Xba), "XBA");
    assert_eq!(format!("{}", CurrencyCode::Xbb), "XBB");
    assert_eq!(format!("{}", CurrencyCode::Xbc), "XBC");
    assert_eq!(format!("{}", CurrencyCode::Xbd), "XBD");
    assert_eq!(format!("{}", CurrencyCode::Xcd), "XCD");
    assert_eq!(format!("{}", CurrencyCode::Xdr), "XDR");
    assert_eq!(format!("{}", CurrencyCode::Xeu), "XEU");
    assert_eq!(format!("{}", CurrencyCode::Xfo), "XFO");
    assert_eq!(format!("{}", CurrencyCode::Xfu), "XFU");
    assert_eq!(format!("{}", CurrencyCode::Xof), "XOF");
    assert_eq!(format!("{}", CurrencyCode::Xpd), "XPD");
    assert_eq!(format!("{}", CurrencyCode::Xpf), "XPF");
    assert_eq!(format!("{}", CurrencyCode::Xpt), "XPT");
    assert_eq!(format!("{}", CurrencyCode::Xre), "XRE");
    assert_eq!(format!("{}", CurrencyCode::Xsu), "XSU");
    assert_eq!(format!("{}", CurrencyCode::Xts), "XTS");
    assert_eq!(format!("{}", CurrencyCode::Xua), "XUA");
    assert_eq!(format!("{}", CurrencyCode::Xxx), "XXX");
    assert_eq!(format!("{}", CurrencyCode::Ydd), "YDD");
    assert_eq!(format!("{}", CurrencyCode::Yer), "YER");
    assert_eq!(format!("{}", CurrencyCode::Yud), "YUD");
    assert_eq!(format!("{}", CurrencyCode::Yum), "YUM");
    assert_eq!(format!("{}", CurrencyCode::Yun), "YUN");
    assert_eq!(format!("{}", CurrencyCode::Zal), "ZAL");
    assert_eq!(format!("{}", CurrencyCode::Zar), "ZAR");
    assert_eq!(format!("{}", CurrencyCode::Zmk), "ZMK");
    assert_eq!(format!("{}", CurrencyCode::Zmw), "ZMW");
    assert_eq!(format!("{}", CurrencyCode::Zrn), "ZRN");
    assert_eq!(format!("{}", CurrencyCode::Zrz), "ZRZ");
    assert_eq!(format!("{}", CurrencyCode::Zwc), "ZWC");
    assert_eq!(format!("{}", CurrencyCode::Zwd), "ZWD");
    assert_eq!(format!("{}", CurrencyCode::Zwl), "ZWL");
    assert_eq!(format!("{}", CurrencyCode::Zwn), "ZWN");
    assert_eq!(format!("{}", CurrencyCode::Zwr), "ZWR");
}

#[test]
fn test_currencycode_fromstr() {
    use std::str::FromStr;
    assert_eq!(CurrencyCode::from_str("ADP").unwrap(), CurrencyCode::Adp);
    assert_eq!(CurrencyCode::from_str("AED").unwrap(), CurrencyCode::Aed);
    assert_eq!(CurrencyCode::from_str("AFA").unwrap(), CurrencyCode::Afa);
    assert_eq!(CurrencyCode::from_str("AFN").unwrap(), CurrencyCode::Afn);
    assert_eq!(CurrencyCode::from_str("ALK").unwrap(), CurrencyCode::Alk);
    assert_eq!(CurrencyCode::from_str("ALL").unwrap(), CurrencyCode::All);
    assert_eq!(CurrencyCode::from_str("AMD").unwrap(), CurrencyCode::Amd);
    assert_eq!(CurrencyCode::from_str("ANG").unwrap(), CurrencyCode::Ang);
    assert_eq!(CurrencyCode::from_str("AOA").unwrap(), CurrencyCode::Aoa);
    assert_eq!(CurrencyCode::from_str("AOK").unwrap(), CurrencyCode::Aok);
    assert_eq!(CurrencyCode::from_str("AON").unwrap(), CurrencyCode::Aon);
    assert_eq!(CurrencyCode::from_str("AOR").unwrap(), CurrencyCode::Aor);
    assert_eq!(CurrencyCode::from_str("ARA").unwrap(), CurrencyCode::Ara);
    assert_eq!(CurrencyCode::from_str("ARP").unwrap(), CurrencyCode::Arp);
    assert_eq!(CurrencyCode::from_str("ARS").unwrap(), CurrencyCode::Ars);
    assert_eq!(CurrencyCode::from_str("ARY").unwrap(), CurrencyCode::Ary);
    assert_eq!(CurrencyCode::from_str("ATS").unwrap(), CurrencyCode::Ats);
    assert_eq!(CurrencyCode::from_str("AUD").unwrap(), CurrencyCode::Aud);
    assert_eq!(CurrencyCode::from_str("AWG").unwrap(), CurrencyCode::Awg);
    assert_eq!(CurrencyCode::from_str("AYM").unwrap(), CurrencyCode::Aym);
    assert_eq!(CurrencyCode::from_str("AZM").unwrap(), CurrencyCode::Azm);
    assert_eq!(CurrencyCode::from_str("AZN").unwrap(), CurrencyCode::Azn);
    assert_eq!(CurrencyCode::from_str("BAD").unwrap(), CurrencyCode::Bad);
    assert_eq!(CurrencyCode::from_str("BAM").unwrap(), CurrencyCode::Bam);
    assert_eq!(CurrencyCode::from_str("BBD").unwrap(), CurrencyCode::Bbd);
    assert_eq!(CurrencyCode::from_str("BDT").unwrap(), CurrencyCode::Bdt);
    assert_eq!(CurrencyCode::from_str("BEC").unwrap(), CurrencyCode::Bec);
    assert_eq!(CurrencyCode::from_str("BEF").unwrap(), CurrencyCode::Bef);
    assert_eq!(CurrencyCode::from_str("BEL").unwrap(), CurrencyCode::Bel);
    assert_eq!(CurrencyCode::from_str("BGJ").unwrap(), CurrencyCode::Bgj);
    assert_eq!(CurrencyCode::from_str("BGK").unwrap(), CurrencyCode::Bgk);
    assert_eq!(CurrencyCode::from_str("BGL").unwrap(), CurrencyCode::Bgl);
    assert_eq!(CurrencyCode::from_str("BGN").unwrap(), CurrencyCode::Bgn);
    assert_eq!(CurrencyCode::from_str("BHD").unwrap(), CurrencyCode::Bhd);
    assert_eq!(CurrencyCode::from_str("BIF").unwrap(), CurrencyCode::Bif);
    assert_eq!(CurrencyCode::from_str("BMD").unwrap(), CurrencyCode::Bmd);
    assert_eq!(CurrencyCode::from_str("BND").unwrap(), CurrencyCode::Bnd);
    assert_eq!(CurrencyCode::from_str("BOB").unwrap(), CurrencyCode::Bob);
    assert_eq!(CurrencyCode::from_str("BOP").unwrap(), CurrencyCode::Bop);
    assert_eq!(CurrencyCode::from_str("BOV").unwrap(), CurrencyCode::Bov);
    assert_eq!(CurrencyCode::from_str("BRB").unwrap(), CurrencyCode::Brb);
    assert_eq!(CurrencyCode::from_str("BRC").unwrap(), CurrencyCode::Brc);
    assert_eq!(CurrencyCode::from_str("BRE").unwrap(), CurrencyCode::Bre);
    assert_eq!(CurrencyCode::from_str("BRL").unwrap(), CurrencyCode::Brl);
    assert_eq!(CurrencyCode::from_str("BRN").unwrap(), CurrencyCode::Brn);
    assert_eq!(CurrencyCode::from_str("BRR").unwrap(), CurrencyCode::Brr);
    assert_eq!(CurrencyCode::from_str("BSD").unwrap(), CurrencyCode::Bsd);
    assert_eq!(CurrencyCode::from_str("BTN").unwrap(), CurrencyCode::Btn);
    assert_eq!(CurrencyCode::from_str("BUK").unwrap(), CurrencyCode::Buk);
    assert_eq!(CurrencyCode::from_str("BWP").unwrap(), CurrencyCode::Bwp);
    assert_eq!(CurrencyCode::from_str("BYB").unwrap(), CurrencyCode::Byb);
    assert_eq!(CurrencyCode::from_str("BYN").unwrap(), CurrencyCode::Byn);
    assert_eq!(CurrencyCode::from_str("BYR").unwrap(), CurrencyCode::Byr);
    assert_eq!(CurrencyCode::from_str("BZD").unwrap(), CurrencyCode::Bzd);
    assert_eq!(CurrencyCode::from_str("CAD").unwrap(), CurrencyCode::Cad);
    assert_eq!(CurrencyCode::from_str("CDF").unwrap(), CurrencyCode::Cdf);
    assert_eq!(CurrencyCode::from_str("CHC").unwrap(), CurrencyCode::Chc);
    assert_eq!(CurrencyCode::from_str("CHE").unwrap(), CurrencyCode::Che);
    assert_eq!(CurrencyCode::from_str("CHF").unwrap(), CurrencyCode::Chf);
    assert_eq!(CurrencyCode::from_str("CHW").unwrap(), CurrencyCode::Chw);
    assert_eq!(CurrencyCode::from_str("CLF").unwrap(), CurrencyCode::Clf);
    assert_eq!(CurrencyCode::from_str("CLP").unwrap(), CurrencyCode::Clp);
    assert_eq!(CurrencyCode::from_str("CNY").unwrap(), CurrencyCode::Cny);
    assert_eq!(CurrencyCode::from_str("COP").unwrap(), CurrencyCode::Cop);
    assert_eq!(CurrencyCode::from_str("COU").unwrap(), CurrencyCode::Cou);
    assert_eq!(CurrencyCode::from_str("CRC").unwrap(), CurrencyCode::Crc);
    assert_eq!(CurrencyCode::from_str("CSD").unwrap(), CurrencyCode::Csd);
    assert_eq!(CurrencyCode::from_str("CSJ").unwrap(), CurrencyCode::Csj);
    assert_eq!(CurrencyCode::from_str("CSK").unwrap(), CurrencyCode::Csk);
    assert_eq!(CurrencyCode::from_str("CUC").unwrap(), CurrencyCode::Cuc);
    assert_eq!(CurrencyCode::from_str("CUP").unwrap(), CurrencyCode::Cup);
    assert_eq!(CurrencyCode::from_str("CVE").unwrap(), CurrencyCode::Cve);
    assert_eq!(CurrencyCode::from_str("CYP").unwrap(), CurrencyCode::Cyp);
    assert_eq!(CurrencyCode::from_str("CZK").unwrap(), CurrencyCode::Czk);
    assert_eq!(CurrencyCode::from_str("DDM").unwrap(), CurrencyCode::Ddm);
    assert_eq!(CurrencyCode::from_str("DEM").unwrap(), CurrencyCode::Dem);
    assert_eq!(CurrencyCode::from_str("DJF").unwrap(), CurrencyCode::Djf);
    assert_eq!(CurrencyCode::from_str("DKK").unwrap(), CurrencyCode::Dkk);
    assert_eq!(CurrencyCode::from_str("DOP").unwrap(), CurrencyCode::Dop);
    assert_eq!(CurrencyCode::from_str("DZD").unwrap(), CurrencyCode::Dzd);
    assert_eq!(CurrencyCode::from_str("ECS").unwrap(), CurrencyCode::Ecs);
    assert_eq!(CurrencyCode::from_str("ECV").unwrap(), CurrencyCode::Ecv);
    assert_eq!(CurrencyCode::from_str("EEK").unwrap(), CurrencyCode::Eek);
    assert_eq!(CurrencyCode::from_str("EGP").unwrap(), CurrencyCode::Egp);
    assert_eq!(CurrencyCode::from_str("ERN").unwrap(), CurrencyCode::Ern);
    assert_eq!(CurrencyCode::from_str("ESA").unwrap(), CurrencyCode::Esa);
    assert_eq!(CurrencyCode::from_str("ESB").unwrap(), CurrencyCode::Esb);
    assert_eq!(CurrencyCode::from_str("ESP").unwrap(), CurrencyCode::Esp);
    assert_eq!(CurrencyCode::from_str("ETB").unwrap(), CurrencyCode::Etb);
    assert_eq!(CurrencyCode::from_str("EUR").unwrap(), CurrencyCode::Eur);
    assert_eq!(CurrencyCode::from_str("FIM").unwrap(), CurrencyCode::Fim);
    assert_eq!(CurrencyCode::from_str("FJD").unwrap(), CurrencyCode::Fjd);
    assert_eq!(CurrencyCode::from_str("FKP").unwrap(), CurrencyCode::Fkp);
    assert_eq!(CurrencyCode::from_str("FRF").unwrap(), CurrencyCode::Frf);
    assert_eq!(CurrencyCode::from_str("GBP").unwrap(), CurrencyCode::Gbp);
    assert_eq!(CurrencyCode::from_str("GEK").unwrap(), CurrencyCode::Gek);
    assert_eq!(CurrencyCode::from_str("GEL").unwrap(), CurrencyCode::Gel);
    assert_eq!(CurrencyCode::from_str("GHC").unwrap(), CurrencyCode::Ghc);
    assert_eq!(CurrencyCode::from_str("GHP").unwrap(), CurrencyCode::Ghp);
    assert_eq!(CurrencyCode::from_str("GHS").unwrap(), CurrencyCode::Ghs);
    assert_eq!(CurrencyCode::from_str("GIP").unwrap(), CurrencyCode::Gip);
    assert_eq!(CurrencyCode::from_str("GMD").unwrap(), CurrencyCode::Gmd);
    assert_eq!(CurrencyCode::from_str("GNE").unwrap(), CurrencyCode::Gne);
    assert_eq!(CurrencyCode::from_str("GNF").unwrap(), CurrencyCode::Gnf);
    assert_eq!(CurrencyCode::from_str("GNS").unwrap(), CurrencyCode::Gns);
    assert_eq!(CurrencyCode::from_str("GQE").unwrap(), CurrencyCode::Gqe);
    assert_eq!(CurrencyCode::from_str("GRD").unwrap(), CurrencyCode::Grd);
    assert_eq!(CurrencyCode::from_str("GTQ").unwrap(), CurrencyCode::Gtq);
    assert_eq!(CurrencyCode::from_str("GWE").unwrap(), CurrencyCode::Gwe);
    assert_eq!(CurrencyCode::from_str("GWP").unwrap(), CurrencyCode::Gwp);
    assert_eq!(CurrencyCode::from_str("GYD").unwrap(), CurrencyCode::Gyd);
    assert_eq!(CurrencyCode::from_str("HKD").unwrap(), CurrencyCode::Hkd);
    assert_eq!(CurrencyCode::from_str("HNL").unwrap(), CurrencyCode::Hnl);
    assert_eq!(CurrencyCode::from_str("HRD").unwrap(), CurrencyCode::Hrd);
    assert_eq!(CurrencyCode::from_str("HRK").unwrap(), CurrencyCode::Hrk);
    assert_eq!(CurrencyCode::from_str("HTG").unwrap(), CurrencyCode::Htg);
    assert_eq!(CurrencyCode::from_str("HUF").unwrap(), CurrencyCode::Huf);
    assert_eq!(CurrencyCode::from_str("IDR").unwrap(), CurrencyCode::Idr);
    assert_eq!(CurrencyCode::from_str("IEP").unwrap(), CurrencyCode::Iep);
    assert_eq!(CurrencyCode::from_str("ILP").unwrap(), CurrencyCode::Ilp);
    assert_eq!(CurrencyCode::from_str("ILR").unwrap(), CurrencyCode::Ilr);
    assert_eq!(CurrencyCode::from_str("ILS").unwrap(), CurrencyCode::Ils);
    assert_eq!(CurrencyCode::from_str("INR").unwrap(), CurrencyCode::Inr);
    assert_eq!(CurrencyCode::from_str("IQD").unwrap(), CurrencyCode::Iqd);
    assert_eq!(CurrencyCode::from_str("IRR").unwrap(), CurrencyCode::Irr);
    assert_eq!(CurrencyCode::from_str("ISJ").unwrap(), CurrencyCode::Isj);
    assert_eq!(CurrencyCode::from_str("ISK").unwrap(), CurrencyCode::Isk);
    assert_eq!(CurrencyCode::from_str("ITL").unwrap(), CurrencyCode::Itl);
    assert_eq!(CurrencyCode::from_str("JMD").unwrap(), CurrencyCode::Jmd);
    assert_eq!(CurrencyCode::from_str("JOD").unwrap(), CurrencyCode::Jod);
    assert_eq!(CurrencyCode::from_str("JPY").unwrap(), CurrencyCode::Jpy);
    assert_eq!(CurrencyCode::from_str("KES").unwrap(), CurrencyCode::Kes);
    assert_eq!(CurrencyCode::from_str("KGS").unwrap(), CurrencyCode::Kgs);
    assert_eq!(CurrencyCode::from_str("KHR").unwrap(), CurrencyCode::Khr);
    assert_eq!(CurrencyCode::from_str("KMF").unwrap(), CurrencyCode::Kmf);
    assert_eq!(CurrencyCode::from_str("KPW").unwrap(), CurrencyCode::Kpw);
    assert_eq!(CurrencyCode::from_str("KRW").unwrap(), CurrencyCode::Krw);
    assert_eq!(CurrencyCode::from_str("KWD").unwrap(), CurrencyCode::Kwd);
    assert_eq!(CurrencyCode::from_str("KYD").unwrap(), CurrencyCode::Kyd);
    assert_eq!(CurrencyCode::from_str("KZT").unwrap(), CurrencyCode::Kzt);
    assert_eq!(CurrencyCode::from_str("LAJ").unwrap(), CurrencyCode::Laj);
    assert_eq!(CurrencyCode::from_str("LAK").unwrap(), CurrencyCode::Lak);
    assert_eq!(CurrencyCode::from_str("LBP").unwrap(), CurrencyCode::Lbp);
    assert_eq!(CurrencyCode::from_str("LKR").unwrap(), CurrencyCode::Lkr);
    assert_eq!(CurrencyCode::from_str("LRD").unwrap(), CurrencyCode::Lrd);
    assert_eq!(CurrencyCode::from_str("LSL").unwrap(), CurrencyCode::Lsl);
    assert_eq!(CurrencyCode::from_str("LSM").unwrap(), CurrencyCode::Lsm);
    assert_eq!(CurrencyCode::from_str("LTL").unwrap(), CurrencyCode::Ltl);
    assert_eq!(CurrencyCode::from_str("LTT").unwrap(), CurrencyCode::Ltt);
    assert_eq!(CurrencyCode::from_str("LUC").unwrap(), CurrencyCode::Luc);
    assert_eq!(CurrencyCode::from_str("LUF").unwrap(), CurrencyCode::Luf);
    assert_eq!(CurrencyCode::from_str("LUL").unwrap(), CurrencyCode::Lul);
    assert_eq!(CurrencyCode::from_str("LVL").unwrap(), CurrencyCode::Lvl);
    assert_eq!(CurrencyCode::from_str("LVR").unwrap(), CurrencyCode::Lvr);
    assert_eq!(CurrencyCode::from_str("LYD").unwrap(), CurrencyCode::Lyd);
    assert_eq!(CurrencyCode::from_str("MAD").unwrap(), CurrencyCode::Mad);
    assert_eq!(CurrencyCode::from_str("MDL").unwrap(), CurrencyCode::Mdl);
    assert_eq!(CurrencyCode::from_str("MGA").unwrap(), CurrencyCode::Mga);
    assert_eq!(CurrencyCode::from_str("MGF").unwrap(), CurrencyCode::Mgf);
    assert_eq!(CurrencyCode::from_str("MKD").unwrap(), CurrencyCode::Mkd);
    assert_eq!(CurrencyCode::from_str("MLF").unwrap(), CurrencyCode::Mlf);
    assert_eq!(CurrencyCode::from_str("MMK").unwrap(), CurrencyCode::Mmk);
    assert_eq!(CurrencyCode::from_str("MNT").unwrap(), CurrencyCode::Mnt);
    assert_eq!(CurrencyCode::from_str("MOP").unwrap(), CurrencyCode::Mop);
    assert_eq!(CurrencyCode::from_str("MRO").unwrap(), CurrencyCode::Mro);
    assert_eq!(CurrencyCode::from_str("MRU").unwrap(), CurrencyCode::Mru);
    assert_eq!(CurrencyCode::from_str("MTL").unwrap(), CurrencyCode::Mtl);
    assert_eq!(CurrencyCode::from_str("MTP").unwrap(), CurrencyCode::Mtp);
    assert_eq!(CurrencyCode::from_str("MUR").unwrap(), CurrencyCode::Mur);
    assert_eq!(CurrencyCode::from_str("MVQ").unwrap(), CurrencyCode::Mvq);
    assert_eq!(CurrencyCode::from_str("MVR").unwrap(), CurrencyCode::Mvr);
    assert_eq!(CurrencyCode::from_str("MWK").unwrap(), CurrencyCode::Mwk);
    assert_eq!(CurrencyCode::from_str("MXN").unwrap(), CurrencyCode::Mxn);
    assert_eq!(CurrencyCode::from_str("MXP").unwrap(), CurrencyCode::Mxp);
    assert_eq!(CurrencyCode::from_str("MXV").unwrap(), CurrencyCode::Mxv);
    assert_eq!(CurrencyCode::from_str("MYR").unwrap(), CurrencyCode::Myr);
    assert_eq!(CurrencyCode::from_str("MZE").unwrap(), CurrencyCode::Mze);
    assert_eq!(CurrencyCode::from_str("MZM").unwrap(), CurrencyCode::Mzm);
    assert_eq!(CurrencyCode::from_str("MZN").unwrap(), CurrencyCode::Mzn);
    assert_eq!(CurrencyCode::from_str("NAD").unwrap(), CurrencyCode::Nad);
    assert_eq!(CurrencyCode::from_str("NGN").unwrap(), CurrencyCode::Ngn);
    assert_eq!(CurrencyCode::from_str("NIC").unwrap(), CurrencyCode::Nic);
    assert_eq!(CurrencyCode::from_str("NIO").unwrap(), CurrencyCode::Nio);
    assert_eq!(CurrencyCode::from_str("NLG").unwrap(), CurrencyCode::Nlg);
    assert_eq!(CurrencyCode::from_str("NOK").unwrap(), CurrencyCode::Nok);
    assert_eq!(CurrencyCode::from_str("NPR").unwrap(), CurrencyCode::Npr);
    assert_eq!(CurrencyCode::from_str("NZD").unwrap(), CurrencyCode::Nzd);
    assert_eq!(CurrencyCode::from_str("OMR").unwrap(), CurrencyCode::Omr);
    assert_eq!(CurrencyCode::from_str("PAB").unwrap(), CurrencyCode::Pab);
    assert_eq!(CurrencyCode::from_str("PEH").unwrap(), CurrencyCode::Peh);
    assert_eq!(CurrencyCode::from_str("PEI").unwrap(), CurrencyCode::Pei);
    assert_eq!(CurrencyCode::from_str("PEN").unwrap(), CurrencyCode::Pen);
    assert_eq!(CurrencyCode::from_str("PES").unwrap(), CurrencyCode::Pes);
    assert_eq!(CurrencyCode::from_str("PGK").unwrap(), CurrencyCode::Pgk);
    assert_eq!(CurrencyCode::from_str("PHP").unwrap(), CurrencyCode::Php);
    assert_eq!(CurrencyCode::from_str("PKR").unwrap(), CurrencyCode::Pkr);
    assert_eq!(CurrencyCode::from_str("PLN").unwrap(), CurrencyCode::Pln);
    assert_eq!(CurrencyCode::from_str("PLZ").unwrap(), CurrencyCode::Plz);
    assert_eq!(CurrencyCode::from_str("PTE").unwrap(), CurrencyCode::Pte);
    assert_eq!(CurrencyCode::from_str("PYG").unwrap(), CurrencyCode::Pyg);
    assert_eq!(CurrencyCode::from_str("QAR").unwrap(), CurrencyCode::Qar);
    assert_eq!(CurrencyCode::from_str("RHD").unwrap(), CurrencyCode::Rhd);
    assert_eq!(CurrencyCode::from_str("ROK").unwrap(), CurrencyCode::Rok);
    assert_eq!(CurrencyCode::from_str("ROL").unwrap(), CurrencyCode::Rol);
    assert_eq!(CurrencyCode::from_str("RON").unwrap(), CurrencyCode::Ron);
    assert_eq!(CurrencyCode::from_str("RSD").unwrap(), CurrencyCode::Rsd);
    assert_eq!(CurrencyCode::from_str("RUB").unwrap(), CurrencyCode::Rub);
    assert_eq!(CurrencyCode::from_str("RUR").unwrap(), CurrencyCode::Rur);
    assert_eq!(CurrencyCode::from_str("RWF").unwrap(), CurrencyCode::Rwf);
    assert_eq!(CurrencyCode::from_str("SAR").unwrap(), CurrencyCode::Sar);
    assert_eq!(CurrencyCode::from_str("SBD").unwrap(), CurrencyCode::Sbd);
    assert_eq!(CurrencyCode::from_str("SCR").unwrap(), CurrencyCode::Scr);
    assert_eq!(CurrencyCode::from_str("SDD").unwrap(), CurrencyCode::Sdd);
    assert_eq!(CurrencyCode::from_str("SDG").unwrap(), CurrencyCode::Sdg);
    assert_eq!(CurrencyCode::from_str("SDP").unwrap(), CurrencyCode::Sdp);
    assert_eq!(CurrencyCode::from_str("SEK").unwrap(), CurrencyCode::Sek);
    assert_eq!(CurrencyCode::from_str("SGD").unwrap(), CurrencyCode::Sgd);
    assert_eq!(CurrencyCode::from_str("SHP").unwrap(), CurrencyCode::Shp);
    assert_eq!(CurrencyCode::from_str("SIT").unwrap(), CurrencyCode::Sit);
    assert_eq!(CurrencyCode::from_str("SKK").unwrap(), CurrencyCode::Skk);
    assert_eq!(CurrencyCode::from_str("SLL").unwrap(), CurrencyCode::Sll);
    assert_eq!(CurrencyCode::from_str("SOS").unwrap(), CurrencyCode::Sos);
    assert_eq!(CurrencyCode::from_str("SRD").unwrap(), CurrencyCode::Srd);
    assert_eq!(CurrencyCode::from_str("SRG").unwrap(), CurrencyCode::Srg);
    assert_eq!(CurrencyCode::from_str("SSP").unwrap(), CurrencyCode::Ssp);
    assert_eq!(CurrencyCode::from_str("STD").unwrap(), CurrencyCode::Std);
    assert_eq!(CurrencyCode::from_str("STN").unwrap(), CurrencyCode::Stn);
    assert_eq!(CurrencyCode::from_str("SUR").unwrap(), CurrencyCode::Sur);
    assert_eq!(CurrencyCode::from_str("SVC").unwrap(), CurrencyCode::Svc);
    assert_eq!(CurrencyCode::from_str("SYP").unwrap(), CurrencyCode::Syp);
    assert_eq!(CurrencyCode::from_str("SZL").unwrap(), CurrencyCode::Szl);
    assert_eq!(CurrencyCode::from_str("THB").unwrap(), CurrencyCode::Thb);
    assert_eq!(CurrencyCode::from_str("TJR").unwrap(), CurrencyCode::Tjr);
    assert_eq!(CurrencyCode::from_str("TJS").unwrap(), CurrencyCode::Tjs);
    assert_eq!(CurrencyCode::from_str("TMM").unwrap(), CurrencyCode::Tmm);
    assert_eq!(CurrencyCode::from_str("TMT").unwrap(), CurrencyCode::Tmt);
    assert_eq!(CurrencyCode::from_str("TND").unwrap(), CurrencyCode::Tnd);
    assert_eq!(CurrencyCode::from_str("TOP").unwrap(), CurrencyCode::Top);
    assert_eq!(CurrencyCode::from_str("TPE").unwrap(), CurrencyCode::Tpe);
    assert_eq!(CurrencyCode::from_str("TRL").unwrap(), CurrencyCode::Trl);
    assert_eq!(CurrencyCode::from_str("TRY").unwrap(), CurrencyCode::Try);
    assert_eq!(CurrencyCode::from_str("TTD").unwrap(), CurrencyCode::Ttd);
    assert_eq!(CurrencyCode::from_str("TWD").unwrap(), CurrencyCode::Twd);
    assert_eq!(CurrencyCode::from_str("TZS").unwrap(), CurrencyCode::Tzs);
    assert_eq!(CurrencyCode::from_str("UAH").unwrap(), CurrencyCode::Uah);
    assert_eq!(CurrencyCode::from_str("UAK").unwrap(), CurrencyCode::Uak);
    assert_eq!(CurrencyCode::from_str("UGS").unwrap(), CurrencyCode::Ugs);
    assert_eq!(CurrencyCode::from_str("UGW").unwrap(), CurrencyCode::Ugw);
    assert_eq!(CurrencyCode::from_str("UGX").unwrap(), CurrencyCode::Ugx);
    assert_eq!(CurrencyCode::from_str("USD").unwrap(), CurrencyCode::Usd);
    assert_eq!(CurrencyCode::from_str("USN").unwrap(), CurrencyCode::Usn);
    assert_eq!(CurrencyCode::from_str("USS").unwrap(), CurrencyCode::Uss);
    assert_eq!(CurrencyCode::from_str("UYI").unwrap(), CurrencyCode::Uyi);
    assert_eq!(CurrencyCode::from_str("UYN").unwrap(), CurrencyCode::Uyn);
    assert_eq!(CurrencyCode::from_str("UYP").unwrap(), CurrencyCode::Uyp);
    assert_eq!(CurrencyCode::from_str("UYU").unwrap(), CurrencyCode::Uyu);
    assert_eq!(CurrencyCode::from_str("UYW").unwrap(), CurrencyCode::Uyw);
    assert_eq!(CurrencyCode::from_str("UZS").unwrap(), CurrencyCode::Uzs);
    assert_eq!(CurrencyCode::from_str("VEB").unwrap(), CurrencyCode::Veb);
    assert_eq!(CurrencyCode::from_str("VEF").unwrap(), CurrencyCode::Vef);
    assert_eq!(CurrencyCode::from_str("VES").unwrap(), CurrencyCode::Ves);
    assert_eq!(CurrencyCode::from_str("VNC").unwrap(), CurrencyCode::Vnc);
    assert_eq!(CurrencyCode::from_str("VND").unwrap(), CurrencyCode::Vnd);
    assert_eq!(CurrencyCode::from_str("VUV").unwrap(), CurrencyCode::Vuv);
    assert_eq!(CurrencyCode::from_str("WST").unwrap(), CurrencyCode::Wst);
    assert_eq!(CurrencyCode::from_str("XAF").unwrap(), CurrencyCode::Xaf);
    assert_eq!(CurrencyCode::from_str("XAG").unwrap(), CurrencyCode::Xag);
    assert_eq!(CurrencyCode::from_str("XAU").unwrap(), CurrencyCode::Xau);
    assert_eq!(CurrencyCode::from_str("XBA").unwrap(), CurrencyCode::Xba);
    assert_eq!(CurrencyCode::from_str("XBB").unwrap(), CurrencyCode::Xbb);
    assert_eq!(CurrencyCode::from_str("XBC").unwrap(), CurrencyCode::Xbc);
    assert_eq!(CurrencyCode::from_str("XBD").unwrap(), CurrencyCode::Xbd);
    assert_eq!(CurrencyCode::from_str("XCD").unwrap(), CurrencyCode::Xcd);
    assert_eq!(CurrencyCode::from_str("XDR").unwrap(), CurrencyCode::Xdr);
    assert_eq!(CurrencyCode::from_str("XEU").unwrap(), CurrencyCode::Xeu);
    assert_eq!(CurrencyCode::from_str("XFO").unwrap(), CurrencyCode::Xfo);
    assert_eq!(CurrencyCode::from_str("XFU").unwrap(), CurrencyCode::Xfu);
    assert_eq!(CurrencyCode::from_str("XOF").unwrap(), CurrencyCode::Xof);
    assert_eq!(CurrencyCode::from_str("XPD").unwrap(), CurrencyCode::Xpd);
    assert_eq!(CurrencyCode::from_str("XPF").unwrap(), CurrencyCode::Xpf);
    assert_eq!(CurrencyCode::from_str("XPT").unwrap(), CurrencyCode::Xpt);
    assert_eq!(CurrencyCode::from_str("XRE").unwrap(), CurrencyCode::Xre);
    assert_eq!(CurrencyCode::from_str("XSU").unwrap(), CurrencyCode::Xsu);
    assert_eq!(CurrencyCode::from_str("XTS").unwrap(), CurrencyCode::Xts);
    assert_eq!(CurrencyCode::from_str("XUA").unwrap(), CurrencyCode::Xua);
    assert_eq!(CurrencyCode::from_str("XXX").unwrap(), CurrencyCode::Xxx);
    assert_eq!(CurrencyCode::from_str("YDD").unwrap(), CurrencyCode::Ydd);
    assert_eq!(CurrencyCode::from_str("YER").unwrap(), CurrencyCode::Yer);
    assert_eq!(CurrencyCode::from_str("YUD").unwrap(), CurrencyCode::Yud);
    assert_eq!(CurrencyCode::from_str("YUM").unwrap(), CurrencyCode::Yum);
    assert_eq!(CurrencyCode::from_str("YUN").unwrap(), CurrencyCode::Yun);
    assert_eq!(CurrencyCode::from_str("ZAL").unwrap(), CurrencyCode::Zal);
    assert_eq!(CurrencyCode::from_str("ZAR").unwrap(), CurrencyCode::Zar);
    assert_eq!(CurrencyCode::from_str("ZMK").unwrap(), CurrencyCode::Zmk);
    assert_eq!(CurrencyCode::from_str("ZMW").unwrap(), CurrencyCode::Zmw);
    assert_eq!(CurrencyCode::from_str("ZRN").unwrap(), CurrencyCode::Zrn);
    assert_eq!(CurrencyCode::from_str("ZRZ").unwrap(), CurrencyCode::Zrz);
    assert_eq!(CurrencyCode::from_str("ZWC").unwrap(), CurrencyCode::Zwc);
    assert_eq!(CurrencyCode::from_str("ZWD").unwrap(), CurrencyCode::Zwd);
    assert_eq!(CurrencyCode::from_str("ZWL").unwrap(), CurrencyCode::Zwl);
    assert_eq!(CurrencyCode::from_str("ZWN").unwrap(), CurrencyCode::Zwn);
    assert_eq!(CurrencyCode::from_str("ZWR").unwrap(), CurrencyCode::Zwr);
}

#[cfg(feature = "backend")]
pub mod crud;
