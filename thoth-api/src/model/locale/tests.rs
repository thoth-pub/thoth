use super::*;

mod conversions {
    use super::*;
    #[cfg(feature = "backend")]
    use crate::model::tests::db::setup_test_db;
    #[cfg(feature = "backend")]
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};
    use strum::IntoEnumIterator;

    #[test]
    fn locale_to_language_code_maps_basic_english() {
        let lang: LanguageCode = LocaleCode::En.into();
        assert_eq!(lang, LanguageCode::Eng);
        assert_eq!(lang.to_string().to_lowercase(), "eng");
    }

    #[test]
    fn locale_to_language_code_maps_regional_variants() {
        // English variants should all map to Eng (eng)
        let lang: LanguageCode = LocaleCode::EnUs.into();
        assert_eq!(lang, LanguageCode::Eng);
        let lang: LanguageCode = LocaleCode::EnGb.into();
        assert_eq!(lang, LanguageCode::Eng);
        let lang: LanguageCode = LocaleCode::EnCa.into();
        assert_eq!(lang, LanguageCode::Eng);
        let lang: LanguageCode = LocaleCode::EnAu.into();
        assert_eq!(lang, LanguageCode::Eng);

        // French variants should all map to Fre (fre) - ISO 639-2/B
        let lang: LanguageCode = LocaleCode::Fr.into();
        assert_eq!(lang, LanguageCode::Fre);
        let lang: LanguageCode = LocaleCode::FrFr.into();
        assert_eq!(lang, LanguageCode::Fre);
        let lang: LanguageCode = LocaleCode::FrCa.into();
        assert_eq!(lang, LanguageCode::Fre);
        let lang: LanguageCode = LocaleCode::FrBe.into();
        assert_eq!(lang, LanguageCode::Fre);

        // Spanish variants should all map to Spa (spa)
        let lang: LanguageCode = LocaleCode::Es.into();
        assert_eq!(lang, LanguageCode::Spa);
        let lang: LanguageCode = LocaleCode::EsEs.into();
        assert_eq!(lang, LanguageCode::Spa);
        let lang: LanguageCode = LocaleCode::EsMx.into();
        assert_eq!(lang, LanguageCode::Spa);
        let lang: LanguageCode = LocaleCode::EsAr.into();
        assert_eq!(lang, LanguageCode::Spa);
    }

    #[test]
    fn locale_to_language_code_maps_major_languages() {
        // Test a variety of major world languages (ISO 639-2/B codes)
        let lang: LanguageCode = LocaleCode::De.into();
        assert_eq!(lang, LanguageCode::Ger); // German
        let lang: LanguageCode = LocaleCode::It.into();
        assert_eq!(lang, LanguageCode::Ita); // Italian
        let lang: LanguageCode = LocaleCode::Pt.into();
        assert_eq!(lang, LanguageCode::Por); // Portuguese
        let lang: LanguageCode = LocaleCode::Ru.into();
        assert_eq!(lang, LanguageCode::Rus); // Russian
        let lang: LanguageCode = LocaleCode::Zh.into();
        assert_eq!(lang, LanguageCode::Chi); // Chinese
        let lang: LanguageCode = LocaleCode::Ja.into();
        assert_eq!(lang, LanguageCode::Jpn); // Japanese
        let lang: LanguageCode = LocaleCode::Ko.into();
        assert_eq!(lang, LanguageCode::Kor); // Korean
        let lang: LanguageCode = LocaleCode::Ar.into();
        assert_eq!(lang, LanguageCode::Ara); // Arabic
        let lang: LanguageCode = LocaleCode::Hi.into();
        assert_eq!(lang, LanguageCode::Hin); // Hindi
        let lang: LanguageCode = LocaleCode::Nl.into();
        assert_eq!(lang, LanguageCode::Dut); // Dutch
        let lang: LanguageCode = LocaleCode::Sv.into();
        assert_eq!(lang, LanguageCode::Swe); // Swedish
        let lang: LanguageCode = LocaleCode::Pl.into();
        assert_eq!(lang, LanguageCode::Pol); // Polish
    }

    #[test]
    fn locale_to_language_code_maps_less_common_languages() {
        // Test some less common languages (ISO 639-2/B codes)
        let lang: LanguageCode = LocaleCode::Cy.into();
        assert_eq!(lang, LanguageCode::Wel); // Welsh
        let lang: LanguageCode = LocaleCode::Ga.into();
        assert_eq!(lang, LanguageCode::Gle); // Irish
        let lang: LanguageCode = LocaleCode::Eu.into();
        assert_eq!(lang, LanguageCode::Baq); // Basque
        let lang: LanguageCode = LocaleCode::Is.into();
        assert_eq!(lang, LanguageCode::Ice); // Icelandic
        let lang: LanguageCode = LocaleCode::Ka.into();
        assert_eq!(lang, LanguageCode::Geo); // Georgian
        let lang: LanguageCode = LocaleCode::Hy.into();
        assert_eq!(lang, LanguageCode::Arm); // Armenian
        let lang: LanguageCode = LocaleCode::Bo.into();
        assert_eq!(lang, LanguageCode::Tib); // Tibetan
        let lang: LanguageCode = LocaleCode::Si.into();
        assert_eq!(lang, LanguageCode::Sin); // Sinhala
    }

    #[test]
    fn locale_to_language_code_maps_all_variants() {
        for locale in LocaleCode::iter() {
            let lang: LanguageCode = locale.into();
            let code = lang.to_string();
            assert_eq!(code.len(), 3);
            assert!(code.chars().all(|c| c.is_ascii_uppercase()));
        }
    }

    #[cfg(feature = "backend")]
    #[test]
    fn localecode_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(LocaleCode::En);
    }

    #[cfg(feature = "backend")]
    #[test]
    fn localecode_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<LocaleCode, crate::schema::sql_types::LocaleCode>(
            pool.as_ref(),
            "'en'::locale_code",
            LocaleCode::En,
        );
    }
}
