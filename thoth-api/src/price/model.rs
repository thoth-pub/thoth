use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
#[cfg(feature = "backend")]
use crate::schema::price;

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Price {
    pub price_id: Uuid,
    pub publication_id: Uuid,
    pub currency_code: CurrencyCode,
    pub unit_price: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "price"
)]
pub struct NewPrice {
    pub publication_id: Uuid,
    pub currency_code: CurrencyCode,
    pub unit_price: f64,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "price"
)]
pub struct PatchPrice {
    pub price_id: Uuid,
    pub publication_id: Uuid,
    pub currency_code: CurrencyCode,
    pub unit_price: f64,
}

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Currency_code")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CurrencyCode {
    Adp,
    Aed,
    Afa,
    Afn,
    Alk,
    All,
    Amd,
    Ang,
    Aoa,
    Aok,
    Aon,
    Aor,
    Ara,
    Arp,
    Ars,
    Ary,
    Ats,
    Aud,
    Awg,
    Aym,
    Azm,
    Azn,
    Bad,
    Bam,
    Bbd,
    Bdt,
    Bec,
    Bef,
    Bel,
    Bgj,
    Bgk,
    Bgl,
    Bgn,
    Bhd,
    Bif,
    Bmd,
    Bnd,
    Bob,
    Bop,
    Bov,
    Brb,
    Brc,
    Bre,
    Brl,
    Brn,
    Brr,
    Bsd,
    Btn,
    Buk,
    Bwp,
    Byb,
    Byn,
    Byr,
    Bzd,
    Cad,
    Cdf,
    Chc,
    Che,
    Chf,
    Chw,
    Clf,
    Clp,
    Cny,
    Cop,
    Cou,
    Crc,
    Csd,
    Csj,
    Csk,
    Cuc,
    Cup,
    Cve,
    Cyp,
    Czk,
    Ddm,
    Dem,
    Djf,
    Dkk,
    Dop,
    Dzd,
    Ecs,
    Ecv,
    Eek,
    Egp,
    Ern,
    Esa,
    Esb,
    Esp,
    Etb,
    Eur,
    Fim,
    Fjd,
    Fkp,
    Frf,
    Gbp,
    Gek,
    Gel,
    Ghc,
    Ghp,
    Ghs,
    Gip,
    Gmd,
    Gne,
    Gnf,
    Gns,
    Gqe,
    Grd,
    Gtq,
    Gwe,
    Gwp,
    Gyd,
    Hkd,
    Hnl,
    Hrd,
    Hrk,
    Htg,
    Huf,
    Idr,
    Iep,
    Ilp,
    Ilr,
    Ils,
    Inr,
    Iqd,
    Irr,
    Isj,
    Isk,
    Itl,
    Jmd,
    Jod,
    Jpy,
    Kes,
    Kgs,
    Khr,
    Kmf,
    Kpw,
    Krw,
    Kwd,
    Kyd,
    Kzt,
    Laj,
    Lak,
    Lbp,
    Lkr,
    Lrd,
    Lsl,
    Lsm,
    Ltl,
    Ltt,
    Luc,
    Luf,
    Lul,
    Lvl,
    Lvr,
    Lyd,
    Mad,
    Mdl,
    Mga,
    Mgf,
    Mkd,
    Mlf,
    Mmk,
    Mnt,
    Mop,
    Mro,
    Mru,
    Mtl,
    Mtp,
    Mur,
    Mvq,
    Mvr,
    Mwk,
    Mxn,
    Mxp,
    Mxv,
    Myr,
    Mze,
    Mzm,
    Mzn,
    Nad,
    Ngn,
    Nic,
    Nio,
    Nlg,
    Nok,
    Npr,
    Nzd,
    Omr,
    Pab,
    Peh,
    Pei,
    Pen,
    Pes,
    Pgk,
    Php,
    Pkr,
    Pln,
    Plz,
    Pte,
    Pyg,
    Qar,
    Rhd,
    Rok,
    Rol,
    Ron,
    Rsd,
    Rub,
    Rur,
    Rwf,
    Sar,
    Sbd,
    Scr,
    Sdd,
    Sdg,
    Sdp,
    Sek,
    Sgd,
    Shp,
    Sit,
    Skk,
    Sll,
    Sos,
    Srd,
    Srg,
    Ssp,
    Std,
    Stn,
    Sur,
    Svc,
    Syp,
    Szl,
    Thb,
    Tjr,
    Tjs,
    Tmm,
    Tmt,
    Tnd,
    Top,
    Tpe,
    Trl,
    Try,
    Ttd,
    Twd,
    Tzs,
    Uah,
    Uak,
    Ugs,
    Ugw,
    Ugx,
    Usd,
    Usn,
    Uss,
    Uyi,
    Uyn,
    Uyp,
    Uyu,
    Uyw,
    Uzs,
    Veb,
    Vef,
    Ves,
    Vnc,
    Vnd,
    Vuv,
    Wst,
    Xaf,
    Xag,
    Xau,
    Xba,
    Xbb,
    Xbc,
    Xbd,
    Xcd,
    Xdr,
    Xeu,
    Xfo,
    Xfu,
    Xof,
    Xpd,
    Xpf,
    Xpt,
    Xre,
    Xsu,
    Xts,
    Xua,
    Xxx,
    Ydd,
    Yer,
    Yud,
    Yum,
    Yun,
    Zal,
    Zar,
    Zmk,
    Zmw,
    Zrn,
    Zrz,
    Zwc,
    Zwd,
    Zwl,
    Zwn,
    Zwr,
}

impl Default for CurrencyCode {
    fn default() -> CurrencyCode {
        CurrencyCode::Gbp
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrencyCode::Adp => write!(f, "ADP"),
            CurrencyCode::Aed => write!(f, "AED"),
            CurrencyCode::Afa => write!(f, "AFA"),
            CurrencyCode::Afn => write!(f, "AFN"),
            CurrencyCode::Alk => write!(f, "ALK"),
            CurrencyCode::All => write!(f, "ALL"),
            CurrencyCode::Amd => write!(f, "AMD"),
            CurrencyCode::Ang => write!(f, "ANG"),
            CurrencyCode::Aoa => write!(f, "AOA"),
            CurrencyCode::Aok => write!(f, "AOK"),
            CurrencyCode::Aon => write!(f, "AON"),
            CurrencyCode::Aor => write!(f, "AOR"),
            CurrencyCode::Ara => write!(f, "ARA"),
            CurrencyCode::Arp => write!(f, "ARP"),
            CurrencyCode::Ars => write!(f, "ARS"),
            CurrencyCode::Ary => write!(f, "ARY"),
            CurrencyCode::Ats => write!(f, "ATS"),
            CurrencyCode::Aud => write!(f, "AUD"),
            CurrencyCode::Awg => write!(f, "AWG"),
            CurrencyCode::Aym => write!(f, "AYM"),
            CurrencyCode::Azm => write!(f, "AZM"),
            CurrencyCode::Azn => write!(f, "AZN"),
            CurrencyCode::Bad => write!(f, "BAD"),
            CurrencyCode::Bam => write!(f, "BAM"),
            CurrencyCode::Bbd => write!(f, "BBD"),
            CurrencyCode::Bdt => write!(f, "BDT"),
            CurrencyCode::Bec => write!(f, "BEC"),
            CurrencyCode::Bef => write!(f, "BEF"),
            CurrencyCode::Bel => write!(f, "BEL"),
            CurrencyCode::Bgj => write!(f, "BGJ"),
            CurrencyCode::Bgk => write!(f, "BGK"),
            CurrencyCode::Bgl => write!(f, "BGL"),
            CurrencyCode::Bgn => write!(f, "BGN"),
            CurrencyCode::Bhd => write!(f, "BHD"),
            CurrencyCode::Bif => write!(f, "BIF"),
            CurrencyCode::Bmd => write!(f, "BMD"),
            CurrencyCode::Bnd => write!(f, "BND"),
            CurrencyCode::Bob => write!(f, "BOB"),
            CurrencyCode::Bop => write!(f, "BOP"),
            CurrencyCode::Bov => write!(f, "BOV"),
            CurrencyCode::Brb => write!(f, "BRB"),
            CurrencyCode::Brc => write!(f, "BRC"),
            CurrencyCode::Bre => write!(f, "BRE"),
            CurrencyCode::Brl => write!(f, "BRL"),
            CurrencyCode::Brn => write!(f, "BRN"),
            CurrencyCode::Brr => write!(f, "BRR"),
            CurrencyCode::Bsd => write!(f, "BSD"),
            CurrencyCode::Btn => write!(f, "BTN"),
            CurrencyCode::Buk => write!(f, "BUK"),
            CurrencyCode::Bwp => write!(f, "BWP"),
            CurrencyCode::Byb => write!(f, "BYB"),
            CurrencyCode::Byn => write!(f, "BYN"),
            CurrencyCode::Byr => write!(f, "BYR"),
            CurrencyCode::Bzd => write!(f, "BZD"),
            CurrencyCode::Cad => write!(f, "CAD"),
            CurrencyCode::Cdf => write!(f, "CDF"),
            CurrencyCode::Chc => write!(f, "CHC"),
            CurrencyCode::Che => write!(f, "CHE"),
            CurrencyCode::Chf => write!(f, "CHF"),
            CurrencyCode::Chw => write!(f, "CHW"),
            CurrencyCode::Clf => write!(f, "CLF"),
            CurrencyCode::Clp => write!(f, "CLP"),
            CurrencyCode::Cny => write!(f, "CNY"),
            CurrencyCode::Cop => write!(f, "COP"),
            CurrencyCode::Cou => write!(f, "COU"),
            CurrencyCode::Crc => write!(f, "CRC"),
            CurrencyCode::Csd => write!(f, "CSD"),
            CurrencyCode::Csj => write!(f, "CSJ"),
            CurrencyCode::Csk => write!(f, "CSK"),
            CurrencyCode::Cuc => write!(f, "CUC"),
            CurrencyCode::Cup => write!(f, "CUP"),
            CurrencyCode::Cve => write!(f, "CVE"),
            CurrencyCode::Cyp => write!(f, "CYP"),
            CurrencyCode::Czk => write!(f, "CZK"),
            CurrencyCode::Ddm => write!(f, "DDM"),
            CurrencyCode::Dem => write!(f, "DEM"),
            CurrencyCode::Djf => write!(f, "DJF"),
            CurrencyCode::Dkk => write!(f, "DKK"),
            CurrencyCode::Dop => write!(f, "DOP"),
            CurrencyCode::Dzd => write!(f, "DZD"),
            CurrencyCode::Ecs => write!(f, "ECS"),
            CurrencyCode::Ecv => write!(f, "ECV"),
            CurrencyCode::Eek => write!(f, "EEK"),
            CurrencyCode::Egp => write!(f, "EGP"),
            CurrencyCode::Ern => write!(f, "ERN"),
            CurrencyCode::Esa => write!(f, "ESA"),
            CurrencyCode::Esb => write!(f, "ESB"),
            CurrencyCode::Esp => write!(f, "ESP"),
            CurrencyCode::Etb => write!(f, "ETB"),
            CurrencyCode::Eur => write!(f, "EUR"),
            CurrencyCode::Fim => write!(f, "FIM"),
            CurrencyCode::Fjd => write!(f, "FJD"),
            CurrencyCode::Fkp => write!(f, "FKP"),
            CurrencyCode::Frf => write!(f, "FRF"),
            CurrencyCode::Gbp => write!(f, "GBP"),
            CurrencyCode::Gek => write!(f, "GEK"),
            CurrencyCode::Gel => write!(f, "GEL"),
            CurrencyCode::Ghc => write!(f, "GHC"),
            CurrencyCode::Ghp => write!(f, "GHP"),
            CurrencyCode::Ghs => write!(f, "GHS"),
            CurrencyCode::Gip => write!(f, "GIP"),
            CurrencyCode::Gmd => write!(f, "GMD"),
            CurrencyCode::Gne => write!(f, "GNE"),
            CurrencyCode::Gnf => write!(f, "GNF"),
            CurrencyCode::Gns => write!(f, "GNS"),
            CurrencyCode::Gqe => write!(f, "GQE"),
            CurrencyCode::Grd => write!(f, "GRD"),
            CurrencyCode::Gtq => write!(f, "GTQ"),
            CurrencyCode::Gwe => write!(f, "GWE"),
            CurrencyCode::Gwp => write!(f, "GWP"),
            CurrencyCode::Gyd => write!(f, "GYD"),
            CurrencyCode::Hkd => write!(f, "HKD"),
            CurrencyCode::Hnl => write!(f, "HNL"),
            CurrencyCode::Hrd => write!(f, "HRD"),
            CurrencyCode::Hrk => write!(f, "HRK"),
            CurrencyCode::Htg => write!(f, "HTG"),
            CurrencyCode::Huf => write!(f, "HUF"),
            CurrencyCode::Idr => write!(f, "IDR"),
            CurrencyCode::Iep => write!(f, "IEP"),
            CurrencyCode::Ilp => write!(f, "ILP"),
            CurrencyCode::Ilr => write!(f, "ILR"),
            CurrencyCode::Ils => write!(f, "ILS"),
            CurrencyCode::Inr => write!(f, "INR"),
            CurrencyCode::Iqd => write!(f, "IQD"),
            CurrencyCode::Irr => write!(f, "IRR"),
            CurrencyCode::Isj => write!(f, "ISJ"),
            CurrencyCode::Isk => write!(f, "ISK"),
            CurrencyCode::Itl => write!(f, "ITL"),
            CurrencyCode::Jmd => write!(f, "JMD"),
            CurrencyCode::Jod => write!(f, "JOD"),
            CurrencyCode::Jpy => write!(f, "JPY"),
            CurrencyCode::Kes => write!(f, "KES"),
            CurrencyCode::Kgs => write!(f, "KGS"),
            CurrencyCode::Khr => write!(f, "KHR"),
            CurrencyCode::Kmf => write!(f, "KMF"),
            CurrencyCode::Kpw => write!(f, "KPW"),
            CurrencyCode::Krw => write!(f, "KRW"),
            CurrencyCode::Kwd => write!(f, "KWD"),
            CurrencyCode::Kyd => write!(f, "KYD"),
            CurrencyCode::Kzt => write!(f, "KZT"),
            CurrencyCode::Laj => write!(f, "LAJ"),
            CurrencyCode::Lak => write!(f, "LAK"),
            CurrencyCode::Lbp => write!(f, "LBP"),
            CurrencyCode::Lkr => write!(f, "LKR"),
            CurrencyCode::Lrd => write!(f, "LRD"),
            CurrencyCode::Lsl => write!(f, "LSL"),
            CurrencyCode::Lsm => write!(f, "LSM"),
            CurrencyCode::Ltl => write!(f, "LTL"),
            CurrencyCode::Ltt => write!(f, "LTT"),
            CurrencyCode::Luc => write!(f, "LUC"),
            CurrencyCode::Luf => write!(f, "LUF"),
            CurrencyCode::Lul => write!(f, "LUL"),
            CurrencyCode::Lvl => write!(f, "LVL"),
            CurrencyCode::Lvr => write!(f, "LVR"),
            CurrencyCode::Lyd => write!(f, "LYD"),
            CurrencyCode::Mad => write!(f, "MAD"),
            CurrencyCode::Mdl => write!(f, "MDL"),
            CurrencyCode::Mga => write!(f, "MGA"),
            CurrencyCode::Mgf => write!(f, "MGF"),
            CurrencyCode::Mkd => write!(f, "MKD"),
            CurrencyCode::Mlf => write!(f, "MLF"),
            CurrencyCode::Mmk => write!(f, "MMK"),
            CurrencyCode::Mnt => write!(f, "MNT"),
            CurrencyCode::Mop => write!(f, "MOP"),
            CurrencyCode::Mro => write!(f, "MRO"),
            CurrencyCode::Mru => write!(f, "MRU"),
            CurrencyCode::Mtl => write!(f, "MTL"),
            CurrencyCode::Mtp => write!(f, "MTP"),
            CurrencyCode::Mur => write!(f, "MUR"),
            CurrencyCode::Mvq => write!(f, "MVQ"),
            CurrencyCode::Mvr => write!(f, "MVR"),
            CurrencyCode::Mwk => write!(f, "MWK"),
            CurrencyCode::Mxn => write!(f, "MXN"),
            CurrencyCode::Mxp => write!(f, "MXP"),
            CurrencyCode::Mxv => write!(f, "MXV"),
            CurrencyCode::Myr => write!(f, "MYR"),
            CurrencyCode::Mze => write!(f, "MZE"),
            CurrencyCode::Mzm => write!(f, "MZM"),
            CurrencyCode::Mzn => write!(f, "MZN"),
            CurrencyCode::Nad => write!(f, "NAD"),
            CurrencyCode::Ngn => write!(f, "NGN"),
            CurrencyCode::Nic => write!(f, "NIC"),
            CurrencyCode::Nio => write!(f, "NIO"),
            CurrencyCode::Nlg => write!(f, "NLG"),
            CurrencyCode::Nok => write!(f, "NOK"),
            CurrencyCode::Npr => write!(f, "NPR"),
            CurrencyCode::Nzd => write!(f, "NZD"),
            CurrencyCode::Omr => write!(f, "OMR"),
            CurrencyCode::Pab => write!(f, "PAB"),
            CurrencyCode::Peh => write!(f, "PEH"),
            CurrencyCode::Pei => write!(f, "PEI"),
            CurrencyCode::Pen => write!(f, "PEN"),
            CurrencyCode::Pes => write!(f, "PES"),
            CurrencyCode::Pgk => write!(f, "PGK"),
            CurrencyCode::Php => write!(f, "PHP"),
            CurrencyCode::Pkr => write!(f, "PKR"),
            CurrencyCode::Pln => write!(f, "PLN"),
            CurrencyCode::Plz => write!(f, "PLZ"),
            CurrencyCode::Pte => write!(f, "PTE"),
            CurrencyCode::Pyg => write!(f, "PYG"),
            CurrencyCode::Qar => write!(f, "QAR"),
            CurrencyCode::Rhd => write!(f, "RHD"),
            CurrencyCode::Rok => write!(f, "ROK"),
            CurrencyCode::Rol => write!(f, "ROL"),
            CurrencyCode::Ron => write!(f, "RON"),
            CurrencyCode::Rsd => write!(f, "RSD"),
            CurrencyCode::Rub => write!(f, "RUB"),
            CurrencyCode::Rur => write!(f, "RUR"),
            CurrencyCode::Rwf => write!(f, "RWF"),
            CurrencyCode::Sar => write!(f, "SAR"),
            CurrencyCode::Sbd => write!(f, "SBD"),
            CurrencyCode::Scr => write!(f, "SCR"),
            CurrencyCode::Sdd => write!(f, "SDD"),
            CurrencyCode::Sdg => write!(f, "SDG"),
            CurrencyCode::Sdp => write!(f, "SDP"),
            CurrencyCode::Sek => write!(f, "SEK"),
            CurrencyCode::Sgd => write!(f, "SGD"),
            CurrencyCode::Shp => write!(f, "SHP"),
            CurrencyCode::Sit => write!(f, "SIT"),
            CurrencyCode::Skk => write!(f, "SKK"),
            CurrencyCode::Sll => write!(f, "SLL"),
            CurrencyCode::Sos => write!(f, "SOS"),
            CurrencyCode::Srd => write!(f, "SRD"),
            CurrencyCode::Srg => write!(f, "SRG"),
            CurrencyCode::Ssp => write!(f, "SSP"),
            CurrencyCode::Std => write!(f, "STD"),
            CurrencyCode::Stn => write!(f, "STN"),
            CurrencyCode::Sur => write!(f, "SUR"),
            CurrencyCode::Svc => write!(f, "SVC"),
            CurrencyCode::Syp => write!(f, "SYP"),
            CurrencyCode::Szl => write!(f, "SZL"),
            CurrencyCode::Thb => write!(f, "THB"),
            CurrencyCode::Tjr => write!(f, "TJR"),
            CurrencyCode::Tjs => write!(f, "TJS"),
            CurrencyCode::Tmm => write!(f, "TMM"),
            CurrencyCode::Tmt => write!(f, "TMT"),
            CurrencyCode::Tnd => write!(f, "TND"),
            CurrencyCode::Top => write!(f, "TOP"),
            CurrencyCode::Tpe => write!(f, "TPE"),
            CurrencyCode::Trl => write!(f, "TRL"),
            CurrencyCode::Try => write!(f, "TRY"),
            CurrencyCode::Ttd => write!(f, "TTD"),
            CurrencyCode::Twd => write!(f, "TWD"),
            CurrencyCode::Tzs => write!(f, "TZS"),
            CurrencyCode::Uah => write!(f, "UAH"),
            CurrencyCode::Uak => write!(f, "UAK"),
            CurrencyCode::Ugs => write!(f, "UGS"),
            CurrencyCode::Ugw => write!(f, "UGW"),
            CurrencyCode::Ugx => write!(f, "UGX"),
            CurrencyCode::Usd => write!(f, "USD"),
            CurrencyCode::Usn => write!(f, "USN"),
            CurrencyCode::Uss => write!(f, "USS"),
            CurrencyCode::Uyi => write!(f, "UYI"),
            CurrencyCode::Uyn => write!(f, "UYN"),
            CurrencyCode::Uyp => write!(f, "UYP"),
            CurrencyCode::Uyu => write!(f, "UYU"),
            CurrencyCode::Uyw => write!(f, "UYW"),
            CurrencyCode::Uzs => write!(f, "UZS"),
            CurrencyCode::Veb => write!(f, "VEB"),
            CurrencyCode::Vef => write!(f, "VEF"),
            CurrencyCode::Ves => write!(f, "VES"),
            CurrencyCode::Vnc => write!(f, "VNC"),
            CurrencyCode::Vnd => write!(f, "VND"),
            CurrencyCode::Vuv => write!(f, "VUV"),
            CurrencyCode::Wst => write!(f, "WST"),
            CurrencyCode::Xaf => write!(f, "XAF"),
            CurrencyCode::Xag => write!(f, "XAG"),
            CurrencyCode::Xau => write!(f, "XAU"),
            CurrencyCode::Xba => write!(f, "XBA"),
            CurrencyCode::Xbb => write!(f, "XBB"),
            CurrencyCode::Xbc => write!(f, "XBC"),
            CurrencyCode::Xbd => write!(f, "XBD"),
            CurrencyCode::Xcd => write!(f, "XCD"),
            CurrencyCode::Xdr => write!(f, "XDR"),
            CurrencyCode::Xeu => write!(f, "XEU"),
            CurrencyCode::Xfo => write!(f, "XFO"),
            CurrencyCode::Xfu => write!(f, "XFU"),
            CurrencyCode::Xof => write!(f, "XOF"),
            CurrencyCode::Xpd => write!(f, "XPD"),
            CurrencyCode::Xpf => write!(f, "XPF"),
            CurrencyCode::Xpt => write!(f, "XPT"),
            CurrencyCode::Xre => write!(f, "XRE"),
            CurrencyCode::Xsu => write!(f, "XSU"),
            CurrencyCode::Xts => write!(f, "XTS"),
            CurrencyCode::Xua => write!(f, "XUA"),
            CurrencyCode::Xxx => write!(f, "XXX"),
            CurrencyCode::Ydd => write!(f, "YDD"),
            CurrencyCode::Yer => write!(f, "YER"),
            CurrencyCode::Yud => write!(f, "YUD"),
            CurrencyCode::Yum => write!(f, "YUM"),
            CurrencyCode::Yun => write!(f, "YUN"),
            CurrencyCode::Zal => write!(f, "ZAL"),
            CurrencyCode::Zar => write!(f, "ZAR"),
            CurrencyCode::Zmk => write!(f, "ZMK"),
            CurrencyCode::Zmw => write!(f, "ZMW"),
            CurrencyCode::Zrn => write!(f, "ZRN"),
            CurrencyCode::Zrz => write!(f, "ZRZ"),
            CurrencyCode::Zwc => write!(f, "ZWC"),
            CurrencyCode::Zwd => write!(f, "ZWD"),
            CurrencyCode::Zwl => write!(f, "ZWL"),
            CurrencyCode::Zwn => write!(f, "ZWN"),
            CurrencyCode::Zwr => write!(f, "ZWR"),
        }
    }
}

impl FromStr for CurrencyCode {
    type Err = ThothError;

    fn from_str(input: &str) -> std::result::Result<CurrencyCode, ThothError> {
        match input {
            "ADP" => Ok(CurrencyCode::Adp),
            "AED" => Ok(CurrencyCode::Aed),
            "AFA" => Ok(CurrencyCode::Afa),
            "AFN" => Ok(CurrencyCode::Afn),
            "ALK" => Ok(CurrencyCode::Alk),
            "ALL" => Ok(CurrencyCode::All),
            "AMD" => Ok(CurrencyCode::Amd),
            "ANG" => Ok(CurrencyCode::Ang),
            "AOA" => Ok(CurrencyCode::Aoa),
            "AOK" => Ok(CurrencyCode::Aok),
            "AON" => Ok(CurrencyCode::Aon),
            "AOR" => Ok(CurrencyCode::Aor),
            "ARA" => Ok(CurrencyCode::Ara),
            "ARP" => Ok(CurrencyCode::Arp),
            "ARS" => Ok(CurrencyCode::Ars),
            "ARY" => Ok(CurrencyCode::Ary),
            "ATS" => Ok(CurrencyCode::Ats),
            "AUD" => Ok(CurrencyCode::Aud),
            "AWG" => Ok(CurrencyCode::Awg),
            "AYM" => Ok(CurrencyCode::Aym),
            "AZM" => Ok(CurrencyCode::Azm),
            "AZN" => Ok(CurrencyCode::Azn),
            "BAD" => Ok(CurrencyCode::Bad),
            "BAM" => Ok(CurrencyCode::Bam),
            "BBD" => Ok(CurrencyCode::Bbd),
            "BDT" => Ok(CurrencyCode::Bdt),
            "BEC" => Ok(CurrencyCode::Bec),
            "BEF" => Ok(CurrencyCode::Bef),
            "BEL" => Ok(CurrencyCode::Bel),
            "BGJ" => Ok(CurrencyCode::Bgj),
            "BGK" => Ok(CurrencyCode::Bgk),
            "BGL" => Ok(CurrencyCode::Bgl),
            "BGN" => Ok(CurrencyCode::Bgn),
            "BHD" => Ok(CurrencyCode::Bhd),
            "BIF" => Ok(CurrencyCode::Bif),
            "BMD" => Ok(CurrencyCode::Bmd),
            "BND" => Ok(CurrencyCode::Bnd),
            "BOB" => Ok(CurrencyCode::Bob),
            "BOP" => Ok(CurrencyCode::Bop),
            "BOV" => Ok(CurrencyCode::Bov),
            "BRB" => Ok(CurrencyCode::Brb),
            "BRC" => Ok(CurrencyCode::Brc),
            "BRE" => Ok(CurrencyCode::Bre),
            "BRL" => Ok(CurrencyCode::Brl),
            "BRN" => Ok(CurrencyCode::Brn),
            "BRR" => Ok(CurrencyCode::Brr),
            "BSD" => Ok(CurrencyCode::Bsd),
            "BTN" => Ok(CurrencyCode::Btn),
            "BUK" => Ok(CurrencyCode::Buk),
            "BWP" => Ok(CurrencyCode::Bwp),
            "BYB" => Ok(CurrencyCode::Byb),
            "BYN" => Ok(CurrencyCode::Byn),
            "BYR" => Ok(CurrencyCode::Byr),
            "BZD" => Ok(CurrencyCode::Bzd),
            "CAD" => Ok(CurrencyCode::Cad),
            "CDF" => Ok(CurrencyCode::Cdf),
            "CHC" => Ok(CurrencyCode::Chc),
            "CHE" => Ok(CurrencyCode::Che),
            "CHF" => Ok(CurrencyCode::Chf),
            "CHW" => Ok(CurrencyCode::Chw),
            "CLF" => Ok(CurrencyCode::Clf),
            "CLP" => Ok(CurrencyCode::Clp),
            "CNY" => Ok(CurrencyCode::Cny),
            "COP" => Ok(CurrencyCode::Cop),
            "COU" => Ok(CurrencyCode::Cou),
            "CRC" => Ok(CurrencyCode::Crc),
            "CSD" => Ok(CurrencyCode::Csd),
            "CSJ" => Ok(CurrencyCode::Csj),
            "CSK" => Ok(CurrencyCode::Csk),
            "CUC" => Ok(CurrencyCode::Cuc),
            "CUP" => Ok(CurrencyCode::Cup),
            "CVE" => Ok(CurrencyCode::Cve),
            "CYP" => Ok(CurrencyCode::Cyp),
            "CZK" => Ok(CurrencyCode::Czk),
            "DDM" => Ok(CurrencyCode::Ddm),
            "DEM" => Ok(CurrencyCode::Dem),
            "DJF" => Ok(CurrencyCode::Djf),
            "DKK" => Ok(CurrencyCode::Dkk),
            "DOP" => Ok(CurrencyCode::Dop),
            "DZD" => Ok(CurrencyCode::Dzd),
            "ECS" => Ok(CurrencyCode::Ecs),
            "ECV" => Ok(CurrencyCode::Ecv),
            "EEK" => Ok(CurrencyCode::Eek),
            "EGP" => Ok(CurrencyCode::Egp),
            "ERN" => Ok(CurrencyCode::Ern),
            "ESA" => Ok(CurrencyCode::Esa),
            "ESB" => Ok(CurrencyCode::Esb),
            "ESP" => Ok(CurrencyCode::Esp),
            "ETB" => Ok(CurrencyCode::Etb),
            "EUR" => Ok(CurrencyCode::Eur),
            "FIM" => Ok(CurrencyCode::Fim),
            "FJD" => Ok(CurrencyCode::Fjd),
            "FKP" => Ok(CurrencyCode::Fkp),
            "FRF" => Ok(CurrencyCode::Frf),
            "GBP" => Ok(CurrencyCode::Gbp),
            "GEK" => Ok(CurrencyCode::Gek),
            "GEL" => Ok(CurrencyCode::Gel),
            "GHC" => Ok(CurrencyCode::Ghc),
            "GHP" => Ok(CurrencyCode::Ghp),
            "GHS" => Ok(CurrencyCode::Ghs),
            "GIP" => Ok(CurrencyCode::Gip),
            "GMD" => Ok(CurrencyCode::Gmd),
            "GNE" => Ok(CurrencyCode::Gne),
            "GNF" => Ok(CurrencyCode::Gnf),
            "GNS" => Ok(CurrencyCode::Gns),
            "GQE" => Ok(CurrencyCode::Gqe),
            "GRD" => Ok(CurrencyCode::Grd),
            "GTQ" => Ok(CurrencyCode::Gtq),
            "GWE" => Ok(CurrencyCode::Gwe),
            "GWP" => Ok(CurrencyCode::Gwp),
            "GYD" => Ok(CurrencyCode::Gyd),
            "HKD" => Ok(CurrencyCode::Hkd),
            "HNL" => Ok(CurrencyCode::Hnl),
            "HRD" => Ok(CurrencyCode::Hrd),
            "HRK" => Ok(CurrencyCode::Hrk),
            "HTG" => Ok(CurrencyCode::Htg),
            "HUF" => Ok(CurrencyCode::Huf),
            "IDR" => Ok(CurrencyCode::Idr),
            "IEP" => Ok(CurrencyCode::Iep),
            "ILP" => Ok(CurrencyCode::Ilp),
            "ILR" => Ok(CurrencyCode::Ilr),
            "ILS" => Ok(CurrencyCode::Ils),
            "INR" => Ok(CurrencyCode::Inr),
            "IQD" => Ok(CurrencyCode::Iqd),
            "IRR" => Ok(CurrencyCode::Irr),
            "ISJ" => Ok(CurrencyCode::Isj),
            "ISK" => Ok(CurrencyCode::Isk),
            "ITL" => Ok(CurrencyCode::Itl),
            "JMD" => Ok(CurrencyCode::Jmd),
            "JOD" => Ok(CurrencyCode::Jod),
            "JPY" => Ok(CurrencyCode::Jpy),
            "KES" => Ok(CurrencyCode::Kes),
            "KGS" => Ok(CurrencyCode::Kgs),
            "KHR" => Ok(CurrencyCode::Khr),
            "KMF" => Ok(CurrencyCode::Kmf),
            "KPW" => Ok(CurrencyCode::Kpw),
            "KRW" => Ok(CurrencyCode::Krw),
            "KWD" => Ok(CurrencyCode::Kwd),
            "KYD" => Ok(CurrencyCode::Kyd),
            "KZT" => Ok(CurrencyCode::Kzt),
            "LAJ" => Ok(CurrencyCode::Laj),
            "LAK" => Ok(CurrencyCode::Lak),
            "LBP" => Ok(CurrencyCode::Lbp),
            "LKR" => Ok(CurrencyCode::Lkr),
            "LRD" => Ok(CurrencyCode::Lrd),
            "LSL" => Ok(CurrencyCode::Lsl),
            "LSM" => Ok(CurrencyCode::Lsm),
            "LTL" => Ok(CurrencyCode::Ltl),
            "LTT" => Ok(CurrencyCode::Ltt),
            "LUC" => Ok(CurrencyCode::Luc),
            "LUF" => Ok(CurrencyCode::Luf),
            "LUL" => Ok(CurrencyCode::Lul),
            "LVL" => Ok(CurrencyCode::Lvl),
            "LVR" => Ok(CurrencyCode::Lvr),
            "LYD" => Ok(CurrencyCode::Lyd),
            "MAD" => Ok(CurrencyCode::Mad),
            "MDL" => Ok(CurrencyCode::Mdl),
            "MGA" => Ok(CurrencyCode::Mga),
            "MGF" => Ok(CurrencyCode::Mgf),
            "MKD" => Ok(CurrencyCode::Mkd),
            "MLF" => Ok(CurrencyCode::Mlf),
            "MMK" => Ok(CurrencyCode::Mmk),
            "MNT" => Ok(CurrencyCode::Mnt),
            "MOP" => Ok(CurrencyCode::Mop),
            "MRO" => Ok(CurrencyCode::Mro),
            "MRU" => Ok(CurrencyCode::Mru),
            "MTL" => Ok(CurrencyCode::Mtl),
            "MTP" => Ok(CurrencyCode::Mtp),
            "MUR" => Ok(CurrencyCode::Mur),
            "MVQ" => Ok(CurrencyCode::Mvq),
            "MVR" => Ok(CurrencyCode::Mvr),
            "MWK" => Ok(CurrencyCode::Mwk),
            "MXN" => Ok(CurrencyCode::Mxn),
            "MXP" => Ok(CurrencyCode::Mxp),
            "MXV" => Ok(CurrencyCode::Mxv),
            "MYR" => Ok(CurrencyCode::Myr),
            "MZE" => Ok(CurrencyCode::Mze),
            "MZM" => Ok(CurrencyCode::Mzm),
            "MZN" => Ok(CurrencyCode::Mzn),
            "NAD" => Ok(CurrencyCode::Nad),
            "NGN" => Ok(CurrencyCode::Ngn),
            "NIC" => Ok(CurrencyCode::Nic),
            "NIO" => Ok(CurrencyCode::Nio),
            "NLG" => Ok(CurrencyCode::Nlg),
            "NOK" => Ok(CurrencyCode::Nok),
            "NPR" => Ok(CurrencyCode::Npr),
            "NZD" => Ok(CurrencyCode::Nzd),
            "OMR" => Ok(CurrencyCode::Omr),
            "PAB" => Ok(CurrencyCode::Pab),
            "PEH" => Ok(CurrencyCode::Peh),
            "PEI" => Ok(CurrencyCode::Pei),
            "PEN" => Ok(CurrencyCode::Pen),
            "PES" => Ok(CurrencyCode::Pes),
            "PGK" => Ok(CurrencyCode::Pgk),
            "PHP" => Ok(CurrencyCode::Php),
            "PKR" => Ok(CurrencyCode::Pkr),
            "PLN" => Ok(CurrencyCode::Pln),
            "PLZ" => Ok(CurrencyCode::Plz),
            "PTE" => Ok(CurrencyCode::Pte),
            "PYG" => Ok(CurrencyCode::Pyg),
            "QAR" => Ok(CurrencyCode::Qar),
            "RHD" => Ok(CurrencyCode::Rhd),
            "ROK" => Ok(CurrencyCode::Rok),
            "ROL" => Ok(CurrencyCode::Rol),
            "RON" => Ok(CurrencyCode::Ron),
            "RSD" => Ok(CurrencyCode::Rsd),
            "RUB" => Ok(CurrencyCode::Rub),
            "RUR" => Ok(CurrencyCode::Rur),
            "RWF" => Ok(CurrencyCode::Rwf),
            "SAR" => Ok(CurrencyCode::Sar),
            "SBD" => Ok(CurrencyCode::Sbd),
            "SCR" => Ok(CurrencyCode::Scr),
            "SDD" => Ok(CurrencyCode::Sdd),
            "SDG" => Ok(CurrencyCode::Sdg),
            "SDP" => Ok(CurrencyCode::Sdp),
            "SEK" => Ok(CurrencyCode::Sek),
            "SGD" => Ok(CurrencyCode::Sgd),
            "SHP" => Ok(CurrencyCode::Shp),
            "SIT" => Ok(CurrencyCode::Sit),
            "SKK" => Ok(CurrencyCode::Skk),
            "SLL" => Ok(CurrencyCode::Sll),
            "SOS" => Ok(CurrencyCode::Sos),
            "SRD" => Ok(CurrencyCode::Srd),
            "SRG" => Ok(CurrencyCode::Srg),
            "SSP" => Ok(CurrencyCode::Ssp),
            "STD" => Ok(CurrencyCode::Std),
            "STN" => Ok(CurrencyCode::Stn),
            "SUR" => Ok(CurrencyCode::Sur),
            "SVC" => Ok(CurrencyCode::Svc),
            "SYP" => Ok(CurrencyCode::Syp),
            "SZL" => Ok(CurrencyCode::Szl),
            "THB" => Ok(CurrencyCode::Thb),
            "TJR" => Ok(CurrencyCode::Tjr),
            "TJS" => Ok(CurrencyCode::Tjs),
            "TMM" => Ok(CurrencyCode::Tmm),
            "TMT" => Ok(CurrencyCode::Tmt),
            "TND" => Ok(CurrencyCode::Tnd),
            "TOP" => Ok(CurrencyCode::Top),
            "TPE" => Ok(CurrencyCode::Tpe),
            "TRL" => Ok(CurrencyCode::Trl),
            "TRY" => Ok(CurrencyCode::Try),
            "TTD" => Ok(CurrencyCode::Ttd),
            "TWD" => Ok(CurrencyCode::Twd),
            "TZS" => Ok(CurrencyCode::Tzs),
            "UAH" => Ok(CurrencyCode::Uah),
            "UAK" => Ok(CurrencyCode::Uak),
            "UGS" => Ok(CurrencyCode::Ugs),
            "UGW" => Ok(CurrencyCode::Ugw),
            "UGX" => Ok(CurrencyCode::Ugx),
            "USD" => Ok(CurrencyCode::Usd),
            "USN" => Ok(CurrencyCode::Usn),
            "USS" => Ok(CurrencyCode::Uss),
            "UYI" => Ok(CurrencyCode::Uyi),
            "UYN" => Ok(CurrencyCode::Uyn),
            "UYP" => Ok(CurrencyCode::Uyp),
            "UYU" => Ok(CurrencyCode::Uyu),
            "UYW" => Ok(CurrencyCode::Uyw),
            "UZS" => Ok(CurrencyCode::Uzs),
            "VEB" => Ok(CurrencyCode::Veb),
            "VEF" => Ok(CurrencyCode::Vef),
            "VES" => Ok(CurrencyCode::Ves),
            "VNC" => Ok(CurrencyCode::Vnc),
            "VND" => Ok(CurrencyCode::Vnd),
            "VUV" => Ok(CurrencyCode::Vuv),
            "WST" => Ok(CurrencyCode::Wst),
            "XAF" => Ok(CurrencyCode::Xaf),
            "XAG" => Ok(CurrencyCode::Xag),
            "XAU" => Ok(CurrencyCode::Xau),
            "XBA" => Ok(CurrencyCode::Xba),
            "XBB" => Ok(CurrencyCode::Xbb),
            "XBC" => Ok(CurrencyCode::Xbc),
            "XBD" => Ok(CurrencyCode::Xbd),
            "XCD" => Ok(CurrencyCode::Xcd),
            "XDR" => Ok(CurrencyCode::Xdr),
            "XEU" => Ok(CurrencyCode::Xeu),
            "XFO" => Ok(CurrencyCode::Xfo),
            "XFU" => Ok(CurrencyCode::Xfu),
            "XOF" => Ok(CurrencyCode::Xof),
            "XPD" => Ok(CurrencyCode::Xpd),
            "XPF" => Ok(CurrencyCode::Xpf),
            "XPT" => Ok(CurrencyCode::Xpt),
            "XRE" => Ok(CurrencyCode::Xre),
            "XSU" => Ok(CurrencyCode::Xsu),
            "XTS" => Ok(CurrencyCode::Xts),
            "XUA" => Ok(CurrencyCode::Xua),
            "XXX" => Ok(CurrencyCode::Xxx),
            "YDD" => Ok(CurrencyCode::Ydd),
            "YER" => Ok(CurrencyCode::Yer),
            "YUD" => Ok(CurrencyCode::Yud),
            "YUM" => Ok(CurrencyCode::Yum),
            "YUN" => Ok(CurrencyCode::Yun),
            "ZAL" => Ok(CurrencyCode::Zal),
            "ZAR" => Ok(CurrencyCode::Zar),
            "ZMK" => Ok(CurrencyCode::Zmk),
            "ZMW" => Ok(CurrencyCode::Zmw),
            "ZRN" => Ok(CurrencyCode::Zrn),
            "ZRZ" => Ok(CurrencyCode::Zrz),
            "ZWC" => Ok(CurrencyCode::Zwc),
            "ZWD" => Ok(CurrencyCode::Zwd),
            "ZWL" => Ok(CurrencyCode::Zwl),
            "ZWN" => Ok(CurrencyCode::Zwn),
            "ZWR" => Ok(CurrencyCode::Zwr),
            _ => Err(ThothError::InvalidCurrencyCode(input.to_string())),
        }
    }
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
