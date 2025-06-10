use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;

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
    #[cfg_attr(feature = "backend", graphql(description = "Afrikaans (af)"))]
    Af,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Afrikaans (Namibia) (af-NA)")
    )]
    AfNa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Afrikaans (South Africa) (af-ZA)")
    )]
    AfZa,
    #[cfg_attr(feature = "backend", graphql(description = "Aghem (agq)"))]
    Agq,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Aghem (Cameroon) (agq-CM)")
    )]
    AgqCm,
    #[cfg_attr(feature = "backend", graphql(description = "Akan (ak)"))]
    Ak,
    #[cfg_attr(feature = "backend", graphql(description = "Akan (Ghana) (ak-GH)"))]
    AkGh,
    #[cfg_attr(feature = "backend", graphql(description = "Albanian (sq)"))]
    Sq,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Albanian (Albania) (sq-AL)")
    )]
    SqAl,
    #[cfg_attr(feature = "backend", graphql(description = "Amharic (am)"))]
    Am,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Amharic (Ethiopia) (am-ET)")
    )]
    AmEt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Antigua and Barbuda Creole English")
    )]
    Aig,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (ar)"))]
    Ar,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Algeria) (ar-DZ)"))]
    ArDz,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Bahrain) (ar-BH)"))]
    ArBh,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Egypt) (ar-EG)"))]
    ArEg,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Iraq) (ar-IQ)"))]
    ArIq,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Jordan) (ar-JO)"))]
    ArJo,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Kuwait) (ar-KW)"))]
    ArKw,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Lebanon) (ar-LB)"))]
    ArLb,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Libya) (ar-LY)"))]
    ArLy,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Morocco) (ar-MA)"))]
    ArMa,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Oman) (ar-OM)"))]
    ArOm,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Qatar) (ar-QA)"))]
    ArQa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Arabic (Saudi Arabia) (ar-SA)")
    )]
    ArSa,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Sudan) (ar-SD)"))]
    ArSd,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Syria) (ar-SY)"))]
    ArSy,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Tunisia) (ar-TN)"))]
    ArTn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Arabic (United Arab Emirates) (ar-AE)")
    )]
    ArAe,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (World) (ar-001)"))]
    Ar001,
    #[cfg_attr(feature = "backend", graphql(description = "Arabic (Yemen) (ar-YE)"))]
    ArYe,
    #[cfg_attr(feature = "backend", graphql(description = "Armenian (hy)"))]
    Hy,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Armenian (Armenia) (hy-AM)")
    )]
    HyAm,
    #[cfg_attr(feature = "backend", graphql(description = "Assamese (as)"))]
    As,
    #[cfg_attr(feature = "backend", graphql(description = "Assamese (India) (as-IN)"))]
    AsIn,
    #[cfg_attr(feature = "backend", graphql(description = "Asturian (ast)"))]
    Ast,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Asturian (Spain) (ast-ES)")
    )]
    AstEs,
    #[cfg_attr(feature = "backend", graphql(description = "Asu (asa)"))]
    Asa,
    #[cfg_attr(feature = "backend", graphql(description = "Asu (Tanzania) (asa-TZ)"))]
    AsaTz,
    #[cfg_attr(feature = "backend", graphql(description = "Azerbaijani (az)"))]
    Az,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Azerbaijani (Cyrillic) (az-Cyrl)")
    )]
    AzCyrl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Azerbaijani (Cyrillic, Azerbaijan) (az-Cyrl-AZ)")
    )]
    AzCyrlAz,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Azerbaijani (Latin) (az-Latn)")
    )]
    AzLatn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Azerbaijani (Latin, Azerbaijan) (az-Latn-AZ)")
    )]
    AzLatnAz,
    #[cfg_attr(feature = "backend", graphql(description = "Bafia (ksf)"))]
    Ksf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Bafia (Cameroon) (ksf-CM)")
    )]
    KsfCm,
    #[cfg_attr(feature = "backend", graphql(description = "Bahamas Creole English"))]
    Bah,
    #[cfg_attr(feature = "backend", graphql(description = "Bambara (bm)"))]
    Bm,
    #[cfg_attr(feature = "backend", graphql(description = "Bambara (Mali) (bm-ML)"))]
    BmMl,
    #[cfg_attr(feature = "backend", graphql(description = "Basaa (bas)"))]
    Bas,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Basaa (Cameroon) (bas-CM)")
    )]
    BasCm,
    #[cfg_attr(feature = "backend", graphql(description = "Basque (eu)"))]
    Eu,
    #[cfg_attr(feature = "backend", graphql(description = "Basque (Spain) (eu-ES)"))]
    EuEs,
    #[cfg_attr(feature = "backend", graphql(description = "Belarusian (be)"))]
    Be,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Belarusian (Belarus) (be-BY)")
    )]
    BeBy,
    #[cfg_attr(feature = "backend", graphql(description = "Bemba (bem)"))]
    Bem,
    #[cfg_attr(feature = "backend", graphql(description = "Bemba (Zambia) (bem-ZM)"))]
    BemZm,
    #[cfg_attr(feature = "backend", graphql(description = "Bena (bez)"))]
    Bez,
    #[cfg_attr(feature = "backend", graphql(description = "Bena (Tanzania) (bez-TZ)"))]
    BezTz,
    #[cfg_attr(feature = "backend", graphql(description = "Bengali (bn)"))]
    Bn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Bengali (Bangladesh) (bn-BD)")
    )]
    BnBd,
    #[cfg_attr(feature = "backend", graphql(description = "Bengali (India) (bn-IN)"))]
    BnIn,
    #[cfg_attr(feature = "backend", graphql(description = "Bodo (brx)"))]
    Brx,
    #[cfg_attr(feature = "backend", graphql(description = "Bodo (India) (brx-IN)"))]
    BrxIn,
    #[cfg_attr(feature = "backend", graphql(description = "Bosnian (bs)"))]
    Bs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Bosnian (Bosnia and Herzegovina) (bs-BA)")
    )]
    BsBa,
    #[cfg_attr(feature = "backend", graphql(description = "Breton (br)"))]
    Br,
    #[cfg_attr(feature = "backend", graphql(description = "Breton (France) (br-FR)"))]
    BrFr,
    #[cfg_attr(feature = "backend", graphql(description = "Bulgarian (bg)"))]
    Bg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Bulgarian (Bulgaria) (bg-BG)")
    )]
    BgBg,
    #[cfg_attr(feature = "backend", graphql(description = "Burmese (my)"))]
    My,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Burmese (Myanmar [Burma]) (my-MM)")
    )]
    MyMm,
    #[cfg_attr(feature = "backend", graphql(description = "Catalan (ca)"))]
    Ca,
    #[cfg_attr(feature = "backend", graphql(description = "Catalan (Spain) (ca-ES)"))]
    CaEs,
    #[cfg_attr(feature = "backend", graphql(description = "Central Kurdish (ckb)"))]
    Ckb,
    #[cfg_attr(feature = "backend", graphql(description = "Northern Kurdish (kmr)"))]
    Kmr,
    #[cfg_attr(feature = "backend", graphql(description = "Southern Kurdish (sdh)"))]
    Sdh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Central Morocco Tamazight (tzm)")
    )]
    Tzm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Central Morocco Tamazight (Latin) (tzm-Latn)")
    )]
    TzmLatn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Central Morocco Tamazight (Latin, Morocco) (tzm-Latn-MA) ")
    )]
    TzmLatnMa,
    #[cfg_attr(feature = "backend", graphql(description = "Cherokee (chr)"))]
    Chr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Cherokee (United States) (chr-US)")
    )]
    ChrUs,
    #[cfg_attr(feature = "backend", graphql(description = "Chiga (cgg)"))]
    Cgg,
    #[cfg_attr(feature = "backend", graphql(description = "Chiga (Uganda) (cgg-UG)"))]
    CggUg,
    #[cfg_attr(feature = "backend", graphql(description = "Chinese (zh)"))]
    Zh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Simplified) (zh-Hans)")
    )]
    ZhHans,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Simplified, China) (zh-CN)")
    )]
    ZhCn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Simplified, China) (zh-Hans-CN)")
    )]
    ZhHansCn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Simplified, Hong Kong SAR China) (zh-Hans-HK)")
    )]
    ZhHansHk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Simplified, Macau SAR China) (zh-Hans-MO) ")
    )]
    ZhHansMo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Simplified, Singapore) (zh-Hans-SG)")
    )]
    ZhHansSg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Traditional) (zh-Hant)")
    )]
    ZhHant,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Traditional, Hong Kong SAR China) (zh-Hant-HK) ")
    )]
    ZhHantHk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Traditional, Macau SAR China) (zh-Hant-MO) ")
    )]
    ZhHantMo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Chinese (Traditional, Taiwan) (zh-Hant-TW)")
    )]
    ZhHantTw,
    #[cfg_attr(feature = "backend", graphql(description = "Congo Swahili (swc)"))]
    Swc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Congo Swahili (Congo - Kinshasa) (swc-CD)")
    )]
    SwcCd,
    #[cfg_attr(feature = "backend", graphql(description = "Cornish (kw)"))]
    Kw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Cornish (United Kingdom) (kw-GB)")
    )]
    KwGb,
    #[cfg_attr(feature = "backend", graphql(description = "Croatian (hr)"))]
    Hr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Croatian (Croatia) (hr-HR)")
    )]
    HrHr,
    #[cfg_attr(feature = "backend", graphql(description = "Czech (cs)"))]
    Cs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Czech (Czech Republic) (cs-CZ)")
    )]
    CsCz,
    #[cfg_attr(feature = "backend", graphql(description = "Danish (da)"))]
    Da,
    #[cfg_attr(feature = "backend", graphql(description = "Danish (Denmark) (da-DK)"))]
    DaDk,
    #[cfg_attr(feature = "backend", graphql(description = "Duala (dua)"))]
    Dua,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Duala (Cameroon) (dua-CM)")
    )]
    DuaCm,
    #[cfg_attr(feature = "backend", graphql(description = "Dhivehi (Maldives)"))]
    Dv,
    #[cfg_attr(feature = "backend", graphql(description = "Dutch (nl)"))]
    Nl,
    #[cfg_attr(feature = "backend", graphql(description = "Dutch (Aruba) (nl-AW)"))]
    NlAw,
    #[cfg_attr(feature = "backend", graphql(description = "Dutch (Belgium) (nl-BE)"))]
    NlBe,
    #[cfg_attr(feature = "backend", graphql(description = "Dutch (Curaçao) (nl-CW)"))]
    NlCw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Dutch (Netherlands) (nl-NL)")
    )]
    NlNl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Dutch (Sint Maarten) (nl-SX)")
    )]
    NlSx,
    #[cfg_attr(feature = "backend", graphql(description = "Embu (ebu)"))]
    Ebu,
    #[cfg_attr(feature = "backend", graphql(description = "Embu (Kenya) (ebu-KE)"))]
    EbuKe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Anguilla) (en-AI)")
    )]
    EnAi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (American Samoa) (en-AS)")
    )]
    EnAs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Australia) (en-AU)")
    )]
    EnAu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Austria) (en-AT)")
    )]
    EnAt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Barbados) (en-BB)")
    )]
    EnBb,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Belgium) (en-BE)")
    )]
    EnBe,
    #[cfg_attr(feature = "backend", graphql(description = "English (Belize) (en-BZ)"))]
    EnBz,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Bermuda) (en-BM)")
    )]
    EnBm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Botswana) (en-BW)")
    )]
    EnBw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (British Indian Ocean Territory) (en-IO)")
    )]
    EnIo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Burundi) (en-BI)")
    )]
    EnBi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Cameroon) (en-CM)")
    )]
    EnCm,
    #[cfg_attr(feature = "backend", graphql(description = "English (Canada) (en-CA)"))]
    EnCa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Cayman Islands) (en-KY)")
    )]
    EnKy,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Christmas Island) (en-CX)")
    )]
    EnCx,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Cocos [Keeling] Islands) (en-CC)")
    )]
    EnCc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Cook Islands) (en-CK)")
    )]
    EnCk,
    #[cfg_attr(feature = "backend", graphql(description = "English (Cyprus) (en-CY)"))]
    EnCy,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Denmark) (en-DK)")
    )]
    EnDk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Diego Garcia) (en-DG)")
    )]
    EnDg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Dominica) (en-DM)")
    )]
    EnDm,
    #[cfg_attr(feature = "backend", graphql(description = "English (Egypt) (en-EG)"))]
    EnEg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Eritrea) (en-ER)")
    )]
    EnEr,
    #[cfg_attr(feature = "backend", graphql(description = "English (Europe) (en-EU)"))]
    EnEu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Falkland Islands) (en-FK)")
    )]
    EnFk,
    #[cfg_attr(feature = "backend", graphql(description = "English (Fiji) (en-FJ)"))]
    EnFj,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Finland) (en-FI)")
    )]
    EnFi,
    #[cfg_attr(feature = "backend", graphql(description = "English (Gambia) (en-GM)"))]
    EnGm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Germany) (en-DE)")
    )]
    EnDe,
    #[cfg_attr(feature = "backend", graphql(description = "English (Ghana) (en-GH)"))]
    EnGh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Gibraltar) (en-GI)")
    )]
    EnGi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Grenada) (en-GD)")
    )]
    EnGd,
    #[cfg_attr(feature = "backend", graphql(description = "English (Guam) (en-GU)"))]
    EnGu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Guernsey) (en-GG)")
    )]
    EnGg,
    #[cfg_attr(feature = "backend", graphql(description = "English (Guyana) (en-GY)"))]
    EnGy,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Hong Kong SAR China) (en-HK)")
    )]
    EnHk,
    #[cfg_attr(feature = "backend", graphql(description = "English (India) (en-IN)"))]
    EnIn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Ireland) (en-IE)")
    )]
    EnIe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Isle of Man) (en-IM)")
    )]
    EnIm,
    #[cfg_attr(feature = "backend", graphql(description = "English (Israel) (en-IL)"))]
    EnIl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Jamaica) (en-JM)")
    )]
    EnJm,
    #[cfg_attr(feature = "backend", graphql(description = "English (Jersey) (en-JE)"))]
    EnJe,
    #[cfg_attr(feature = "backend", graphql(description = "English (Kenya) (en-KE)"))]
    EnKe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Kiribati) (en-KI)")
    )]
    EnKi,
    #[cfg_attr(feature = "backend", graphql(description = "English (Kuwait) (en-KW)"))]
    EnKw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Lesotho) (en-LS)")
    )]
    EnLs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Macao SAR China) (en-MO)")
    )]
    EnMo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Madagascar) (en-MG)")
    )]
    EnMg,
    #[cfg_attr(feature = "backend", graphql(description = "English (Malawi) (en-MW)"))]
    EnMw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Malaysia) (en-MY)")
    )]
    EnMy,
    #[cfg_attr(feature = "backend", graphql(description = "English (Malta) (en-MT)"))]
    EnMt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Marshall Islands) (en-MH)")
    )]
    EnMh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Mauritius) (en-MU)")
    )]
    EnMu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Micronesia) (en-FM)")
    )]
    EnFm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Montserrat) (en-MS)")
    )]
    EnMs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Namibia) (en-NA)")
    )]
    EnNa,
    #[cfg_attr(feature = "backend", graphql(description = "English (Nauru) (en-NR)"))]
    EnNr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Netherlands) (en-NL)")
    )]
    EnNl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (New Zealand) (en-NZ)")
    )]
    EnNz,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Nigeria) (en-NG)")
    )]
    EnNg,
    #[cfg_attr(feature = "backend", graphql(description = "English (Niue) (en-NU)"))]
    EnNu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Norfolk Island) (en-NF)")
    )]
    EnNf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Northern Mariana Islands) (en-MP)")
    )]
    EnMp,
    #[cfg_attr(feature = "backend", graphql(description = "English (Norway) (en-NO)"))]
    EnNo,
    #[cfg_attr(feature = "backend", graphql(description = "English (Panama) (en-PA)"))]
    EnPa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Pakistan) (en-PK)")
    )]
    EnPk,
    #[cfg_attr(feature = "backend", graphql(description = "English (Palau) (en-PW)"))]
    EnPw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Papua New Guinea) (en-PG)")
    )]
    EnPg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Philippines) (en-PH)")
    )]
    EnPh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Pitcairn Islands) (en-PN)")
    )]
    EnPn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Puerto Rico) (en-PR)")
    )]
    EnPr,
    #[cfg_attr(feature = "backend", graphql(description = "English (Rwanda) (en-RW)"))]
    EnRw,
    #[cfg_attr(feature = "backend", graphql(description = "English (Samoa) (en-WS)"))]
    EnWs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Saudi Arabia) (en-SA)")
    )]
    EnSa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Seychelles) (en-SC)")
    )]
    EnSc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Sierra Leone) (en-SL)")
    )]
    EnSl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Singapore) (en-SG)")
    )]
    EnSg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Sint Maarten) (en-SX)")
    )]
    EnSx,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Slovenia) (en-SI)")
    )]
    EnSi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Solomon Islands) (en-SB)")
    )]
    EnSb,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (South Sudan) (en-SS)")
    )]
    EnSs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (St Helena) (en-SH)")
    )]
    EnSh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (St Kitts & Nevis) (en-KN)")
    )]
    EnKn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (St Lucia) (en-LC)")
    )]
    EnLc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Vincentian Creole English")
    )]
    Svc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Virgin Islands Creole English")
    )]
    Vic,
    #[cfg_attr(feature = "backend", graphql(description = "English (Sudan) (en-SD)"))]
    EnSd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Swaziland) (en-SZ)")
    )]
    EnSz,
    #[cfg_attr(feature = "backend", graphql(description = "English (Sweden) (en-SE)"))]
    EnSe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Switzerland) (en-CH)")
    )]
    EnCh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Tanzania) (en-TZ)")
    )]
    EnTz,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Tokelau) (en-TK)")
    )]
    EnTk,
    #[cfg_attr(feature = "backend", graphql(description = "English (Tonga) (en-TO)"))]
    EnTo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Trinidad and Tobago) (en-TT)")
    )]
    EnTt,
    #[cfg_attr(feature = "backend", graphql(description = "English (Tuvalu) (en-TV)"))]
    EnTv,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (South Africa) (en-ZA)")
    )]
    EnZa,
    #[cfg_attr(feature = "backend", graphql(description = "English (U.A.E.) (en-AE)"))]
    EnAe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (U.S. Minor Outlying Islands) (en-UM)")
    )]
    EnUm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (U.S. Virgin Islands) (en-VI)")
    )]
    EnVi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (U.S., Computer) (en-US-POSIX)")
    )]
    EnUsPosix,
    #[cfg_attr(feature = "backend", graphql(description = "English (Uganda) (en-UG)"))]
    EnUg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (United Kingdom) (en-GB)")
    )]
    EnGb,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (United States) (en-US)")
    )]
    EnUs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Vanuatu) (en-VU)")
    )]
    EnVu,
    #[cfg_attr(feature = "backend", graphql(description = "English (Zambia) (en-ZM)"))]
    EnZm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "English (Zimbabwe) (en-ZW)")
    )]
    EnZw,
    #[cfg_attr(feature = "backend", graphql(description = "Esperanto (eo)"))]
    Eo,
    #[cfg_attr(feature = "backend", graphql(description = "Estonian (et)"))]
    Et,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Estonian (Estonia) (et-EE)")
    )]
    EtEe,
    #[cfg_attr(feature = "backend", graphql(description = "Ewe (ee)"))]
    Ee,
    #[cfg_attr(feature = "backend", graphql(description = "Ewe (Ghana) (ee-GH)"))]
    EeGh,
    #[cfg_attr(feature = "backend", graphql(description = "Ewe (Togo) (ee-TG)"))]
    EeTg,
    #[cfg_attr(feature = "backend", graphql(description = "Ewondo (ewo)"))]
    Ewo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Ewondo (Cameroon) (ewo-CM)")
    )]
    EwoCm,
    #[cfg_attr(feature = "backend", graphql(description = "Faroese (fo)"))]
    Fo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Faroese (Faroe Islands) (fo-FO)")
    )]
    FoFo,
    #[cfg_attr(feature = "backend", graphql(description = "Filipino (fil)"))]
    Fil,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Filipino (Philippines) (fil-PH)")
    )]
    FilPh,
    #[cfg_attr(feature = "backend", graphql(description = "Finnish (fi)"))]
    Fi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Finnish (Finland) (fi-FI)")
    )]
    FiFi,
    #[cfg_attr(feature = "backend", graphql(description = "French (fr)"))]
    Fr,
    #[cfg_attr(feature = "backend", graphql(description = "French (Belgium) (fr-BE)"))]
    FrBe,
    #[cfg_attr(feature = "backend", graphql(description = "French (Benin) (fr-BJ)"))]
    FrBj,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Burkina Faso) (fr-BF)")
    )]
    FrBf,
    #[cfg_attr(feature = "backend", graphql(description = "French (Burundi) (fr-BI)"))]
    FrBi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Cameroon) (fr-CM)")
    )]
    FrCm,
    #[cfg_attr(feature = "backend", graphql(description = "French (Canada) (fr-CA)"))]
    FrCa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Central African Republic) (fr-CF)")
    )]
    FrCf,
    #[cfg_attr(feature = "backend", graphql(description = "French (Chad) (fr-TD)"))]
    FrTd,
    #[cfg_attr(feature = "backend", graphql(description = "French (Comoros) (fr-KM)"))]
    FrKm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Congo - Brazzaville) (fr-CG)")
    )]
    FrCg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Congo - Kinshasa) (fr-CD)")
    )]
    FrCd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Côte d’Ivoire) (fr-CI)")
    )]
    FrCi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Djibouti) (fr-DJ)")
    )]
    FrDj,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Equatorial Guinea) (fr-GQ)")
    )]
    FrGq,
    #[cfg_attr(feature = "backend", graphql(description = "French (France) (fr-FR)"))]
    FrFr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (French Guiana) (fr-GF)")
    )]
    FrGf,
    #[cfg_attr(feature = "backend", graphql(description = "French (Gabon) (fr-GA)"))]
    FrGa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Guadeloupe) (fr-GP)")
    )]
    FrGp,
    #[cfg_attr(feature = "backend", graphql(description = "French (Guinea) (fr-GN)"))]
    FrGn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Luxembourg) (fr-LU)")
    )]
    FrLu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Madagascar) (fr-MG)")
    )]
    FrMg,
    #[cfg_attr(feature = "backend", graphql(description = "French (Mali) (fr-ML)"))]
    FrMl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Martinique) (fr-MQ)")
    )]
    FrMq,
    #[cfg_attr(feature = "backend", graphql(description = "French (Mayotte) (fr-YT)"))]
    FrYt,
    #[cfg_attr(feature = "backend", graphql(description = "French (Monaco) (fr-MC)"))]
    FrMc,
    #[cfg_attr(feature = "backend", graphql(description = "French (Niger) (fr-NE)"))]
    FrNe,
    #[cfg_attr(feature = "backend", graphql(description = "French (Rwanda) (fr-RW)"))]
    FrRw,
    #[cfg_attr(feature = "backend", graphql(description = "French (Réunion) (fr-RE)"))]
    FrRe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Saint Barthélemy) (fr-BL)")
    )]
    FrBl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Saint Martin) (fr-MF)")
    )]
    FrMf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Mauritius) (fr-MU)")
    )]
    FrMu,
    #[cfg_attr(feature = "backend", graphql(description = "French (Senegal) (fr-SN)"))]
    FrSn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "French (Switzerland) (fr-CH)")
    )]
    FrCh,
    #[cfg_attr(feature = "backend", graphql(description = "French (Togo) (fr-TG)"))]
    FrTg,
    #[cfg_attr(feature = "backend", graphql(description = "Fulah (ff)"))]
    Ff,
    #[cfg_attr(feature = "backend", graphql(description = "Fulah (Senegal) (ff-SN)"))]
    FfSn,
    #[cfg_attr(feature = "backend", graphql(description = "Galician (gl)"))]
    Gl,
    #[cfg_attr(feature = "backend", graphql(description = "Galician (Spain) (gl-ES)"))]
    GlEs,
    #[cfg_attr(feature = "backend", graphql(description = "Laotian (Laos) (lao)"))]
    Lao,
    #[cfg_attr(feature = "backend", graphql(description = "Ganda (lg)"))]
    Lg,
    #[cfg_attr(feature = "backend", graphql(description = "Ganda (Uganda) (lg-UG)"))]
    LgUg,
    #[cfg_attr(feature = "backend", graphql(description = "Georgian (ka)"))]
    Ka,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Georgian (Georgia) (ka-GE)")
    )]
    KaGe,
    #[cfg_attr(feature = "backend", graphql(description = "German (de)"))]
    De,
    #[cfg_attr(feature = "backend", graphql(description = "German (Austria) (de-AT)"))]
    DeAt,
    #[cfg_attr(feature = "backend", graphql(description = "German (Belgium) (de-BE)"))]
    DeBe,
    #[cfg_attr(feature = "backend", graphql(description = "German (Germany) (de-DE)"))]
    DeDe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "German (Liechtenstein) (de-LI)")
    )]
    DeLi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "German (Luxembourg) (de-LU)")
    )]
    DeLu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "German (Switzerland) (de-CH)")
    )]
    DeCh,
    #[cfg_attr(feature = "backend", graphql(description = "Greek (el)"))]
    El,
    #[cfg_attr(feature = "backend", graphql(description = "Greek (Cyprus) (el-CY)"))]
    ElCy,
    #[cfg_attr(feature = "backend", graphql(description = "Greek (Greece) (el-GR)"))]
    ElGr,
    #[cfg_attr(feature = "backend", graphql(description = "Gujarati (gu)"))]
    Gu,
    #[cfg_attr(feature = "backend", graphql(description = "Gujarati (India) (gu-IN)"))]
    GuIn,
    #[cfg_attr(feature = "backend", graphql(description = "Gusii (guz)"))]
    Guz,
    #[cfg_attr(feature = "backend", graphql(description = "Gusii (Kenya) (guz-KE)"))]
    GuzKe,
    #[cfg_attr(feature = "backend", graphql(description = "Hausa (ha)"))]
    Ha,
    #[cfg_attr(feature = "backend", graphql(description = "Hausa (Latin) (ha-Latn)"))]
    HaLatn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Hausa (Latin, Ghana) (ha-Latn-GH)")
    )]
    HaLatnGh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Hausa (Latin, Niger) (ha-Latn-NE)")
    )]
    HaLatnNe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Hausa (Latin, Nigeria) (ha-Latn-NG)")
    )]
    HaLatnNg,
    #[cfg_attr(feature = "backend", graphql(description = "Hawaiian (haw)"))]
    Haw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Hawaiian (United States) (haw-US)")
    )]
    HawUs,
    #[cfg_attr(feature = "backend", graphql(description = "Hebrew (he)"))]
    He,
    #[cfg_attr(feature = "backend", graphql(description = "Hebrew (Israel) (he-IL)"))]
    HeIl,
    #[cfg_attr(feature = "backend", graphql(description = "Hindi (hi)"))]
    Hi,
    #[cfg_attr(feature = "backend", graphql(description = "Hindi (India) (hi-IN)"))]
    HiIn,
    #[cfg_attr(feature = "backend", graphql(description = "Hungarian (hu)"))]
    Hu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Hungarian (Hungary) (hu-HU)")
    )]
    HuHu,
    #[cfg_attr(feature = "backend", graphql(description = "Icelandic (is)"))]
    Is,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Icelandic (Iceland) (is-IS)")
    )]
    IsIs,
    #[cfg_attr(feature = "backend", graphql(description = "Igbo (ig)"))]
    Ig,
    #[cfg_attr(feature = "backend", graphql(description = "Igbo (Nigeria) (ig-NG)"))]
    IgNg,
    #[cfg_attr(feature = "backend", graphql(description = "Inari Sami"))]
    Smn,
    #[cfg_attr(feature = "backend", graphql(description = "Inari Sami (Finland)"))]
    SmnFi,
    #[cfg_attr(feature = "backend", graphql(description = "Indonesian (id)"))]
    Id,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Indonesian (Indonesia) (id-ID)")
    )]
    IdId,
    #[cfg_attr(feature = "backend", graphql(description = "Irish (ga)"))]
    Ga,
    #[cfg_attr(feature = "backend", graphql(description = "Irish (Ireland) (ga-IE)"))]
    GaIe,
    #[cfg_attr(feature = "backend", graphql(description = "Italian (it)"))]
    It,
    #[cfg_attr(feature = "backend", graphql(description = "Italian (Italy) (it-IT)"))]
    ItIt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Italian (Switzerland) (it-CH)")
    )]
    ItCh,
    #[cfg_attr(feature = "backend", graphql(description = "Japanese (ja)"))]
    Ja,
    #[cfg_attr(feature = "backend", graphql(description = "Japanese (Japan) (ja-JP)"))]
    JaJp,
    #[cfg_attr(feature = "backend", graphql(description = "Jola-Fonyi (dyo)"))]
    Dyo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Jola-Fonyi (Senegal) (dyo-SN)")
    )]
    DyoSn,
    #[cfg_attr(feature = "backend", graphql(description = "Kabuverdianu (kea)"))]
    Kea,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Kabuverdianu (Cape Verde) (kea-CV)")
    )]
    KeaCv,
    #[cfg_attr(feature = "backend", graphql(description = "Kabyle (kab)"))]
    Kab,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Kabyle (Algeria) (kab-DZ)")
    )]
    KabDz,
    #[cfg_attr(feature = "backend", graphql(description = "Kalaallisut (kl)"))]
    Kl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Kalaallisut (Greenland) (kl-GL)")
    )]
    KlGl,
    #[cfg_attr(feature = "backend", graphql(description = "Kalenjin (kln)"))]
    Kln,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Kalenjin (Kenya) (kln-KE)")
    )]
    KlnKe,
    #[cfg_attr(feature = "backend", graphql(description = "Kamba (kam)"))]
    Kam,
    #[cfg_attr(feature = "backend", graphql(description = "Kamba (Kenya) (kam-KE)"))]
    KamKe,
    #[cfg_attr(feature = "backend", graphql(description = "Kannada (kn)"))]
    Kn,
    #[cfg_attr(feature = "backend", graphql(description = "Kannada (India) (kn-IN)"))]
    KnIn,
    #[cfg_attr(feature = "backend", graphql(description = "Kara-Kalpak (kaa)"))]
    Kaa,
    #[cfg_attr(feature = "backend", graphql(description = "Kazakh (kk)"))]
    Kk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Kazakh (Cyrillic) (kk-Cyrl)")
    )]
    KkCyrl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Kazakh (Cyrillic, Kazakhstan) (kk-Cyrl-KZ)")
    )]
    KkCyrlKz,
    #[cfg_attr(feature = "backend", graphql(description = "Khmer (km)"))]
    Km,
    #[cfg_attr(feature = "backend", graphql(description = "Khmer (Cambodia) (km-KH)"))]
    KmKh,
    #[cfg_attr(feature = "backend", graphql(description = "Kikuyu (ki)"))]
    Ki,
    #[cfg_attr(feature = "backend", graphql(description = "Kikuyu (Kenya) (ki-KE)"))]
    KiKe,
    #[cfg_attr(feature = "backend", graphql(description = "Kinyarwanda (rw)"))]
    Rw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Kinyarwanda (Rwanda) (rw-RW)")
    )]
    RwRw,
    #[cfg_attr(feature = "backend", graphql(description = "Konkani (kok)"))]
    Kok,
    #[cfg_attr(feature = "backend", graphql(description = "Konkani (India) (kok-IN)"))]
    KokIn,
    #[cfg_attr(feature = "backend", graphql(description = "Korean (ko)"))]
    Ko,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Korean (South Korea) (ko-KR)")
    )]
    KoKr,
    #[cfg_attr(feature = "backend", graphql(description = "Koyra Chiini (khq)"))]
    Khq,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Koyra Chiini (Mali) (khq-ML)")
    )]
    KhqMl,
    #[cfg_attr(feature = "backend", graphql(description = "Koyraboro Senni (ses)"))]
    Ses,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Koyraboro Senni (Mali) (ses-ML)")
    )]
    SesMl,
    #[cfg_attr(feature = "backend", graphql(description = "Kwasio (nmg)"))]
    Nmg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Kwasio (Cameroon) (nmg-CM)")
    )]
    NmgCm,
    #[cfg_attr(feature = "backend", graphql(description = "Kyrgyz (ky)"))]
    Ky,
    #[cfg_attr(feature = "backend", graphql(description = "Langi (lag)"))]
    Lag,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Langi (Tanzania) (lag-TZ)")
    )]
    LagTz,
    #[cfg_attr(feature = "backend", graphql(description = "Latvian (lv)"))]
    Lv,
    #[cfg_attr(feature = "backend", graphql(description = "Latvian (Latvia) (lv-LV)"))]
    LvLv,
    #[cfg_attr(feature = "backend", graphql(description = "Liberian English"))]
    Lir,
    #[cfg_attr(feature = "backend", graphql(description = "Lingala (ln)"))]
    Ln,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Lingala (Congo - Brazzaville) (ln-CG)")
    )]
    LnCg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Lingala (Congo - Kinshasa) (ln-CD)")
    )]
    LnCd,
    #[cfg_attr(feature = "backend", graphql(description = "Lithuanian (lt)"))]
    Lt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Lithuanian (Lithuania) (lt-LT)")
    )]
    LtLt,
    #[cfg_attr(feature = "backend", graphql(description = "Luba-Katanga (lu)"))]
    Lu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Luba-Katanga (Congo - Kinshasa) (lu-CD)")
    )]
    LuCd,
    #[cfg_attr(feature = "backend", graphql(description = "Luo (luo)"))]
    Luo,
    #[cfg_attr(feature = "backend", graphql(description = "Luo (Kenya) (luo-KE)"))]
    LuoKe,
    #[cfg_attr(feature = "backend", graphql(description = "Luyia (luy)"))]
    Luy,
    #[cfg_attr(feature = "backend", graphql(description = "Luyia (Kenya) (luy-KE)"))]
    LuyKe,
    #[cfg_attr(feature = "backend", graphql(description = "Macedonian (mk)"))]
    Mk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Macedonian (Macedonia) (mk-MK)")
    )]
    MkMk,
    #[cfg_attr(feature = "backend", graphql(description = "Machame (jmc)"))]
    Jmc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Machame (Tanzania) (jmc-TZ)")
    )]
    JmcTz,
    #[cfg_attr(feature = "backend", graphql(description = "Makhuwa-Meetto (mgh)"))]
    Mgh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Makhuwa-Meetto (Mozambique) (mgh-MZ)")
    )]
    MghMz,
    #[cfg_attr(feature = "backend", graphql(description = "Makonde (kde)"))]
    Kde,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Makonde (Tanzania) (kde-TZ)")
    )]
    KdeTz,
    #[cfg_attr(feature = "backend", graphql(description = "Malagasy (mg)"))]
    Mg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Malagasy (Madagascar) (mg-MG)")
    )]
    MgMg,
    #[cfg_attr(feature = "backend", graphql(description = "Malay (ms)"))]
    Ms,
    #[cfg_attr(feature = "backend", graphql(description = "Malay (Brunei) (ms-BN)"))]
    MsBn,
    #[cfg_attr(feature = "backend", graphql(description = "Malay (Malaysia) (ms-MY)"))]
    MsMy,
    #[cfg_attr(feature = "backend", graphql(description = "Malayalam (ml)"))]
    Ml,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Malayalam (India) (ml-IN)")
    )]
    MlIn,
    #[cfg_attr(feature = "backend", graphql(description = "Maltese (mt)"))]
    Mt,
    #[cfg_attr(feature = "backend", graphql(description = "Maltese (Malta) (mt-MT)"))]
    MtMt,
    #[cfg_attr(feature = "backend", graphql(description = "Manx (gv)"))]
    Gv,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Manx (United Kingdom) (gv-GB)")
    )]
    GvGb,
    #[cfg_attr(feature = "backend", graphql(description = "Marathi (mr)"))]
    Mr,
    #[cfg_attr(feature = "backend", graphql(description = "Marathi (India) (mr-IN)"))]
    MrIn,
    #[cfg_attr(feature = "backend", graphql(description = "Masai (mas)"))]
    Mas,
    #[cfg_attr(feature = "backend", graphql(description = "Masai (Kenya) (mas-KE)"))]
    MasKe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Masai (Tanzania) (mas-TZ)")
    )]
    MasTz,
    #[cfg_attr(feature = "backend", graphql(description = "Meru (mer)"))]
    Mer,
    #[cfg_attr(feature = "backend", graphql(description = "Meru (Kenya) (mer-KE)"))]
    MerKe,
    #[cfg_attr(feature = "backend", graphql(description = "Mongolian (mn)"))]
    Mn,
    #[cfg_attr(feature = "backend", graphql(description = "Morisyen (mfe)"))]
    Mfe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Morisyen (Mauritius) (mfe-MU)")
    )]
    MfeMu,
    #[cfg_attr(feature = "backend", graphql(description = "Mundang (mua)"))]
    Mua,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Mundang (Cameroon) (mua-CM)")
    )]
    MuaCm,
    #[cfg_attr(feature = "backend", graphql(description = "Nama (naq)"))]
    Naq,
    #[cfg_attr(feature = "backend", graphql(description = "Nama (Namibia) (naq-NA)"))]
    NaqNa,
    #[cfg_attr(feature = "backend", graphql(description = "Nepali (ne)"))]
    Ne,
    #[cfg_attr(feature = "backend", graphql(description = "Nepali (India) (ne-IN)"))]
    NeIn,
    #[cfg_attr(feature = "backend", graphql(description = "Nepali (Nepal) (ne-NP)"))]
    NeNp,
    #[cfg_attr(feature = "backend", graphql(description = "Northern Sami"))]
    Se,
    #[cfg_attr(feature = "backend", graphql(description = "Northern Sami (Finland)"))]
    SeFi,
    #[cfg_attr(feature = "backend", graphql(description = "Northern Sami (Norway)"))]
    SeNo,
    #[cfg_attr(feature = "backend", graphql(description = "Northern Sami (Sweden)"))]
    SeSe,
    #[cfg_attr(feature = "backend", graphql(description = "North Ndebele (nd)"))]
    Nd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "North Ndebele (Zimbabwe) (nd-ZW)")
    )]
    NdZw,
    #[cfg_attr(feature = "backend", graphql(description = "Norwegian Bokmål (nb)"))]
    Nb,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Norwegian Bokmål (Norway) (nb-NO)")
    )]
    NbNo,
    #[cfg_attr(feature = "backend", graphql(description = "Norwegian Nynorsk (nn)"))]
    Nn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Norwegian Nynorsk (Norway) (nn-NO)")
    )]
    NnNo,
    #[cfg_attr(feature = "backend", graphql(description = "Nuer (nus)"))]
    Nus,
    #[cfg_attr(feature = "backend", graphql(description = "Nuer (Sudan) (nus-SD)"))]
    NusSd,
    #[cfg_attr(feature = "backend", graphql(description = "Nyankole (nyn)"))]
    Nyn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Nyankole (Uganda) (nyn-UG)")
    )]
    NynUg,
    #[cfg_attr(feature = "backend", graphql(description = "Oriya (or)"))]
    Or,
    #[cfg_attr(feature = "backend", graphql(description = "Oriya (India) (or-IN)"))]
    OrIn,
    #[cfg_attr(feature = "backend", graphql(description = "Oromo (om)"))]
    Om,
    #[cfg_attr(feature = "backend", graphql(description = "Oromo (Ethiopia) (om-ET)"))]
    OmEt,
    #[cfg_attr(feature = "backend", graphql(description = "Oromo (Kenya) (om-KE)"))]
    OmKe,
    #[cfg_attr(feature = "backend", graphql(description = "Pashto (ps)"))]
    Ps,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Pashto (Afghanistan) (ps-AF)")
    )]
    PsAf,
    #[cfg_attr(feature = "backend", graphql(description = "Persian (fa)"))]
    Fa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Persian (Afghanistan) (fa-AF)")
    )]
    FaAf,
    #[cfg_attr(feature = "backend", graphql(description = "Persian (Iran) (fa-IR)"))]
    FaIr,
    #[cfg_attr(feature = "backend", graphql(description = "Polish (pl)"))]
    Pl,
    #[cfg_attr(feature = "backend", graphql(description = "Polish (Poland) (pl-PL)"))]
    PlPl,
    #[cfg_attr(feature = "backend", graphql(description = "Portuguese (pt)"))]
    Pt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Portuguese (Angola) (pt-AO)")
    )]
    PtAo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Portuguese (Brazil) (pt-BR)")
    )]
    PtBr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Portuguese (Guinea-Bissau) (pt-GW)")
    )]
    PtGw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Portuguese (Mozambique) (pt-MZ)")
    )]
    PtMz,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Portuguese (Portugal) (pt-PT)")
    )]
    PtPt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Portuguese (São Tomé and Príncipe) (pt-ST)")
    )]
    PtSt,
    #[cfg_attr(feature = "backend", graphql(description = "Punjabi (pa)"))]
    Pa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Punjabi (Arabic) (pa-Arab)")
    )]
    PaArab,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Punjabi (Arabic, Pakistan) (pa-Arab-PK)")
    )]
    PaArabPk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Punjabi (Gurmukhi) (pa-Guru)")
    )]
    PaGuru,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Punjabi (Gurmukhi, India) (pa-Guru-IN)")
    )]
    PaGuruIn,
    #[cfg_attr(feature = "backend", graphql(description = "Romanian (ro)"))]
    Ro,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Romanian (Moldova) (ro-MD)")
    )]
    RoMd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Romanian (Romania) (ro-RO)")
    )]
    RoRo,
    #[cfg_attr(feature = "backend", graphql(description = "Romansh (rm)"))]
    Rm,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Romansh (Switzerland) (rm-CH)")
    )]
    RmCh,
    #[cfg_attr(feature = "backend", graphql(description = "Rombo (rof)"))]
    Rof,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Rombo (Tanzania) (rof-TZ)")
    )]
    RofTz,
    #[cfg_attr(feature = "backend", graphql(description = "Rundi (rn)"))]
    Rn,
    #[cfg_attr(feature = "backend", graphql(description = "Rundi (Burundi) (rn-BI)"))]
    RnBi,
    #[cfg_attr(feature = "backend", graphql(description = "Russian (ru)"))]
    Ru,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Russian (Moldova) (ru-MD)")
    )]
    RuMd,
    #[cfg_attr(feature = "backend", graphql(description = "Russian (Russia) (ru-RU)"))]
    RuRu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Russian (Ukraine) (ru-UA)")
    )]
    RuUa,
    #[cfg_attr(feature = "backend", graphql(description = "Rwa (rwk)"))]
    Rwk,
    #[cfg_attr(feature = "backend", graphql(description = "Rwa (Tanzania) (rwk-TZ)"))]
    RwkTz,
    #[cfg_attr(feature = "backend", graphql(description = "Samburu (saq)"))]
    Saq,
    #[cfg_attr(feature = "backend", graphql(description = "Samburu (Kenya) (saq-KE)"))]
    SaqKe,
    #[cfg_attr(feature = "backend", graphql(description = "Sango (sg)"))]
    Sg,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Sango (Central African Republic) (sg-CF)")
    )]
    SgCf,
    #[cfg_attr(feature = "backend", graphql(description = "Sangu (sbp)"))]
    Sbp,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Sangu (Tanzania) (sbp-TZ)")
    )]
    SbpTz,
    #[cfg_attr(feature = "backend", graphql(description = "Sanskrit (sa)"))]
    Sa,
    #[cfg_attr(feature = "backend", graphql(description = "Scottish Gaelic (gd)"))]
    Gd,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Scottish Gaelic (United Kingdom)")
    )]
    GdGb,
    #[cfg_attr(feature = "backend", graphql(description = "Sena (seh)"))]
    Seh,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Sena (Mozambique) (seh-MZ)")
    )]
    SehMz,
    #[cfg_attr(feature = "backend", graphql(description = "Serbian (sr)"))]
    Sr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Serbian (Cyrillic) (sr-Cyrl)")
    )]
    SrCyrl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Serbian (Cyrillic, Bosnia and Herzegovina)(sr-Cyrl-BA) ")
    )]
    SrCyrlBa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Serbian (Cyrillic, Montenegro) (sr-Cyrl-ME)")
    )]
    SrCyrlMe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Serbian (Cyrillic, Serbia) (sr-Cyrl-RS)")
    )]
    SrCyrlRs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Serbian (Latin) (sr-Latn)")
    )]
    SrLatn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Serbian (Latin, Bosnia and Herzegovina) (sr-Latn-BA) ")
    )]
    SrLatnBa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Serbian (Latin, Montenegro) (sr-Latn-ME)")
    )]
    SrLatnMe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Serbian (Latin, Serbia) (sr-Latn-RS)")
    )]
    SrLatnRs,
    #[cfg_attr(feature = "backend", graphql(description = "Shambala (ksb)"))]
    Ksb,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Shambala (Tanzania) (ksb-TZ)")
    )]
    KsbTz,
    #[cfg_attr(feature = "backend", graphql(description = "Shona (sn)"))]
    Sn,
    #[cfg_attr(feature = "backend", graphql(description = "Shona (Zimbabwe) (sn-ZW)"))]
    SnZw,
    #[cfg_attr(feature = "backend", graphql(description = "Sichuan Yi (ii)"))]
    Ii,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Sichuan Yi (China) (ii-CN)")
    )]
    IiCn,
    #[cfg_attr(feature = "backend", graphql(description = "Sinhala (si)"))]
    Si,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Sinhala (Sri Lanka) (si-LK)")
    )]
    SiLk,
    #[cfg_attr(feature = "backend", graphql(description = "Slovak (sk)"))]
    Sk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Slovak (Slovakia) (sk-SK)")
    )]
    SkSk,
    #[cfg_attr(feature = "backend", graphql(description = "Slovenian (sl)"))]
    Sl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Slovenian (Slovenia) (sl-SI)")
    )]
    SlSi,
    #[cfg_attr(feature = "backend", graphql(description = "Soga (xog)"))]
    Xog,
    #[cfg_attr(feature = "backend", graphql(description = "Soga (Uganda) (xog-UG)"))]
    XogUg,
    #[cfg_attr(feature = "backend", graphql(description = "Somali (so)"))]
    So,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Somali (Djibouti) (so-DJ)")
    )]
    SoDj,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Somali (Ethiopia) (so-ET)")
    )]
    SoEt,
    #[cfg_attr(feature = "backend", graphql(description = "Somali (Kenya) (so-KE)"))]
    SoKe,
    #[cfg_attr(feature = "backend", graphql(description = "Somali (Somalia) (so-SO)"))]
    SoSo,
    #[cfg_attr(feature = "backend", graphql(description = "Spanish (es)"))]
    Es,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Argentina) (es-AR)")
    )]
    EsAr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Bolivia) (es-BO)")
    )]
    EsBo,
    #[cfg_attr(feature = "backend", graphql(description = "Spanish (Chile) (es-CL)"))]
    EsCl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Colombia) (es-CO)")
    )]
    EsCo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Costa Rica) (es-CR)")
    )]
    EsCr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Dominican Republic) (es-DO)")
    )]
    EsDo,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Ecuador) (es-EC)")
    )]
    EsEc,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (El Salvador) (es-SV)")
    )]
    EsSv,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Equatorial Guinea) (es-GQ)")
    )]
    EsGq,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Guatemala) (es-GT)")
    )]
    EsGt,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Honduras) (es-HN)")
    )]
    EsHn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Latin America) (es-419)")
    )]
    Es419,
    #[cfg_attr(feature = "backend", graphql(description = "Spanish (Mexico) (es-MX)"))]
    EsMx,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Nicaragua) (es-NI)")
    )]
    EsNi,
    #[cfg_attr(feature = "backend", graphql(description = "Spanish (Panama) (es-PA)"))]
    EsPa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Paraguay) (es-PY)")
    )]
    EsPy,
    #[cfg_attr(feature = "backend", graphql(description = "Spanish (Peru) (es-PE)"))]
    EsPe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Puerto Rico) (es-PR)")
    )]
    EsPr,
    #[cfg_attr(feature = "backend", graphql(description = "Spanish (Spain) (es-ES)"))]
    EsEs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (United States) (es-US)")
    )]
    EsUs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Uruguay) (es-UY)")
    )]
    EsUy,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Spanish (Venezuela) (es-VE)")
    )]
    EsVe,
    #[cfg_attr(feature = "backend", graphql(description = "Swahili (sw)"))]
    Sw,
    #[cfg_attr(feature = "backend", graphql(description = "Swahili (Kenya) (sw-KE)"))]
    SwKe,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Swahili (Tanzania) (sw-TZ)")
    )]
    SwTz,
    #[cfg_attr(feature = "backend", graphql(description = "Swedish (sv)"))]
    Sv,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Swedish (Finland) (sv-FI)")
    )]
    SvFi,
    #[cfg_attr(feature = "backend", graphql(description = "Swedish (Sweden) (sv-SE)"))]
    SvSe,
    #[cfg_attr(feature = "backend", graphql(description = "Swiss German (gsw)"))]
    Gsw,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Swiss German (Switzerland) (gsw-CH)")
    )]
    GswCh,
    #[cfg_attr(feature = "backend", graphql(description = "Tachelhit (shi)"))]
    Shi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Tachelhit (Latin) (shi-Latn)")
    )]
    ShiLatn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Tachelhit (Latin, Morocco) (shi-Latn-MA)")
    )]
    ShiLatnMa,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Tachelhit (Tifinagh) (shi-Tfng)")
    )]
    ShiTfng,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Tachelhit (Tifinagh, Morocco) (shi-Tfng-MA)")
    )]
    ShiTfngMa,
    #[cfg_attr(feature = "backend", graphql(description = "Taita (dav)"))]
    Dav,
    #[cfg_attr(feature = "backend", graphql(description = "Taita (Kenya) (dav-KE)"))]
    DavKe,
    #[cfg_attr(feature = "backend", graphql(description = "Tajik (tg)"))]
    Tg,
    #[cfg_attr(feature = "backend", graphql(description = "Tamil (ta)"))]
    Ta,
    #[cfg_attr(feature = "backend", graphql(description = "Tamil (India) (ta-IN)"))]
    TaIn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Tamil (Sri Lanka) (ta-LK)")
    )]
    TaLk,
    #[cfg_attr(feature = "backend", graphql(description = "Tasawaq (twq)"))]
    Twq,
    #[cfg_attr(feature = "backend", graphql(description = "Tasawaq (Niger) (twq-NE)"))]
    TwqNe,
    #[cfg_attr(feature = "backend", graphql(description = "Te Reo Māori (mi)"))]
    Mi,
    #[cfg_attr(feature = "backend", graphql(description = "Telugu (te)"))]
    Te,
    #[cfg_attr(feature = "backend", graphql(description = "Telugu (India) (te-IN)"))]
    TeIn,
    #[cfg_attr(feature = "backend", graphql(description = "Teso (teo)"))]
    Teo,
    #[cfg_attr(feature = "backend", graphql(description = "Teso (Kenya) (teo-KE)"))]
    TeoKe,
    #[cfg_attr(feature = "backend", graphql(description = "Teso (Uganda) (teo-UG)"))]
    TeoUg,
    #[cfg_attr(feature = "backend", graphql(description = "Thai (th)"))]
    Th,
    #[cfg_attr(feature = "backend", graphql(description = "Thai (Thailand) (th-TH)"))]
    ThTh,
    #[cfg_attr(feature = "backend", graphql(description = "Tibetan (bo)"))]
    Bo,
    #[cfg_attr(feature = "backend", graphql(description = "Tibetan (China) (bo-CN)"))]
    BoCn,
    #[cfg_attr(feature = "backend", graphql(description = "Tibetan (India) (bo-IN)"))]
    BoIn,
    #[cfg_attr(feature = "backend", graphql(description = "Tigrinya (ti)"))]
    Ti,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Tigrinya (Eritrea) (ti-ER)")
    )]
    TiEr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Tigrinya (Ethiopia) (ti-ET)")
    )]
    TiEt,
    #[cfg_attr(feature = "backend", graphql(description = "Tongan (to)"))]
    To,
    #[cfg_attr(feature = "backend", graphql(description = "Tongan (Tonga) (to-TO)"))]
    ToTo,
    #[cfg_attr(feature = "backend", graphql(description = "Turkish (tr)"))]
    Tr,
    #[cfg_attr(feature = "backend", graphql(description = "Turkmen (tk)"))]
    Tk,
    #[cfg_attr(feature = "backend", graphql(description = "Turkish (Turkey) (tr-TR)"))]
    TrTr,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Turks And Caicos Creole English")
    )]
    Tch,
    #[cfg_attr(feature = "backend", graphql(description = "Ukrainian (uk)"))]
    Uk,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Ukrainian (Ukraine) (uk-UA)")
    )]
    UkUa,
    #[cfg_attr(feature = "backend", graphql(description = "Urdu (ur)"))]
    Ur,
    #[cfg_attr(feature = "backend", graphql(description = "Urdu (India) (ur-IN)"))]
    UrIn,
    #[cfg_attr(feature = "backend", graphql(description = "Urdu (Pakistan) (ur-PK)"))]
    UrPk,
    #[cfg_attr(feature = "backend", graphql(description = "Uyghur"))]
    Ug,
    #[cfg_attr(feature = "backend", graphql(description = "Uyghur (China)"))]
    UgCn,
    #[cfg_attr(feature = "backend", graphql(description = "Uzbek (uz)"))]
    Uz,
    #[cfg_attr(feature = "backend", graphql(description = "Uzbek (Arabic) (uz-Arab)"))]
    UzArab,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Uzbek (Arabic, Afghanistan) (uz-Arab-AF)")
    )]
    UzArabAf,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Uzbek (Cyrillic) (uz-Cyrl)")
    )]
    UzCyrl,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Uzbek (Cyrillic, Uzbekistan) (uz-Cyrl-UZ)")
    )]
    UzCyrlUz,
    #[cfg_attr(feature = "backend", graphql(description = "Uzbek (Latin) (uz-Latn)"))]
    UzLatn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Uzbek (Latin, Uzbekistan) (uz-Latn-UZ)")
    )]
    UzLatnUz,
    #[cfg_attr(feature = "backend", graphql(description = "Vai (vai)"))]
    Vai,
    #[cfg_attr(feature = "backend", graphql(description = "Vai (Latin) (vai-Latn)"))]
    VaiLatn,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Vai (Latin, Liberia) (vai-Latn-LR)")
    )]
    VaiLatnLr,
    #[cfg_attr(feature = "backend", graphql(description = "Vai (Vai) (vai-Vaii)"))]
    VaiVaii,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Vai (Vai, Liberia) (vai-Vaii-LR)")
    )]
    VaiVaiiLr,
    #[cfg_attr(feature = "backend", graphql(description = "Valencian (val)"))]
    Val,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Valencian (Spain) (val-ES)")
    )]
    ValEs,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Valencian (Spain Catalan) (ca-ES-valencia)")
    )]
    CaEsValencia,
    #[cfg_attr(feature = "backend", graphql(description = "Vietnamese (vi)"))]
    Vi,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Vietnamese (Vietnam) (vi-VN)")
    )]
    ViVn,
    #[cfg_attr(feature = "backend", graphql(description = "Vunjo (vun)"))]
    Vun,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Vunjo (Tanzania) (vun-TZ)")
    )]
    VunTz,
    #[cfg_attr(feature = "backend", graphql(description = "Welsh (cy)"))]
    Cy,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Welsh (United Kingdom) (cy-GB)")
    )]
    CyGb,
    #[cfg_attr(feature = "backend", graphql(description = "Wolof (wo)"))]
    Wo,
    #[cfg_attr(feature = "backend", graphql(description = "Xhosa (xh)"))]
    Xh,
    #[cfg_attr(feature = "backend", graphql(description = "Yangben (yav)"))]
    Yav,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Yangben (Cameroon) (yav-CM)")
    )]
    YavCm,
    #[cfg_attr(feature = "backend", graphql(description = "Yoruba (yo)"))]
    Yo,
    #[cfg_attr(feature = "backend", graphql(description = "Yoruba (Nigeria) (yo-NG)"))]
    YoNg,
    #[cfg_attr(feature = "backend", graphql(description = "Zarma (dje)"))]
    Dje,
    #[cfg_attr(feature = "backend", graphql(description = "Zarma (Niger) (dje-NE)"))]
    DjeNe,
    #[cfg_attr(feature = "backend", graphql(description = "Zulu (zu)"))]
    Zu,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Zulu (South Africa) (zu-ZA)")
    )]
    ZuZa,
}
