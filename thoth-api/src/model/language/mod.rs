use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::language;
#[cfg(feature = "backend")]
use crate::schema::language_history;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Relation between a language and the original language of a text"),
    ExistingTypePath = "crate::schema::sql_types::LanguageRelation"
)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum LanguageRelation {
    #[default]
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
    LanguageId,
    WorkId,
    LanguageCode,
    LanguageRelation,
    MainLanguage,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub language_id: Uuid,
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new description of a work's language"),
    diesel(table_name = language)
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
    graphql(description = "Set of values required to update an existing description of a work's language"),
    diesel(table_name = language, treat_none_as_null = true)
)]
pub struct PatchLanguage {
    pub language_id: Uuid,
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
}

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Three-letter ISO 639 code representing a language"),
    ExistingTypePath = "crate::schema::sql_types::LanguageCode"
)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum LanguageCode {
    #[cfg_attr(feature = "backend", graphql(description = "Afar"))]
    Aar,
    #[cfg_attr(feature = "backend", graphql(description = "Abkhazian"))]
    Abk,
    #[cfg_attr(feature = "backend", graphql(description = "Achinese"))]
    Ace,
    #[cfg_attr(feature = "backend", graphql(description = "Acoli"))]
    Ach,
    #[cfg_attr(feature = "backend", graphql(description = "Adangme"))]
    Ada,
    #[cfg_attr(feature = "backend", graphql(description = "Adyghe"))]
    Ady,
    #[cfg_attr(feature = "backend", graphql(description = "Afro-Asiatic languages"))]
    Afa,
    #[cfg_attr(feature = "backend", graphql(description = "Afrihili"))]
    Afh,
    #[cfg_attr(feature = "backend", graphql(description = "Afrikaans"))]
    Afr,
    #[cfg_attr(feature = "backend", graphql(description = "Ainu (Japan)"))]
    Ain,
    #[cfg_attr(feature = "backend", graphql(description = "Akan"))]
    Aka,
    #[cfg_attr(feature = "backend", graphql(description = "Akkadian"))]
    Akk,
    #[cfg_attr(feature = "backend", graphql(description = "Albanian"))]
    Alb,
    #[cfg_attr(feature = "backend", graphql(description = "Aleut"))]
    Ale,
    #[cfg_attr(feature = "backend", graphql(description = "Algonquian languages"))]
    Alg,
    #[cfg_attr(feature = "backend", graphql(description = "Southern Altai"))]
    Alt,
    #[cfg_attr(feature = "backend", graphql(description = "Amharic"))]
    Amh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Old English (ca. 450-1100)")
    )]
    Ang,
    #[cfg_attr(feature = "backend", graphql(description = "Angika"))]
    Anp,
    #[cfg_attr(feature = "backend", graphql(description = "Apache languages"))]
    Apa,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic"))]
    Ara,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Official Aramaic (700-300 BCE)")
    )]
    Arc,
    #[cfg_attr(feature = "backend", graphql(description = "Aragonese"))]
    Arg,
    #[cfg_attr(feature = "backend", graphql(description = "Armenian"))]
    Arm,
    #[cfg_attr(feature = "backend", graphql(description = "Mapudungun"))]
    Arn,
    #[cfg_attr(feature = "backend", graphql(description = "Arapaho"))]
    Arp,
    #[cfg_attr(feature = "backend", graphql(description = "Artificial languages"))]
    Art,
    #[cfg_attr(feature = "backend", graphql(description = "Arawak"))]
    Arw,
    #[cfg_attr(feature = "backend", graphql(description = "Assamese"))]
    Asm,
    #[cfg_attr(feature = "backend", graphql(description = "Asturian"))]
    Ast,
    #[cfg_attr(feature = "backend", graphql(description = "Athapascan languages"))]
    Ath,
    #[cfg_attr(feature = "backend", graphql(description = "Australian languages"))]
    Aus,
    #[cfg_attr(feature = "backend", graphql(description = "Avaric"))]
    Ava,
    #[cfg_attr(feature = "backend", graphql(description = "Avestan"))]
    Ave,
    #[cfg_attr(feature = "backend", graphql(description = "Awadhi"))]
    Awa,
    #[cfg_attr(feature = "backend", graphql(description = "Aymara"))]
    Aym,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijani"))]
    Aze,
    #[cfg_attr(feature = "backend", graphql(description = "Banda languages"))]
    Bad,
    #[cfg_attr(feature = "backend", graphql(description = "Bamileke languages"))]
    Bai,
    #[cfg_attr(feature = "backend", graphql(description = "Bashkir"))]
    Bak,
    #[cfg_attr(feature = "backend", graphql(description = "Baluchi"))]
    Bal,
    #[cfg_attr(feature = "backend", graphql(description = "Bambara"))]
    Bam,
    #[cfg_attr(feature = "backend", graphql(description = "Balinese"))]
    Ban,
    #[cfg_attr(feature = "backend", graphql(description = "Basque"))]
    Baq,
    #[cfg_attr(feature = "backend", graphql(description = "Basa (Cameroon)"))]
    Bas,
    #[cfg_attr(feature = "backend", graphql(description = "Baltic languages"))]
    Bat,
    #[cfg_attr(feature = "backend", graphql(description = "Beja"))]
    Bej,
    #[cfg_attr(feature = "backend", graphql(description = "Belarusian"))]
    Bel,
    #[cfg_attr(feature = "backend", graphql(description = "Bemba (Zambia)"))]
    Bem,
    #[cfg_attr(feature = "backend", graphql(description = "Bengali"))]
    Ben,
    #[cfg_attr(feature = "backend", graphql(description = "Berber languages"))]
    Ber,
    #[cfg_attr(feature = "backend", graphql(description = "Bhojpuri"))]
    Bho,
    #[cfg_attr(feature = "backend", graphql(description = "Bihari languages"))]
    Bih,
    #[cfg_attr(feature = "backend", graphql(description = "Bikol"))]
    Bik,
    #[cfg_attr(feature = "backend", graphql(description = "Bini"))]
    Bin,
    #[cfg_attr(feature = "backend", graphql(description = "Bislama"))]
    Bis,
    #[cfg_attr(feature = "backend", graphql(description = "Siksika"))]
    Bla,
    #[cfg_attr(feature = "backend", graphql(description = "Bantu languages"))]
    Bnt,
    #[cfg_attr(feature = "backend", graphql(description = "Bosnian"))]
    Bos,
    #[cfg_attr(feature = "backend", graphql(description = "Braj"))]
    Bra,
    #[cfg_attr(feature = "backend", graphql(description = "Breton"))]
    Bre,
    #[cfg_attr(feature = "backend", graphql(description = "Batak languages"))]
    Btk,
    #[cfg_attr(feature = "backend", graphql(description = "Buriat"))]
    Bua,
    #[cfg_attr(feature = "backend", graphql(description = "Buginese"))]
    Bug,
    #[cfg_attr(feature = "backend", graphql(description = "Bulgarian"))]
    Bul,
    #[cfg_attr(feature = "backend", graphql(description = "Burmese"))]
    Bur,
    #[cfg_attr(feature = "backend", graphql(description = "Bilin"))]
    Byn,
    #[cfg_attr(feature = "backend", graphql(description = "Caddo"))]
    Cad,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Central American Indian languages")
    )]
    Cai,
    #[cfg_attr(feature = "backend", graphql(description = "Galibi Carib"))]
    Car,
    #[cfg_attr(feature = "backend", graphql(description = "Catalan"))]
    Cat,
    #[cfg_attr(feature = "backend", graphql(description = "Caucasian languages"))]
    Cau,
    #[cfg_attr(feature = "backend", graphql(description = "Cebuano"))]
    Ceb,
    #[cfg_attr(feature = "backend", graphql(description = "Celtic languages"))]
    Cel,
    #[cfg_attr(feature = "backend", graphql(description = "Chamorro"))]
    Cha,
    #[cfg_attr(feature = "backend", graphql(description = "Chibcha"))]
    Chb,
    #[cfg_attr(feature = "backend", graphql(description = "Chechen"))]
    Che,
    #[cfg_attr(feature = "backend", graphql(description = "Chagatai"))]
    Chg,
    #[cfg_attr(feature = "backend", graphql(description = "Chinese"))]
    Chi,
    #[cfg_attr(feature = "backend", graphql(description = "Chuukese"))]
    Chk,
    #[cfg_attr(feature = "backend", graphql(description = "Mari (Russia)"))]
    Chm,
    #[cfg_attr(feature = "backend", graphql(description = "Chinook jargon"))]
    Chn,
    #[cfg_attr(feature = "backend", graphql(description = "Choctaw"))]
    Cho,
    #[cfg_attr(feature = "backend", graphql(description = "Chipewyan"))]
    Chp,
    #[cfg_attr(feature = "backend", graphql(description = "Cherokee"))]
    Chr,
    #[cfg_attr(feature = "backend", graphql(description = "Church Slavic"))]
    Chu,
    #[cfg_attr(feature = "backend", graphql(description = "Chuvash"))]
    Chv,
    #[cfg_attr(feature = "backend", graphql(description = "Cheyenne"))]
    Chy,
    #[cfg_attr(feature = "backend", graphql(description = "Chamic languages"))]
    Cmc,
    #[cfg_attr(feature = "backend", graphql(description = "Montenegrin"))]
    Cnr,
    #[cfg_attr(feature = "backend", graphql(description = "Coptic"))]
    Cop,
    #[cfg_attr(feature = "backend", graphql(description = "Cornish"))]
    Cor,
    #[cfg_attr(feature = "backend", graphql(description = "Corsican"))]
    Cos,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Creoles and pidgins, English‑based")
    )]
    Cpe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Creoles and pidgins, French‑based")
    )]
    Cpf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Creoles and pidgins, Portuguese-based")
    )]
    Cpp,
    #[cfg_attr(feature = "backend", graphql(description = "Cree"))]
    Cre,
    #[cfg_attr(feature = "backend", graphql(description = "Crimean Tatar"))]
    Crh,
    #[cfg_attr(feature = "backend", graphql(description = "Creoles and pidgins"))]
    Crp,
    #[cfg_attr(feature = "backend", graphql(description = "Kashubian"))]
    Csb,
    #[cfg_attr(feature = "backend", graphql(description = "Cushitic languages"))]
    Cus,
    #[cfg_attr(feature = "backend", graphql(description = "Czech"))]
    Cze,
    #[cfg_attr(feature = "backend", graphql(description = "Dakota"))]
    Dak,
    #[cfg_attr(feature = "backend", graphql(description = "Danish"))]
    Dan,
    #[cfg_attr(feature = "backend", graphql(description = "Dargwa"))]
    Dar,
    #[cfg_attr(feature = "backend", graphql(description = "Land Dayak languages"))]
    Day,
    #[cfg_attr(feature = "backend", graphql(description = "Delaware"))]
    Del,
    #[cfg_attr(feature = "backend", graphql(description = "Slave (Athapascan)"))]
    Den,
    #[cfg_attr(feature = "backend", graphql(description = "Tlicho"))]
    Dgr,
    #[cfg_attr(feature = "backend", graphql(description = "Dinka"))]
    Din,
    #[cfg_attr(feature = "backend", graphql(description = "Dhivehi"))]
    Div,
    #[cfg_attr(feature = "backend", graphql(description = "Dogri (macrolanguage)"))]
    Doi,
    #[cfg_attr(feature = "backend", graphql(description = "Dravidian languages"))]
    Dra,
    #[cfg_attr(feature = "backend", graphql(description = "Lower Sorbian"))]
    Dsb,
    #[cfg_attr(feature = "backend", graphql(description = "Duala"))]
    Dua,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Middle Dutch (ca. 1050-1350)")
    )]
    Dum,
    #[cfg_attr(feature = "backend", graphql(description = "Dutch"))]
    Dut,
    #[cfg_attr(feature = "backend", graphql(description = "Dyula"))]
    Dyu,
    #[cfg_attr(feature = "backend", graphql(description = "Dzongkha"))]
    Dzo,
    #[cfg_attr(feature = "backend", graphql(description = "Efik"))]
    Efi,
    #[cfg_attr(feature = "backend", graphql(description = "Egyptian (Ancient)"))]
    Egy,
    #[cfg_attr(feature = "backend", graphql(description = "Ekajuk"))]
    Eka,
    #[cfg_attr(feature = "backend", graphql(description = "Elamite"))]
    Elx,
    #[default]
    #[cfg_attr(feature = "backend", graphql(description = "English"))]
    Eng,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Middle English (1100-1500)")
    )]
    Enm,
    #[cfg_attr(feature = "backend", graphql(description = "Esperanto"))]
    Epo,
    #[cfg_attr(feature = "backend", graphql(description = "Estonian"))]
    Est,
    #[cfg_attr(feature = "backend", graphql(description = "Ewe"))]
    Ewe,
    #[cfg_attr(feature = "backend", graphql(description = "Ewondo"))]
    Ewo,
    #[cfg_attr(feature = "backend", graphql(description = "Fang (Equatorial Guinea)"))]
    Fan,
    #[cfg_attr(feature = "backend", graphql(description = "Faroese"))]
    Fao,
    #[cfg_attr(feature = "backend", graphql(description = "Fanti"))]
    Fat,
    #[cfg_attr(feature = "backend", graphql(description = "Fijian"))]
    Fij,
    #[cfg_attr(feature = "backend", graphql(description = "Filipino"))]
    Fil,
    #[cfg_attr(feature = "backend", graphql(description = "Finnish"))]
    Fin,
    #[cfg_attr(feature = "backend", graphql(description = "Finno-Ugrian languages"))]
    Fiu,
    #[cfg_attr(feature = "backend", graphql(description = "Fon"))]
    Fon,
    #[cfg_attr(feature = "backend", graphql(description = "French"))]
    Fre,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Middle French (ca. 1400-1600)")
    )]
    Frm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Old French (842-ca. 1400)")
    )]
    Fro,
    #[cfg_attr(feature = "backend", graphql(description = "Northern Frisian"))]
    Frr,
    #[cfg_attr(feature = "backend", graphql(description = "Eastern Frisian"))]
    Frs,
    #[cfg_attr(feature = "backend", graphql(description = "Western Frisian"))]
    Fry,
    #[cfg_attr(feature = "backend", graphql(description = "Fulah"))]
    Ful,
    #[cfg_attr(feature = "backend", graphql(description = "Friulian"))]
    Fur,
    #[cfg_attr(feature = "backend", graphql(description = "Ga"))]
    Gaa,
    #[cfg_attr(feature = "backend", graphql(description = "Gayo"))]
    Gay,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Gbaya (Central African Republic)")
    )]
    Gba,
    #[cfg_attr(feature = "backend", graphql(description = "Germanic languages"))]
    Gem,
    #[cfg_attr(feature = "backend", graphql(description = "Georgian"))]
    Geo,
    #[cfg_attr(feature = "backend", graphql(description = "German"))]
    Ger,
    #[cfg_attr(feature = "backend", graphql(description = "Geez"))]
    Gez,
    #[cfg_attr(feature = "backend", graphql(description = "Gilbertese"))]
    Gil,
    #[cfg_attr(feature = "backend", graphql(description = "Scottish Gaelic"))]
    Gla,
    #[cfg_attr(feature = "backend", graphql(description = "Irish"))]
    Gle,
    #[cfg_attr(feature = "backend", graphql(description = "Galician"))]
    Glg,
    #[cfg_attr(feature = "backend", graphql(description = "Manx"))]
    Glv,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Middle High German (ca. 1050-1500)")
    )]
    Gmh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Old High German (ca. 750-1050)")
    )]
    Goh,
    #[cfg_attr(feature = "backend", graphql(description = "Gondi"))]
    Gon,
    #[cfg_attr(feature = "backend", graphql(description = "Gorontalo"))]
    Gor,
    #[cfg_attr(feature = "backend", graphql(description = "Gothic"))]
    Got,
    #[cfg_attr(feature = "backend", graphql(description = "Grebo"))]
    Grb,
    #[cfg_attr(feature = "backend", graphql(description = "Ancient Greek (to 1453)"))]
    Grc,
    #[cfg_attr(feature = "backend", graphql(description = "Modern Greek (1453-)"))]
    Gre,
    #[cfg_attr(feature = "backend", graphql(description = "Guarani"))]
    Grn,
    #[cfg_attr(feature = "backend", graphql(description = "Swiss German"))]
    Gsw,
    #[cfg_attr(feature = "backend", graphql(description = "Gujarati"))]
    Guj,
    #[cfg_attr(feature = "backend", graphql(description = "Gwichʼin"))]
    Gwi,
    #[cfg_attr(feature = "backend", graphql(description = "Haida"))]
    Hai,
    #[cfg_attr(feature = "backend", graphql(description = "Haitian"))]
    Hat,
    #[cfg_attr(feature = "backend", graphql(description = "Hausa"))]
    Hau,
    #[cfg_attr(feature = "backend", graphql(description = "Hawaiian"))]
    Haw,
    #[cfg_attr(feature = "backend", graphql(description = "Hebrew"))]
    Heb,
    #[cfg_attr(feature = "backend", graphql(description = "Herero"))]
    Her,
    #[cfg_attr(feature = "backend", graphql(description = "Hiligaynon"))]
    Hil,
    #[cfg_attr(feature = "backend", graphql(description = "Himachali languages"))]
    Him,
    #[cfg_attr(feature = "backend", graphql(description = "Hindi"))]
    Hin,
    #[cfg_attr(feature = "backend", graphql(description = "Hittite"))]
    Hit,
    #[cfg_attr(feature = "backend", graphql(description = "Hmong"))]
    Hmn,
    #[cfg_attr(feature = "backend", graphql(description = "Hiri Motu"))]
    Hmo,
    #[cfg_attr(feature = "backend", graphql(description = "Croatian"))]
    Hrv,
    #[cfg_attr(feature = "backend", graphql(description = "Upper Sorbian"))]
    Hsb,
    #[cfg_attr(feature = "backend", graphql(description = "Hungarian"))]
    Hun,
    #[cfg_attr(feature = "backend", graphql(description = "Hupa"))]
    Hup,
    #[cfg_attr(feature = "backend", graphql(description = "Iban"))]
    Iba,
    #[cfg_attr(feature = "backend", graphql(description = "Igbo"))]
    Ibo,
    #[cfg_attr(feature = "backend", graphql(description = "Icelandic"))]
    Ice,
    #[cfg_attr(feature = "backend", graphql(description = "Ido"))]
    Ido,
    #[cfg_attr(feature = "backend", graphql(description = "Sichuan Yi"))]
    Iii,
    #[cfg_attr(feature = "backend", graphql(description = "Ijo languages"))]
    Ijo,
    #[cfg_attr(feature = "backend", graphql(description = "Inuktitut"))]
    Iku,
    #[cfg_attr(feature = "backend", graphql(description = "Interlingue"))]
    Ile,
    #[cfg_attr(feature = "backend", graphql(description = "Iloko"))]
    Ilo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Interlingua (International Auxiliary Language Association)")
    )]
    Ina,
    #[cfg_attr(feature = "backend", graphql(description = "Indic languages"))]
    Inc,
    #[cfg_attr(feature = "backend", graphql(description = "Indonesian"))]
    Ind,
    #[cfg_attr(feature = "backend", graphql(description = "Indo-European languages"))]
    Ine,
    #[cfg_attr(feature = "backend", graphql(description = "Ingush"))]
    Inh,
    #[cfg_attr(feature = "backend", graphql(description = "Inupiaq"))]
    Ipk,
    #[cfg_attr(feature = "backend", graphql(description = "Iranian languages"))]
    Ira,
    #[cfg_attr(feature = "backend", graphql(description = "Iroquoian languages"))]
    Iro,
    #[cfg_attr(feature = "backend", graphql(description = "Italian"))]
    Ita,
    #[cfg_attr(feature = "backend", graphql(description = "Javanese"))]
    Jav,
    #[cfg_attr(feature = "backend", graphql(description = "Lojban"))]
    Jbo,
    #[cfg_attr(feature = "backend", graphql(description = "Japanese"))]
    Jpn,
    #[cfg_attr(feature = "backend", graphql(description = "Judeo-Persian"))]
    Jpr,
    #[cfg_attr(feature = "backend", graphql(description = "Judeo-Arabic"))]
    Jrb,
    #[cfg_attr(feature = "backend", graphql(description = "Kara-Kalpak"))]
    Kaa,
    #[cfg_attr(feature = "backend", graphql(description = "Kabyle"))]
    Kab,
    #[cfg_attr(feature = "backend", graphql(description = "Kachin"))]
    Kac,
    #[cfg_attr(feature = "backend", graphql(description = "Kalaallisut"))]
    Kal,
    #[cfg_attr(feature = "backend", graphql(description = "Kamba (Kenya)"))]
    Kam,
    #[cfg_attr(feature = "backend", graphql(description = "Kannada"))]
    Kan,
    #[cfg_attr(feature = "backend", graphql(description = "Karen languages"))]
    Kar,
    #[cfg_attr(feature = "backend", graphql(description = "Kashmiri"))]
    Kas,
    #[cfg_attr(feature = "backend", graphql(description = "Kanuri"))]
    Kau,
    #[cfg_attr(feature = "backend", graphql(description = "Kawi"))]
    Kaw,
    #[cfg_attr(feature = "backend", graphql(description = "Kazakh"))]
    Kaz,
    #[cfg_attr(feature = "backend", graphql(description = "Kabardian"))]
    Kbd,
    #[cfg_attr(feature = "backend", graphql(description = "Khasi"))]
    Kha,
    #[cfg_attr(feature = "backend", graphql(description = "Khoisan languages"))]
    Khi,
    #[cfg_attr(feature = "backend", graphql(description = "Khmer"))]
    Khm,
    #[cfg_attr(feature = "backend", graphql(description = "Khotanese"))]
    Kho,
    #[cfg_attr(feature = "backend", graphql(description = "Kikuyu"))]
    Kik,
    #[cfg_attr(feature = "backend", graphql(description = "Kinyarwanda"))]
    Kin,
    #[cfg_attr(feature = "backend", graphql(description = "Kirghiz"))]
    Kir,
    #[cfg_attr(feature = "backend", graphql(description = "Kimbundu"))]
    Kmb,
    #[cfg_attr(feature = "backend", graphql(description = "Konkani (macrolanguage)"))]
    Kok,
    #[cfg_attr(feature = "backend", graphql(description = "Komi"))]
    Kom,
    #[cfg_attr(feature = "backend", graphql(description = "Kongo"))]
    Kon,
    #[cfg_attr(feature = "backend", graphql(description = "Korean"))]
    Kor,
    #[cfg_attr(feature = "backend", graphql(description = "Kosraean"))]
    Kos,
    #[cfg_attr(feature = "backend", graphql(description = "Kpelle"))]
    Kpe,
    #[cfg_attr(feature = "backend", graphql(description = "Karachay-Balkar"))]
    Krc,
    #[cfg_attr(feature = "backend", graphql(description = "Karelian"))]
    Krl,
    #[cfg_attr(feature = "backend", graphql(description = "Kru languages"))]
    Kro,
    #[cfg_attr(feature = "backend", graphql(description = "Kurukh"))]
    Kru,
    #[cfg_attr(feature = "backend", graphql(description = "Kuanyama"))]
    Kua,
    #[cfg_attr(feature = "backend", graphql(description = "Kumyk"))]
    Kum,
    #[cfg_attr(feature = "backend", graphql(description = "Kurdish"))]
    Kur,
    #[cfg_attr(feature = "backend", graphql(description = "Kutenai"))]
    Kut,
    #[cfg_attr(feature = "backend", graphql(description = "Ladino"))]
    Lad,
    #[cfg_attr(feature = "backend", graphql(description = "Lahnda"))]
    Lah,
    #[cfg_attr(feature = "backend", graphql(description = "Lamba"))]
    Lam,
    #[cfg_attr(feature = "backend", graphql(description = "Lao"))]
    Lao,
    #[cfg_attr(feature = "backend", graphql(description = "Latin"))]
    Lat,
    #[cfg_attr(feature = "backend", graphql(description = "Latvian"))]
    Lav,
    #[cfg_attr(feature = "backend", graphql(description = "Lezghian"))]
    Lez,
    #[cfg_attr(feature = "backend", graphql(description = "Limburgan"))]
    Lim,
    #[cfg_attr(feature = "backend", graphql(description = "Lingala"))]
    Lin,
    #[cfg_attr(feature = "backend", graphql(description = "Lithuanian"))]
    Lit,
    #[cfg_attr(feature = "backend", graphql(description = "Mongo"))]
    Lol,
    #[cfg_attr(feature = "backend", graphql(description = "Lozi"))]
    Loz,
    #[cfg_attr(feature = "backend", graphql(description = "Luxembourgish"))]
    Ltz,
    #[cfg_attr(feature = "backend", graphql(description = "Luba-Lulua"))]
    Lua,
    #[cfg_attr(feature = "backend", graphql(description = "Luba-Katanga"))]
    Lub,
    #[cfg_attr(feature = "backend", graphql(description = "Ganda"))]
    Lug,
    #[cfg_attr(feature = "backend", graphql(description = "Luiseno"))]
    Lui,
    #[cfg_attr(feature = "backend", graphql(description = "Lunda"))]
    Lun,
    #[cfg_attr(feature = "backend", graphql(description = "Luo (Kenya and Tanzania)"))]
    Luo,
    #[cfg_attr(feature = "backend", graphql(description = "Lushai"))]
    Lus,
    #[cfg_attr(feature = "backend", graphql(description = "Macedonian"))]
    Mac,
    #[cfg_attr(feature = "backend", graphql(description = "Madurese"))]
    Mad,
    #[cfg_attr(feature = "backend", graphql(description = "Magahi"))]
    Mag,
    #[cfg_attr(feature = "backend", graphql(description = "Marshallese"))]
    Mah,
    #[cfg_attr(feature = "backend", graphql(description = "Maithili"))]
    Mai,
    #[cfg_attr(feature = "backend", graphql(description = "Makasar"))]
    Mak,
    #[cfg_attr(feature = "backend", graphql(description = "Malayalam"))]
    Mal,
    #[cfg_attr(feature = "backend", graphql(description = "Mandingo"))]
    Man,
    #[cfg_attr(feature = "backend", graphql(description = "Maori"))]
    Mao,
    #[cfg_attr(feature = "backend", graphql(description = "Austronesian languages"))]
    Map,
    #[cfg_attr(feature = "backend", graphql(description = "Marathi"))]
    Mar,
    #[cfg_attr(feature = "backend", graphql(description = "Masai"))]
    Mas,
    #[cfg_attr(feature = "backend", graphql(description = "Malay (macrolanguage)"))]
    May,
    #[cfg_attr(feature = "backend", graphql(description = "Moksha"))]
    Mdf,
    #[cfg_attr(feature = "backend", graphql(description = "Mandar"))]
    Mdr,
    #[cfg_attr(feature = "backend", graphql(description = "Mende (Sierra Leone)"))]
    Men,
    #[cfg_attr(feature = "backend", graphql(description = "Middle Irish (900-1200)"))]
    Mga,
    #[cfg_attr(feature = "backend", graphql(description = "Mi'kmaq"))]
    Mic,
    #[cfg_attr(feature = "backend", graphql(description = "Minangkabau"))]
    Min,
    #[cfg_attr(feature = "backend", graphql(description = "Uncoded languages"))]
    Mis,
    #[cfg_attr(feature = "backend", graphql(description = "Mon-Khmer languages"))]
    Mkh,
    #[cfg_attr(feature = "backend", graphql(description = "Malagasy"))]
    Mlg,
    #[cfg_attr(feature = "backend", graphql(description = "Maltese"))]
    Mlt,
    #[cfg_attr(feature = "backend", graphql(description = "Manchu"))]
    Mnc,
    #[cfg_attr(feature = "backend", graphql(description = "Manipuri"))]
    Mni,
    #[cfg_attr(feature = "backend", graphql(description = "Manobo languages"))]
    Mno,
    #[cfg_attr(feature = "backend", graphql(description = "Mohawk"))]
    Moh,
    #[cfg_attr(feature = "backend", graphql(description = "Mongolian"))]
    Mon,
    #[cfg_attr(feature = "backend", graphql(description = "Mossi"))]
    Mos,
    #[cfg_attr(feature = "backend", graphql(description = "Multiple languages"))]
    Mul,
    #[cfg_attr(feature = "backend", graphql(description = "Munda languages"))]
    Mun,
    #[cfg_attr(feature = "backend", graphql(description = "Creek"))]
    Mus,
    #[cfg_attr(feature = "backend", graphql(description = "Mirandese"))]
    Mwl,
    #[cfg_attr(feature = "backend", graphql(description = "Marwari"))]
    Mwr,
    #[cfg_attr(feature = "backend", graphql(description = "Mayan languages"))]
    Myn,
    #[cfg_attr(feature = "backend", graphql(description = "Erzya"))]
    Myv,
    #[cfg_attr(feature = "backend", graphql(description = "Nahuatl languages"))]
    Nah,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "North American Indian languages")
    )]
    Nai,
    #[cfg_attr(feature = "backend", graphql(description = "Neapolitan"))]
    Nap,
    #[cfg_attr(feature = "backend", graphql(description = "Nauru"))]
    Nau,
    #[cfg_attr(feature = "backend", graphql(description = "Navajo"))]
    Nav,
    #[cfg_attr(feature = "backend", graphql(description = "South Ndebele"))]
    Nbl,
    #[cfg_attr(feature = "backend", graphql(description = "North Ndebele"))]
    Nde,
    #[cfg_attr(feature = "backend", graphql(description = "Ndonga"))]
    Ndo,
    #[cfg_attr(feature = "backend", graphql(description = "Low German"))]
    Nds,
    #[cfg_attr(feature = "backend", graphql(description = "Nepali (macrolanguage)"))]
    Nep,
    #[cfg_attr(feature = "backend", graphql(description = "Newari"))]
    New,
    #[cfg_attr(feature = "backend", graphql(description = "Nias"))]
    Nia,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Niger-Kordofanian languages")
    )]
    Nic,
    #[cfg_attr(feature = "backend", graphql(description = "Niuean"))]
    Niu,
    #[cfg_attr(feature = "backend", graphql(description = "Norwegian Nynorsk"))]
    Nno,
    #[cfg_attr(feature = "backend", graphql(description = "Norwegian Bokmål"))]
    Nob,
    #[cfg_attr(feature = "backend", graphql(description = "Nogai"))]
    Nog,
    #[cfg_attr(feature = "backend", graphql(description = "Old Norse"))]
    Non,
    #[cfg_attr(feature = "backend", graphql(description = "Norwegian"))]
    Nor,
    #[cfg_attr(feature = "backend", graphql(description = "N'Ko"))]
    Nqo,
    #[cfg_attr(feature = "backend", graphql(description = "Pedi"))]
    Nso,
    #[cfg_attr(feature = "backend", graphql(description = "Nubian languages"))]
    Nub,
    #[cfg_attr(feature = "backend", graphql(description = "Classical Newari"))]
    Nwc,
    #[cfg_attr(feature = "backend", graphql(description = "Nyanja"))]
    Nya,
    #[cfg_attr(feature = "backend", graphql(description = "Nyamwezi"))]
    Nym,
    #[cfg_attr(feature = "backend", graphql(description = "Nyankole"))]
    Nyn,
    #[cfg_attr(feature = "backend", graphql(description = "Nyoro"))]
    Nyo,
    #[cfg_attr(feature = "backend", graphql(description = "Nzima"))]
    Nzi,
    #[cfg_attr(feature = "backend", graphql(description = "Occitan (post 1500)"))]
    Oci,
    #[cfg_attr(feature = "backend", graphql(description = "Ojibwa"))]
    Oji,
    #[cfg_attr(feature = "backend", graphql(description = "Oriya (macrolanguage)"))]
    Ori,
    #[cfg_attr(feature = "backend", graphql(description = "Oromo"))]
    Orm,
    #[cfg_attr(feature = "backend", graphql(description = "Osage"))]
    Osa,
    #[cfg_attr(feature = "backend", graphql(description = "Ossetian"))]
    Oss,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Ottoman Turkish (1500-1928)")
    )]
    Ota,
    #[cfg_attr(feature = "backend", graphql(description = "Otomian languages"))]
    Oto,
    #[cfg_attr(feature = "backend", graphql(description = "Papuan languages"))]
    Paa,
    #[cfg_attr(feature = "backend", graphql(description = "Pangasinan"))]
    Pag,
    #[cfg_attr(feature = "backend", graphql(description = "Pahlavi"))]
    Pal,
    #[cfg_attr(feature = "backend", graphql(description = "Pampanga"))]
    Pam,
    #[cfg_attr(feature = "backend", graphql(description = "Panjabi"))]
    Pan,
    #[cfg_attr(feature = "backend", graphql(description = "Papiamento"))]
    Pap,
    #[cfg_attr(feature = "backend", graphql(description = "Palauan"))]
    Pau,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Old Persian (ca. 600-400 B.C.)")
    )]
    Peo,
    #[cfg_attr(feature = "backend", graphql(description = "Persian"))]
    Per,
    #[cfg_attr(feature = "backend", graphql(description = "Philippine languages"))]
    Phi,
    #[cfg_attr(feature = "backend", graphql(description = "Phoenician"))]
    Phn,
    #[cfg_attr(feature = "backend", graphql(description = "Pali"))]
    Pli,
    #[cfg_attr(feature = "backend", graphql(description = "Polish"))]
    Pol,
    #[cfg_attr(feature = "backend", graphql(description = "Pohnpeian"))]
    Pon,
    #[cfg_attr(feature = "backend", graphql(description = "Portuguese"))]
    Por,
    #[cfg_attr(feature = "backend", graphql(description = "Prakrit languages"))]
    Pra,
    #[cfg_attr(feature = "backend", graphql(description = "Old Provençal (to 1500)"))]
    Pro,
    #[cfg_attr(feature = "backend", graphql(description = "Pushto"))]
    Pus,
    #[cfg_attr(feature = "backend", graphql(description = "Reserved for local use"))]
    Qaa,
    #[cfg_attr(feature = "backend", graphql(description = "Quechua"))]
    Que,
    #[cfg_attr(feature = "backend", graphql(description = "Rajasthani"))]
    Raj,
    #[cfg_attr(feature = "backend", graphql(description = "Rapanui"))]
    Rap,
    #[cfg_attr(feature = "backend", graphql(description = "Rarotongan"))]
    Rar,
    #[cfg_attr(feature = "backend", graphql(description = "Romance languages"))]
    Roa,
    #[cfg_attr(feature = "backend", graphql(description = "Romansh"))]
    Roh,
    #[cfg_attr(feature = "backend", graphql(description = "Romany"))]
    Rom,
    #[cfg_attr(feature = "backend", graphql(description = "Romanian"))]
    Rum,
    #[cfg_attr(feature = "backend", graphql(description = "Rundi"))]
    Run,
    #[cfg_attr(feature = "backend", graphql(description = "Macedo-Romanian"))]
    Rup,
    #[cfg_attr(feature = "backend", graphql(description = "Russian"))]
    Rus,
    #[cfg_attr(feature = "backend", graphql(description = "Sandawe"))]
    Sad,
    #[cfg_attr(feature = "backend", graphql(description = "Sango"))]
    Sag,
    #[cfg_attr(feature = "backend", graphql(description = "Yakut"))]
    Sah,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "South American Indian languages")
    )]
    Sai,
    #[cfg_attr(feature = "backend", graphql(description = "Salishan languages"))]
    Sal,
    #[cfg_attr(feature = "backend", graphql(description = "Samaritan Aramaic"))]
    Sam,
    #[cfg_attr(feature = "backend", graphql(description = "Sanskrit"))]
    San,
    #[cfg_attr(feature = "backend", graphql(description = "Sasak"))]
    Sas,
    #[cfg_attr(feature = "backend", graphql(description = "Santali"))]
    Sat,
    #[cfg_attr(feature = "backend", graphql(description = "Sicilian"))]
    Scn,
    #[cfg_attr(feature = "backend", graphql(description = "Scots"))]
    Sco,
    #[cfg_attr(feature = "backend", graphql(description = "Selkup"))]
    Sel,
    #[cfg_attr(feature = "backend", graphql(description = "Semitic languages"))]
    Sem,
    #[cfg_attr(feature = "backend", graphql(description = "Old Irish (to 900)"))]
    Sga,
    #[cfg_attr(feature = "backend", graphql(description = "sign languages"))]
    Sgn,
    #[cfg_attr(feature = "backend", graphql(description = "Shan"))]
    Shn,
    #[cfg_attr(feature = "backend", graphql(description = "Sidamo"))]
    Sid,
    #[cfg_attr(feature = "backend", graphql(description = "Sinhala"))]
    Sin,
    #[cfg_attr(feature = "backend", graphql(description = "Siouan languages"))]
    Sio,
    #[cfg_attr(feature = "backend", graphql(description = "Sino-Tibetan languages"))]
    Sit,
    #[cfg_attr(feature = "backend", graphql(description = "Slavic languages"))]
    Sla,
    #[cfg_attr(feature = "backend", graphql(description = "Slovak"))]
    Slo,
    #[cfg_attr(feature = "backend", graphql(description = "Slovenian"))]
    Slv,
    #[cfg_attr(feature = "backend", graphql(description = "Southern Sami"))]
    Sma,
    #[cfg_attr(feature = "backend", graphql(description = "Northern Sami"))]
    Sme,
    #[cfg_attr(feature = "backend", graphql(description = "Sami languages"))]
    Smi,
    #[cfg_attr(feature = "backend", graphql(description = "Lule Sami"))]
    Smj,
    #[cfg_attr(feature = "backend", graphql(description = "Inari Sami"))]
    Smn,
    #[cfg_attr(feature = "backend", graphql(description = "Samoan"))]
    Smo,
    #[cfg_attr(feature = "backend", graphql(description = "Skolt Sami"))]
    Sms,
    #[cfg_attr(feature = "backend", graphql(description = "Shona"))]
    Sna,
    #[cfg_attr(feature = "backend", graphql(description = "Sindhi"))]
    Snd,
    #[cfg_attr(feature = "backend", graphql(description = "Soninke"))]
    Snk,
    #[cfg_attr(feature = "backend", graphql(description = "Sogdian"))]
    Sog,
    #[cfg_attr(feature = "backend", graphql(description = "Somali"))]
    Som,
    #[cfg_attr(feature = "backend", graphql(description = "Songhai languages"))]
    Son,
    #[cfg_attr(feature = "backend", graphql(description = "Southern Sotho"))]
    Sot,
    #[cfg_attr(feature = "backend", graphql(description = "Spanish"))]
    Spa,
    #[cfg_attr(feature = "backend", graphql(description = "Sardinian"))]
    Srd,
    #[cfg_attr(feature = "backend", graphql(description = "Sranan Tongo"))]
    Srn,
    #[cfg_attr(feature = "backend", graphql(description = "Serbian"))]
    Srp,
    #[cfg_attr(feature = "backend", graphql(description = "Serer"))]
    Srr,
    #[cfg_attr(feature = "backend", graphql(description = "Nilo-Saharan languages"))]
    Ssa,
    #[cfg_attr(feature = "backend", graphql(description = "Swati"))]
    Ssw,
    #[cfg_attr(feature = "backend", graphql(description = "Sukuma"))]
    Suk,
    #[cfg_attr(feature = "backend", graphql(description = "Sundanese"))]
    Sun,
    #[cfg_attr(feature = "backend", graphql(description = "Susu"))]
    Sus,
    #[cfg_attr(feature = "backend", graphql(description = "Sumerian"))]
    Sux,
    #[cfg_attr(feature = "backend", graphql(description = "Swahili (macrolanguage)"))]
    Swa,
    #[cfg_attr(feature = "backend", graphql(description = "Swedish"))]
    Swe,
    #[cfg_attr(feature = "backend", graphql(description = "Classical Syriac"))]
    Syc,
    #[cfg_attr(feature = "backend", graphql(description = "Syriac"))]
    Syr,
    #[cfg_attr(feature = "backend", graphql(description = "Tahitian"))]
    Tah,
    #[cfg_attr(feature = "backend", graphql(description = "Tai languages"))]
    Tai,
    #[cfg_attr(feature = "backend", graphql(description = "Tamil"))]
    Tam,
    #[cfg_attr(feature = "backend", graphql(description = "Tatar"))]
    Tat,
    #[cfg_attr(feature = "backend", graphql(description = "Telugu"))]
    Tel,
    #[cfg_attr(feature = "backend", graphql(description = "Timne"))]
    Tem,
    #[cfg_attr(feature = "backend", graphql(description = "Tereno"))]
    Ter,
    #[cfg_attr(feature = "backend", graphql(description = "Tetum"))]
    Tet,
    #[cfg_attr(feature = "backend", graphql(description = "Tajik"))]
    Tgk,
    #[cfg_attr(feature = "backend", graphql(description = "Tagalog"))]
    Tgl,
    #[cfg_attr(feature = "backend", graphql(description = "Thai"))]
    Tha,
    #[cfg_attr(feature = "backend", graphql(description = "Tibetan"))]
    Tib,
    #[cfg_attr(feature = "backend", graphql(description = "Tigre"))]
    Tig,
    #[cfg_attr(feature = "backend", graphql(description = "Tigrinya"))]
    Tir,
    #[cfg_attr(feature = "backend", graphql(description = "Tiv"))]
    Tiv,
    #[cfg_attr(feature = "backend", graphql(description = "Tokelau"))]
    Tkl,
    #[cfg_attr(feature = "backend", graphql(description = "Klingon"))]
    Tlh,
    #[cfg_attr(feature = "backend", graphql(description = "Tlingit"))]
    Tli,
    #[cfg_attr(feature = "backend", graphql(description = "Tamashek"))]
    Tmh,
    #[cfg_attr(feature = "backend", graphql(description = "Tonga (Nyasa)"))]
    Tog,
    #[cfg_attr(feature = "backend", graphql(description = "Tonga (Tonga Islands)"))]
    Ton,
    #[cfg_attr(feature = "backend", graphql(description = "Tok Pisin"))]
    Tpi,
    #[cfg_attr(feature = "backend", graphql(description = "Tsimshian"))]
    Tsi,
    #[cfg_attr(feature = "backend", graphql(description = "Tswana"))]
    Tsn,
    #[cfg_attr(feature = "backend", graphql(description = "Tsonga"))]
    Tso,
    #[cfg_attr(feature = "backend", graphql(description = "Turkmen"))]
    Tuk,
    #[cfg_attr(feature = "backend", graphql(description = "Tumbuka"))]
    Tum,
    #[cfg_attr(feature = "backend", graphql(description = "Tupi languages"))]
    Tup,
    #[cfg_attr(feature = "backend", graphql(description = "Turkish"))]
    Tur,
    #[cfg_attr(feature = "backend", graphql(description = "Altaic languages"))]
    Tut,
    #[cfg_attr(feature = "backend", graphql(description = "Tuvalu"))]
    Tvl,
    #[cfg_attr(feature = "backend", graphql(description = "Twi"))]
    Twi,
    #[cfg_attr(feature = "backend", graphql(description = "Tuvinian"))]
    Tyv,
    #[cfg_attr(feature = "backend", graphql(description = "Udmurt"))]
    Udm,
    #[cfg_attr(feature = "backend", graphql(description = "Ugaritic"))]
    Uga,
    #[cfg_attr(feature = "backend", graphql(description = "Uighur"))]
    Uig,
    #[cfg_attr(feature = "backend", graphql(description = "Ukrainian"))]
    Ukr,
    #[cfg_attr(feature = "backend", graphql(description = "Umbundu"))]
    Umb,
    #[cfg_attr(feature = "backend", graphql(description = "Undetermined"))]
    Und,
    #[cfg_attr(feature = "backend", graphql(description = "Urdu"))]
    Urd,
    #[cfg_attr(feature = "backend", graphql(description = "Uzbek"))]
    Uzb,
    #[cfg_attr(feature = "backend", graphql(description = "Vai"))]
    Vai,
    #[cfg_attr(feature = "backend", graphql(description = "Venda"))]
    Ven,
    #[cfg_attr(feature = "backend", graphql(description = "Vietnamese"))]
    Vie,
    #[cfg_attr(feature = "backend", graphql(description = "Volapük"))]
    Vol,
    #[cfg_attr(feature = "backend", graphql(description = "Votic"))]
    Vot,
    #[cfg_attr(feature = "backend", graphql(description = "Wakashan languages"))]
    Wak,
    #[cfg_attr(feature = "backend", graphql(description = "Wolaytta"))]
    Wal,
    #[cfg_attr(feature = "backend", graphql(description = "Waray (Philippines)"))]
    War,
    #[cfg_attr(feature = "backend", graphql(description = "Washo"))]
    Was,
    #[cfg_attr(feature = "backend", graphql(description = "Welsh"))]
    Wel,
    #[cfg_attr(feature = "backend", graphql(description = "Sorbian languages"))]
    Wen,
    #[cfg_attr(feature = "backend", graphql(description = "Walloon"))]
    Wln,
    #[cfg_attr(feature = "backend", graphql(description = "Wolof"))]
    Wol,
    #[cfg_attr(feature = "backend", graphql(description = "Kalmyk"))]
    Xal,
    #[cfg_attr(feature = "backend", graphql(description = "Xhosa"))]
    Xho,
    #[cfg_attr(feature = "backend", graphql(description = "Yao"))]
    Yao,
    #[cfg_attr(feature = "backend", graphql(description = "Yapese"))]
    Yap,
    #[cfg_attr(feature = "backend", graphql(description = "Yiddish"))]
    Yid,
    #[cfg_attr(feature = "backend", graphql(description = "Yoruba"))]
    Yor,
    #[cfg_attr(feature = "backend", graphql(description = "Yupik languages"))]
    Ypk,
    #[cfg_attr(feature = "backend", graphql(description = "Zapotec"))]
    Zap,
    #[cfg_attr(feature = "backend", graphql(description = "Blissymbols"))]
    Zbl,
    #[cfg_attr(feature = "backend", graphql(description = "Zenaga"))]
    Zen,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Standard Moroccan Tamazight")
    )]
    Zgh,
    #[cfg_attr(feature = "backend", graphql(description = "Zhuang"))]
    Zha,
    #[cfg_attr(feature = "backend", graphql(description = "Zande languages"))]
    Znd,
    #[cfg_attr(feature = "backend", graphql(description = "Zulu"))]
    Zul,
    #[cfg_attr(feature = "backend", graphql(description = "Zuni"))]
    Zun,
    #[cfg_attr(feature = "backend", graphql(description = "No linguistic content"))]
    Zxx,
    #[cfg_attr(feature = "backend", graphql(description = "Zaza"))]
    Zza,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct LanguageHistory {
    pub language_history_id: Uuid,
    pub language_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = language_history)
)]
pub struct NewLanguageHistory {
    pub language_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

impl Default for Language {
    fn default() -> Language {
        Language {
            language_id: Default::default(),
            work_id: Default::default(),
            language_code: Default::default(),
            language_relation: Default::default(),
            main_language: true,
            created_at: Default::default(),
            updated_at: Default::default(),
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
    use std::str::FromStr;
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
    use std::str::FromStr;
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

#[cfg(feature = "backend")]
pub mod crud;
