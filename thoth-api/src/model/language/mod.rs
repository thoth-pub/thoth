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
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(
        description = "Relation between a language listed for a work and the original language of the work's text"
    ),
    ExistingTypePath = "crate::schema::sql_types::LanguageRelation"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum LanguageRelation {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Original language of the text")
    )]
    #[default]
    Original,
    #[cfg_attr(
        feature = "backend",
        db_rename = "translated-from",
        graphql(description = "Language from which the text was translated")
    )]
    TranslatedFrom,
    #[cfg_attr(
        feature = "backend",
        db_rename = "translated-into",
        graphql(description = "Language into which the text has been translated")
    )]
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
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub language_id: Uuid,
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new description of a work's language"),
    diesel(table_name = language)
)]
pub struct NewLanguage {
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing description of a work's language"),
    diesel(table_name = language, treat_none_as_null = true)
)]
pub struct PatchLanguage {
    pub language_id: Uuid,
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Three-letter ISO 639 code representing a language"),
    ExistingTypePath = "crate::schema::sql_types::LanguageCode"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
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

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct LanguageHistory {
    pub language_history_id: Uuid,
    pub language_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = language_history)
)]
pub struct NewLanguageHistory {
    pub language_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::LanguagePolicy;
#[cfg(test)]
mod tests;
