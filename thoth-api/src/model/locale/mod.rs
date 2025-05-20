use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use strum::Display;
use strum::EnumString;

use crate::schema::locale;
use crate::schema::sql_types::LocaleCode as LocaleCodeSql;


#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "BCP-47 code representing locale"),
    ExistingTypePath = "crate::schema::sql_types::LocaleCode"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum LocaleCode {
    #[default]
    #[cfg_attr(feature = "backend", graphql(description = "English"))]
    En,
    #[cfg_attr(feature = "backend", graphql(description = "Afrikaans"))]
    Af,
    #[cfg_attr(feature = "backend", graphql(description = "Afrikaans (Namibia)"))]
    AfNa,
    #[cfg_attr(feature = "backend", graphql(description = "Afrikaans (South Africa)"))]
    AfZa,
    #[cfg_attr(feature = "backend", graphql(description = "Aghem"))]
    Agq,
    #[cfg_attr(feature = "backend", graphql(description = "Aghem (Cameroon)"))]
    AgqCm,
    #[cfg_attr(feature = "backend", graphql(description = "Akan"))]
    Ak,
    #[cfg_attr(feature = "backend", graphql(description = "Akan (Ghana)"))]
    AkGh,
    #[cfg_attr(feature = "backend", graphql(description = "Albanian"))]
    Sq,
    #[cfg_attr(feature = "backend", graphql(description = "Albanian (Albania)"))]
    SqAl,
    #[cfg_attr(feature = "backend", graphql(description = "Amharic"))]
    Am,
    #[cfg_attr(feature = "backend", graphql(description = "Amharic (Ethiopia)"))]
    AmEt,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic"))]
    Ar,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Algeria)"))]
    ArDz,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Bahrain)"))]
    ArBh,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Egypt)"))]
    ArEg,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Iraq)"))]
    ArIq,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Jordan)"))]
    ArJo,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Kuwait)"))]
    ArKw,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Lebanon)"))]
    ArLb,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Libya)"))]
    ArLy,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Morocco)"))]
    ArMa,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Oman)"))]
    ArOm,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Qatar)"))]
    ArQa,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Saudi Arabia)"))]
    ArSa,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Sudan)"))]
    ArSd,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Syria)"))]
    ArSy,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Tunisia)"))]
    ArTn,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (United Arab Emirates)"))]
    ArAe,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (World)"))]
    Ar001,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Yemen)"))]
    ArYe,
    #[cfg_attr(feature = "backend", graphql(description = "Armenian"))]
    Hy,
    #[cfg_attr(feature = "backend", graphql(description = "Armenian (Armenia)"))]
    HyAm,
    #[cfg_attr(feature = "backend", graphql(description = "Assamese"))]
    As,
    #[cfg_attr(feature = "backend", graphql(description = "Assamese (India)"))]
    AsIn,
    #[cfg_attr(feature = "backend", graphql(description = "Asturian"))]
    Ast,
    #[cfg_attr(feature = "backend", graphql(description = "Asturian (Spain)"))]
    AstEs,
    #[cfg_attr(feature = "backend", graphql(description = "Asu"))]
    Asa,
    #[cfg_attr(feature = "backend", graphql(description = "Asu (Tanzania)"))]
    AsaTz,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijani"))]
    Az,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijani (Cyrillic)"))]
    AzCyrl,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijani (Cyrillic, Azerbaijan)"))]
    AzCyrlAz,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijani (Latin)"))]
    AzLatn,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijani (Latin, Azerbaijan)"))]
    AzLatnAz,
    #[cfg_attr(feature = "backend", graphql(description = "Bafia"))]
    Ksf,
    #[cfg_attr(feature = "backend", graphql(description = "Bafia (Cameroon)"))]
    KsfCm,
    #[cfg_attr(feature = "backend", graphql(description = "Bambara"))]
    Bm,
    #[cfg_attr(feature = "backend", graphql(description = "Bambara (Mali)"))]
    BmMl,
    #[cfg_attr(feature = "backend", graphql(description = "Basaa"))]
    Bas,
    #[cfg_attr(feature = "backend", graphql(description = "Basaa (Cameroon)"))]
    BasCm,
    #[cfg_attr(feature = "backend", graphql(description = "Basque"))]
    Eu,
    #[cfg_attr(feature = "backend", graphql(description = "Basque (Spain)"))]
    EuEs,
    #[cfg_attr(feature = "backend", graphql(description = "Belarusian"))]
    Be,
    #[cfg_attr(feature = "backend", graphql(description = "Belarusian (Belarus)"))]
    BeBy,
    #[cfg_attr(feature = "backend", graphql(description = "Bemba"))]
    Bem,
    #[cfg_attr(feature = "backend", graphql(description = "Bemba (Zambia)"))]
    BemZm,
    #[cfg_attr(feature = "backend", graphql(description = "Bena"))]
    Bez,
    #[cfg_attr(feature = "backend", graphql(description = "Bena (Tanzania)"))]
    BezTz,
    #[cfg_attr(feature = "backend", graphql(description = "Bengali"))]
    Bn,
    #[cfg_attr(feature = "backend", graphql(description = "Bengali (Bangladesh)"))]
    BnBd,
    #[cfg_attr(feature = "backend", graphql(description = "Bengali (India)"))]
    BnIn,
}

impl LocaleCode {
    pub fn to_string(&self) -> String {
        self.to_string().to_lowercase()
    }
}
