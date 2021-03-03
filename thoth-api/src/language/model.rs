use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
#[cfg(feature = "backend")]
use crate::schema::language;
#[cfg(feature = "backend")]
use crate::schema::language_history;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Language_relation")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LanguageRelation {
    Original,
    #[cfg_attr(feature = "backend", db_rename = "translated-from")]
    TranslatedFrom,
    #[cfg_attr(feature = "backend", db_rename = "translated-into")]
    TranslatedInto,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting languages list")
)]
pub enum LanguageField {
    LanguageID,
    WorkID,
    LanguageCode,
    LanguageRelation,
    MainLanguage,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Serialize, Deserialize)]
pub struct Language {
    pub language_id: Uuid,
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "language"
)]
pub struct NewLanguage {
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "language"
)]
pub struct PatchLanguage {
    pub language_id: Uuid,
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
}

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Language_code")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LanguageCode {
    Aar,
    Abk,
    Ace,
    Ach,
    Ada,
    Ady,
    Afa,
    Afh,
    Afr,
    Ain,
    Aka,
    Akk,
    Alb,
    Ale,
    Alg,
    Alt,
    Amh,
    Ang,
    Anp,
    Apa,
    Ara,
    Arc,
    Arg,
    Arm,
    Arn,
    Arp,
    Art,
    Arw,
    Asm,
    Ast,
    Ath,
    Aus,
    Ava,
    Ave,
    Awa,
    Aym,
    Aze,
    Bad,
    Bai,
    Bak,
    Bal,
    Bam,
    Ban,
    Baq,
    Bas,
    Bat,
    Bej,
    Bel,
    Bem,
    Ben,
    Ber,
    Bho,
    Bih,
    Bik,
    Bin,
    Bis,
    Bla,
    Bnt,
    Bos,
    Bra,
    Bre,
    Btk,
    Bua,
    Bug,
    Bul,
    Bur,
    Byn,
    Cad,
    Cai,
    Car,
    Cat,
    Cau,
    Ceb,
    Cel,
    Cha,
    Chb,
    Che,
    Chg,
    Chi,
    Chk,
    Chm,
    Chn,
    Cho,
    Chp,
    Chr,
    Chu,
    Chv,
    Chy,
    Cmc,
    Cnr,
    Cop,
    Cor,
    Cos,
    Cpe,
    Cpf,
    Cpp,
    Cre,
    Crh,
    Crp,
    Csb,
    Cus,
    Cze,
    Dak,
    Dan,
    Dar,
    Day,
    Del,
    Den,
    Dgr,
    Din,
    Div,
    Doi,
    Dra,
    Dsb,
    Dua,
    Dum,
    Dut,
    Dyu,
    Dzo,
    Efi,
    Egy,
    Eka,
    Elx,
    Eng,
    Enm,
    Epo,
    Est,
    Ewe,
    Ewo,
    Fan,
    Fao,
    Fat,
    Fij,
    Fil,
    Fin,
    Fiu,
    Fon,
    Fre,
    Frm,
    Fro,
    Frr,
    Frs,
    Fry,
    Ful,
    Fur,
    Gaa,
    Gay,
    Gba,
    Gem,
    Geo,
    Ger,
    Gez,
    Gil,
    Gla,
    Gle,
    Glg,
    Glv,
    Gmh,
    Goh,
    Gon,
    Gor,
    Got,
    Grb,
    Grc,
    Gre,
    Grn,
    Gsw,
    Guj,
    Gwi,
    Hai,
    Hat,
    Hau,
    Haw,
    Heb,
    Her,
    Hil,
    Him,
    Hin,
    Hit,
    Hmn,
    Hmo,
    Hrv,
    Hsb,
    Hun,
    Hup,
    Iba,
    Ibo,
    Ice,
    Ido,
    Iii,
    Ijo,
    Iku,
    Ile,
    Ilo,
    Ina,
    Inc,
    Ind,
    Ine,
    Inh,
    Ipk,
    Ira,
    Iro,
    Ita,
    Jav,
    Jbo,
    Jpn,
    Jpr,
    Jrb,
    Kaa,
    Kab,
    Kac,
    Kal,
    Kam,
    Kan,
    Kar,
    Kas,
    Kau,
    Kaw,
    Kaz,
    Kbd,
    Kha,
    Khi,
    Khm,
    Kho,
    Kik,
    Kin,
    Kir,
    Kmb,
    Kok,
    Kom,
    Kon,
    Kor,
    Kos,
    Kpe,
    Krc,
    Krl,
    Kro,
    Kru,
    Kua,
    Kum,
    Kur,
    Kut,
    Lad,
    Lah,
    Lam,
    Lao,
    Lat,
    Lav,
    Lez,
    Lim,
    Lin,
    Lit,
    Lol,
    Loz,
    Ltz,
    Lua,
    Lub,
    Lug,
    Lui,
    Lun,
    Luo,
    Lus,
    Mac,
    Mad,
    Mag,
    Mah,
    Mai,
    Mak,
    Mal,
    Man,
    Mao,
    Map,
    Mar,
    Mas,
    May,
    Mdf,
    Mdr,
    Men,
    Mga,
    Mic,
    Min,
    Mis,
    Mkh,
    Mlg,
    Mlt,
    Mnc,
    Mni,
    Mno,
    Moh,
    Mon,
    Mos,
    Mul,
    Mun,
    Mus,
    Mwl,
    Mwr,
    Myn,
    Myv,
    Nah,
    Nai,
    Nap,
    Nau,
    Nav,
    Nbl,
    Nde,
    Ndo,
    Nds,
    Nep,
    New,
    Nia,
    Nic,
    Niu,
    Nno,
    Nob,
    Nog,
    Non,
    Nor,
    Nqo,
    Nso,
    Nub,
    Nwc,
    Nya,
    Nym,
    Nyn,
    Nyo,
    Nzi,
    Oci,
    Oji,
    Ori,
    Orm,
    Osa,
    Oss,
    Ota,
    Oto,
    Paa,
    Pag,
    Pal,
    Pam,
    Pan,
    Pap,
    Pau,
    Peo,
    Per,
    Phi,
    Phn,
    Pli,
    Pol,
    Pon,
    Por,
    Pra,
    Pro,
    Pus,
    Qaa,
    Que,
    Raj,
    Rap,
    Rar,
    Roa,
    Roh,
    Rom,
    Rum,
    Run,
    Rup,
    Rus,
    Sad,
    Sag,
    Sah,
    Sai,
    Sal,
    Sam,
    San,
    Sas,
    Sat,
    Scn,
    Sco,
    Sel,
    Sem,
    Sga,
    Sgn,
    Shn,
    Sid,
    Sin,
    Sio,
    Sit,
    Sla,
    Slo,
    Slv,
    Sma,
    Sme,
    Smi,
    Smj,
    Smn,
    Smo,
    Sms,
    Sna,
    Snd,
    Snk,
    Sog,
    Som,
    Son,
    Sot,
    Spa,
    Srd,
    Srn,
    Srp,
    Srr,
    Ssa,
    Ssw,
    Suk,
    Sun,
    Sus,
    Sux,
    Swa,
    Swe,
    Syc,
    Syr,
    Tah,
    Tai,
    Tam,
    Tat,
    Tel,
    Tem,
    Ter,
    Tet,
    Tgk,
    Tgl,
    Tha,
    Tib,
    Tig,
    Tir,
    Tiv,
    Tkl,
    Tlh,
    Tli,
    Tmh,
    Tog,
    Ton,
    Tpi,
    Tsi,
    Tsn,
    Tso,
    Tuk,
    Tum,
    Tup,
    Tur,
    Tut,
    Tvl,
    Twi,
    Tyv,
    Udm,
    Uga,
    Uig,
    Ukr,
    Umb,
    Und,
    Urd,
    Uzb,
    Vai,
    Ven,
    Vie,
    Vol,
    Vot,
    Wak,
    Wal,
    War,
    Was,
    Wel,
    Wen,
    Wln,
    Wol,
    Xal,
    Xho,
    Yao,
    Yap,
    Yid,
    Yor,
    Ypk,
    Zap,
    Zbl,
    Zen,
    Zgh,
    Zha,
    Znd,
    Zul,
    Zun,
    Zxx,
    Zza,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct LanguageHistory {
    pub language_history_id: Uuid,
    pub language_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "language_history"
)]
pub struct NewLanguageHistory {
    pub language_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

impl Default for LanguageCode {
    fn default() -> LanguageCode {
        LanguageCode::Eng
    }
}

impl Default for LanguageRelation {
    fn default() -> LanguageRelation {
        LanguageRelation::Original
    }
}

impl fmt::Display for LanguageRelation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LanguageRelation::Original => write!(f, "Original"),
            LanguageRelation::TranslatedFrom => write!(f, "Translated From"),
            LanguageRelation::TranslatedInto => write!(f, "Translated Into"),
        }
    }
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LanguageCode::Aar => write!(f, "AAR"),
            LanguageCode::Abk => write!(f, "ABK"),
            LanguageCode::Ace => write!(f, "ACE"),
            LanguageCode::Ach => write!(f, "ACH"),
            LanguageCode::Ada => write!(f, "ADA"),
            LanguageCode::Ady => write!(f, "ADY"),
            LanguageCode::Afa => write!(f, "AFA"),
            LanguageCode::Afh => write!(f, "AFH"),
            LanguageCode::Afr => write!(f, "AFR"),
            LanguageCode::Ain => write!(f, "AIN"),
            LanguageCode::Aka => write!(f, "AKA"),
            LanguageCode::Akk => write!(f, "AKK"),
            LanguageCode::Alb => write!(f, "ALB"),
            LanguageCode::Ale => write!(f, "ALE"),
            LanguageCode::Alg => write!(f, "ALG"),
            LanguageCode::Alt => write!(f, "ALT"),
            LanguageCode::Amh => write!(f, "AMH"),
            LanguageCode::Ang => write!(f, "ANG"),
            LanguageCode::Anp => write!(f, "ANP"),
            LanguageCode::Apa => write!(f, "APA"),
            LanguageCode::Ara => write!(f, "ARA"),
            LanguageCode::Arc => write!(f, "ARC"),
            LanguageCode::Arg => write!(f, "ARG"),
            LanguageCode::Arm => write!(f, "ARM"),
            LanguageCode::Arn => write!(f, "ARN"),
            LanguageCode::Arp => write!(f, "ARP"),
            LanguageCode::Art => write!(f, "ART"),
            LanguageCode::Arw => write!(f, "ARW"),
            LanguageCode::Asm => write!(f, "ASM"),
            LanguageCode::Ast => write!(f, "AST"),
            LanguageCode::Ath => write!(f, "ATH"),
            LanguageCode::Aus => write!(f, "AUS"),
            LanguageCode::Ava => write!(f, "AVA"),
            LanguageCode::Ave => write!(f, "AVE"),
            LanguageCode::Awa => write!(f, "AWA"),
            LanguageCode::Aym => write!(f, "AYM"),
            LanguageCode::Aze => write!(f, "AZE"),
            LanguageCode::Bad => write!(f, "BAD"),
            LanguageCode::Bai => write!(f, "BAI"),
            LanguageCode::Bak => write!(f, "BAK"),
            LanguageCode::Bal => write!(f, "BAL"),
            LanguageCode::Bam => write!(f, "BAM"),
            LanguageCode::Ban => write!(f, "BAN"),
            LanguageCode::Baq => write!(f, "BAQ"),
            LanguageCode::Bas => write!(f, "BAS"),
            LanguageCode::Bat => write!(f, "BAT"),
            LanguageCode::Bej => write!(f, "BEJ"),
            LanguageCode::Bel => write!(f, "BEL"),
            LanguageCode::Bem => write!(f, "BEM"),
            LanguageCode::Ben => write!(f, "BEN"),
            LanguageCode::Ber => write!(f, "BER"),
            LanguageCode::Bho => write!(f, "BHO"),
            LanguageCode::Bih => write!(f, "BIH"),
            LanguageCode::Bik => write!(f, "BIK"),
            LanguageCode::Bin => write!(f, "BIN"),
            LanguageCode::Bis => write!(f, "BIS"),
            LanguageCode::Bla => write!(f, "BLA"),
            LanguageCode::Bnt => write!(f, "BNT"),
            LanguageCode::Bos => write!(f, "BOS"),
            LanguageCode::Bra => write!(f, "BRA"),
            LanguageCode::Bre => write!(f, "BRE"),
            LanguageCode::Btk => write!(f, "BTK"),
            LanguageCode::Bua => write!(f, "BUA"),
            LanguageCode::Bug => write!(f, "BUG"),
            LanguageCode::Bul => write!(f, "BUL"),
            LanguageCode::Bur => write!(f, "BUR"),
            LanguageCode::Byn => write!(f, "BYN"),
            LanguageCode::Cad => write!(f, "CAD"),
            LanguageCode::Cai => write!(f, "CAI"),
            LanguageCode::Car => write!(f, "CAR"),
            LanguageCode::Cat => write!(f, "CAT"),
            LanguageCode::Cau => write!(f, "CAU"),
            LanguageCode::Ceb => write!(f, "CEB"),
            LanguageCode::Cel => write!(f, "CEL"),
            LanguageCode::Cha => write!(f, "CHA"),
            LanguageCode::Chb => write!(f, "CHB"),
            LanguageCode::Che => write!(f, "CHE"),
            LanguageCode::Chg => write!(f, "CHG"),
            LanguageCode::Chi => write!(f, "CHI"),
            LanguageCode::Chk => write!(f, "CHK"),
            LanguageCode::Chm => write!(f, "CHM"),
            LanguageCode::Chn => write!(f, "CHN"),
            LanguageCode::Cho => write!(f, "CHO"),
            LanguageCode::Chp => write!(f, "CHP"),
            LanguageCode::Chr => write!(f, "CHR"),
            LanguageCode::Chu => write!(f, "CHU"),
            LanguageCode::Chv => write!(f, "CHV"),
            LanguageCode::Chy => write!(f, "CHY"),
            LanguageCode::Cmc => write!(f, "CMC"),
            LanguageCode::Cnr => write!(f, "CNR"),
            LanguageCode::Cop => write!(f, "COP"),
            LanguageCode::Cor => write!(f, "COR"),
            LanguageCode::Cos => write!(f, "COS"),
            LanguageCode::Cpe => write!(f, "CPE"),
            LanguageCode::Cpf => write!(f, "CPF"),
            LanguageCode::Cpp => write!(f, "CPP"),
            LanguageCode::Cre => write!(f, "CRE"),
            LanguageCode::Crh => write!(f, "CRH"),
            LanguageCode::Crp => write!(f, "CRP"),
            LanguageCode::Csb => write!(f, "CSB"),
            LanguageCode::Cus => write!(f, "CUS"),
            LanguageCode::Cze => write!(f, "CZE"),
            LanguageCode::Dak => write!(f, "DAK"),
            LanguageCode::Dan => write!(f, "DAN"),
            LanguageCode::Dar => write!(f, "DAR"),
            LanguageCode::Day => write!(f, "DAY"),
            LanguageCode::Del => write!(f, "DEL"),
            LanguageCode::Den => write!(f, "DEN"),
            LanguageCode::Dgr => write!(f, "DGR"),
            LanguageCode::Din => write!(f, "DIN"),
            LanguageCode::Div => write!(f, "DIV"),
            LanguageCode::Doi => write!(f, "DOI"),
            LanguageCode::Dra => write!(f, "DRA"),
            LanguageCode::Dsb => write!(f, "DSB"),
            LanguageCode::Dua => write!(f, "DUA"),
            LanguageCode::Dum => write!(f, "DUM"),
            LanguageCode::Dut => write!(f, "DUT"),
            LanguageCode::Dyu => write!(f, "DYU"),
            LanguageCode::Dzo => write!(f, "DZO"),
            LanguageCode::Efi => write!(f, "EFI"),
            LanguageCode::Egy => write!(f, "EGY"),
            LanguageCode::Eka => write!(f, "EKA"),
            LanguageCode::Elx => write!(f, "ELX"),
            LanguageCode::Eng => write!(f, "ENG"),
            LanguageCode::Enm => write!(f, "ENM"),
            LanguageCode::Epo => write!(f, "EPO"),
            LanguageCode::Est => write!(f, "EST"),
            LanguageCode::Ewe => write!(f, "EWE"),
            LanguageCode::Ewo => write!(f, "EWO"),
            LanguageCode::Fan => write!(f, "FAN"),
            LanguageCode::Fao => write!(f, "FAO"),
            LanguageCode::Fat => write!(f, "FAT"),
            LanguageCode::Fij => write!(f, "FIJ"),
            LanguageCode::Fil => write!(f, "FIL"),
            LanguageCode::Fin => write!(f, "FIN"),
            LanguageCode::Fiu => write!(f, "FIU"),
            LanguageCode::Fon => write!(f, "FON"),
            LanguageCode::Fre => write!(f, "FRE"),
            LanguageCode::Frm => write!(f, "FRM"),
            LanguageCode::Fro => write!(f, "FRO"),
            LanguageCode::Frr => write!(f, "FRR"),
            LanguageCode::Frs => write!(f, "FRS"),
            LanguageCode::Fry => write!(f, "FRY"),
            LanguageCode::Ful => write!(f, "FUL"),
            LanguageCode::Fur => write!(f, "FUR"),
            LanguageCode::Gaa => write!(f, "GAA"),
            LanguageCode::Gay => write!(f, "GAY"),
            LanguageCode::Gba => write!(f, "GBA"),
            LanguageCode::Gem => write!(f, "GEM"),
            LanguageCode::Geo => write!(f, "GEO"),
            LanguageCode::Ger => write!(f, "GER"),
            LanguageCode::Gez => write!(f, "GEZ"),
            LanguageCode::Gil => write!(f, "GIL"),
            LanguageCode::Gla => write!(f, "GLA"),
            LanguageCode::Gle => write!(f, "GLE"),
            LanguageCode::Glg => write!(f, "GLG"),
            LanguageCode::Glv => write!(f, "GLV"),
            LanguageCode::Gmh => write!(f, "GMH"),
            LanguageCode::Goh => write!(f, "GOH"),
            LanguageCode::Gon => write!(f, "GON"),
            LanguageCode::Gor => write!(f, "GOR"),
            LanguageCode::Got => write!(f, "GOT"),
            LanguageCode::Grb => write!(f, "GRB"),
            LanguageCode::Grc => write!(f, "GRC"),
            LanguageCode::Gre => write!(f, "GRE"),
            LanguageCode::Grn => write!(f, "GRN"),
            LanguageCode::Gsw => write!(f, "GSW"),
            LanguageCode::Guj => write!(f, "GUJ"),
            LanguageCode::Gwi => write!(f, "GWI"),
            LanguageCode::Hai => write!(f, "HAI"),
            LanguageCode::Hat => write!(f, "HAT"),
            LanguageCode::Hau => write!(f, "HAU"),
            LanguageCode::Haw => write!(f, "HAW"),
            LanguageCode::Heb => write!(f, "HEB"),
            LanguageCode::Her => write!(f, "HER"),
            LanguageCode::Hil => write!(f, "HIL"),
            LanguageCode::Him => write!(f, "HIM"),
            LanguageCode::Hin => write!(f, "HIN"),
            LanguageCode::Hit => write!(f, "HIT"),
            LanguageCode::Hmn => write!(f, "HMN"),
            LanguageCode::Hmo => write!(f, "HMO"),
            LanguageCode::Hrv => write!(f, "HRV"),
            LanguageCode::Hsb => write!(f, "HSB"),
            LanguageCode::Hun => write!(f, "HUN"),
            LanguageCode::Hup => write!(f, "HUP"),
            LanguageCode::Iba => write!(f, "IBA"),
            LanguageCode::Ibo => write!(f, "IBO"),
            LanguageCode::Ice => write!(f, "ICE"),
            LanguageCode::Ido => write!(f, "IDO"),
            LanguageCode::Iii => write!(f, "III"),
            LanguageCode::Ijo => write!(f, "IJO"),
            LanguageCode::Iku => write!(f, "IKU"),
            LanguageCode::Ile => write!(f, "ILE"),
            LanguageCode::Ilo => write!(f, "ILO"),
            LanguageCode::Ina => write!(f, "INA"),
            LanguageCode::Inc => write!(f, "INC"),
            LanguageCode::Ind => write!(f, "IND"),
            LanguageCode::Ine => write!(f, "INE"),
            LanguageCode::Inh => write!(f, "INH"),
            LanguageCode::Ipk => write!(f, "IPK"),
            LanguageCode::Ira => write!(f, "IRA"),
            LanguageCode::Iro => write!(f, "IRO"),
            LanguageCode::Ita => write!(f, "ITA"),
            LanguageCode::Jav => write!(f, "JAV"),
            LanguageCode::Jbo => write!(f, "JBO"),
            LanguageCode::Jpn => write!(f, "JPN"),
            LanguageCode::Jpr => write!(f, "JPR"),
            LanguageCode::Jrb => write!(f, "JRB"),
            LanguageCode::Kaa => write!(f, "KAA"),
            LanguageCode::Kab => write!(f, "KAB"),
            LanguageCode::Kac => write!(f, "KAC"),
            LanguageCode::Kal => write!(f, "KAL"),
            LanguageCode::Kam => write!(f, "KAM"),
            LanguageCode::Kan => write!(f, "KAN"),
            LanguageCode::Kar => write!(f, "KAR"),
            LanguageCode::Kas => write!(f, "KAS"),
            LanguageCode::Kau => write!(f, "KAU"),
            LanguageCode::Kaw => write!(f, "KAW"),
            LanguageCode::Kaz => write!(f, "KAZ"),
            LanguageCode::Kbd => write!(f, "KBD"),
            LanguageCode::Kha => write!(f, "KHA"),
            LanguageCode::Khi => write!(f, "KHI"),
            LanguageCode::Khm => write!(f, "KHM"),
            LanguageCode::Kho => write!(f, "KHO"),
            LanguageCode::Kik => write!(f, "KIK"),
            LanguageCode::Kin => write!(f, "KIN"),
            LanguageCode::Kir => write!(f, "KIR"),
            LanguageCode::Kmb => write!(f, "KMB"),
            LanguageCode::Kok => write!(f, "KOK"),
            LanguageCode::Kom => write!(f, "KOM"),
            LanguageCode::Kon => write!(f, "KON"),
            LanguageCode::Kor => write!(f, "KOR"),
            LanguageCode::Kos => write!(f, "KOS"),
            LanguageCode::Kpe => write!(f, "KPE"),
            LanguageCode::Krc => write!(f, "KRC"),
            LanguageCode::Krl => write!(f, "KRL"),
            LanguageCode::Kro => write!(f, "KRO"),
            LanguageCode::Kru => write!(f, "KRU"),
            LanguageCode::Kua => write!(f, "KUA"),
            LanguageCode::Kum => write!(f, "KUM"),
            LanguageCode::Kur => write!(f, "KUR"),
            LanguageCode::Kut => write!(f, "KUT"),
            LanguageCode::Lad => write!(f, "LAD"),
            LanguageCode::Lah => write!(f, "LAH"),
            LanguageCode::Lam => write!(f, "LAM"),
            LanguageCode::Lao => write!(f, "LAO"),
            LanguageCode::Lat => write!(f, "LAT"),
            LanguageCode::Lav => write!(f, "LAV"),
            LanguageCode::Lez => write!(f, "LEZ"),
            LanguageCode::Lim => write!(f, "LIM"),
            LanguageCode::Lin => write!(f, "LIN"),
            LanguageCode::Lit => write!(f, "LIT"),
            LanguageCode::Lol => write!(f, "LOL"),
            LanguageCode::Loz => write!(f, "LOZ"),
            LanguageCode::Ltz => write!(f, "LTZ"),
            LanguageCode::Lua => write!(f, "LUA"),
            LanguageCode::Lub => write!(f, "LUB"),
            LanguageCode::Lug => write!(f, "LUG"),
            LanguageCode::Lui => write!(f, "LUI"),
            LanguageCode::Lun => write!(f, "LUN"),
            LanguageCode::Luo => write!(f, "LUO"),
            LanguageCode::Lus => write!(f, "LUS"),
            LanguageCode::Mac => write!(f, "MAC"),
            LanguageCode::Mad => write!(f, "MAD"),
            LanguageCode::Mag => write!(f, "MAG"),
            LanguageCode::Mah => write!(f, "MAH"),
            LanguageCode::Mai => write!(f, "MAI"),
            LanguageCode::Mak => write!(f, "MAK"),
            LanguageCode::Mal => write!(f, "MAL"),
            LanguageCode::Man => write!(f, "MAN"),
            LanguageCode::Mao => write!(f, "MAO"),
            LanguageCode::Map => write!(f, "MAP"),
            LanguageCode::Mar => write!(f, "MAR"),
            LanguageCode::Mas => write!(f, "MAS"),
            LanguageCode::May => write!(f, "MAY"),
            LanguageCode::Mdf => write!(f, "MDF"),
            LanguageCode::Mdr => write!(f, "MDR"),
            LanguageCode::Men => write!(f, "MEN"),
            LanguageCode::Mga => write!(f, "MGA"),
            LanguageCode::Mic => write!(f, "MIC"),
            LanguageCode::Min => write!(f, "MIN"),
            LanguageCode::Mis => write!(f, "MIS"),
            LanguageCode::Mkh => write!(f, "MKH"),
            LanguageCode::Mlg => write!(f, "MLG"),
            LanguageCode::Mlt => write!(f, "MLT"),
            LanguageCode::Mnc => write!(f, "MNC"),
            LanguageCode::Mni => write!(f, "MNI"),
            LanguageCode::Mno => write!(f, "MNO"),
            LanguageCode::Moh => write!(f, "MOH"),
            LanguageCode::Mon => write!(f, "MON"),
            LanguageCode::Mos => write!(f, "MOS"),
            LanguageCode::Mul => write!(f, "MUL"),
            LanguageCode::Mun => write!(f, "MUN"),
            LanguageCode::Mus => write!(f, "MUS"),
            LanguageCode::Mwl => write!(f, "MWL"),
            LanguageCode::Mwr => write!(f, "MWR"),
            LanguageCode::Myn => write!(f, "MYN"),
            LanguageCode::Myv => write!(f, "MYV"),
            LanguageCode::Nah => write!(f, "NAH"),
            LanguageCode::Nai => write!(f, "NAI"),
            LanguageCode::Nap => write!(f, "NAP"),
            LanguageCode::Nau => write!(f, "NAU"),
            LanguageCode::Nav => write!(f, "NAV"),
            LanguageCode::Nbl => write!(f, "NBL"),
            LanguageCode::Nde => write!(f, "NDE"),
            LanguageCode::Ndo => write!(f, "NDO"),
            LanguageCode::Nds => write!(f, "NDS"),
            LanguageCode::Nep => write!(f, "NEP"),
            LanguageCode::New => write!(f, "NEW"),
            LanguageCode::Nia => write!(f, "NIA"),
            LanguageCode::Nic => write!(f, "NIC"),
            LanguageCode::Niu => write!(f, "NIU"),
            LanguageCode::Nno => write!(f, "NNO"),
            LanguageCode::Nob => write!(f, "NOB"),
            LanguageCode::Nog => write!(f, "NOG"),
            LanguageCode::Non => write!(f, "NON"),
            LanguageCode::Nor => write!(f, "NOR"),
            LanguageCode::Nqo => write!(f, "NQO"),
            LanguageCode::Nso => write!(f, "NSO"),
            LanguageCode::Nub => write!(f, "NUB"),
            LanguageCode::Nwc => write!(f, "NWC"),
            LanguageCode::Nya => write!(f, "NYA"),
            LanguageCode::Nym => write!(f, "NYM"),
            LanguageCode::Nyn => write!(f, "NYN"),
            LanguageCode::Nyo => write!(f, "NYO"),
            LanguageCode::Nzi => write!(f, "NZI"),
            LanguageCode::Oci => write!(f, "OCI"),
            LanguageCode::Oji => write!(f, "OJI"),
            LanguageCode::Ori => write!(f, "ORI"),
            LanguageCode::Orm => write!(f, "ORM"),
            LanguageCode::Osa => write!(f, "OSA"),
            LanguageCode::Oss => write!(f, "OSS"),
            LanguageCode::Ota => write!(f, "OTA"),
            LanguageCode::Oto => write!(f, "OTO"),
            LanguageCode::Paa => write!(f, "PAA"),
            LanguageCode::Pag => write!(f, "PAG"),
            LanguageCode::Pal => write!(f, "PAL"),
            LanguageCode::Pam => write!(f, "PAM"),
            LanguageCode::Pan => write!(f, "PAN"),
            LanguageCode::Pap => write!(f, "PAP"),
            LanguageCode::Pau => write!(f, "PAU"),
            LanguageCode::Peo => write!(f, "PEO"),
            LanguageCode::Per => write!(f, "PER"),
            LanguageCode::Phi => write!(f, "PHI"),
            LanguageCode::Phn => write!(f, "PHN"),
            LanguageCode::Pli => write!(f, "PLI"),
            LanguageCode::Pol => write!(f, "POL"),
            LanguageCode::Pon => write!(f, "PON"),
            LanguageCode::Por => write!(f, "POR"),
            LanguageCode::Pra => write!(f, "PRA"),
            LanguageCode::Pro => write!(f, "PRO"),
            LanguageCode::Pus => write!(f, "PUS"),
            LanguageCode::Qaa => write!(f, "QAA"),
            LanguageCode::Que => write!(f, "QUE"),
            LanguageCode::Raj => write!(f, "RAJ"),
            LanguageCode::Rap => write!(f, "RAP"),
            LanguageCode::Rar => write!(f, "RAR"),
            LanguageCode::Roa => write!(f, "ROA"),
            LanguageCode::Roh => write!(f, "ROH"),
            LanguageCode::Rom => write!(f, "ROM"),
            LanguageCode::Rum => write!(f, "RUM"),
            LanguageCode::Run => write!(f, "RUN"),
            LanguageCode::Rup => write!(f, "RUP"),
            LanguageCode::Rus => write!(f, "RUS"),
            LanguageCode::Sad => write!(f, "SAD"),
            LanguageCode::Sag => write!(f, "SAG"),
            LanguageCode::Sah => write!(f, "SAH"),
            LanguageCode::Sai => write!(f, "SAI"),
            LanguageCode::Sal => write!(f, "SAL"),
            LanguageCode::Sam => write!(f, "SAM"),
            LanguageCode::San => write!(f, "SAN"),
            LanguageCode::Sas => write!(f, "SAS"),
            LanguageCode::Sat => write!(f, "SAT"),
            LanguageCode::Scn => write!(f, "SCN"),
            LanguageCode::Sco => write!(f, "SCO"),
            LanguageCode::Sel => write!(f, "SEL"),
            LanguageCode::Sem => write!(f, "SEM"),
            LanguageCode::Sga => write!(f, "SGA"),
            LanguageCode::Sgn => write!(f, "SGN"),
            LanguageCode::Shn => write!(f, "SHN"),
            LanguageCode::Sid => write!(f, "SID"),
            LanguageCode::Sin => write!(f, "SIN"),
            LanguageCode::Sio => write!(f, "SIO"),
            LanguageCode::Sit => write!(f, "SIT"),
            LanguageCode::Sla => write!(f, "SLA"),
            LanguageCode::Slo => write!(f, "SLO"),
            LanguageCode::Slv => write!(f, "SLV"),
            LanguageCode::Sma => write!(f, "SMA"),
            LanguageCode::Sme => write!(f, "SME"),
            LanguageCode::Smi => write!(f, "SMI"),
            LanguageCode::Smj => write!(f, "SMJ"),
            LanguageCode::Smn => write!(f, "SMN"),
            LanguageCode::Smo => write!(f, "SMO"),
            LanguageCode::Sms => write!(f, "SMS"),
            LanguageCode::Sna => write!(f, "SNA"),
            LanguageCode::Snd => write!(f, "SND"),
            LanguageCode::Snk => write!(f, "SNK"),
            LanguageCode::Sog => write!(f, "SOG"),
            LanguageCode::Som => write!(f, "SOM"),
            LanguageCode::Son => write!(f, "SON"),
            LanguageCode::Sot => write!(f, "SOT"),
            LanguageCode::Spa => write!(f, "SPA"),
            LanguageCode::Srd => write!(f, "SRD"),
            LanguageCode::Srn => write!(f, "SRN"),
            LanguageCode::Srp => write!(f, "SRP"),
            LanguageCode::Srr => write!(f, "SRR"),
            LanguageCode::Ssa => write!(f, "SSA"),
            LanguageCode::Ssw => write!(f, "SSW"),
            LanguageCode::Suk => write!(f, "SUK"),
            LanguageCode::Sun => write!(f, "SUN"),
            LanguageCode::Sus => write!(f, "SUS"),
            LanguageCode::Sux => write!(f, "SUX"),
            LanguageCode::Swa => write!(f, "SWA"),
            LanguageCode::Swe => write!(f, "SWE"),
            LanguageCode::Syc => write!(f, "SYC"),
            LanguageCode::Syr => write!(f, "SYR"),
            LanguageCode::Tah => write!(f, "TAH"),
            LanguageCode::Tai => write!(f, "TAI"),
            LanguageCode::Tam => write!(f, "TAM"),
            LanguageCode::Tat => write!(f, "TAT"),
            LanguageCode::Tel => write!(f, "TEL"),
            LanguageCode::Tem => write!(f, "TEM"),
            LanguageCode::Ter => write!(f, "TER"),
            LanguageCode::Tet => write!(f, "TET"),
            LanguageCode::Tgk => write!(f, "TGK"),
            LanguageCode::Tgl => write!(f, "TGL"),
            LanguageCode::Tha => write!(f, "THA"),
            LanguageCode::Tib => write!(f, "TIB"),
            LanguageCode::Tig => write!(f, "TIG"),
            LanguageCode::Tir => write!(f, "TIR"),
            LanguageCode::Tiv => write!(f, "TIV"),
            LanguageCode::Tkl => write!(f, "TKL"),
            LanguageCode::Tlh => write!(f, "TLH"),
            LanguageCode::Tli => write!(f, "TLI"),
            LanguageCode::Tmh => write!(f, "TMH"),
            LanguageCode::Tog => write!(f, "TOG"),
            LanguageCode::Ton => write!(f, "TON"),
            LanguageCode::Tpi => write!(f, "TPI"),
            LanguageCode::Tsi => write!(f, "TSI"),
            LanguageCode::Tsn => write!(f, "TSN"),
            LanguageCode::Tso => write!(f, "TSO"),
            LanguageCode::Tuk => write!(f, "TUK"),
            LanguageCode::Tum => write!(f, "TUM"),
            LanguageCode::Tup => write!(f, "TUP"),
            LanguageCode::Tur => write!(f, "TUR"),
            LanguageCode::Tut => write!(f, "TUT"),
            LanguageCode::Tvl => write!(f, "TVL"),
            LanguageCode::Twi => write!(f, "TWI"),
            LanguageCode::Tyv => write!(f, "TYV"),
            LanguageCode::Udm => write!(f, "UDM"),
            LanguageCode::Uga => write!(f, "UGA"),
            LanguageCode::Uig => write!(f, "UIG"),
            LanguageCode::Ukr => write!(f, "UKR"),
            LanguageCode::Umb => write!(f, "UMB"),
            LanguageCode::Und => write!(f, "UND"),
            LanguageCode::Urd => write!(f, "URD"),
            LanguageCode::Uzb => write!(f, "UZB"),
            LanguageCode::Vai => write!(f, "VAI"),
            LanguageCode::Ven => write!(f, "VEN"),
            LanguageCode::Vie => write!(f, "VIE"),
            LanguageCode::Vol => write!(f, "VOL"),
            LanguageCode::Vot => write!(f, "VOT"),
            LanguageCode::Wak => write!(f, "WAK"),
            LanguageCode::Wal => write!(f, "WAL"),
            LanguageCode::War => write!(f, "WAR"),
            LanguageCode::Was => write!(f, "WAS"),
            LanguageCode::Wel => write!(f, "WEL"),
            LanguageCode::Wen => write!(f, "WEN"),
            LanguageCode::Wln => write!(f, "WLN"),
            LanguageCode::Wol => write!(f, "WOL"),
            LanguageCode::Xal => write!(f, "XAL"),
            LanguageCode::Xho => write!(f, "XHO"),
            LanguageCode::Yao => write!(f, "YAO"),
            LanguageCode::Yap => write!(f, "YAP"),
            LanguageCode::Yid => write!(f, "YID"),
            LanguageCode::Yor => write!(f, "YOR"),
            LanguageCode::Ypk => write!(f, "YPK"),
            LanguageCode::Zap => write!(f, "ZAP"),
            LanguageCode::Zbl => write!(f, "ZBL"),
            LanguageCode::Zen => write!(f, "ZEN"),
            LanguageCode::Zgh => write!(f, "ZGH"),
            LanguageCode::Zha => write!(f, "ZHA"),
            LanguageCode::Znd => write!(f, "ZND"),
            LanguageCode::Zul => write!(f, "ZUL"),
            LanguageCode::Zun => write!(f, "ZUN"),
            LanguageCode::Zxx => write!(f, "ZXX"),
            LanguageCode::Zza => write!(f, "ZZA"),
        }
    }
}

impl FromStr for LanguageRelation {
    type Err = ThothError;

    fn from_str(input: &str) -> std::result::Result<LanguageRelation, ThothError> {
        match input {
            "Original" => Ok(LanguageRelation::Original),
            "Translated From" => Ok(LanguageRelation::TranslatedFrom),
            "Translated Into" => Ok(LanguageRelation::TranslatedInto),
            _ => Err(ThothError::InvalidLanguageRelation(input.to_string())),
        }
    }
}

impl FromStr for LanguageCode {
    type Err = ThothError;

    fn from_str(input: &str) -> std::result::Result<LanguageCode, ThothError> {
        match input {
            "AAR" => Ok(LanguageCode::Aar),
            "ABK" => Ok(LanguageCode::Abk),
            "ACE" => Ok(LanguageCode::Ace),
            "ACH" => Ok(LanguageCode::Ach),
            "ADA" => Ok(LanguageCode::Ada),
            "ADY" => Ok(LanguageCode::Ady),
            "AFA" => Ok(LanguageCode::Afa),
            "AFH" => Ok(LanguageCode::Afh),
            "AFR" => Ok(LanguageCode::Afr),
            "AIN" => Ok(LanguageCode::Ain),
            "AKA" => Ok(LanguageCode::Aka),
            "AKK" => Ok(LanguageCode::Akk),
            "ALB" => Ok(LanguageCode::Alb),
            "ALE" => Ok(LanguageCode::Ale),
            "ALG" => Ok(LanguageCode::Alg),
            "ALT" => Ok(LanguageCode::Alt),
            "AMH" => Ok(LanguageCode::Amh),
            "ANG" => Ok(LanguageCode::Ang),
            "ANP" => Ok(LanguageCode::Anp),
            "APA" => Ok(LanguageCode::Apa),
            "ARA" => Ok(LanguageCode::Ara),
            "ARC" => Ok(LanguageCode::Arc),
            "ARG" => Ok(LanguageCode::Arg),
            "ARM" => Ok(LanguageCode::Arm),
            "ARN" => Ok(LanguageCode::Arn),
            "ARP" => Ok(LanguageCode::Arp),
            "ART" => Ok(LanguageCode::Art),
            "ARW" => Ok(LanguageCode::Arw),
            "ASM" => Ok(LanguageCode::Asm),
            "AST" => Ok(LanguageCode::Ast),
            "ATH" => Ok(LanguageCode::Ath),
            "AUS" => Ok(LanguageCode::Aus),
            "AVA" => Ok(LanguageCode::Ava),
            "AVE" => Ok(LanguageCode::Ave),
            "AWA" => Ok(LanguageCode::Awa),
            "AYM" => Ok(LanguageCode::Aym),
            "AZE" => Ok(LanguageCode::Aze),
            "BAD" => Ok(LanguageCode::Bad),
            "BAI" => Ok(LanguageCode::Bai),
            "BAK" => Ok(LanguageCode::Bak),
            "BAL" => Ok(LanguageCode::Bal),
            "BAM" => Ok(LanguageCode::Bam),
            "BAN" => Ok(LanguageCode::Ban),
            "BAQ" => Ok(LanguageCode::Baq),
            "BAS" => Ok(LanguageCode::Bas),
            "BAT" => Ok(LanguageCode::Bat),
            "BEJ" => Ok(LanguageCode::Bej),
            "BEL" => Ok(LanguageCode::Bel),
            "BEM" => Ok(LanguageCode::Bem),
            "BEN" => Ok(LanguageCode::Ben),
            "BER" => Ok(LanguageCode::Ber),
            "BHO" => Ok(LanguageCode::Bho),
            "BIH" => Ok(LanguageCode::Bih),
            "BIK" => Ok(LanguageCode::Bik),
            "BIN" => Ok(LanguageCode::Bin),
            "BIS" => Ok(LanguageCode::Bis),
            "BLA" => Ok(LanguageCode::Bla),
            "BNT" => Ok(LanguageCode::Bnt),
            "BOS" => Ok(LanguageCode::Bos),
            "BRA" => Ok(LanguageCode::Bra),
            "BRE" => Ok(LanguageCode::Bre),
            "BTK" => Ok(LanguageCode::Btk),
            "BUA" => Ok(LanguageCode::Bua),
            "BUG" => Ok(LanguageCode::Bug),
            "BUL" => Ok(LanguageCode::Bul),
            "BUR" => Ok(LanguageCode::Bur),
            "BYN" => Ok(LanguageCode::Byn),
            "CAD" => Ok(LanguageCode::Cad),
            "CAI" => Ok(LanguageCode::Cai),
            "CAR" => Ok(LanguageCode::Car),
            "CAT" => Ok(LanguageCode::Cat),
            "CAU" => Ok(LanguageCode::Cau),
            "CEB" => Ok(LanguageCode::Ceb),
            "CEL" => Ok(LanguageCode::Cel),
            "CHA" => Ok(LanguageCode::Cha),
            "CHB" => Ok(LanguageCode::Chb),
            "CHE" => Ok(LanguageCode::Che),
            "CHG" => Ok(LanguageCode::Chg),
            "CHI" => Ok(LanguageCode::Chi),
            "CHK" => Ok(LanguageCode::Chk),
            "CHM" => Ok(LanguageCode::Chm),
            "CHN" => Ok(LanguageCode::Chn),
            "CHO" => Ok(LanguageCode::Cho),
            "CHP" => Ok(LanguageCode::Chp),
            "CHR" => Ok(LanguageCode::Chr),
            "CHU" => Ok(LanguageCode::Chu),
            "CHV" => Ok(LanguageCode::Chv),
            "CHY" => Ok(LanguageCode::Chy),
            "CMC" => Ok(LanguageCode::Cmc),
            "CNR" => Ok(LanguageCode::Cnr),
            "COP" => Ok(LanguageCode::Cop),
            "COR" => Ok(LanguageCode::Cor),
            "COS" => Ok(LanguageCode::Cos),
            "CPE" => Ok(LanguageCode::Cpe),
            "CPF" => Ok(LanguageCode::Cpf),
            "CPP" => Ok(LanguageCode::Cpp),
            "CRE" => Ok(LanguageCode::Cre),
            "CRH" => Ok(LanguageCode::Crh),
            "CRP" => Ok(LanguageCode::Crp),
            "CSB" => Ok(LanguageCode::Csb),
            "CUS" => Ok(LanguageCode::Cus),
            "CZE" => Ok(LanguageCode::Cze),
            "DAK" => Ok(LanguageCode::Dak),
            "DAN" => Ok(LanguageCode::Dan),
            "DAR" => Ok(LanguageCode::Dar),
            "DAY" => Ok(LanguageCode::Day),
            "DEL" => Ok(LanguageCode::Del),
            "DEN" => Ok(LanguageCode::Den),
            "DGR" => Ok(LanguageCode::Dgr),
            "DIN" => Ok(LanguageCode::Din),
            "DIV" => Ok(LanguageCode::Div),
            "DOI" => Ok(LanguageCode::Doi),
            "DRA" => Ok(LanguageCode::Dra),
            "DSB" => Ok(LanguageCode::Dsb),
            "DUA" => Ok(LanguageCode::Dua),
            "DUM" => Ok(LanguageCode::Dum),
            "DUT" => Ok(LanguageCode::Dut),
            "DYU" => Ok(LanguageCode::Dyu),
            "DZO" => Ok(LanguageCode::Dzo),
            "EFI" => Ok(LanguageCode::Efi),
            "EGY" => Ok(LanguageCode::Egy),
            "EKA" => Ok(LanguageCode::Eka),
            "ELX" => Ok(LanguageCode::Elx),
            "ENG" => Ok(LanguageCode::Eng),
            "ENM" => Ok(LanguageCode::Enm),
            "EPO" => Ok(LanguageCode::Epo),
            "EST" => Ok(LanguageCode::Est),
            "EWE" => Ok(LanguageCode::Ewe),
            "EWO" => Ok(LanguageCode::Ewo),
            "FAN" => Ok(LanguageCode::Fan),
            "FAO" => Ok(LanguageCode::Fao),
            "FAT" => Ok(LanguageCode::Fat),
            "FIJ" => Ok(LanguageCode::Fij),
            "FIL" => Ok(LanguageCode::Fil),
            "FIN" => Ok(LanguageCode::Fin),
            "FIU" => Ok(LanguageCode::Fiu),
            "FON" => Ok(LanguageCode::Fon),
            "FRE" => Ok(LanguageCode::Fre),
            "FRM" => Ok(LanguageCode::Frm),
            "FRO" => Ok(LanguageCode::Fro),
            "FRR" => Ok(LanguageCode::Frr),
            "FRS" => Ok(LanguageCode::Frs),
            "FRY" => Ok(LanguageCode::Fry),
            "FUL" => Ok(LanguageCode::Ful),
            "FUR" => Ok(LanguageCode::Fur),
            "GAA" => Ok(LanguageCode::Gaa),
            "GAY" => Ok(LanguageCode::Gay),
            "GBA" => Ok(LanguageCode::Gba),
            "GEM" => Ok(LanguageCode::Gem),
            "GEO" => Ok(LanguageCode::Geo),
            "GER" => Ok(LanguageCode::Ger),
            "GEZ" => Ok(LanguageCode::Gez),
            "GIL" => Ok(LanguageCode::Gil),
            "GLA" => Ok(LanguageCode::Gla),
            "GLE" => Ok(LanguageCode::Gle),
            "GLG" => Ok(LanguageCode::Glg),
            "GLV" => Ok(LanguageCode::Glv),
            "GMH" => Ok(LanguageCode::Gmh),
            "GOH" => Ok(LanguageCode::Goh),
            "GON" => Ok(LanguageCode::Gon),
            "GOR" => Ok(LanguageCode::Gor),
            "GOT" => Ok(LanguageCode::Got),
            "GRB" => Ok(LanguageCode::Grb),
            "GRC" => Ok(LanguageCode::Grc),
            "GRE" => Ok(LanguageCode::Gre),
            "GRN" => Ok(LanguageCode::Grn),
            "GSW" => Ok(LanguageCode::Gsw),
            "GUJ" => Ok(LanguageCode::Guj),
            "GWI" => Ok(LanguageCode::Gwi),
            "HAI" => Ok(LanguageCode::Hai),
            "HAT" => Ok(LanguageCode::Hat),
            "HAU" => Ok(LanguageCode::Hau),
            "HAW" => Ok(LanguageCode::Haw),
            "HEB" => Ok(LanguageCode::Heb),
            "HER" => Ok(LanguageCode::Her),
            "HIL" => Ok(LanguageCode::Hil),
            "HIM" => Ok(LanguageCode::Him),
            "HIN" => Ok(LanguageCode::Hin),
            "HIT" => Ok(LanguageCode::Hit),
            "HMN" => Ok(LanguageCode::Hmn),
            "HMO" => Ok(LanguageCode::Hmo),
            "HRV" => Ok(LanguageCode::Hrv),
            "HSB" => Ok(LanguageCode::Hsb),
            "HUN" => Ok(LanguageCode::Hun),
            "HUP" => Ok(LanguageCode::Hup),
            "IBA" => Ok(LanguageCode::Iba),
            "IBO" => Ok(LanguageCode::Ibo),
            "ICE" => Ok(LanguageCode::Ice),
            "IDO" => Ok(LanguageCode::Ido),
            "III" => Ok(LanguageCode::Iii),
            "IJO" => Ok(LanguageCode::Ijo),
            "IKU" => Ok(LanguageCode::Iku),
            "ILE" => Ok(LanguageCode::Ile),
            "ILO" => Ok(LanguageCode::Ilo),
            "INA" => Ok(LanguageCode::Ina),
            "INC" => Ok(LanguageCode::Inc),
            "IND" => Ok(LanguageCode::Ind),
            "INE" => Ok(LanguageCode::Ine),
            "INH" => Ok(LanguageCode::Inh),
            "IPK" => Ok(LanguageCode::Ipk),
            "IRA" => Ok(LanguageCode::Ira),
            "IRO" => Ok(LanguageCode::Iro),
            "ITA" => Ok(LanguageCode::Ita),
            "JAV" => Ok(LanguageCode::Jav),
            "JBO" => Ok(LanguageCode::Jbo),
            "JPN" => Ok(LanguageCode::Jpn),
            "JPR" => Ok(LanguageCode::Jpr),
            "JRB" => Ok(LanguageCode::Jrb),
            "KAA" => Ok(LanguageCode::Kaa),
            "KAB" => Ok(LanguageCode::Kab),
            "KAC" => Ok(LanguageCode::Kac),
            "KAL" => Ok(LanguageCode::Kal),
            "KAM" => Ok(LanguageCode::Kam),
            "KAN" => Ok(LanguageCode::Kan),
            "KAR" => Ok(LanguageCode::Kar),
            "KAS" => Ok(LanguageCode::Kas),
            "KAU" => Ok(LanguageCode::Kau),
            "KAW" => Ok(LanguageCode::Kaw),
            "KAZ" => Ok(LanguageCode::Kaz),
            "KBD" => Ok(LanguageCode::Kbd),
            "KHA" => Ok(LanguageCode::Kha),
            "KHI" => Ok(LanguageCode::Khi),
            "KHM" => Ok(LanguageCode::Khm),
            "KHO" => Ok(LanguageCode::Kho),
            "KIK" => Ok(LanguageCode::Kik),
            "KIN" => Ok(LanguageCode::Kin),
            "KIR" => Ok(LanguageCode::Kir),
            "KMB" => Ok(LanguageCode::Kmb),
            "KOK" => Ok(LanguageCode::Kok),
            "KOM" => Ok(LanguageCode::Kom),
            "KON" => Ok(LanguageCode::Kon),
            "KOR" => Ok(LanguageCode::Kor),
            "KOS" => Ok(LanguageCode::Kos),
            "KPE" => Ok(LanguageCode::Kpe),
            "KRC" => Ok(LanguageCode::Krc),
            "KRL" => Ok(LanguageCode::Krl),
            "KRO" => Ok(LanguageCode::Kro),
            "KRU" => Ok(LanguageCode::Kru),
            "KUA" => Ok(LanguageCode::Kua),
            "KUM" => Ok(LanguageCode::Kum),
            "KUR" => Ok(LanguageCode::Kur),
            "KUT" => Ok(LanguageCode::Kut),
            "LAD" => Ok(LanguageCode::Lad),
            "LAH" => Ok(LanguageCode::Lah),
            "LAM" => Ok(LanguageCode::Lam),
            "LAO" => Ok(LanguageCode::Lao),
            "LAT" => Ok(LanguageCode::Lat),
            "LAV" => Ok(LanguageCode::Lav),
            "LEZ" => Ok(LanguageCode::Lez),
            "LIM" => Ok(LanguageCode::Lim),
            "LIN" => Ok(LanguageCode::Lin),
            "LIT" => Ok(LanguageCode::Lit),
            "LOL" => Ok(LanguageCode::Lol),
            "LOZ" => Ok(LanguageCode::Loz),
            "LTZ" => Ok(LanguageCode::Ltz),
            "LUA" => Ok(LanguageCode::Lua),
            "LUB" => Ok(LanguageCode::Lub),
            "LUG" => Ok(LanguageCode::Lug),
            "LUI" => Ok(LanguageCode::Lui),
            "LUN" => Ok(LanguageCode::Lun),
            "LUO" => Ok(LanguageCode::Luo),
            "LUS" => Ok(LanguageCode::Lus),
            "MAC" => Ok(LanguageCode::Mac),
            "MAD" => Ok(LanguageCode::Mad),
            "MAG" => Ok(LanguageCode::Mag),
            "MAH" => Ok(LanguageCode::Mah),
            "MAI" => Ok(LanguageCode::Mai),
            "MAK" => Ok(LanguageCode::Mak),
            "MAL" => Ok(LanguageCode::Mal),
            "MAN" => Ok(LanguageCode::Man),
            "MAO" => Ok(LanguageCode::Mao),
            "MAP" => Ok(LanguageCode::Map),
            "MAR" => Ok(LanguageCode::Mar),
            "MAS" => Ok(LanguageCode::Mas),
            "MAY" => Ok(LanguageCode::May),
            "MDF" => Ok(LanguageCode::Mdf),
            "MDR" => Ok(LanguageCode::Mdr),
            "MEN" => Ok(LanguageCode::Men),
            "MGA" => Ok(LanguageCode::Mga),
            "MIC" => Ok(LanguageCode::Mic),
            "MIN" => Ok(LanguageCode::Min),
            "MIS" => Ok(LanguageCode::Mis),
            "MKH" => Ok(LanguageCode::Mkh),
            "MLG" => Ok(LanguageCode::Mlg),
            "MLT" => Ok(LanguageCode::Mlt),
            "MNC" => Ok(LanguageCode::Mnc),
            "MNI" => Ok(LanguageCode::Mni),
            "MNO" => Ok(LanguageCode::Mno),
            "MOH" => Ok(LanguageCode::Moh),
            "MON" => Ok(LanguageCode::Mon),
            "MOS" => Ok(LanguageCode::Mos),
            "MUL" => Ok(LanguageCode::Mul),
            "MUN" => Ok(LanguageCode::Mun),
            "MUS" => Ok(LanguageCode::Mus),
            "MWL" => Ok(LanguageCode::Mwl),
            "MWR" => Ok(LanguageCode::Mwr),
            "MYN" => Ok(LanguageCode::Myn),
            "MYV" => Ok(LanguageCode::Myv),
            "NAH" => Ok(LanguageCode::Nah),
            "NAI" => Ok(LanguageCode::Nai),
            "NAP" => Ok(LanguageCode::Nap),
            "NAU" => Ok(LanguageCode::Nau),
            "NAV" => Ok(LanguageCode::Nav),
            "NBL" => Ok(LanguageCode::Nbl),
            "NDE" => Ok(LanguageCode::Nde),
            "NDO" => Ok(LanguageCode::Ndo),
            "NDS" => Ok(LanguageCode::Nds),
            "NEP" => Ok(LanguageCode::Nep),
            "NEW" => Ok(LanguageCode::New),
            "NIA" => Ok(LanguageCode::Nia),
            "NIC" => Ok(LanguageCode::Nic),
            "NIU" => Ok(LanguageCode::Niu),
            "NNO" => Ok(LanguageCode::Nno),
            "NOB" => Ok(LanguageCode::Nob),
            "NOG" => Ok(LanguageCode::Nog),
            "NON" => Ok(LanguageCode::Non),
            "NOR" => Ok(LanguageCode::Nor),
            "NQO" => Ok(LanguageCode::Nqo),
            "NSO" => Ok(LanguageCode::Nso),
            "NUB" => Ok(LanguageCode::Nub),
            "NWC" => Ok(LanguageCode::Nwc),
            "NYA" => Ok(LanguageCode::Nya),
            "NYM" => Ok(LanguageCode::Nym),
            "NYN" => Ok(LanguageCode::Nyn),
            "NYO" => Ok(LanguageCode::Nyo),
            "NZI" => Ok(LanguageCode::Nzi),
            "OCI" => Ok(LanguageCode::Oci),
            "OJI" => Ok(LanguageCode::Oji),
            "ORI" => Ok(LanguageCode::Ori),
            "ORM" => Ok(LanguageCode::Orm),
            "OSA" => Ok(LanguageCode::Osa),
            "OSS" => Ok(LanguageCode::Oss),
            "OTA" => Ok(LanguageCode::Ota),
            "OTO" => Ok(LanguageCode::Oto),
            "PAA" => Ok(LanguageCode::Paa),
            "PAG" => Ok(LanguageCode::Pag),
            "PAL" => Ok(LanguageCode::Pal),
            "PAM" => Ok(LanguageCode::Pam),
            "PAN" => Ok(LanguageCode::Pan),
            "PAP" => Ok(LanguageCode::Pap),
            "PAU" => Ok(LanguageCode::Pau),
            "PEO" => Ok(LanguageCode::Peo),
            "PER" => Ok(LanguageCode::Per),
            "PHI" => Ok(LanguageCode::Phi),
            "PHN" => Ok(LanguageCode::Phn),
            "PLI" => Ok(LanguageCode::Pli),
            "POL" => Ok(LanguageCode::Pol),
            "PON" => Ok(LanguageCode::Pon),
            "POR" => Ok(LanguageCode::Por),
            "PRA" => Ok(LanguageCode::Pra),
            "PRO" => Ok(LanguageCode::Pro),
            "PUS" => Ok(LanguageCode::Pus),
            "QAA" => Ok(LanguageCode::Qaa),
            "QUE" => Ok(LanguageCode::Que),
            "RAJ" => Ok(LanguageCode::Raj),
            "RAP" => Ok(LanguageCode::Rap),
            "RAR" => Ok(LanguageCode::Rar),
            "ROA" => Ok(LanguageCode::Roa),
            "ROH" => Ok(LanguageCode::Roh),
            "ROM" => Ok(LanguageCode::Rom),
            "RUM" => Ok(LanguageCode::Rum),
            "RUN" => Ok(LanguageCode::Run),
            "RUP" => Ok(LanguageCode::Rup),
            "RUS" => Ok(LanguageCode::Rus),
            "SAD" => Ok(LanguageCode::Sad),
            "SAG" => Ok(LanguageCode::Sag),
            "SAH" => Ok(LanguageCode::Sah),
            "SAI" => Ok(LanguageCode::Sai),
            "SAL" => Ok(LanguageCode::Sal),
            "SAM" => Ok(LanguageCode::Sam),
            "SAN" => Ok(LanguageCode::San),
            "SAS" => Ok(LanguageCode::Sas),
            "SAT" => Ok(LanguageCode::Sat),
            "SCN" => Ok(LanguageCode::Scn),
            "SCO" => Ok(LanguageCode::Sco),
            "SEL" => Ok(LanguageCode::Sel),
            "SEM" => Ok(LanguageCode::Sem),
            "SGA" => Ok(LanguageCode::Sga),
            "SGN" => Ok(LanguageCode::Sgn),
            "SHN" => Ok(LanguageCode::Shn),
            "SID" => Ok(LanguageCode::Sid),
            "SIN" => Ok(LanguageCode::Sin),
            "SIO" => Ok(LanguageCode::Sio),
            "SIT" => Ok(LanguageCode::Sit),
            "SLA" => Ok(LanguageCode::Sla),
            "SLO" => Ok(LanguageCode::Slo),
            "SLV" => Ok(LanguageCode::Slv),
            "SMA" => Ok(LanguageCode::Sma),
            "SME" => Ok(LanguageCode::Sme),
            "SMI" => Ok(LanguageCode::Smi),
            "SMJ" => Ok(LanguageCode::Smj),
            "SMN" => Ok(LanguageCode::Smn),
            "SMO" => Ok(LanguageCode::Smo),
            "SMS" => Ok(LanguageCode::Sms),
            "SNA" => Ok(LanguageCode::Sna),
            "SND" => Ok(LanguageCode::Snd),
            "SNK" => Ok(LanguageCode::Snk),
            "SOG" => Ok(LanguageCode::Sog),
            "SOM" => Ok(LanguageCode::Som),
            "SON" => Ok(LanguageCode::Son),
            "SOT" => Ok(LanguageCode::Sot),
            "SPA" => Ok(LanguageCode::Spa),
            "SRD" => Ok(LanguageCode::Srd),
            "SRN" => Ok(LanguageCode::Srn),
            "SRP" => Ok(LanguageCode::Srp),
            "SRR" => Ok(LanguageCode::Srr),
            "SSA" => Ok(LanguageCode::Ssa),
            "SSW" => Ok(LanguageCode::Ssw),
            "SUK" => Ok(LanguageCode::Suk),
            "SUN" => Ok(LanguageCode::Sun),
            "SUS" => Ok(LanguageCode::Sus),
            "SUX" => Ok(LanguageCode::Sux),
            "SWA" => Ok(LanguageCode::Swa),
            "SWE" => Ok(LanguageCode::Swe),
            "SYC" => Ok(LanguageCode::Syc),
            "SYR" => Ok(LanguageCode::Syr),
            "TAH" => Ok(LanguageCode::Tah),
            "TAI" => Ok(LanguageCode::Tai),
            "TAM" => Ok(LanguageCode::Tam),
            "TAT" => Ok(LanguageCode::Tat),
            "TEL" => Ok(LanguageCode::Tel),
            "TEM" => Ok(LanguageCode::Tem),
            "TER" => Ok(LanguageCode::Ter),
            "TET" => Ok(LanguageCode::Tet),
            "TGK" => Ok(LanguageCode::Tgk),
            "TGL" => Ok(LanguageCode::Tgl),
            "THA" => Ok(LanguageCode::Tha),
            "TIB" => Ok(LanguageCode::Tib),
            "TIG" => Ok(LanguageCode::Tig),
            "TIR" => Ok(LanguageCode::Tir),
            "TIV" => Ok(LanguageCode::Tiv),
            "TKL" => Ok(LanguageCode::Tkl),
            "TLH" => Ok(LanguageCode::Tlh),
            "TLI" => Ok(LanguageCode::Tli),
            "TMH" => Ok(LanguageCode::Tmh),
            "TOG" => Ok(LanguageCode::Tog),
            "TON" => Ok(LanguageCode::Ton),
            "TPI" => Ok(LanguageCode::Tpi),
            "TSI" => Ok(LanguageCode::Tsi),
            "TSN" => Ok(LanguageCode::Tsn),
            "TSO" => Ok(LanguageCode::Tso),
            "TUK" => Ok(LanguageCode::Tuk),
            "TUM" => Ok(LanguageCode::Tum),
            "TUP" => Ok(LanguageCode::Tup),
            "TUR" => Ok(LanguageCode::Tur),
            "TUT" => Ok(LanguageCode::Tut),
            "TVL" => Ok(LanguageCode::Tvl),
            "TWI" => Ok(LanguageCode::Twi),
            "TYV" => Ok(LanguageCode::Tyv),
            "UDM" => Ok(LanguageCode::Udm),
            "UGA" => Ok(LanguageCode::Uga),
            "UIG" => Ok(LanguageCode::Uig),
            "UKR" => Ok(LanguageCode::Ukr),
            "UMB" => Ok(LanguageCode::Umb),
            "UND" => Ok(LanguageCode::Und),
            "URD" => Ok(LanguageCode::Urd),
            "UZB" => Ok(LanguageCode::Uzb),
            "VAI" => Ok(LanguageCode::Vai),
            "VEN" => Ok(LanguageCode::Ven),
            "VIE" => Ok(LanguageCode::Vie),
            "VOL" => Ok(LanguageCode::Vol),
            "VOT" => Ok(LanguageCode::Vot),
            "WAK" => Ok(LanguageCode::Wak),
            "WAL" => Ok(LanguageCode::Wal),
            "WAR" => Ok(LanguageCode::War),
            "WAS" => Ok(LanguageCode::Was),
            "WEL" => Ok(LanguageCode::Wel),
            "WEN" => Ok(LanguageCode::Wen),
            "WLN" => Ok(LanguageCode::Wln),
            "WOL" => Ok(LanguageCode::Wol),
            "XAL" => Ok(LanguageCode::Xal),
            "XHO" => Ok(LanguageCode::Xho),
            "YAO" => Ok(LanguageCode::Yao),
            "YAP" => Ok(LanguageCode::Yap),
            "YID" => Ok(LanguageCode::Yid),
            "YOR" => Ok(LanguageCode::Yor),
            "YPK" => Ok(LanguageCode::Ypk),
            "ZAP" => Ok(LanguageCode::Zap),
            "ZBL" => Ok(LanguageCode::Zbl),
            "ZEN" => Ok(LanguageCode::Zen),
            "ZGH" => Ok(LanguageCode::Zgh),
            "ZHA" => Ok(LanguageCode::Zha),
            "ZND" => Ok(LanguageCode::Znd),
            "ZUL" => Ok(LanguageCode::Zul),
            "ZUN" => Ok(LanguageCode::Zun),
            "ZXX" => Ok(LanguageCode::Zxx),
            "ZZA" => Ok(LanguageCode::Zza),
            _ => Err(ThothError::InvalidLanguageCode(input.to_string())),
        }
    }
}

#[test]
fn test_languagecode_default() {
    let langcode: LanguageCode = Default::default();
    assert_eq!(langcode, LanguageCode::Eng);
}

#[test]
fn test_languagerelation_default() {
    let langrelation: LanguageRelation = Default::default();
    assert_eq!(langrelation, LanguageRelation::Original);
}

#[test]
fn test_languagerelation_display() {
    assert_eq!(format!("{}", LanguageRelation::Original), "Original");
    assert_eq!(
        format!("{}", LanguageRelation::TranslatedFrom),
        "Translated From"
    );
    assert_eq!(
        format!("{}", LanguageRelation::TranslatedInto),
        "Translated Into"
    );
}

#[test]
fn test_languagecode_display() {
    assert_eq!(format!("{}", LanguageCode::Aar), "AAR");
    assert_eq!(format!("{}", LanguageCode::Abk), "ABK");
    assert_eq!(format!("{}", LanguageCode::Ace), "ACE");
    assert_eq!(format!("{}", LanguageCode::Ach), "ACH");
    assert_eq!(format!("{}", LanguageCode::Ada), "ADA");
    assert_eq!(format!("{}", LanguageCode::Ady), "ADY");
    assert_eq!(format!("{}", LanguageCode::Afa), "AFA");
    assert_eq!(format!("{}", LanguageCode::Afh), "AFH");
    assert_eq!(format!("{}", LanguageCode::Afr), "AFR");
    assert_eq!(format!("{}", LanguageCode::Ain), "AIN");
    assert_eq!(format!("{}", LanguageCode::Aka), "AKA");
    assert_eq!(format!("{}", LanguageCode::Akk), "AKK");
    assert_eq!(format!("{}", LanguageCode::Alb), "ALB");
    assert_eq!(format!("{}", LanguageCode::Ale), "ALE");
    assert_eq!(format!("{}", LanguageCode::Alg), "ALG");
    assert_eq!(format!("{}", LanguageCode::Alt), "ALT");
    assert_eq!(format!("{}", LanguageCode::Amh), "AMH");
    assert_eq!(format!("{}", LanguageCode::Ang), "ANG");
    assert_eq!(format!("{}", LanguageCode::Anp), "ANP");
    assert_eq!(format!("{}", LanguageCode::Apa), "APA");
    assert_eq!(format!("{}", LanguageCode::Ara), "ARA");
    assert_eq!(format!("{}", LanguageCode::Arc), "ARC");
    assert_eq!(format!("{}", LanguageCode::Arg), "ARG");
    assert_eq!(format!("{}", LanguageCode::Arm), "ARM");
    assert_eq!(format!("{}", LanguageCode::Arn), "ARN");
    assert_eq!(format!("{}", LanguageCode::Arp), "ARP");
    assert_eq!(format!("{}", LanguageCode::Art), "ART");
    assert_eq!(format!("{}", LanguageCode::Arw), "ARW");
    assert_eq!(format!("{}", LanguageCode::Asm), "ASM");
    assert_eq!(format!("{}", LanguageCode::Ast), "AST");
    assert_eq!(format!("{}", LanguageCode::Ath), "ATH");
    assert_eq!(format!("{}", LanguageCode::Aus), "AUS");
    assert_eq!(format!("{}", LanguageCode::Ava), "AVA");
    assert_eq!(format!("{}", LanguageCode::Ave), "AVE");
    assert_eq!(format!("{}", LanguageCode::Awa), "AWA");
    assert_eq!(format!("{}", LanguageCode::Aym), "AYM");
    assert_eq!(format!("{}", LanguageCode::Aze), "AZE");
    assert_eq!(format!("{}", LanguageCode::Bad), "BAD");
    assert_eq!(format!("{}", LanguageCode::Bai), "BAI");
    assert_eq!(format!("{}", LanguageCode::Bak), "BAK");
    assert_eq!(format!("{}", LanguageCode::Bal), "BAL");
    assert_eq!(format!("{}", LanguageCode::Bam), "BAM");
    assert_eq!(format!("{}", LanguageCode::Ban), "BAN");
    assert_eq!(format!("{}", LanguageCode::Baq), "BAQ");
    assert_eq!(format!("{}", LanguageCode::Bas), "BAS");
    assert_eq!(format!("{}", LanguageCode::Bat), "BAT");
    assert_eq!(format!("{}", LanguageCode::Bej), "BEJ");
    assert_eq!(format!("{}", LanguageCode::Bel), "BEL");
    assert_eq!(format!("{}", LanguageCode::Bem), "BEM");
    assert_eq!(format!("{}", LanguageCode::Ben), "BEN");
    assert_eq!(format!("{}", LanguageCode::Ber), "BER");
    assert_eq!(format!("{}", LanguageCode::Bho), "BHO");
    assert_eq!(format!("{}", LanguageCode::Bih), "BIH");
    assert_eq!(format!("{}", LanguageCode::Bik), "BIK");
    assert_eq!(format!("{}", LanguageCode::Bin), "BIN");
    assert_eq!(format!("{}", LanguageCode::Bis), "BIS");
    assert_eq!(format!("{}", LanguageCode::Bla), "BLA");
    assert_eq!(format!("{}", LanguageCode::Bnt), "BNT");
    assert_eq!(format!("{}", LanguageCode::Bos), "BOS");
    assert_eq!(format!("{}", LanguageCode::Bra), "BRA");
    assert_eq!(format!("{}", LanguageCode::Bre), "BRE");
    assert_eq!(format!("{}", LanguageCode::Btk), "BTK");
    assert_eq!(format!("{}", LanguageCode::Bua), "BUA");
    assert_eq!(format!("{}", LanguageCode::Bug), "BUG");
    assert_eq!(format!("{}", LanguageCode::Bul), "BUL");
    assert_eq!(format!("{}", LanguageCode::Bur), "BUR");
    assert_eq!(format!("{}", LanguageCode::Byn), "BYN");
    assert_eq!(format!("{}", LanguageCode::Cad), "CAD");
    assert_eq!(format!("{}", LanguageCode::Cai), "CAI");
    assert_eq!(format!("{}", LanguageCode::Car), "CAR");
    assert_eq!(format!("{}", LanguageCode::Cat), "CAT");
    assert_eq!(format!("{}", LanguageCode::Cau), "CAU");
    assert_eq!(format!("{}", LanguageCode::Ceb), "CEB");
    assert_eq!(format!("{}", LanguageCode::Cel), "CEL");
    assert_eq!(format!("{}", LanguageCode::Cha), "CHA");
    assert_eq!(format!("{}", LanguageCode::Chb), "CHB");
    assert_eq!(format!("{}", LanguageCode::Che), "CHE");
    assert_eq!(format!("{}", LanguageCode::Chg), "CHG");
    assert_eq!(format!("{}", LanguageCode::Chi), "CHI");
    assert_eq!(format!("{}", LanguageCode::Chk), "CHK");
    assert_eq!(format!("{}", LanguageCode::Chm), "CHM");
    assert_eq!(format!("{}", LanguageCode::Chn), "CHN");
    assert_eq!(format!("{}", LanguageCode::Cho), "CHO");
    assert_eq!(format!("{}", LanguageCode::Chp), "CHP");
    assert_eq!(format!("{}", LanguageCode::Chr), "CHR");
    assert_eq!(format!("{}", LanguageCode::Chu), "CHU");
    assert_eq!(format!("{}", LanguageCode::Chv), "CHV");
    assert_eq!(format!("{}", LanguageCode::Chy), "CHY");
    assert_eq!(format!("{}", LanguageCode::Cmc), "CMC");
    assert_eq!(format!("{}", LanguageCode::Cnr), "CNR");
    assert_eq!(format!("{}", LanguageCode::Cop), "COP");
    assert_eq!(format!("{}", LanguageCode::Cor), "COR");
    assert_eq!(format!("{}", LanguageCode::Cos), "COS");
    assert_eq!(format!("{}", LanguageCode::Cpe), "CPE");
    assert_eq!(format!("{}", LanguageCode::Cpf), "CPF");
    assert_eq!(format!("{}", LanguageCode::Cpp), "CPP");
    assert_eq!(format!("{}", LanguageCode::Cre), "CRE");
    assert_eq!(format!("{}", LanguageCode::Crh), "CRH");
    assert_eq!(format!("{}", LanguageCode::Crp), "CRP");
    assert_eq!(format!("{}", LanguageCode::Csb), "CSB");
    assert_eq!(format!("{}", LanguageCode::Cus), "CUS");
    assert_eq!(format!("{}", LanguageCode::Cze), "CZE");
    assert_eq!(format!("{}", LanguageCode::Dak), "DAK");
    assert_eq!(format!("{}", LanguageCode::Dan), "DAN");
    assert_eq!(format!("{}", LanguageCode::Dar), "DAR");
    assert_eq!(format!("{}", LanguageCode::Day), "DAY");
    assert_eq!(format!("{}", LanguageCode::Del), "DEL");
    assert_eq!(format!("{}", LanguageCode::Den), "DEN");
    assert_eq!(format!("{}", LanguageCode::Dgr), "DGR");
    assert_eq!(format!("{}", LanguageCode::Din), "DIN");
    assert_eq!(format!("{}", LanguageCode::Div), "DIV");
    assert_eq!(format!("{}", LanguageCode::Doi), "DOI");
    assert_eq!(format!("{}", LanguageCode::Dra), "DRA");
    assert_eq!(format!("{}", LanguageCode::Dsb), "DSB");
    assert_eq!(format!("{}", LanguageCode::Dua), "DUA");
    assert_eq!(format!("{}", LanguageCode::Dum), "DUM");
    assert_eq!(format!("{}", LanguageCode::Dut), "DUT");
    assert_eq!(format!("{}", LanguageCode::Dyu), "DYU");
    assert_eq!(format!("{}", LanguageCode::Dzo), "DZO");
    assert_eq!(format!("{}", LanguageCode::Efi), "EFI");
    assert_eq!(format!("{}", LanguageCode::Egy), "EGY");
    assert_eq!(format!("{}", LanguageCode::Eka), "EKA");
    assert_eq!(format!("{}", LanguageCode::Elx), "ELX");
    assert_eq!(format!("{}", LanguageCode::Eng), "ENG");
    assert_eq!(format!("{}", LanguageCode::Enm), "ENM");
    assert_eq!(format!("{}", LanguageCode::Epo), "EPO");
    assert_eq!(format!("{}", LanguageCode::Est), "EST");
    assert_eq!(format!("{}", LanguageCode::Ewe), "EWE");
    assert_eq!(format!("{}", LanguageCode::Ewo), "EWO");
    assert_eq!(format!("{}", LanguageCode::Fan), "FAN");
    assert_eq!(format!("{}", LanguageCode::Fao), "FAO");
    assert_eq!(format!("{}", LanguageCode::Fat), "FAT");
    assert_eq!(format!("{}", LanguageCode::Fij), "FIJ");
    assert_eq!(format!("{}", LanguageCode::Fil), "FIL");
    assert_eq!(format!("{}", LanguageCode::Fin), "FIN");
    assert_eq!(format!("{}", LanguageCode::Fiu), "FIU");
    assert_eq!(format!("{}", LanguageCode::Fon), "FON");
    assert_eq!(format!("{}", LanguageCode::Fre), "FRE");
    assert_eq!(format!("{}", LanguageCode::Frm), "FRM");
    assert_eq!(format!("{}", LanguageCode::Fro), "FRO");
    assert_eq!(format!("{}", LanguageCode::Frr), "FRR");
    assert_eq!(format!("{}", LanguageCode::Frs), "FRS");
    assert_eq!(format!("{}", LanguageCode::Fry), "FRY");
    assert_eq!(format!("{}", LanguageCode::Ful), "FUL");
    assert_eq!(format!("{}", LanguageCode::Fur), "FUR");
    assert_eq!(format!("{}", LanguageCode::Gaa), "GAA");
    assert_eq!(format!("{}", LanguageCode::Gay), "GAY");
    assert_eq!(format!("{}", LanguageCode::Gba), "GBA");
    assert_eq!(format!("{}", LanguageCode::Gem), "GEM");
    assert_eq!(format!("{}", LanguageCode::Geo), "GEO");
    assert_eq!(format!("{}", LanguageCode::Ger), "GER");
    assert_eq!(format!("{}", LanguageCode::Gez), "GEZ");
    assert_eq!(format!("{}", LanguageCode::Gil), "GIL");
    assert_eq!(format!("{}", LanguageCode::Gla), "GLA");
    assert_eq!(format!("{}", LanguageCode::Gle), "GLE");
    assert_eq!(format!("{}", LanguageCode::Glg), "GLG");
    assert_eq!(format!("{}", LanguageCode::Glv), "GLV");
    assert_eq!(format!("{}", LanguageCode::Gmh), "GMH");
    assert_eq!(format!("{}", LanguageCode::Goh), "GOH");
    assert_eq!(format!("{}", LanguageCode::Gon), "GON");
    assert_eq!(format!("{}", LanguageCode::Gor), "GOR");
    assert_eq!(format!("{}", LanguageCode::Got), "GOT");
    assert_eq!(format!("{}", LanguageCode::Grb), "GRB");
    assert_eq!(format!("{}", LanguageCode::Grc), "GRC");
    assert_eq!(format!("{}", LanguageCode::Gre), "GRE");
    assert_eq!(format!("{}", LanguageCode::Grn), "GRN");
    assert_eq!(format!("{}", LanguageCode::Gsw), "GSW");
    assert_eq!(format!("{}", LanguageCode::Guj), "GUJ");
    assert_eq!(format!("{}", LanguageCode::Gwi), "GWI");
    assert_eq!(format!("{}", LanguageCode::Hai), "HAI");
    assert_eq!(format!("{}", LanguageCode::Hat), "HAT");
    assert_eq!(format!("{}", LanguageCode::Hau), "HAU");
    assert_eq!(format!("{}", LanguageCode::Haw), "HAW");
    assert_eq!(format!("{}", LanguageCode::Heb), "HEB");
    assert_eq!(format!("{}", LanguageCode::Her), "HER");
    assert_eq!(format!("{}", LanguageCode::Hil), "HIL");
    assert_eq!(format!("{}", LanguageCode::Him), "HIM");
    assert_eq!(format!("{}", LanguageCode::Hin), "HIN");
    assert_eq!(format!("{}", LanguageCode::Hit), "HIT");
    assert_eq!(format!("{}", LanguageCode::Hmn), "HMN");
    assert_eq!(format!("{}", LanguageCode::Hmo), "HMO");
    assert_eq!(format!("{}", LanguageCode::Hrv), "HRV");
    assert_eq!(format!("{}", LanguageCode::Hsb), "HSB");
    assert_eq!(format!("{}", LanguageCode::Hun), "HUN");
    assert_eq!(format!("{}", LanguageCode::Hup), "HUP");
    assert_eq!(format!("{}", LanguageCode::Iba), "IBA");
    assert_eq!(format!("{}", LanguageCode::Ibo), "IBO");
    assert_eq!(format!("{}", LanguageCode::Ice), "ICE");
    assert_eq!(format!("{}", LanguageCode::Ido), "IDO");
    assert_eq!(format!("{}", LanguageCode::Iii), "III");
    assert_eq!(format!("{}", LanguageCode::Ijo), "IJO");
    assert_eq!(format!("{}", LanguageCode::Iku), "IKU");
    assert_eq!(format!("{}", LanguageCode::Ile), "ILE");
    assert_eq!(format!("{}", LanguageCode::Ilo), "ILO");
    assert_eq!(format!("{}", LanguageCode::Ina), "INA");
    assert_eq!(format!("{}", LanguageCode::Inc), "INC");
    assert_eq!(format!("{}", LanguageCode::Ind), "IND");
    assert_eq!(format!("{}", LanguageCode::Ine), "INE");
    assert_eq!(format!("{}", LanguageCode::Inh), "INH");
    assert_eq!(format!("{}", LanguageCode::Ipk), "IPK");
    assert_eq!(format!("{}", LanguageCode::Ira), "IRA");
    assert_eq!(format!("{}", LanguageCode::Iro), "IRO");
    assert_eq!(format!("{}", LanguageCode::Ita), "ITA");
    assert_eq!(format!("{}", LanguageCode::Jav), "JAV");
    assert_eq!(format!("{}", LanguageCode::Jbo), "JBO");
    assert_eq!(format!("{}", LanguageCode::Jpn), "JPN");
    assert_eq!(format!("{}", LanguageCode::Jpr), "JPR");
    assert_eq!(format!("{}", LanguageCode::Jrb), "JRB");
    assert_eq!(format!("{}", LanguageCode::Kaa), "KAA");
    assert_eq!(format!("{}", LanguageCode::Kab), "KAB");
    assert_eq!(format!("{}", LanguageCode::Kac), "KAC");
    assert_eq!(format!("{}", LanguageCode::Kal), "KAL");
    assert_eq!(format!("{}", LanguageCode::Kam), "KAM");
    assert_eq!(format!("{}", LanguageCode::Kan), "KAN");
    assert_eq!(format!("{}", LanguageCode::Kar), "KAR");
    assert_eq!(format!("{}", LanguageCode::Kas), "KAS");
    assert_eq!(format!("{}", LanguageCode::Kau), "KAU");
    assert_eq!(format!("{}", LanguageCode::Kaw), "KAW");
    assert_eq!(format!("{}", LanguageCode::Kaz), "KAZ");
    assert_eq!(format!("{}", LanguageCode::Kbd), "KBD");
    assert_eq!(format!("{}", LanguageCode::Kha), "KHA");
    assert_eq!(format!("{}", LanguageCode::Khi), "KHI");
    assert_eq!(format!("{}", LanguageCode::Khm), "KHM");
    assert_eq!(format!("{}", LanguageCode::Kho), "KHO");
    assert_eq!(format!("{}", LanguageCode::Kik), "KIK");
    assert_eq!(format!("{}", LanguageCode::Kin), "KIN");
    assert_eq!(format!("{}", LanguageCode::Kir), "KIR");
    assert_eq!(format!("{}", LanguageCode::Kmb), "KMB");
    assert_eq!(format!("{}", LanguageCode::Kok), "KOK");
    assert_eq!(format!("{}", LanguageCode::Kom), "KOM");
    assert_eq!(format!("{}", LanguageCode::Kon), "KON");
    assert_eq!(format!("{}", LanguageCode::Kor), "KOR");
    assert_eq!(format!("{}", LanguageCode::Kos), "KOS");
    assert_eq!(format!("{}", LanguageCode::Kpe), "KPE");
    assert_eq!(format!("{}", LanguageCode::Krc), "KRC");
    assert_eq!(format!("{}", LanguageCode::Krl), "KRL");
    assert_eq!(format!("{}", LanguageCode::Kro), "KRO");
    assert_eq!(format!("{}", LanguageCode::Kru), "KRU");
    assert_eq!(format!("{}", LanguageCode::Kua), "KUA");
    assert_eq!(format!("{}", LanguageCode::Kum), "KUM");
    assert_eq!(format!("{}", LanguageCode::Kur), "KUR");
    assert_eq!(format!("{}", LanguageCode::Kut), "KUT");
    assert_eq!(format!("{}", LanguageCode::Lad), "LAD");
    assert_eq!(format!("{}", LanguageCode::Lah), "LAH");
    assert_eq!(format!("{}", LanguageCode::Lam), "LAM");
    assert_eq!(format!("{}", LanguageCode::Lao), "LAO");
    assert_eq!(format!("{}", LanguageCode::Lat), "LAT");
    assert_eq!(format!("{}", LanguageCode::Lav), "LAV");
    assert_eq!(format!("{}", LanguageCode::Lez), "LEZ");
    assert_eq!(format!("{}", LanguageCode::Lim), "LIM");
    assert_eq!(format!("{}", LanguageCode::Lin), "LIN");
    assert_eq!(format!("{}", LanguageCode::Lit), "LIT");
    assert_eq!(format!("{}", LanguageCode::Lol), "LOL");
    assert_eq!(format!("{}", LanguageCode::Loz), "LOZ");
    assert_eq!(format!("{}", LanguageCode::Ltz), "LTZ");
    assert_eq!(format!("{}", LanguageCode::Lua), "LUA");
    assert_eq!(format!("{}", LanguageCode::Lub), "LUB");
    assert_eq!(format!("{}", LanguageCode::Lug), "LUG");
    assert_eq!(format!("{}", LanguageCode::Lui), "LUI");
    assert_eq!(format!("{}", LanguageCode::Lun), "LUN");
    assert_eq!(format!("{}", LanguageCode::Luo), "LUO");
    assert_eq!(format!("{}", LanguageCode::Lus), "LUS");
    assert_eq!(format!("{}", LanguageCode::Mac), "MAC");
    assert_eq!(format!("{}", LanguageCode::Mad), "MAD");
    assert_eq!(format!("{}", LanguageCode::Mag), "MAG");
    assert_eq!(format!("{}", LanguageCode::Mah), "MAH");
    assert_eq!(format!("{}", LanguageCode::Mai), "MAI");
    assert_eq!(format!("{}", LanguageCode::Mak), "MAK");
    assert_eq!(format!("{}", LanguageCode::Mal), "MAL");
    assert_eq!(format!("{}", LanguageCode::Man), "MAN");
    assert_eq!(format!("{}", LanguageCode::Mao), "MAO");
    assert_eq!(format!("{}", LanguageCode::Map), "MAP");
    assert_eq!(format!("{}", LanguageCode::Mar), "MAR");
    assert_eq!(format!("{}", LanguageCode::Mas), "MAS");
    assert_eq!(format!("{}", LanguageCode::May), "MAY");
    assert_eq!(format!("{}", LanguageCode::Mdf), "MDF");
    assert_eq!(format!("{}", LanguageCode::Mdr), "MDR");
    assert_eq!(format!("{}", LanguageCode::Men), "MEN");
    assert_eq!(format!("{}", LanguageCode::Mga), "MGA");
    assert_eq!(format!("{}", LanguageCode::Mic), "MIC");
    assert_eq!(format!("{}", LanguageCode::Min), "MIN");
    assert_eq!(format!("{}", LanguageCode::Mis), "MIS");
    assert_eq!(format!("{}", LanguageCode::Mkh), "MKH");
    assert_eq!(format!("{}", LanguageCode::Mlg), "MLG");
    assert_eq!(format!("{}", LanguageCode::Mlt), "MLT");
    assert_eq!(format!("{}", LanguageCode::Mnc), "MNC");
    assert_eq!(format!("{}", LanguageCode::Mni), "MNI");
    assert_eq!(format!("{}", LanguageCode::Mno), "MNO");
    assert_eq!(format!("{}", LanguageCode::Moh), "MOH");
    assert_eq!(format!("{}", LanguageCode::Mon), "MON");
    assert_eq!(format!("{}", LanguageCode::Mos), "MOS");
    assert_eq!(format!("{}", LanguageCode::Mul), "MUL");
    assert_eq!(format!("{}", LanguageCode::Mun), "MUN");
    assert_eq!(format!("{}", LanguageCode::Mus), "MUS");
    assert_eq!(format!("{}", LanguageCode::Mwl), "MWL");
    assert_eq!(format!("{}", LanguageCode::Mwr), "MWR");
    assert_eq!(format!("{}", LanguageCode::Myn), "MYN");
    assert_eq!(format!("{}", LanguageCode::Myv), "MYV");
    assert_eq!(format!("{}", LanguageCode::Nah), "NAH");
    assert_eq!(format!("{}", LanguageCode::Nai), "NAI");
    assert_eq!(format!("{}", LanguageCode::Nap), "NAP");
    assert_eq!(format!("{}", LanguageCode::Nau), "NAU");
    assert_eq!(format!("{}", LanguageCode::Nav), "NAV");
    assert_eq!(format!("{}", LanguageCode::Nbl), "NBL");
    assert_eq!(format!("{}", LanguageCode::Nde), "NDE");
    assert_eq!(format!("{}", LanguageCode::Ndo), "NDO");
    assert_eq!(format!("{}", LanguageCode::Nds), "NDS");
    assert_eq!(format!("{}", LanguageCode::Nep), "NEP");
    assert_eq!(format!("{}", LanguageCode::New), "NEW");
    assert_eq!(format!("{}", LanguageCode::Nia), "NIA");
    assert_eq!(format!("{}", LanguageCode::Nic), "NIC");
    assert_eq!(format!("{}", LanguageCode::Niu), "NIU");
    assert_eq!(format!("{}", LanguageCode::Nno), "NNO");
    assert_eq!(format!("{}", LanguageCode::Nob), "NOB");
    assert_eq!(format!("{}", LanguageCode::Nog), "NOG");
    assert_eq!(format!("{}", LanguageCode::Non), "NON");
    assert_eq!(format!("{}", LanguageCode::Nor), "NOR");
    assert_eq!(format!("{}", LanguageCode::Nqo), "NQO");
    assert_eq!(format!("{}", LanguageCode::Nso), "NSO");
    assert_eq!(format!("{}", LanguageCode::Nub), "NUB");
    assert_eq!(format!("{}", LanguageCode::Nwc), "NWC");
    assert_eq!(format!("{}", LanguageCode::Nya), "NYA");
    assert_eq!(format!("{}", LanguageCode::Nym), "NYM");
    assert_eq!(format!("{}", LanguageCode::Nyn), "NYN");
    assert_eq!(format!("{}", LanguageCode::Nyo), "NYO");
    assert_eq!(format!("{}", LanguageCode::Nzi), "NZI");
    assert_eq!(format!("{}", LanguageCode::Oci), "OCI");
    assert_eq!(format!("{}", LanguageCode::Oji), "OJI");
    assert_eq!(format!("{}", LanguageCode::Ori), "ORI");
    assert_eq!(format!("{}", LanguageCode::Orm), "ORM");
    assert_eq!(format!("{}", LanguageCode::Osa), "OSA");
    assert_eq!(format!("{}", LanguageCode::Oss), "OSS");
    assert_eq!(format!("{}", LanguageCode::Ota), "OTA");
    assert_eq!(format!("{}", LanguageCode::Oto), "OTO");
    assert_eq!(format!("{}", LanguageCode::Paa), "PAA");
    assert_eq!(format!("{}", LanguageCode::Pag), "PAG");
    assert_eq!(format!("{}", LanguageCode::Pal), "PAL");
    assert_eq!(format!("{}", LanguageCode::Pam), "PAM");
    assert_eq!(format!("{}", LanguageCode::Pan), "PAN");
    assert_eq!(format!("{}", LanguageCode::Pap), "PAP");
    assert_eq!(format!("{}", LanguageCode::Pau), "PAU");
    assert_eq!(format!("{}", LanguageCode::Peo), "PEO");
    assert_eq!(format!("{}", LanguageCode::Per), "PER");
    assert_eq!(format!("{}", LanguageCode::Phi), "PHI");
    assert_eq!(format!("{}", LanguageCode::Phn), "PHN");
    assert_eq!(format!("{}", LanguageCode::Pli), "PLI");
    assert_eq!(format!("{}", LanguageCode::Pol), "POL");
    assert_eq!(format!("{}", LanguageCode::Pon), "PON");
    assert_eq!(format!("{}", LanguageCode::Por), "POR");
    assert_eq!(format!("{}", LanguageCode::Pra), "PRA");
    assert_eq!(format!("{}", LanguageCode::Pro), "PRO");
    assert_eq!(format!("{}", LanguageCode::Pus), "PUS");
    assert_eq!(format!("{}", LanguageCode::Qaa), "QAA");
    assert_eq!(format!("{}", LanguageCode::Que), "QUE");
    assert_eq!(format!("{}", LanguageCode::Raj), "RAJ");
    assert_eq!(format!("{}", LanguageCode::Rap), "RAP");
    assert_eq!(format!("{}", LanguageCode::Rar), "RAR");
    assert_eq!(format!("{}", LanguageCode::Roa), "ROA");
    assert_eq!(format!("{}", LanguageCode::Roh), "ROH");
    assert_eq!(format!("{}", LanguageCode::Rom), "ROM");
    assert_eq!(format!("{}", LanguageCode::Rum), "RUM");
    assert_eq!(format!("{}", LanguageCode::Run), "RUN");
    assert_eq!(format!("{}", LanguageCode::Rup), "RUP");
    assert_eq!(format!("{}", LanguageCode::Rus), "RUS");
    assert_eq!(format!("{}", LanguageCode::Sad), "SAD");
    assert_eq!(format!("{}", LanguageCode::Sag), "SAG");
    assert_eq!(format!("{}", LanguageCode::Sah), "SAH");
    assert_eq!(format!("{}", LanguageCode::Sai), "SAI");
    assert_eq!(format!("{}", LanguageCode::Sal), "SAL");
    assert_eq!(format!("{}", LanguageCode::Sam), "SAM");
    assert_eq!(format!("{}", LanguageCode::San), "SAN");
    assert_eq!(format!("{}", LanguageCode::Sas), "SAS");
    assert_eq!(format!("{}", LanguageCode::Sat), "SAT");
    assert_eq!(format!("{}", LanguageCode::Scn), "SCN");
    assert_eq!(format!("{}", LanguageCode::Sco), "SCO");
    assert_eq!(format!("{}", LanguageCode::Sel), "SEL");
    assert_eq!(format!("{}", LanguageCode::Sem), "SEM");
    assert_eq!(format!("{}", LanguageCode::Sga), "SGA");
    assert_eq!(format!("{}", LanguageCode::Sgn), "SGN");
    assert_eq!(format!("{}", LanguageCode::Shn), "SHN");
    assert_eq!(format!("{}", LanguageCode::Sid), "SID");
    assert_eq!(format!("{}", LanguageCode::Sin), "SIN");
    assert_eq!(format!("{}", LanguageCode::Sio), "SIO");
    assert_eq!(format!("{}", LanguageCode::Sit), "SIT");
    assert_eq!(format!("{}", LanguageCode::Sla), "SLA");
    assert_eq!(format!("{}", LanguageCode::Slo), "SLO");
    assert_eq!(format!("{}", LanguageCode::Slv), "SLV");
    assert_eq!(format!("{}", LanguageCode::Sma), "SMA");
    assert_eq!(format!("{}", LanguageCode::Sme), "SME");
    assert_eq!(format!("{}", LanguageCode::Smi), "SMI");
    assert_eq!(format!("{}", LanguageCode::Smj), "SMJ");
    assert_eq!(format!("{}", LanguageCode::Smn), "SMN");
    assert_eq!(format!("{}", LanguageCode::Smo), "SMO");
    assert_eq!(format!("{}", LanguageCode::Sms), "SMS");
    assert_eq!(format!("{}", LanguageCode::Sna), "SNA");
    assert_eq!(format!("{}", LanguageCode::Snd), "SND");
    assert_eq!(format!("{}", LanguageCode::Snk), "SNK");
    assert_eq!(format!("{}", LanguageCode::Sog), "SOG");
    assert_eq!(format!("{}", LanguageCode::Som), "SOM");
    assert_eq!(format!("{}", LanguageCode::Son), "SON");
    assert_eq!(format!("{}", LanguageCode::Sot), "SOT");
    assert_eq!(format!("{}", LanguageCode::Spa), "SPA");
    assert_eq!(format!("{}", LanguageCode::Srd), "SRD");
    assert_eq!(format!("{}", LanguageCode::Srn), "SRN");
    assert_eq!(format!("{}", LanguageCode::Srp), "SRP");
    assert_eq!(format!("{}", LanguageCode::Srr), "SRR");
    assert_eq!(format!("{}", LanguageCode::Ssa), "SSA");
    assert_eq!(format!("{}", LanguageCode::Ssw), "SSW");
    assert_eq!(format!("{}", LanguageCode::Suk), "SUK");
    assert_eq!(format!("{}", LanguageCode::Sun), "SUN");
    assert_eq!(format!("{}", LanguageCode::Sus), "SUS");
    assert_eq!(format!("{}", LanguageCode::Sux), "SUX");
    assert_eq!(format!("{}", LanguageCode::Swa), "SWA");
    assert_eq!(format!("{}", LanguageCode::Swe), "SWE");
    assert_eq!(format!("{}", LanguageCode::Syc), "SYC");
    assert_eq!(format!("{}", LanguageCode::Syr), "SYR");
    assert_eq!(format!("{}", LanguageCode::Tah), "TAH");
    assert_eq!(format!("{}", LanguageCode::Tai), "TAI");
    assert_eq!(format!("{}", LanguageCode::Tam), "TAM");
    assert_eq!(format!("{}", LanguageCode::Tat), "TAT");
    assert_eq!(format!("{}", LanguageCode::Tel), "TEL");
    assert_eq!(format!("{}", LanguageCode::Tem), "TEM");
    assert_eq!(format!("{}", LanguageCode::Ter), "TER");
    assert_eq!(format!("{}", LanguageCode::Tet), "TET");
    assert_eq!(format!("{}", LanguageCode::Tgk), "TGK");
    assert_eq!(format!("{}", LanguageCode::Tgl), "TGL");
    assert_eq!(format!("{}", LanguageCode::Tha), "THA");
    assert_eq!(format!("{}", LanguageCode::Tib), "TIB");
    assert_eq!(format!("{}", LanguageCode::Tig), "TIG");
    assert_eq!(format!("{}", LanguageCode::Tir), "TIR");
    assert_eq!(format!("{}", LanguageCode::Tiv), "TIV");
    assert_eq!(format!("{}", LanguageCode::Tkl), "TKL");
    assert_eq!(format!("{}", LanguageCode::Tlh), "TLH");
    assert_eq!(format!("{}", LanguageCode::Tli), "TLI");
    assert_eq!(format!("{}", LanguageCode::Tmh), "TMH");
    assert_eq!(format!("{}", LanguageCode::Tog), "TOG");
    assert_eq!(format!("{}", LanguageCode::Ton), "TON");
    assert_eq!(format!("{}", LanguageCode::Tpi), "TPI");
    assert_eq!(format!("{}", LanguageCode::Tsi), "TSI");
    assert_eq!(format!("{}", LanguageCode::Tsn), "TSN");
    assert_eq!(format!("{}", LanguageCode::Tso), "TSO");
    assert_eq!(format!("{}", LanguageCode::Tuk), "TUK");
    assert_eq!(format!("{}", LanguageCode::Tum), "TUM");
    assert_eq!(format!("{}", LanguageCode::Tup), "TUP");
    assert_eq!(format!("{}", LanguageCode::Tur), "TUR");
    assert_eq!(format!("{}", LanguageCode::Tut), "TUT");
    assert_eq!(format!("{}", LanguageCode::Tvl), "TVL");
    assert_eq!(format!("{}", LanguageCode::Twi), "TWI");
    assert_eq!(format!("{}", LanguageCode::Tyv), "TYV");
    assert_eq!(format!("{}", LanguageCode::Udm), "UDM");
    assert_eq!(format!("{}", LanguageCode::Uga), "UGA");
    assert_eq!(format!("{}", LanguageCode::Uig), "UIG");
    assert_eq!(format!("{}", LanguageCode::Ukr), "UKR");
    assert_eq!(format!("{}", LanguageCode::Umb), "UMB");
    assert_eq!(format!("{}", LanguageCode::Und), "UND");
    assert_eq!(format!("{}", LanguageCode::Urd), "URD");
    assert_eq!(format!("{}", LanguageCode::Uzb), "UZB");
    assert_eq!(format!("{}", LanguageCode::Vai), "VAI");
    assert_eq!(format!("{}", LanguageCode::Ven), "VEN");
    assert_eq!(format!("{}", LanguageCode::Vie), "VIE");
    assert_eq!(format!("{}", LanguageCode::Vol), "VOL");
    assert_eq!(format!("{}", LanguageCode::Vot), "VOT");
    assert_eq!(format!("{}", LanguageCode::Wak), "WAK");
    assert_eq!(format!("{}", LanguageCode::Wal), "WAL");
    assert_eq!(format!("{}", LanguageCode::War), "WAR");
    assert_eq!(format!("{}", LanguageCode::Was), "WAS");
    assert_eq!(format!("{}", LanguageCode::Wel), "WEL");
    assert_eq!(format!("{}", LanguageCode::Wen), "WEN");
    assert_eq!(format!("{}", LanguageCode::Wln), "WLN");
    assert_eq!(format!("{}", LanguageCode::Wol), "WOL");
    assert_eq!(format!("{}", LanguageCode::Xal), "XAL");
    assert_eq!(format!("{}", LanguageCode::Xho), "XHO");
    assert_eq!(format!("{}", LanguageCode::Yao), "YAO");
    assert_eq!(format!("{}", LanguageCode::Yap), "YAP");
    assert_eq!(format!("{}", LanguageCode::Yid), "YID");
    assert_eq!(format!("{}", LanguageCode::Yor), "YOR");
    assert_eq!(format!("{}", LanguageCode::Ypk), "YPK");
    assert_eq!(format!("{}", LanguageCode::Zap), "ZAP");
    assert_eq!(format!("{}", LanguageCode::Zbl), "ZBL");
    assert_eq!(format!("{}", LanguageCode::Zen), "ZEN");
    assert_eq!(format!("{}", LanguageCode::Zgh), "ZGH");
    assert_eq!(format!("{}", LanguageCode::Zha), "ZHA");
    assert_eq!(format!("{}", LanguageCode::Znd), "ZND");
    assert_eq!(format!("{}", LanguageCode::Zul), "ZUL");
    assert_eq!(format!("{}", LanguageCode::Zun), "ZUN");
    assert_eq!(format!("{}", LanguageCode::Zxx), "ZXX");
    assert_eq!(format!("{}", LanguageCode::Zza), "ZZA");
}

#[test]
fn test_languagerelation_fromstr() {
    assert_eq!(
        LanguageRelation::from_str("Original").unwrap(),
        LanguageRelation::Original
    );
    assert_eq!(
        LanguageRelation::from_str("Translated From").unwrap(),
        LanguageRelation::TranslatedFrom
    );
    assert_eq!(
        LanguageRelation::from_str("Translated Into").unwrap(),
        LanguageRelation::TranslatedInto
    );

    assert!(LanguageRelation::from_str("Invented").is_err());
}

#[test]
fn test_languagecode_fromstr() {
    assert_eq!(LanguageCode::from_str("AAR").unwrap(), LanguageCode::Aar);
    assert_eq!(LanguageCode::from_str("ABK").unwrap(), LanguageCode::Abk);
    assert_eq!(LanguageCode::from_str("ACE").unwrap(), LanguageCode::Ace);
    assert_eq!(LanguageCode::from_str("ACH").unwrap(), LanguageCode::Ach);
    assert_eq!(LanguageCode::from_str("ADA").unwrap(), LanguageCode::Ada);
    assert_eq!(LanguageCode::from_str("ADY").unwrap(), LanguageCode::Ady);
    assert_eq!(LanguageCode::from_str("AFA").unwrap(), LanguageCode::Afa);
    assert_eq!(LanguageCode::from_str("AFH").unwrap(), LanguageCode::Afh);
    assert_eq!(LanguageCode::from_str("AFR").unwrap(), LanguageCode::Afr);
    assert_eq!(LanguageCode::from_str("AIN").unwrap(), LanguageCode::Ain);
    assert_eq!(LanguageCode::from_str("AKA").unwrap(), LanguageCode::Aka);
    assert_eq!(LanguageCode::from_str("AKK").unwrap(), LanguageCode::Akk);
    assert_eq!(LanguageCode::from_str("ALB").unwrap(), LanguageCode::Alb);
    assert_eq!(LanguageCode::from_str("ALE").unwrap(), LanguageCode::Ale);
    assert_eq!(LanguageCode::from_str("ALG").unwrap(), LanguageCode::Alg);
    assert_eq!(LanguageCode::from_str("ALT").unwrap(), LanguageCode::Alt);
    assert_eq!(LanguageCode::from_str("AMH").unwrap(), LanguageCode::Amh);
    assert_eq!(LanguageCode::from_str("ANG").unwrap(), LanguageCode::Ang);
    assert_eq!(LanguageCode::from_str("ANP").unwrap(), LanguageCode::Anp);
    assert_eq!(LanguageCode::from_str("APA").unwrap(), LanguageCode::Apa);
    assert_eq!(LanguageCode::from_str("ARA").unwrap(), LanguageCode::Ara);
    assert_eq!(LanguageCode::from_str("ARC").unwrap(), LanguageCode::Arc);
    assert_eq!(LanguageCode::from_str("ARG").unwrap(), LanguageCode::Arg);
    assert_eq!(LanguageCode::from_str("ARM").unwrap(), LanguageCode::Arm);
    assert_eq!(LanguageCode::from_str("ARN").unwrap(), LanguageCode::Arn);
    assert_eq!(LanguageCode::from_str("ARP").unwrap(), LanguageCode::Arp);
    assert_eq!(LanguageCode::from_str("ART").unwrap(), LanguageCode::Art);
    assert_eq!(LanguageCode::from_str("ARW").unwrap(), LanguageCode::Arw);
    assert_eq!(LanguageCode::from_str("ASM").unwrap(), LanguageCode::Asm);
    assert_eq!(LanguageCode::from_str("AST").unwrap(), LanguageCode::Ast);
    assert_eq!(LanguageCode::from_str("ATH").unwrap(), LanguageCode::Ath);
    assert_eq!(LanguageCode::from_str("AUS").unwrap(), LanguageCode::Aus);
    assert_eq!(LanguageCode::from_str("AVA").unwrap(), LanguageCode::Ava);
    assert_eq!(LanguageCode::from_str("AVE").unwrap(), LanguageCode::Ave);
    assert_eq!(LanguageCode::from_str("AWA").unwrap(), LanguageCode::Awa);
    assert_eq!(LanguageCode::from_str("AYM").unwrap(), LanguageCode::Aym);
    assert_eq!(LanguageCode::from_str("AZE").unwrap(), LanguageCode::Aze);
    assert_eq!(LanguageCode::from_str("BAD").unwrap(), LanguageCode::Bad);
    assert_eq!(LanguageCode::from_str("BAI").unwrap(), LanguageCode::Bai);
    assert_eq!(LanguageCode::from_str("BAK").unwrap(), LanguageCode::Bak);
    assert_eq!(LanguageCode::from_str("BAL").unwrap(), LanguageCode::Bal);
    assert_eq!(LanguageCode::from_str("BAM").unwrap(), LanguageCode::Bam);
    assert_eq!(LanguageCode::from_str("BAN").unwrap(), LanguageCode::Ban);
    assert_eq!(LanguageCode::from_str("BAQ").unwrap(), LanguageCode::Baq);
    assert_eq!(LanguageCode::from_str("BAS").unwrap(), LanguageCode::Bas);
    assert_eq!(LanguageCode::from_str("BAT").unwrap(), LanguageCode::Bat);
    assert_eq!(LanguageCode::from_str("BEJ").unwrap(), LanguageCode::Bej);
    assert_eq!(LanguageCode::from_str("BEL").unwrap(), LanguageCode::Bel);
    assert_eq!(LanguageCode::from_str("BEM").unwrap(), LanguageCode::Bem);
    assert_eq!(LanguageCode::from_str("BEN").unwrap(), LanguageCode::Ben);
    assert_eq!(LanguageCode::from_str("BER").unwrap(), LanguageCode::Ber);
    assert_eq!(LanguageCode::from_str("BHO").unwrap(), LanguageCode::Bho);
    assert_eq!(LanguageCode::from_str("BIH").unwrap(), LanguageCode::Bih);
    assert_eq!(LanguageCode::from_str("BIK").unwrap(), LanguageCode::Bik);
    assert_eq!(LanguageCode::from_str("BIN").unwrap(), LanguageCode::Bin);
    assert_eq!(LanguageCode::from_str("BIS").unwrap(), LanguageCode::Bis);
    assert_eq!(LanguageCode::from_str("BLA").unwrap(), LanguageCode::Bla);
    assert_eq!(LanguageCode::from_str("BNT").unwrap(), LanguageCode::Bnt);
    assert_eq!(LanguageCode::from_str("BOS").unwrap(), LanguageCode::Bos);
    assert_eq!(LanguageCode::from_str("BRA").unwrap(), LanguageCode::Bra);
    assert_eq!(LanguageCode::from_str("BRE").unwrap(), LanguageCode::Bre);
    assert_eq!(LanguageCode::from_str("BTK").unwrap(), LanguageCode::Btk);
    assert_eq!(LanguageCode::from_str("BUA").unwrap(), LanguageCode::Bua);
    assert_eq!(LanguageCode::from_str("BUG").unwrap(), LanguageCode::Bug);
    assert_eq!(LanguageCode::from_str("BUL").unwrap(), LanguageCode::Bul);
    assert_eq!(LanguageCode::from_str("BUR").unwrap(), LanguageCode::Bur);
    assert_eq!(LanguageCode::from_str("BYN").unwrap(), LanguageCode::Byn);
    assert_eq!(LanguageCode::from_str("CAD").unwrap(), LanguageCode::Cad);
    assert_eq!(LanguageCode::from_str("CAI").unwrap(), LanguageCode::Cai);
    assert_eq!(LanguageCode::from_str("CAR").unwrap(), LanguageCode::Car);
    assert_eq!(LanguageCode::from_str("CAT").unwrap(), LanguageCode::Cat);
    assert_eq!(LanguageCode::from_str("CAU").unwrap(), LanguageCode::Cau);
    assert_eq!(LanguageCode::from_str("CEB").unwrap(), LanguageCode::Ceb);
    assert_eq!(LanguageCode::from_str("CEL").unwrap(), LanguageCode::Cel);
    assert_eq!(LanguageCode::from_str("CHA").unwrap(), LanguageCode::Cha);
    assert_eq!(LanguageCode::from_str("CHB").unwrap(), LanguageCode::Chb);
    assert_eq!(LanguageCode::from_str("CHE").unwrap(), LanguageCode::Che);
    assert_eq!(LanguageCode::from_str("CHG").unwrap(), LanguageCode::Chg);
    assert_eq!(LanguageCode::from_str("CHI").unwrap(), LanguageCode::Chi);
    assert_eq!(LanguageCode::from_str("CHK").unwrap(), LanguageCode::Chk);
    assert_eq!(LanguageCode::from_str("CHM").unwrap(), LanguageCode::Chm);
    assert_eq!(LanguageCode::from_str("CHN").unwrap(), LanguageCode::Chn);
    assert_eq!(LanguageCode::from_str("CHO").unwrap(), LanguageCode::Cho);
    assert_eq!(LanguageCode::from_str("CHP").unwrap(), LanguageCode::Chp);
    assert_eq!(LanguageCode::from_str("CHR").unwrap(), LanguageCode::Chr);
    assert_eq!(LanguageCode::from_str("CHU").unwrap(), LanguageCode::Chu);
    assert_eq!(LanguageCode::from_str("CHV").unwrap(), LanguageCode::Chv);
    assert_eq!(LanguageCode::from_str("CHY").unwrap(), LanguageCode::Chy);
    assert_eq!(LanguageCode::from_str("CMC").unwrap(), LanguageCode::Cmc);
    assert_eq!(LanguageCode::from_str("CNR").unwrap(), LanguageCode::Cnr);
    assert_eq!(LanguageCode::from_str("COP").unwrap(), LanguageCode::Cop);
    assert_eq!(LanguageCode::from_str("COR").unwrap(), LanguageCode::Cor);
    assert_eq!(LanguageCode::from_str("COS").unwrap(), LanguageCode::Cos);
    assert_eq!(LanguageCode::from_str("CPE").unwrap(), LanguageCode::Cpe);
    assert_eq!(LanguageCode::from_str("CPF").unwrap(), LanguageCode::Cpf);
    assert_eq!(LanguageCode::from_str("CPP").unwrap(), LanguageCode::Cpp);
    assert_eq!(LanguageCode::from_str("CRE").unwrap(), LanguageCode::Cre);
    assert_eq!(LanguageCode::from_str("CRH").unwrap(), LanguageCode::Crh);
    assert_eq!(LanguageCode::from_str("CRP").unwrap(), LanguageCode::Crp);
    assert_eq!(LanguageCode::from_str("CSB").unwrap(), LanguageCode::Csb);
    assert_eq!(LanguageCode::from_str("CUS").unwrap(), LanguageCode::Cus);
    assert_eq!(LanguageCode::from_str("CZE").unwrap(), LanguageCode::Cze);
    assert_eq!(LanguageCode::from_str("DAK").unwrap(), LanguageCode::Dak);
    assert_eq!(LanguageCode::from_str("DAN").unwrap(), LanguageCode::Dan);
    assert_eq!(LanguageCode::from_str("DAR").unwrap(), LanguageCode::Dar);
    assert_eq!(LanguageCode::from_str("DAY").unwrap(), LanguageCode::Day);
    assert_eq!(LanguageCode::from_str("DEL").unwrap(), LanguageCode::Del);
    assert_eq!(LanguageCode::from_str("DEN").unwrap(), LanguageCode::Den);
    assert_eq!(LanguageCode::from_str("DGR").unwrap(), LanguageCode::Dgr);
    assert_eq!(LanguageCode::from_str("DIN").unwrap(), LanguageCode::Din);
    assert_eq!(LanguageCode::from_str("DIV").unwrap(), LanguageCode::Div);
    assert_eq!(LanguageCode::from_str("DOI").unwrap(), LanguageCode::Doi);
    assert_eq!(LanguageCode::from_str("DRA").unwrap(), LanguageCode::Dra);
    assert_eq!(LanguageCode::from_str("DSB").unwrap(), LanguageCode::Dsb);
    assert_eq!(LanguageCode::from_str("DUA").unwrap(), LanguageCode::Dua);
    assert_eq!(LanguageCode::from_str("DUM").unwrap(), LanguageCode::Dum);
    assert_eq!(LanguageCode::from_str("DUT").unwrap(), LanguageCode::Dut);
    assert_eq!(LanguageCode::from_str("DYU").unwrap(), LanguageCode::Dyu);
    assert_eq!(LanguageCode::from_str("DZO").unwrap(), LanguageCode::Dzo);
    assert_eq!(LanguageCode::from_str("EFI").unwrap(), LanguageCode::Efi);
    assert_eq!(LanguageCode::from_str("EGY").unwrap(), LanguageCode::Egy);
    assert_eq!(LanguageCode::from_str("EKA").unwrap(), LanguageCode::Eka);
    assert_eq!(LanguageCode::from_str("ELX").unwrap(), LanguageCode::Elx);
    assert_eq!(LanguageCode::from_str("ENG").unwrap(), LanguageCode::Eng);
    assert_eq!(LanguageCode::from_str("ENM").unwrap(), LanguageCode::Enm);
    assert_eq!(LanguageCode::from_str("EPO").unwrap(), LanguageCode::Epo);
    assert_eq!(LanguageCode::from_str("EST").unwrap(), LanguageCode::Est);
    assert_eq!(LanguageCode::from_str("EWE").unwrap(), LanguageCode::Ewe);
    assert_eq!(LanguageCode::from_str("EWO").unwrap(), LanguageCode::Ewo);
    assert_eq!(LanguageCode::from_str("FAN").unwrap(), LanguageCode::Fan);
    assert_eq!(LanguageCode::from_str("FAO").unwrap(), LanguageCode::Fao);
    assert_eq!(LanguageCode::from_str("FAT").unwrap(), LanguageCode::Fat);
    assert_eq!(LanguageCode::from_str("FIJ").unwrap(), LanguageCode::Fij);
    assert_eq!(LanguageCode::from_str("FIL").unwrap(), LanguageCode::Fil);
    assert_eq!(LanguageCode::from_str("FIN").unwrap(), LanguageCode::Fin);
    assert_eq!(LanguageCode::from_str("FIU").unwrap(), LanguageCode::Fiu);
    assert_eq!(LanguageCode::from_str("FON").unwrap(), LanguageCode::Fon);
    assert_eq!(LanguageCode::from_str("FRE").unwrap(), LanguageCode::Fre);
    assert_eq!(LanguageCode::from_str("FRM").unwrap(), LanguageCode::Frm);
    assert_eq!(LanguageCode::from_str("FRO").unwrap(), LanguageCode::Fro);
    assert_eq!(LanguageCode::from_str("FRR").unwrap(), LanguageCode::Frr);
    assert_eq!(LanguageCode::from_str("FRS").unwrap(), LanguageCode::Frs);
    assert_eq!(LanguageCode::from_str("FRY").unwrap(), LanguageCode::Fry);
    assert_eq!(LanguageCode::from_str("FUL").unwrap(), LanguageCode::Ful);
    assert_eq!(LanguageCode::from_str("FUR").unwrap(), LanguageCode::Fur);
    assert_eq!(LanguageCode::from_str("GAA").unwrap(), LanguageCode::Gaa);
    assert_eq!(LanguageCode::from_str("GAY").unwrap(), LanguageCode::Gay);
    assert_eq!(LanguageCode::from_str("GBA").unwrap(), LanguageCode::Gba);
    assert_eq!(LanguageCode::from_str("GEM").unwrap(), LanguageCode::Gem);
    assert_eq!(LanguageCode::from_str("GEO").unwrap(), LanguageCode::Geo);
    assert_eq!(LanguageCode::from_str("GER").unwrap(), LanguageCode::Ger);
    assert_eq!(LanguageCode::from_str("GEZ").unwrap(), LanguageCode::Gez);
    assert_eq!(LanguageCode::from_str("GIL").unwrap(), LanguageCode::Gil);
    assert_eq!(LanguageCode::from_str("GLA").unwrap(), LanguageCode::Gla);
    assert_eq!(LanguageCode::from_str("GLE").unwrap(), LanguageCode::Gle);
    assert_eq!(LanguageCode::from_str("GLG").unwrap(), LanguageCode::Glg);
    assert_eq!(LanguageCode::from_str("GLV").unwrap(), LanguageCode::Glv);
    assert_eq!(LanguageCode::from_str("GMH").unwrap(), LanguageCode::Gmh);
    assert_eq!(LanguageCode::from_str("GOH").unwrap(), LanguageCode::Goh);
    assert_eq!(LanguageCode::from_str("GON").unwrap(), LanguageCode::Gon);
    assert_eq!(LanguageCode::from_str("GOR").unwrap(), LanguageCode::Gor);
    assert_eq!(LanguageCode::from_str("GOT").unwrap(), LanguageCode::Got);
    assert_eq!(LanguageCode::from_str("GRB").unwrap(), LanguageCode::Grb);
    assert_eq!(LanguageCode::from_str("GRC").unwrap(), LanguageCode::Grc);
    assert_eq!(LanguageCode::from_str("GRE").unwrap(), LanguageCode::Gre);
    assert_eq!(LanguageCode::from_str("GRN").unwrap(), LanguageCode::Grn);
    assert_eq!(LanguageCode::from_str("GSW").unwrap(), LanguageCode::Gsw);
    assert_eq!(LanguageCode::from_str("GUJ").unwrap(), LanguageCode::Guj);
    assert_eq!(LanguageCode::from_str("GWI").unwrap(), LanguageCode::Gwi);
    assert_eq!(LanguageCode::from_str("HAI").unwrap(), LanguageCode::Hai);
    assert_eq!(LanguageCode::from_str("HAT").unwrap(), LanguageCode::Hat);
    assert_eq!(LanguageCode::from_str("HAU").unwrap(), LanguageCode::Hau);
    assert_eq!(LanguageCode::from_str("HAW").unwrap(), LanguageCode::Haw);
    assert_eq!(LanguageCode::from_str("HEB").unwrap(), LanguageCode::Heb);
    assert_eq!(LanguageCode::from_str("HER").unwrap(), LanguageCode::Her);
    assert_eq!(LanguageCode::from_str("HIL").unwrap(), LanguageCode::Hil);
    assert_eq!(LanguageCode::from_str("HIM").unwrap(), LanguageCode::Him);
    assert_eq!(LanguageCode::from_str("HIN").unwrap(), LanguageCode::Hin);
    assert_eq!(LanguageCode::from_str("HIT").unwrap(), LanguageCode::Hit);
    assert_eq!(LanguageCode::from_str("HMN").unwrap(), LanguageCode::Hmn);
    assert_eq!(LanguageCode::from_str("HMO").unwrap(), LanguageCode::Hmo);
    assert_eq!(LanguageCode::from_str("HRV").unwrap(), LanguageCode::Hrv);
    assert_eq!(LanguageCode::from_str("HSB").unwrap(), LanguageCode::Hsb);
    assert_eq!(LanguageCode::from_str("HUN").unwrap(), LanguageCode::Hun);
    assert_eq!(LanguageCode::from_str("HUP").unwrap(), LanguageCode::Hup);
    assert_eq!(LanguageCode::from_str("IBA").unwrap(), LanguageCode::Iba);
    assert_eq!(LanguageCode::from_str("IBO").unwrap(), LanguageCode::Ibo);
    assert_eq!(LanguageCode::from_str("ICE").unwrap(), LanguageCode::Ice);
    assert_eq!(LanguageCode::from_str("IDO").unwrap(), LanguageCode::Ido);
    assert_eq!(LanguageCode::from_str("III").unwrap(), LanguageCode::Iii);
    assert_eq!(LanguageCode::from_str("IJO").unwrap(), LanguageCode::Ijo);
    assert_eq!(LanguageCode::from_str("IKU").unwrap(), LanguageCode::Iku);
    assert_eq!(LanguageCode::from_str("ILE").unwrap(), LanguageCode::Ile);
    assert_eq!(LanguageCode::from_str("ILO").unwrap(), LanguageCode::Ilo);
    assert_eq!(LanguageCode::from_str("INA").unwrap(), LanguageCode::Ina);
    assert_eq!(LanguageCode::from_str("INC").unwrap(), LanguageCode::Inc);
    assert_eq!(LanguageCode::from_str("IND").unwrap(), LanguageCode::Ind);
    assert_eq!(LanguageCode::from_str("INE").unwrap(), LanguageCode::Ine);
    assert_eq!(LanguageCode::from_str("INH").unwrap(), LanguageCode::Inh);
    assert_eq!(LanguageCode::from_str("IPK").unwrap(), LanguageCode::Ipk);
    assert_eq!(LanguageCode::from_str("IRA").unwrap(), LanguageCode::Ira);
    assert_eq!(LanguageCode::from_str("IRO").unwrap(), LanguageCode::Iro);
    assert_eq!(LanguageCode::from_str("ITA").unwrap(), LanguageCode::Ita);
    assert_eq!(LanguageCode::from_str("JAV").unwrap(), LanguageCode::Jav);
    assert_eq!(LanguageCode::from_str("JBO").unwrap(), LanguageCode::Jbo);
    assert_eq!(LanguageCode::from_str("JPN").unwrap(), LanguageCode::Jpn);
    assert_eq!(LanguageCode::from_str("JPR").unwrap(), LanguageCode::Jpr);
    assert_eq!(LanguageCode::from_str("JRB").unwrap(), LanguageCode::Jrb);
    assert_eq!(LanguageCode::from_str("KAA").unwrap(), LanguageCode::Kaa);
    assert_eq!(LanguageCode::from_str("KAB").unwrap(), LanguageCode::Kab);
    assert_eq!(LanguageCode::from_str("KAC").unwrap(), LanguageCode::Kac);
    assert_eq!(LanguageCode::from_str("KAL").unwrap(), LanguageCode::Kal);
    assert_eq!(LanguageCode::from_str("KAM").unwrap(), LanguageCode::Kam);
    assert_eq!(LanguageCode::from_str("KAN").unwrap(), LanguageCode::Kan);
    assert_eq!(LanguageCode::from_str("KAR").unwrap(), LanguageCode::Kar);
    assert_eq!(LanguageCode::from_str("KAS").unwrap(), LanguageCode::Kas);
    assert_eq!(LanguageCode::from_str("KAU").unwrap(), LanguageCode::Kau);
    assert_eq!(LanguageCode::from_str("KAW").unwrap(), LanguageCode::Kaw);
    assert_eq!(LanguageCode::from_str("KAZ").unwrap(), LanguageCode::Kaz);
    assert_eq!(LanguageCode::from_str("KBD").unwrap(), LanguageCode::Kbd);
    assert_eq!(LanguageCode::from_str("KHA").unwrap(), LanguageCode::Kha);
    assert_eq!(LanguageCode::from_str("KHI").unwrap(), LanguageCode::Khi);
    assert_eq!(LanguageCode::from_str("KHM").unwrap(), LanguageCode::Khm);
    assert_eq!(LanguageCode::from_str("KHO").unwrap(), LanguageCode::Kho);
    assert_eq!(LanguageCode::from_str("KIK").unwrap(), LanguageCode::Kik);
    assert_eq!(LanguageCode::from_str("KIN").unwrap(), LanguageCode::Kin);
    assert_eq!(LanguageCode::from_str("KIR").unwrap(), LanguageCode::Kir);
    assert_eq!(LanguageCode::from_str("KMB").unwrap(), LanguageCode::Kmb);
    assert_eq!(LanguageCode::from_str("KOK").unwrap(), LanguageCode::Kok);
    assert_eq!(LanguageCode::from_str("KOM").unwrap(), LanguageCode::Kom);
    assert_eq!(LanguageCode::from_str("KON").unwrap(), LanguageCode::Kon);
    assert_eq!(LanguageCode::from_str("KOR").unwrap(), LanguageCode::Kor);
    assert_eq!(LanguageCode::from_str("KOS").unwrap(), LanguageCode::Kos);
    assert_eq!(LanguageCode::from_str("KPE").unwrap(), LanguageCode::Kpe);
    assert_eq!(LanguageCode::from_str("KRC").unwrap(), LanguageCode::Krc);
    assert_eq!(LanguageCode::from_str("KRL").unwrap(), LanguageCode::Krl);
    assert_eq!(LanguageCode::from_str("KRO").unwrap(), LanguageCode::Kro);
    assert_eq!(LanguageCode::from_str("KRU").unwrap(), LanguageCode::Kru);
    assert_eq!(LanguageCode::from_str("KUA").unwrap(), LanguageCode::Kua);
    assert_eq!(LanguageCode::from_str("KUM").unwrap(), LanguageCode::Kum);
    assert_eq!(LanguageCode::from_str("KUR").unwrap(), LanguageCode::Kur);
    assert_eq!(LanguageCode::from_str("KUT").unwrap(), LanguageCode::Kut);
    assert_eq!(LanguageCode::from_str("LAD").unwrap(), LanguageCode::Lad);
    assert_eq!(LanguageCode::from_str("LAH").unwrap(), LanguageCode::Lah);
    assert_eq!(LanguageCode::from_str("LAM").unwrap(), LanguageCode::Lam);
    assert_eq!(LanguageCode::from_str("LAO").unwrap(), LanguageCode::Lao);
    assert_eq!(LanguageCode::from_str("LAT").unwrap(), LanguageCode::Lat);
    assert_eq!(LanguageCode::from_str("LAV").unwrap(), LanguageCode::Lav);
    assert_eq!(LanguageCode::from_str("LEZ").unwrap(), LanguageCode::Lez);
    assert_eq!(LanguageCode::from_str("LIM").unwrap(), LanguageCode::Lim);
    assert_eq!(LanguageCode::from_str("LIN").unwrap(), LanguageCode::Lin);
    assert_eq!(LanguageCode::from_str("LIT").unwrap(), LanguageCode::Lit);
    assert_eq!(LanguageCode::from_str("LOL").unwrap(), LanguageCode::Lol);
    assert_eq!(LanguageCode::from_str("LOZ").unwrap(), LanguageCode::Loz);
    assert_eq!(LanguageCode::from_str("LTZ").unwrap(), LanguageCode::Ltz);
    assert_eq!(LanguageCode::from_str("LUA").unwrap(), LanguageCode::Lua);
    assert_eq!(LanguageCode::from_str("LUB").unwrap(), LanguageCode::Lub);
    assert_eq!(LanguageCode::from_str("LUG").unwrap(), LanguageCode::Lug);
    assert_eq!(LanguageCode::from_str("LUI").unwrap(), LanguageCode::Lui);
    assert_eq!(LanguageCode::from_str("LUN").unwrap(), LanguageCode::Lun);
    assert_eq!(LanguageCode::from_str("LUO").unwrap(), LanguageCode::Luo);
    assert_eq!(LanguageCode::from_str("LUS").unwrap(), LanguageCode::Lus);
    assert_eq!(LanguageCode::from_str("MAC").unwrap(), LanguageCode::Mac);
    assert_eq!(LanguageCode::from_str("MAD").unwrap(), LanguageCode::Mad);
    assert_eq!(LanguageCode::from_str("MAG").unwrap(), LanguageCode::Mag);
    assert_eq!(LanguageCode::from_str("MAH").unwrap(), LanguageCode::Mah);
    assert_eq!(LanguageCode::from_str("MAI").unwrap(), LanguageCode::Mai);
    assert_eq!(LanguageCode::from_str("MAK").unwrap(), LanguageCode::Mak);
    assert_eq!(LanguageCode::from_str("MAL").unwrap(), LanguageCode::Mal);
    assert_eq!(LanguageCode::from_str("MAN").unwrap(), LanguageCode::Man);
    assert_eq!(LanguageCode::from_str("MAO").unwrap(), LanguageCode::Mao);
    assert_eq!(LanguageCode::from_str("MAP").unwrap(), LanguageCode::Map);
    assert_eq!(LanguageCode::from_str("MAR").unwrap(), LanguageCode::Mar);
    assert_eq!(LanguageCode::from_str("MAS").unwrap(), LanguageCode::Mas);
    assert_eq!(LanguageCode::from_str("MAY").unwrap(), LanguageCode::May);
    assert_eq!(LanguageCode::from_str("MDF").unwrap(), LanguageCode::Mdf);
    assert_eq!(LanguageCode::from_str("MDR").unwrap(), LanguageCode::Mdr);
    assert_eq!(LanguageCode::from_str("MEN").unwrap(), LanguageCode::Men);
    assert_eq!(LanguageCode::from_str("MGA").unwrap(), LanguageCode::Mga);
    assert_eq!(LanguageCode::from_str("MIC").unwrap(), LanguageCode::Mic);
    assert_eq!(LanguageCode::from_str("MIN").unwrap(), LanguageCode::Min);
    assert_eq!(LanguageCode::from_str("MIS").unwrap(), LanguageCode::Mis);
    assert_eq!(LanguageCode::from_str("MKH").unwrap(), LanguageCode::Mkh);
    assert_eq!(LanguageCode::from_str("MLG").unwrap(), LanguageCode::Mlg);
    assert_eq!(LanguageCode::from_str("MLT").unwrap(), LanguageCode::Mlt);
    assert_eq!(LanguageCode::from_str("MNC").unwrap(), LanguageCode::Mnc);
    assert_eq!(LanguageCode::from_str("MNI").unwrap(), LanguageCode::Mni);
    assert_eq!(LanguageCode::from_str("MNO").unwrap(), LanguageCode::Mno);
    assert_eq!(LanguageCode::from_str("MOH").unwrap(), LanguageCode::Moh);
    assert_eq!(LanguageCode::from_str("MON").unwrap(), LanguageCode::Mon);
    assert_eq!(LanguageCode::from_str("MOS").unwrap(), LanguageCode::Mos);
    assert_eq!(LanguageCode::from_str("MUL").unwrap(), LanguageCode::Mul);
    assert_eq!(LanguageCode::from_str("MUN").unwrap(), LanguageCode::Mun);
    assert_eq!(LanguageCode::from_str("MUS").unwrap(), LanguageCode::Mus);
    assert_eq!(LanguageCode::from_str("MWL").unwrap(), LanguageCode::Mwl);
    assert_eq!(LanguageCode::from_str("MWR").unwrap(), LanguageCode::Mwr);
    assert_eq!(LanguageCode::from_str("MYN").unwrap(), LanguageCode::Myn);
    assert_eq!(LanguageCode::from_str("MYV").unwrap(), LanguageCode::Myv);
    assert_eq!(LanguageCode::from_str("NAH").unwrap(), LanguageCode::Nah);
    assert_eq!(LanguageCode::from_str("NAI").unwrap(), LanguageCode::Nai);
    assert_eq!(LanguageCode::from_str("NAP").unwrap(), LanguageCode::Nap);
    assert_eq!(LanguageCode::from_str("NAU").unwrap(), LanguageCode::Nau);
    assert_eq!(LanguageCode::from_str("NAV").unwrap(), LanguageCode::Nav);
    assert_eq!(LanguageCode::from_str("NBL").unwrap(), LanguageCode::Nbl);
    assert_eq!(LanguageCode::from_str("NDE").unwrap(), LanguageCode::Nde);
    assert_eq!(LanguageCode::from_str("NDO").unwrap(), LanguageCode::Ndo);
    assert_eq!(LanguageCode::from_str("NDS").unwrap(), LanguageCode::Nds);
    assert_eq!(LanguageCode::from_str("NEP").unwrap(), LanguageCode::Nep);
    assert_eq!(LanguageCode::from_str("NEW").unwrap(), LanguageCode::New);
    assert_eq!(LanguageCode::from_str("NIA").unwrap(), LanguageCode::Nia);
    assert_eq!(LanguageCode::from_str("NIC").unwrap(), LanguageCode::Nic);
    assert_eq!(LanguageCode::from_str("NIU").unwrap(), LanguageCode::Niu);
    assert_eq!(LanguageCode::from_str("NNO").unwrap(), LanguageCode::Nno);
    assert_eq!(LanguageCode::from_str("NOB").unwrap(), LanguageCode::Nob);
    assert_eq!(LanguageCode::from_str("NOG").unwrap(), LanguageCode::Nog);
    assert_eq!(LanguageCode::from_str("NON").unwrap(), LanguageCode::Non);
    assert_eq!(LanguageCode::from_str("NOR").unwrap(), LanguageCode::Nor);
    assert_eq!(LanguageCode::from_str("NQO").unwrap(), LanguageCode::Nqo);
    assert_eq!(LanguageCode::from_str("NSO").unwrap(), LanguageCode::Nso);
    assert_eq!(LanguageCode::from_str("NUB").unwrap(), LanguageCode::Nub);
    assert_eq!(LanguageCode::from_str("NWC").unwrap(), LanguageCode::Nwc);
    assert_eq!(LanguageCode::from_str("NYA").unwrap(), LanguageCode::Nya);
    assert_eq!(LanguageCode::from_str("NYM").unwrap(), LanguageCode::Nym);
    assert_eq!(LanguageCode::from_str("NYN").unwrap(), LanguageCode::Nyn);
    assert_eq!(LanguageCode::from_str("NYO").unwrap(), LanguageCode::Nyo);
    assert_eq!(LanguageCode::from_str("NZI").unwrap(), LanguageCode::Nzi);
    assert_eq!(LanguageCode::from_str("OCI").unwrap(), LanguageCode::Oci);
    assert_eq!(LanguageCode::from_str("OJI").unwrap(), LanguageCode::Oji);
    assert_eq!(LanguageCode::from_str("ORI").unwrap(), LanguageCode::Ori);
    assert_eq!(LanguageCode::from_str("ORM").unwrap(), LanguageCode::Orm);
    assert_eq!(LanguageCode::from_str("OSA").unwrap(), LanguageCode::Osa);
    assert_eq!(LanguageCode::from_str("OSS").unwrap(), LanguageCode::Oss);
    assert_eq!(LanguageCode::from_str("OTA").unwrap(), LanguageCode::Ota);
    assert_eq!(LanguageCode::from_str("OTO").unwrap(), LanguageCode::Oto);
    assert_eq!(LanguageCode::from_str("PAA").unwrap(), LanguageCode::Paa);
    assert_eq!(LanguageCode::from_str("PAG").unwrap(), LanguageCode::Pag);
    assert_eq!(LanguageCode::from_str("PAL").unwrap(), LanguageCode::Pal);
    assert_eq!(LanguageCode::from_str("PAM").unwrap(), LanguageCode::Pam);
    assert_eq!(LanguageCode::from_str("PAN").unwrap(), LanguageCode::Pan);
    assert_eq!(LanguageCode::from_str("PAP").unwrap(), LanguageCode::Pap);
    assert_eq!(LanguageCode::from_str("PAU").unwrap(), LanguageCode::Pau);
    assert_eq!(LanguageCode::from_str("PEO").unwrap(), LanguageCode::Peo);
    assert_eq!(LanguageCode::from_str("PER").unwrap(), LanguageCode::Per);
    assert_eq!(LanguageCode::from_str("PHI").unwrap(), LanguageCode::Phi);
    assert_eq!(LanguageCode::from_str("PHN").unwrap(), LanguageCode::Phn);
    assert_eq!(LanguageCode::from_str("PLI").unwrap(), LanguageCode::Pli);
    assert_eq!(LanguageCode::from_str("POL").unwrap(), LanguageCode::Pol);
    assert_eq!(LanguageCode::from_str("PON").unwrap(), LanguageCode::Pon);
    assert_eq!(LanguageCode::from_str("POR").unwrap(), LanguageCode::Por);
    assert_eq!(LanguageCode::from_str("PRA").unwrap(), LanguageCode::Pra);
    assert_eq!(LanguageCode::from_str("PRO").unwrap(), LanguageCode::Pro);
    assert_eq!(LanguageCode::from_str("PUS").unwrap(), LanguageCode::Pus);
    assert_eq!(LanguageCode::from_str("QAA").unwrap(), LanguageCode::Qaa);
    assert_eq!(LanguageCode::from_str("QUE").unwrap(), LanguageCode::Que);
    assert_eq!(LanguageCode::from_str("RAJ").unwrap(), LanguageCode::Raj);
    assert_eq!(LanguageCode::from_str("RAP").unwrap(), LanguageCode::Rap);
    assert_eq!(LanguageCode::from_str("RAR").unwrap(), LanguageCode::Rar);
    assert_eq!(LanguageCode::from_str("ROA").unwrap(), LanguageCode::Roa);
    assert_eq!(LanguageCode::from_str("ROH").unwrap(), LanguageCode::Roh);
    assert_eq!(LanguageCode::from_str("ROM").unwrap(), LanguageCode::Rom);
    assert_eq!(LanguageCode::from_str("RUM").unwrap(), LanguageCode::Rum);
    assert_eq!(LanguageCode::from_str("RUN").unwrap(), LanguageCode::Run);
    assert_eq!(LanguageCode::from_str("RUP").unwrap(), LanguageCode::Rup);
    assert_eq!(LanguageCode::from_str("RUS").unwrap(), LanguageCode::Rus);
    assert_eq!(LanguageCode::from_str("SAD").unwrap(), LanguageCode::Sad);
    assert_eq!(LanguageCode::from_str("SAG").unwrap(), LanguageCode::Sag);
    assert_eq!(LanguageCode::from_str("SAH").unwrap(), LanguageCode::Sah);
    assert_eq!(LanguageCode::from_str("SAI").unwrap(), LanguageCode::Sai);
    assert_eq!(LanguageCode::from_str("SAL").unwrap(), LanguageCode::Sal);
    assert_eq!(LanguageCode::from_str("SAM").unwrap(), LanguageCode::Sam);
    assert_eq!(LanguageCode::from_str("SAN").unwrap(), LanguageCode::San);
    assert_eq!(LanguageCode::from_str("SAS").unwrap(), LanguageCode::Sas);
    assert_eq!(LanguageCode::from_str("SAT").unwrap(), LanguageCode::Sat);
    assert_eq!(LanguageCode::from_str("SCN").unwrap(), LanguageCode::Scn);
    assert_eq!(LanguageCode::from_str("SCO").unwrap(), LanguageCode::Sco);
    assert_eq!(LanguageCode::from_str("SEL").unwrap(), LanguageCode::Sel);
    assert_eq!(LanguageCode::from_str("SEM").unwrap(), LanguageCode::Sem);
    assert_eq!(LanguageCode::from_str("SGA").unwrap(), LanguageCode::Sga);
    assert_eq!(LanguageCode::from_str("SGN").unwrap(), LanguageCode::Sgn);
    assert_eq!(LanguageCode::from_str("SHN").unwrap(), LanguageCode::Shn);
    assert_eq!(LanguageCode::from_str("SID").unwrap(), LanguageCode::Sid);
    assert_eq!(LanguageCode::from_str("SIN").unwrap(), LanguageCode::Sin);
    assert_eq!(LanguageCode::from_str("SIO").unwrap(), LanguageCode::Sio);
    assert_eq!(LanguageCode::from_str("SIT").unwrap(), LanguageCode::Sit);
    assert_eq!(LanguageCode::from_str("SLA").unwrap(), LanguageCode::Sla);
    assert_eq!(LanguageCode::from_str("SLO").unwrap(), LanguageCode::Slo);
    assert_eq!(LanguageCode::from_str("SLV").unwrap(), LanguageCode::Slv);
    assert_eq!(LanguageCode::from_str("SMA").unwrap(), LanguageCode::Sma);
    assert_eq!(LanguageCode::from_str("SME").unwrap(), LanguageCode::Sme);
    assert_eq!(LanguageCode::from_str("SMI").unwrap(), LanguageCode::Smi);
    assert_eq!(LanguageCode::from_str("SMJ").unwrap(), LanguageCode::Smj);
    assert_eq!(LanguageCode::from_str("SMN").unwrap(), LanguageCode::Smn);
    assert_eq!(LanguageCode::from_str("SMO").unwrap(), LanguageCode::Smo);
    assert_eq!(LanguageCode::from_str("SMS").unwrap(), LanguageCode::Sms);
    assert_eq!(LanguageCode::from_str("SNA").unwrap(), LanguageCode::Sna);
    assert_eq!(LanguageCode::from_str("SND").unwrap(), LanguageCode::Snd);
    assert_eq!(LanguageCode::from_str("SNK").unwrap(), LanguageCode::Snk);
    assert_eq!(LanguageCode::from_str("SOG").unwrap(), LanguageCode::Sog);
    assert_eq!(LanguageCode::from_str("SOM").unwrap(), LanguageCode::Som);
    assert_eq!(LanguageCode::from_str("SON").unwrap(), LanguageCode::Son);
    assert_eq!(LanguageCode::from_str("SOT").unwrap(), LanguageCode::Sot);
    assert_eq!(LanguageCode::from_str("SPA").unwrap(), LanguageCode::Spa);
    assert_eq!(LanguageCode::from_str("SRD").unwrap(), LanguageCode::Srd);
    assert_eq!(LanguageCode::from_str("SRN").unwrap(), LanguageCode::Srn);
    assert_eq!(LanguageCode::from_str("SRP").unwrap(), LanguageCode::Srp);
    assert_eq!(LanguageCode::from_str("SRR").unwrap(), LanguageCode::Srr);
    assert_eq!(LanguageCode::from_str("SSA").unwrap(), LanguageCode::Ssa);
    assert_eq!(LanguageCode::from_str("SSW").unwrap(), LanguageCode::Ssw);
    assert_eq!(LanguageCode::from_str("SUK").unwrap(), LanguageCode::Suk);
    assert_eq!(LanguageCode::from_str("SUN").unwrap(), LanguageCode::Sun);
    assert_eq!(LanguageCode::from_str("SUS").unwrap(), LanguageCode::Sus);
    assert_eq!(LanguageCode::from_str("SUX").unwrap(), LanguageCode::Sux);
    assert_eq!(LanguageCode::from_str("SWA").unwrap(), LanguageCode::Swa);
    assert_eq!(LanguageCode::from_str("SWE").unwrap(), LanguageCode::Swe);
    assert_eq!(LanguageCode::from_str("SYC").unwrap(), LanguageCode::Syc);
    assert_eq!(LanguageCode::from_str("SYR").unwrap(), LanguageCode::Syr);
    assert_eq!(LanguageCode::from_str("TAH").unwrap(), LanguageCode::Tah);
    assert_eq!(LanguageCode::from_str("TAI").unwrap(), LanguageCode::Tai);
    assert_eq!(LanguageCode::from_str("TAM").unwrap(), LanguageCode::Tam);
    assert_eq!(LanguageCode::from_str("TAT").unwrap(), LanguageCode::Tat);
    assert_eq!(LanguageCode::from_str("TEL").unwrap(), LanguageCode::Tel);
    assert_eq!(LanguageCode::from_str("TEM").unwrap(), LanguageCode::Tem);
    assert_eq!(LanguageCode::from_str("TER").unwrap(), LanguageCode::Ter);
    assert_eq!(LanguageCode::from_str("TET").unwrap(), LanguageCode::Tet);
    assert_eq!(LanguageCode::from_str("TGK").unwrap(), LanguageCode::Tgk);
    assert_eq!(LanguageCode::from_str("TGL").unwrap(), LanguageCode::Tgl);
    assert_eq!(LanguageCode::from_str("THA").unwrap(), LanguageCode::Tha);
    assert_eq!(LanguageCode::from_str("TIB").unwrap(), LanguageCode::Tib);
    assert_eq!(LanguageCode::from_str("TIG").unwrap(), LanguageCode::Tig);
    assert_eq!(LanguageCode::from_str("TIR").unwrap(), LanguageCode::Tir);
    assert_eq!(LanguageCode::from_str("TIV").unwrap(), LanguageCode::Tiv);
    assert_eq!(LanguageCode::from_str("TKL").unwrap(), LanguageCode::Tkl);
    assert_eq!(LanguageCode::from_str("TLH").unwrap(), LanguageCode::Tlh);
    assert_eq!(LanguageCode::from_str("TLI").unwrap(), LanguageCode::Tli);
    assert_eq!(LanguageCode::from_str("TMH").unwrap(), LanguageCode::Tmh);
    assert_eq!(LanguageCode::from_str("TOG").unwrap(), LanguageCode::Tog);
    assert_eq!(LanguageCode::from_str("TON").unwrap(), LanguageCode::Ton);
    assert_eq!(LanguageCode::from_str("TPI").unwrap(), LanguageCode::Tpi);
    assert_eq!(LanguageCode::from_str("TSI").unwrap(), LanguageCode::Tsi);
    assert_eq!(LanguageCode::from_str("TSN").unwrap(), LanguageCode::Tsn);
    assert_eq!(LanguageCode::from_str("TSO").unwrap(), LanguageCode::Tso);
    assert_eq!(LanguageCode::from_str("TUK").unwrap(), LanguageCode::Tuk);
    assert_eq!(LanguageCode::from_str("TUM").unwrap(), LanguageCode::Tum);
    assert_eq!(LanguageCode::from_str("TUP").unwrap(), LanguageCode::Tup);
    assert_eq!(LanguageCode::from_str("TUR").unwrap(), LanguageCode::Tur);
    assert_eq!(LanguageCode::from_str("TUT").unwrap(), LanguageCode::Tut);
    assert_eq!(LanguageCode::from_str("TVL").unwrap(), LanguageCode::Tvl);
    assert_eq!(LanguageCode::from_str("TWI").unwrap(), LanguageCode::Twi);
    assert_eq!(LanguageCode::from_str("TYV").unwrap(), LanguageCode::Tyv);
    assert_eq!(LanguageCode::from_str("UDM").unwrap(), LanguageCode::Udm);
    assert_eq!(LanguageCode::from_str("UGA").unwrap(), LanguageCode::Uga);
    assert_eq!(LanguageCode::from_str("UIG").unwrap(), LanguageCode::Uig);
    assert_eq!(LanguageCode::from_str("UKR").unwrap(), LanguageCode::Ukr);
    assert_eq!(LanguageCode::from_str("UMB").unwrap(), LanguageCode::Umb);
    assert_eq!(LanguageCode::from_str("UND").unwrap(), LanguageCode::Und);
    assert_eq!(LanguageCode::from_str("URD").unwrap(), LanguageCode::Urd);
    assert_eq!(LanguageCode::from_str("UZB").unwrap(), LanguageCode::Uzb);
    assert_eq!(LanguageCode::from_str("VAI").unwrap(), LanguageCode::Vai);
    assert_eq!(LanguageCode::from_str("VEN").unwrap(), LanguageCode::Ven);
    assert_eq!(LanguageCode::from_str("VIE").unwrap(), LanguageCode::Vie);
    assert_eq!(LanguageCode::from_str("VOL").unwrap(), LanguageCode::Vol);
    assert_eq!(LanguageCode::from_str("VOT").unwrap(), LanguageCode::Vot);
    assert_eq!(LanguageCode::from_str("WAK").unwrap(), LanguageCode::Wak);
    assert_eq!(LanguageCode::from_str("WAL").unwrap(), LanguageCode::Wal);
    assert_eq!(LanguageCode::from_str("WAR").unwrap(), LanguageCode::War);
    assert_eq!(LanguageCode::from_str("WAS").unwrap(), LanguageCode::Was);
    assert_eq!(LanguageCode::from_str("WEL").unwrap(), LanguageCode::Wel);
    assert_eq!(LanguageCode::from_str("WEN").unwrap(), LanguageCode::Wen);
    assert_eq!(LanguageCode::from_str("WLN").unwrap(), LanguageCode::Wln);
    assert_eq!(LanguageCode::from_str("WOL").unwrap(), LanguageCode::Wol);
    assert_eq!(LanguageCode::from_str("XAL").unwrap(), LanguageCode::Xal);
    assert_eq!(LanguageCode::from_str("XHO").unwrap(), LanguageCode::Xho);
    assert_eq!(LanguageCode::from_str("YAO").unwrap(), LanguageCode::Yao);
    assert_eq!(LanguageCode::from_str("YAP").unwrap(), LanguageCode::Yap);
    assert_eq!(LanguageCode::from_str("YID").unwrap(), LanguageCode::Yid);
    assert_eq!(LanguageCode::from_str("YOR").unwrap(), LanguageCode::Yor);
    assert_eq!(LanguageCode::from_str("YPK").unwrap(), LanguageCode::Ypk);
    assert_eq!(LanguageCode::from_str("ZAP").unwrap(), LanguageCode::Zap);
    assert_eq!(LanguageCode::from_str("ZBL").unwrap(), LanguageCode::Zbl);
    assert_eq!(LanguageCode::from_str("ZEN").unwrap(), LanguageCode::Zen);
    assert_eq!(LanguageCode::from_str("ZGH").unwrap(), LanguageCode::Zgh);
    assert_eq!(LanguageCode::from_str("ZHA").unwrap(), LanguageCode::Zha);
    assert_eq!(LanguageCode::from_str("ZND").unwrap(), LanguageCode::Znd);
    assert_eq!(LanguageCode::from_str("ZUL").unwrap(), LanguageCode::Zul);
    assert_eq!(LanguageCode::from_str("ZUN").unwrap(), LanguageCode::Zun);
    assert_eq!(LanguageCode::from_str("ZXX").unwrap(), LanguageCode::Zxx);
    assert_eq!(LanguageCode::from_str("ZZA").unwrap(), LanguageCode::Zza);

    assert!(LanguageRelation::from_str("ESP").is_err());
    assert!(LanguageRelation::from_str("ZZZ").is_err());
}
