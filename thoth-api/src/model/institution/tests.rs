use super::*;
use crate::model::{CountryCode, Crud, Doi, Ror};
use uuid::Uuid;

fn make_institution(pool: &crate::db::PgPool, name: String) -> Institution {
    let new_institution = NewInstitution {
        institution_name: name,
        institution_doi: None,
        ror: None,
        country_code: Some(CountryCode::Gbr),
    };

    Institution::create(pool, &new_institution).expect("Failed to create institution")
}

mod defaults {
    use super::*;

    #[test]
    fn institutionfield_default_is_institution_name() {
        let fundfield: InstitutionField = Default::default();
        assert_eq!(fundfield, InstitutionField::InstitutionName);
    }
}

mod display_and_parse {
    use super::*;

    #[test]
    fn institutionfield_display_formats_expected_strings() {
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
    fn institutionfield_fromstr_parses_expected_values() {
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
    fn countrycode_display_formats_expected_strings() {
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
    fn countrycode_fromstr_parses_expected_values() {
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

    #[test]
    fn institution_display_formats_with_optional_ids() {
        let with_ror = Institution {
            institution_name: "Test Institution".to_string(),
            ror: Some(Ror("https://ror.org/0abcdef12".to_string())),
            ..Default::default()
        };
        assert_eq!(format!("{with_ror}"), "Test Institution - 0abcdef12");

        let with_doi = Institution {
            institution_name: "Test Institution".to_string(),
            institution_doi: Some(Doi("https://doi.org/10.1234/abcd".to_string())),
            ..Default::default()
        };
        assert_eq!(format!("{with_doi}"), "Test Institution - 10.1234/abcd");

        let no_ids = Institution {
            institution_name: "Test Institution".to_string(),
            ..Default::default()
        };
        assert_eq!(format!("{no_ids}"), "Test Institution");
    }
}

#[cfg(feature = "backend")]
mod conversions {
    use super::*;
    use crate::model::tests::db::setup_test_db;
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};

    #[test]
    fn countrycode_graphql_roundtrip() {
        assert_graphql_enum_roundtrip(CountryCode::Gbr);
    }

    #[test]
    fn countrycode_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();

        assert_db_enum_roundtrip::<CountryCode, crate::schema::sql_types::CountryCode>(
            pool.as_ref(),
            "'gbr'::country_code",
            CountryCode::Gbr,
        );
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let institution: Institution = Default::default();
        assert_eq!(institution.pk(), institution.institution_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let institution: Institution = Default::default();
        let user_id = "123456".to_string();
        let new_institution_history = institution.new_history_entry(&user_id);
        assert_eq!(
            new_institution_history.institution_id,
            institution.institution_id
        );
        assert_eq!(new_institution_history.user_id, user_id);
        assert_eq!(
            new_institution_history.data,
            serde_json::Value::String(serde_json::to_string(&institution).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::funding::{Funding, NewFunding};
    use crate::model::institution::policy::InstitutionPolicy;
    use crate::model::tests::db::{
        create_imprint, create_publisher, create_work, setup_test_db, test_context,
        test_context_anonymous, test_context_with_user, test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_requires_authentication_for_create_update() {
        let (_guard, pool) = setup_test_db();

        let ctx = test_context_anonymous(pool.clone());

        let new_institution = NewInstitution {
            institution_name: "Institution Policy".to_string(),
            institution_doi: None,
            ror: None,
            country_code: Some(CountryCode::Gbr),
        };

        let institution = Institution::create(pool.as_ref(), &new_institution)
            .expect("Failed to create institution");
        let patch = PatchInstitution {
            institution_id: institution.institution_id,
            institution_name: "Updated Institution".to_string(),
            institution_doi: institution.institution_doi.clone(),
            ror: institution.ror.clone(),
            country_code: institution.country_code,
            ror_status: None,
            ror_successor: juniper::Nullable::ImplicitNull,
        };

        assert!(InstitutionPolicy::can_create(&ctx, &new_institution, ()).is_err());
        assert!(InstitutionPolicy::can_update(&ctx, &institution, &patch, ()).is_err());
    }

    #[test]
    fn crud_policy_allows_authenticated_user_for_create_update() {
        let (_guard, pool) = setup_test_db();

        let ctx = test_context(pool.clone(), "institution-user");

        let new_institution = NewInstitution {
            institution_name: "Institution Policy".to_string(),
            institution_doi: None,
            ror: None,
            country_code: Some(CountryCode::Gbr),
        };

        let institution = Institution::create(pool.as_ref(), &new_institution)
            .expect("Failed to create institution");
        let patch = PatchInstitution {
            institution_id: institution.institution_id,
            institution_name: "Updated Institution".to_string(),
            institution_doi: institution.institution_doi.clone(),
            ror: institution.ror.clone(),
            country_code: institution.country_code,
            ror_status: None,
            ror_successor: juniper::Nullable::ImplicitNull,
        };

        assert!(InstitutionPolicy::can_create(&ctx, &new_institution, ()).is_ok());
        assert!(InstitutionPolicy::can_update(&ctx, &institution, &patch, ()).is_ok());
    }

    #[test]
    fn crud_policy_delete_requires_publisher_membership() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let institution =
            make_institution(pool.as_ref(), format!("Institution {}", Uuid::new_v4()));

        let new_funding = NewFunding {
            work_id: work.work_id,
            institution_id: institution.institution_id,
            program: None,
            project_name: None,
            project_shortname: None,
            grant_number: None,
        };
        Funding::create(pool.as_ref(), &new_funding).expect("Failed to create funding");

        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("institution-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);
        assert!(InstitutionPolicy::can_delete(&ctx, &institution).is_ok());

        let other_user = test_user_with_role("institution-user", Role::PublisherUser, "org-other");
        let other_ctx = test_context_with_user(pool.clone(), other_user);
        assert!(InstitutionPolicy::can_delete(&other_ctx, &institution).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::model::tests::db::{setup_test_db, test_context};
    use crate::model::Crud;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let new_institution = NewInstitution {
            institution_name: format!("Institution {}", Uuid::new_v4()),
            institution_doi: None,
            ror: None,
            country_code: Some(CountryCode::Gbr),
        };

        let institution = Institution::create(pool.as_ref(), &new_institution)
            .expect("Failed to create institution");
        let fetched = Institution::from_id(pool.as_ref(), &institution.institution_id)
            .expect("Failed to fetch");
        assert_eq!(institution.institution_id, fetched.institution_id);

        let patch = PatchInstitution {
            institution_id: institution.institution_id,
            institution_name: "Updated Institution".to_string(),
            institution_doi: institution.institution_doi.clone(),
            ror: institution.ror.clone(),
            country_code: institution.country_code,
            ror_status: None,
            ror_successor: juniper::Nullable::ImplicitNull,
        };

        let ctx = test_context(pool.clone(), "test-user");
        let updated = institution.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.institution_name, patch.institution_name);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Institution::from_id(pool.as_ref(), &deleted.institution_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        make_institution(pool.as_ref(), format!("Institution {}", Uuid::new_v4()));
        make_institution(pool.as_ref(), format!("Institution {}", Uuid::new_v4()));

        let order = InstitutionOrderBy {
            field: InstitutionField::InstitutionId,
            direction: Direction::Asc,
        };

        let first = Institution::all(
            pool.as_ref(),
            1,
            0,
            None,
            order.clone(),
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch institutions");
        let second = Institution::all(
            pool.as_ref(),
            1,
            1,
            None,
            order,
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch institutions");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].institution_id, second[0].institution_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        make_institution(pool.as_ref(), format!("Institution {}", Uuid::new_v4()));
        make_institution(pool.as_ref(), format!("Institution {}", Uuid::new_v4()));

        let count = Institution::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count institutions");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_matches_institution_name() {
        let (_guard, pool) = setup_test_db();

        let marker = format!("Filter {}", Uuid::new_v4());
        let matches = make_institution(pool.as_ref(), format!("Institution {marker}"));
        make_institution(pool.as_ref(), "Other Institution".to_string());

        let filtered = Institution::all(
            pool.as_ref(),
            10,
            0,
            Some(marker),
            InstitutionOrderBy {
                field: InstitutionField::InstitutionId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter institutions");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].institution_id, matches.institution_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let first = make_institution(pool.as_ref(), format!("Institution {}", Uuid::new_v4()));
        let second = make_institution(pool.as_ref(), format!("Institution {}", Uuid::new_v4()));
        let mut ids = [first.institution_id, second.institution_id];
        ids.sort();

        let asc = Institution::all(
            pool.as_ref(),
            2,
            0,
            None,
            InstitutionOrderBy {
                field: InstitutionField::InstitutionId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to order institutions (asc)");

        let desc = Institution::all(
            pool.as_ref(),
            2,
            0,
            None,
            InstitutionOrderBy {
                field: InstitutionField::InstitutionId,
                direction: Direction::Desc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to order institutions (desc)");

        assert_eq!(asc[0].institution_id, ids[0]);
        assert_eq!(desc[0].institution_id, ids[1]);
    }

    #[test]
    fn crud_count_with_filter_matches_ror() {
        let (_guard, pool) = setup_test_db();

        let marker = "0abcd1234";
        Institution::create(
            pool.as_ref(),
            &NewInstitution {
                institution_name: "Ror Match".to_string(),
                institution_doi: None,
                ror: Some(Ror(format!("https://ror.org/{marker}"))),
                country_code: Some(CountryCode::Gbr),
            },
        )
        .expect("Failed to create institution");
        Institution::create(
            pool.as_ref(),
            &NewInstitution {
                institution_name: "Other".to_string(),
                institution_doi: None,
                ror: None,
                country_code: Some(CountryCode::Gbr),
            },
        )
        .expect("Failed to create institution");

        let count = Institution::count(
            pool.as_ref(),
            Some(marker.to_string()),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to count filtered institutions");

        assert_eq!(count, 1);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        Institution::create(
            pool.as_ref(),
            &NewInstitution {
                institution_name: "Institution A".to_string(),
                institution_doi: Some(Doi("https://doi.org/10.1234/A".to_string())),
                ror: Some(Ror("https://ror.org/0aaaa0000".to_string())),
                country_code: Some(CountryCode::Gbr),
            },
        )
        .expect("Failed to create institution");
        Institution::create(
            pool.as_ref(),
            &NewInstitution {
                institution_name: "Institution B".to_string(),
                institution_doi: Some(Doi("https://doi.org/10.1234/B".to_string())),
                ror: Some(Ror("https://ror.org/0bbbb0000".to_string())),
                country_code: Some(CountryCode::Fra),
            },
        )
        .expect("Failed to create institution");

        let fields: Vec<fn() -> InstitutionField> = vec![
            || InstitutionField::InstitutionId,
            || InstitutionField::InstitutionName,
            || InstitutionField::InstitutionDoi,
            || InstitutionField::Ror,
            || InstitutionField::CountryCode,
            || InstitutionField::CreatedAt,
            || InstitutionField::UpdatedAt,
            || InstitutionField::RorStatus,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Institution::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    InstitutionOrderBy {
                        field: field(),
                        direction,
                    },
                    vec![],
                    None,
                    None,
                    vec![],
                    vec![],
                    None,
                    None,
                )
                .expect("Failed to order institutions");

                assert_eq!(results.len(), 2);
            }
        }
    }
}

/// ROR-SYNC-01C (thoth#1024): `ror_status`, `ror_successor`, the
/// `ROR_SYNC_SERVICE` role, history on create, and the P3 trigger. The section
/// numbers in the test names' comments refer to the stage specification's
/// acceptance list (§4).
#[cfg(feature = "backend")]
mod ror_state {
    use super::*;

    use std::collections::{BTreeSet, HashMap};
    use std::path::Path;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use diesel::connection::SimpleConnection;
    use diesel::sql_types::{BigInt, Bool, Text};
    use diesel::{
        Connection, ExpressionMethods, PgConnection, QueryDsl, QueryableByName, RunQueryDsl,
    };
    use diesel_migrations::MigrationHarness;
    use thoth_errors::{ThothError, ThothResult};
    use zitadel::actix::introspection::IntrospectedUser;

    use crate::db::{PgPool, MIGRATIONS};
    use crate::model::affiliation::{Affiliation, NewAffiliation};
    use crate::model::funding::{Funding, NewFunding};
    use crate::model::institution::crud::{
        may_write_ror_state, CREATE_WITHOUT_HISTORY_ERROR, UPDATE_TABLE_LOCK_SQL,
    };
    use crate::model::institution::policy::InstitutionPolicy;
    use crate::model::publisher::Publisher;
    use crate::model::tests::db::{
        create_contribution, create_contributor, create_imprint, create_publisher, create_work,
        setup_test_db, test_db_url, test_superuser,
    };
    use crate::model::tests::{assert_db_enum_roundtrip, assert_graphql_enum_roundtrip};
    use crate::model::work::Work;
    use crate::policy::{CreatePolicy, DeletePolicy, PolicyContext, Role, UpdatePolicy};
    use crate::schema::{affiliation, institution, institution_history};

    const ROR_A: &str = "https://ror.org/0aaaaaa00";
    const ROR_B: &str = "https://ror.org/0bbbbbb00";
    const ROR_C: &str = "https://ror.org/0cccccc00";
    const ROR_D: &str = "https://ror.org/0dddddd00";

    const C_MIGRATION_VERSION: &str = "20261113";
    const C_DOWN_SQL: &str = include_str!("../../../migrations/20261113_v1.10.0/down.sql");

    const STATE_CHECK_MESSAGE: &str =
        "An institution without a ROR ID must be ACTIVE with no successor.";
    const ACTIVE_SUCCESSOR_MESSAGE: &str = "An ACTIVE institution cannot have a ROR successor.";
    const SELF_SUCCESSOR_MESSAGE: &str = "An institution cannot be its own ROR successor.";
    const INVALID_SUCCESSOR_MESSAGE: &str = "Invalid ROR successor.";

    fn ror(value: &str) -> Ror {
        Ror(value.to_string())
    }

    /// A policy context that can move to another thread.
    struct ThreadCtx {
        db: Arc<PgPool>,
        user: Option<IntrospectedUser>,
    }

    impl PolicyContext for ThreadCtx {
        fn db(&self) -> &PgPool {
            &self.db
        }
        fn user(&self) -> Option<&IntrospectedUser> {
            self.user.as_ref()
        }
    }

    fn ctx(pool: &Arc<PgPool>, user: Option<IntrospectedUser>) -> ThreadCtx {
        ThreadCtx {
            db: pool.clone(),
            user,
        }
    }

    fn user_with_project_roles(
        user_id: &str,
        project_roles: Option<HashMap<String, HashMap<String, String>>>,
    ) -> IntrospectedUser {
        let mut user = test_superuser(user_id);
        user.project_roles = project_roles;
        user
    }

    fn unscoped_user(user_id: &str, roles: &[Role]) -> IntrospectedUser {
        let mut project_roles = HashMap::new();
        for role in roles {
            project_roles.insert(role.as_ref().to_string(), HashMap::new());
        }
        user_with_project_roles(user_id, Some(project_roles))
    }

    fn ror_sync_user(user_id: &str) -> IntrospectedUser {
        unscoped_user(user_id, &[Role::RorSyncService])
    }

    /// One caller class of the stage's matrix (S5).
    struct Caller {
        label: &'static str,
        make: Box<dyn Fn() -> Option<IntrospectedUser>>,
        privileged: bool,
        /// Whether the caller may delete an institution linked to the test
        /// publisher (unchanged by this stage).
        deletes_linked: bool,
    }

    impl Caller {
        fn user(&self) -> Option<IntrospectedUser> {
            (self.make)()
        }
    }

    fn callers(org_id: &str) -> Vec<Caller> {
        let org_id = org_id.to_string();
        let scoped_roles = move || {
            let mut scoped_roles = HashMap::new();
            for role in [
                Role::PublisherAdmin,
                Role::PublisherUser,
                Role::WorkLifecycle,
                Role::CdnWrite,
            ] {
                let mut orgs = HashMap::new();
                orgs.insert(org_id.clone(), "label".to_string());
                scoped_roles.insert(role.as_ref().to_string(), orgs);
            }
            scoped_roles
        };
        vec![
            Caller {
                label: "anonymous",
                make: Box::new(|| None),
                privileged: false,
                deletes_linked: false,
            },
            Caller {
                label: "authenticated, no role",
                make: Box::new(|| Some(user_with_project_roles("matrix-no-role", None))),
                privileged: false,
                deletes_linked: false,
            },
            Caller {
                label: "publisher-scoped roles",
                make: Box::new(move || {
                    Some(user_with_project_roles(
                        "matrix-publisher",
                        Some(scoped_roles()),
                    ))
                }),
                privileged: false,
                deletes_linked: true,
            },
            Caller {
                label: "SUPERUSER",
                make: Box::new(|| Some(unscoped_user("matrix-superuser", &[Role::Superuser]))),
                privileged: true,
                deletes_linked: true,
            },
            Caller {
                label: "ROR_SYNC_SERVICE",
                make: Box::new(|| Some(ror_sync_user("matrix-ror-sync"))),
                privileged: true,
                deletes_linked: false,
            },
            Caller {
                label: "DISSEMINATION_WORKER",
                make: Box::new(|| {
                    Some(unscoped_user(
                        "matrix-dissemination",
                        &[Role::DisseminationWorker],
                    ))
                }),
                privileged: false,
                deletes_linked: false,
            },
            Caller {
                label: "METRICS_INGEST_SERVICE",
                make: Box::new(|| {
                    Some(unscoped_user(
                        "matrix-metrics-ingest",
                        &[Role::MetricsIngestService],
                    ))
                }),
                privileged: false,
                deletes_linked: false,
            },
            Caller {
                label: "METRICS_READ_SERVICE",
                make: Box::new(|| {
                    Some(unscoped_user(
                        "matrix-metrics-read",
                        &[Role::MetricsReadService],
                    ))
                }),
                privileged: false,
                deletes_linked: false,
            },
        ]
    }

    fn new_institution(name: &str, ror_id: Option<&str>) -> NewInstitution {
        NewInstitution {
            institution_name: name.to_string(),
            institution_doi: None,
            ror: ror_id.map(ror),
            country_code: Some(CountryCode::Gbr),
        }
    }

    /// A fixture row, created through the unit-test-only `Crud::create`.
    fn seed(pool: &PgPool, name: &str, ror_id: Option<&str>) -> Institution {
        Institution::create(pool, &new_institution(name, ror_id)).expect("seed institution")
    }

    /// Write a status and successor directly, as a privileged update would.
    fn set_ror_state(
        pool: &PgPool,
        institution_id: Uuid,
        status: RorStatus,
        successor: Option<&str>,
    ) -> Institution {
        let mut connection = pool.get().expect("connection");
        diesel::update(institution::table.find(institution_id))
            .set(&InstitutionRorStatePatch {
                ror_status: Some(status),
                ror_successor: Some(successor.map(ror)),
            })
            .get_result(&mut connection)
            .expect("set ROR state")
    }

    fn patch_from(current: &Institution) -> PatchInstitution {
        PatchInstitution {
            institution_id: current.institution_id,
            institution_name: current.institution_name.clone(),
            institution_doi: current.institution_doi.clone(),
            ror: current.ror.clone(),
            country_code: current.country_code,
            ror_status: None,
            ror_successor: juniper::Nullable::ImplicitNull,
        }
    }

    fn renamed(current: &Institution, suffix: &str) -> PatchInstitution {
        let mut patch = patch_from(current);
        patch.institution_name = format!("{} {suffix}", current.institution_name);
        patch
    }

    fn stored(pool: &PgPool, institution_id: Uuid) -> Institution {
        Institution::from_id(pool, &institution_id).expect("stored institution")
    }

    fn history(pool: &PgPool, institution_id: Uuid) -> Vec<InstitutionHistory> {
        let mut connection = pool.get().expect("connection");
        institution_history::table
            .filter(institution_history::institution_id.eq(institution_id))
            .order(institution_history::timestamp.asc())
            .load::<InstitutionHistory>(&mut connection)
            .expect("history rows")
    }

    fn total_history(pool: &PgPool) -> i64 {
        let mut connection = pool.get().expect("connection");
        institution_history::table
            .count()
            .get_result(&mut connection)
            .expect("history count")
    }

    fn total_institutions(pool: &PgPool) -> i64 {
        let mut connection = pool.get().expect("connection");
        institution::table
            .count()
            .get_result(&mut connection)
            .expect("institution count")
    }

    fn history_data(row: &InstitutionHistory) -> Institution {
        let serde_json::Value::String(text) = &row.data else {
            panic!("history data must be a JSON string, got {:?}", row.data);
        };
        serde_json::from_str(text).expect("history data holds a serialised institution")
    }

    fn assert_constraint_error(result: ThothResult<Institution>, message: &str) {
        match result {
            Err(ThothError::DatabaseConstraintError(actual)) => assert_eq!(actual, message),
            other => panic!("expected `{message}`, got {other:?}"),
        }
    }

    fn assert_unauthorised(result: ThothResult<()>, label: &str) {
        assert!(
            matches!(result, Err(ThothError::Unauthorised)),
            "{label}: expected Unauthorised, got {result:?}"
        );
    }

    #[derive(QueryableByName)]
    struct Count {
        #[diesel(sql_type = BigInt)]
        n: i64,
    }

    #[derive(QueryableByName)]
    struct Flag {
        #[diesel(sql_type = Bool)]
        flag: bool,
    }

    #[derive(QueryableByName)]
    struct TextRow {
        #[diesel(sql_type = Text)]
        value: String,
    }

    fn count(connection: &mut PgConnection, sql: &str) -> i64 {
        diesel::sql_query(sql)
            .get_result::<Count>(connection)
            .unwrap_or_else(|error| panic!("{sql}: {error}"))
            .n
    }

    fn flag(connection: &mut PgConnection, sql: &str) -> bool {
        diesel::sql_query(sql)
            .get_result::<Flag>(connection)
            .unwrap_or_else(|error| panic!("{sql}: {error}"))
            .flag
    }

    fn text(connection: &mut PgConnection, sql: &str) -> String {
        diesel::sql_query(sql)
            .get_result::<TextRow>(connection)
            .unwrap_or_else(|error| panic!("{sql}: {error}"))
            .value
    }

    fn connect() -> PgConnection {
        PgConnection::establish(&test_db_url()).expect("a dedicated test connection")
    }

    // ------------------------------------------------------------------
    // Enum, field and schema basics (§4.3)
    // ------------------------------------------------------------------

    #[test]
    fn ror_status_display_parse_and_graphql_conversions() {
        assert_eq!(RorStatus::default(), RorStatus::Active);
        for (label, status) in [
            ("Active", RorStatus::Active),
            ("Inactive", RorStatus::Inactive),
            ("Withdrawn", RorStatus::Withdrawn),
        ] {
            assert_eq!(status.to_string(), label);
            assert_eq!(RorStatus::from_str(label).unwrap(), status);
            assert_graphql_enum_roundtrip(status);
        }
        assert_eq!(InstitutionField::RorStatus.to_string(), "ROR Status");
        assert_eq!(
            InstitutionField::from_str("ROR Status").unwrap(),
            InstitutionField::RorStatus
        );
        assert_eq!(
            serde_json::to_string(&RorStatus::Withdrawn).unwrap(),
            "\"WITHDRAWN\""
        );
    }

    #[test]
    fn ror_status_db_enum_roundtrip() {
        let (_guard, pool) = setup_test_db();
        for (literal, status) in [
            ("'active'::ror_status", RorStatus::Active),
            ("'inactive'::ror_status", RorStatus::Inactive),
            ("'withdrawn'::ror_status", RorStatus::Withdrawn),
        ] {
            assert_db_enum_roundtrip::<RorStatus, crate::schema::sql_types::RorStatus>(
                pool.as_ref(),
                literal,
                status,
            );
        }
    }

    #[test]
    fn schema_rs_the_model_and_the_database_agree_on_the_column_order() {
        let (_guard, pool) = setup_test_db();
        let expected = [
            "institution_id",
            "institution_name",
            "institution_doi",
            "created_at",
            "updated_at",
            "ror",
            "country_code",
            "ror_status",
            "ror_successor",
        ];

        #[derive(QueryableByName)]
        struct Column {
            #[diesel(sql_type = Text)]
            column_name: String,
        }
        let mut connection = pool.get().expect("connection");
        let database: Vec<String> = diesel::sql_query(
            "SELECT column_name::text AS column_name FROM information_schema.columns \
             WHERE table_schema = 'public' AND table_name = 'institution' \
             ORDER BY ordinal_position",
        )
        .load::<Column>(&mut connection)
        .expect("columns")
        .into_iter()
        .map(|column| column.column_name)
        .collect();
        assert_eq!(database, expected);

        let select = diesel::debug_query::<diesel::pg::Pg, _>(
            &institution::table.select(institution::all_columns),
        )
        .to_string();
        let schema_rs: Vec<&str> = select
            .split("\"institution\".\"")
            .skip(1)
            .map(|rest| rest.split('"').next().expect("a column name"))
            .collect();
        assert_eq!(schema_rs, expected);

        // The model reads each column into the field of the same name.
        let created = seed(pool.as_ref(), "Column order", Some(ROR_A));
        set_ror_state(
            pool.as_ref(),
            created.institution_id,
            RorStatus::Withdrawn,
            Some(ROR_B),
        );
        let loaded = stored(pool.as_ref(), created.institution_id);
        assert_eq!(loaded.ror, Some(ror(ROR_A)));
        assert_eq!(loaded.country_code, Some(CountryCode::Gbr));
        assert_eq!(loaded.ror_status, RorStatus::Withdrawn);
        assert_eq!(loaded.ror_successor, Some(ror(ROR_B)));
    }

    // ------------------------------------------------------------------
    // Building the status and successor changeset (S4 step 3, S5)
    // ------------------------------------------------------------------

    #[test]
    fn the_ror_state_patch_follows_privilege_and_ror_changes() {
        let statuses = [None, Some(RorStatus::Inactive)];
        let successors = [
            juniper::Nullable::ImplicitNull,
            juniper::Nullable::ExplicitNull,
            juniper::Nullable::Some(ror(ROR_B)),
        ];
        for privileged in [false, true] {
            for ror_changes in [false, true] {
                for status in statuses {
                    for successor in successors.clone() {
                        let mut patch = patch_from(&Institution::default());
                        patch.ror_status = status;
                        patch.ror_successor = successor.clone();
                        let built =
                            InstitutionRorStatePatch::for_update(&patch, privileged, ror_changes);

                        let supplied_status = if privileged { status } else { None };
                        let supplied_successor = match (privileged, &successor) {
                            (false, _) | (true, juniper::Nullable::ImplicitNull) => None,
                            (true, juniper::Nullable::ExplicitNull) => Some(None),
                            (true, juniper::Nullable::Some(value)) => Some(Some(value.clone())),
                        };
                        let expected = if ror_changes {
                            InstitutionRorStatePatch {
                                ror_status: supplied_status.or(Some(RorStatus::Active)),
                                ror_successor: supplied_successor.or(Some(None)),
                            }
                        } else {
                            InstitutionRorStatePatch {
                                ror_status: supplied_status,
                                ror_successor: supplied_successor,
                            }
                        };
                        assert_eq!(
                            built, expected,
                            "privileged={privileged} ror_changes={ror_changes} \
                             status={status:?} successor={successor:?}"
                        );
                        if !privileged && !ror_changes {
                            assert_eq!(built, InstitutionRorStatePatch::default());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn the_update_is_one_statement_with_or_without_the_state_patch() {
        let id = Uuid::new_v4();
        let changeset = InstitutionChangeset::from(&patch_from(&Institution {
            institution_id: id,
            institution_name: "One statement".to_string(),
            ..Default::default()
        }));

        let empty = diesel::debug_query::<diesel::pg::Pg, _>(
            &diesel::update(institution::table.find(id))
                .set((&changeset, &InstitutionRorStatePatch::default())),
        )
        .to_string();
        assert_eq!(empty.matches("UPDATE").count(), 1, "{empty}");
        assert!(empty.contains("\"institution_name\" = $1"), "{empty}");
        assert!(!empty.contains("ror_status"), "{empty}");
        assert!(!empty.contains("ror_successor"), "{empty}");

        let full = diesel::debug_query::<diesel::pg::Pg, _>(
            &diesel::update(institution::table.find(id)).set((
                &changeset,
                &InstitutionRorStatePatch {
                    ror_status: Some(RorStatus::Active),
                    ror_successor: Some(None),
                },
            )),
        )
        .to_string();
        assert_eq!(full.matches("UPDATE").count(), 1, "{full}");
        for column in ["\"ror\" =", "\"ror_status\" =", "\"ror_successor\" ="] {
            assert!(full.contains(column), "{column} missing from {full}");
        }
    }

    // ------------------------------------------------------------------
    // The policy matrix (§4.4)
    // ------------------------------------------------------------------

    #[test]
    fn matrix_create_any_authenticated_caller_creates_an_active_row_with_history() {
        let (_guard, pool) = setup_test_db();
        for caller in callers("org-matrix") {
            let ctx = ctx(&pool, caller.user());
            let data = new_institution(&format!("Create {}", caller.label), Some(ROR_A));
            let before = (total_institutions(&pool), total_history(&pool));
            let policy = InstitutionPolicy::can_create(&ctx, &data, ());
            let created = Institution::create_with_history(&ctx, &data);
            match caller.user() {
                None => {
                    assert_unauthorised(policy, caller.label);
                    assert!(matches!(created, Err(ThothError::Unauthorised)));
                    assert_eq!(
                        (total_institutions(&pool), total_history(&pool)),
                        before,
                        "{}: a refused create writes nothing",
                        caller.label
                    );
                }
                Some(user) => {
                    assert!(policy.is_ok(), "{}: create allowed", caller.label);
                    let created = created.expect("create");
                    assert_eq!(created.ror_status, RorStatus::Active, "{}", caller.label);
                    assert_eq!(created.ror_successor, None, "{}", caller.label);
                    let rows = history(&pool, created.institution_id);
                    assert_eq!(rows.len(), 1, "{}: one history row", caller.label);
                    assert_eq!(rows[0].user_id, user.user_id);
                    assert_eq!(history_data(&rows[0]), created);
                }
            }
        }
    }

    #[test]
    fn matrix_update_other_fields_is_open_to_every_authenticated_caller_and_keeps_the_state() {
        let (_guard, pool) = setup_test_db();
        for caller in callers("org-matrix") {
            let row = seed(
                pool.as_ref(),
                &format!("Other {}", caller.label),
                Some(ROR_A),
            );
            let row = set_ror_state(
                pool.as_ref(),
                row.institution_id,
                RorStatus::Inactive,
                Some(ROR_B),
            );
            let ctx = ctx(&pool, caller.user());
            let patch = renamed(&row, "renamed");
            let policy = InstitutionPolicy::can_update(&ctx, &row, &patch, ());
            if caller.user().is_none() {
                assert_unauthorised(policy, caller.label);
                continue;
            }
            assert!(policy.is_ok(), "{}", caller.label);
            let updated = row.update(&ctx, &patch).expect("update");
            assert_eq!(updated.institution_name, patch.institution_name);
            assert_eq!(updated.ror_status, RorStatus::Inactive, "{}", caller.label);
            assert_eq!(updated.ror_successor, Some(ror(ROR_B)), "{}", caller.label);
        }
    }

    #[test]
    fn matrix_update_status_differing_is_denied_and_equal_is_dropped_unless_privileged() {
        let (_guard, pool) = setup_test_db();
        for caller in callers("org-matrix") {
            let row = seed(
                pool.as_ref(),
                &format!("Status {}", caller.label),
                Some(ROR_A),
            );
            let ctx = ctx(&pool, caller.user());

            let mut differing = renamed(&row, "differing");
            differing.ror_status = Some(RorStatus::Inactive);
            let mut equal = renamed(&row, "equal");
            equal.ror_status = Some(RorStatus::Active);

            if caller.user().is_none() {
                assert_unauthorised(
                    InstitutionPolicy::can_update(&ctx, &row, &differing, ()),
                    caller.label,
                );
                assert_unauthorised(
                    InstitutionPolicy::can_update(&ctx, &row, &equal, ()),
                    caller.label,
                );
                continue;
            }
            assert_eq!(
                may_write_ror_state(&ctx),
                caller.privileged,
                "{}",
                caller.label
            );

            assert!(InstitutionPolicy::can_update(&ctx, &row, &equal, ()).is_ok());
            let after_equal = row.update(&ctx, &equal).expect("equal value update");
            assert_eq!(
                after_equal.ror_status,
                RorStatus::Active,
                "{}",
                caller.label
            );

            let policy = InstitutionPolicy::can_update(&ctx, &after_equal, &differing, ());
            if caller.privileged {
                assert!(policy.is_ok(), "{}", caller.label);
                let written = after_equal.update(&ctx, &differing).expect("privileged");
                assert_eq!(written.ror_status, RorStatus::Inactive, "{}", caller.label);
            } else {
                assert_unauthorised(policy, caller.label);
                assert_eq!(
                    stored(pool.as_ref(), row.institution_id).ror_status,
                    RorStatus::Active
                );
            }
        }
    }

    #[test]
    fn matrix_update_successor_value_and_explicit_null_follow_the_same_rule() {
        let (_guard, pool) = setup_test_db();
        for caller in callers("org-matrix") {
            let ctx = ctx(&pool, caller.user());
            let row = seed(
                pool.as_ref(),
                &format!("Successor {}", caller.label),
                Some(ROR_A),
            );
            let row = set_ror_state(
                pool.as_ref(),
                row.institution_id,
                RorStatus::Inactive,
                Some(ROR_B),
            );

            let mut differing = renamed(&row, "differing");
            differing.ror_successor = juniper::Nullable::Some(ror(ROR_C));
            let mut equal = renamed(&row, "equal");
            equal.ror_successor = juniper::Nullable::Some(ror(ROR_B));
            let mut clear = renamed(&row, "clear");
            clear.ror_successor = juniper::Nullable::ExplicitNull;

            if caller.user().is_none() {
                for patch in [&differing, &equal, &clear] {
                    assert_unauthorised(
                        InstitutionPolicy::can_update(&ctx, &row, patch, ()),
                        caller.label,
                    );
                }
                continue;
            }

            assert!(InstitutionPolicy::can_update(&ctx, &row, &equal, ()).is_ok());
            let row = row.update(&ctx, &equal).expect("equal successor");
            assert_eq!(row.ror_successor, Some(ror(ROR_B)), "{}", caller.label);

            if caller.privileged {
                assert!(InstitutionPolicy::can_update(&ctx, &row, &differing, ()).is_ok());
                let row = row.update(&ctx, &differing).expect("set successor");
                assert_eq!(row.ror_successor, Some(ror(ROR_C)), "{}", caller.label);
                assert!(InstitutionPolicy::can_update(&ctx, &row, &clear, ()).is_ok());
                let row = row.update(&ctx, &clear).expect("clear successor");
                assert_eq!(
                    row.ror_successor, None,
                    "{}: explicit null clears",
                    caller.label
                );
                assert_eq!(row.ror_status, RorStatus::Inactive);
            } else {
                assert_unauthorised(
                    InstitutionPolicy::can_update(&ctx, &row, &differing, ()),
                    caller.label,
                );
                assert_unauthorised(
                    InstitutionPolicy::can_update(&ctx, &row, &clear, ()),
                    caller.label,
                );
                // On a row whose successor is null, an explicit null is equal,
                // so it passes and is dropped.
                let null_row =
                    set_ror_state(pool.as_ref(), row.institution_id, RorStatus::Inactive, None);
                let mut clear_null = renamed(&null_row, "null");
                clear_null.ror_successor = juniper::Nullable::ExplicitNull;
                assert!(InstitutionPolicy::can_update(&ctx, &null_row, &clear_null, ()).is_ok());
                let after = null_row.update(&ctx, &clear_null).expect("dropped null");
                assert_eq!(after.ror_successor, None);
                assert_eq!(after.ror_status, RorStatus::Inactive);
            }
        }
    }

    #[test]
    fn matrix_delete_is_unchanged() {
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher.zitadel_id.clone().expect("org id");
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let linked = seed(pool.as_ref(), "Linked", Some(ROR_A));
        Funding::create(
            pool.as_ref(),
            &NewFunding {
                work_id: work.work_id,
                institution_id: linked.institution_id,
                program: None,
                project_name: None,
                project_shortname: None,
                grant_number: None,
            },
        )
        .expect("funding");
        let unlinked = seed(pool.as_ref(), "Unlinked", Some(ROR_B));

        for caller in callers(&org_id) {
            let ctx = ctx(&pool, caller.user());
            let on_unlinked = InstitutionPolicy::can_delete(&ctx, &unlinked);
            let on_linked = InstitutionPolicy::can_delete(&ctx, &linked);
            if caller.user().is_none() {
                assert_unauthorised(on_unlinked, caller.label);
                assert_unauthorised(on_linked, caller.label);
                continue;
            }
            assert!(on_unlinked.is_ok(), "{}: unlinked row", caller.label);
            if caller.deletes_linked {
                assert!(on_linked.is_ok(), "{}: linked row", caller.label);
            } else {
                assert_unauthorised(on_linked, caller.label);
            }
        }
    }

    #[test]
    fn a_privileged_write_between_an_unprivileged_load_and_its_update_survives() {
        let (_guard, pool) = setup_test_db();
        let row = seed(pool.as_ref(), "Race", Some(ROR_A));
        let unprivileged = ctx(&pool, Some(user_with_project_roles("race-user", None)));
        let sync = ctx(&pool, Some(ror_sync_user("race-sync")));

        // The unprivileged caller's resolver loads the row: ACTIVE, no successor.
        let loaded = Institution::from_id(pool.as_ref(), &row.institution_id).expect("load");

        // The sync commits a status change in between.
        let mut sync_patch = patch_from(&loaded);
        sync_patch.ror_status = Some(RorStatus::Withdrawn);
        sync_patch.ror_successor = juniper::Nullable::Some(ror(ROR_B));
        loaded.update(&sync, &sync_patch).expect("sync write");

        // The unprivileged caller sends the values it loaded, as a client that
        // echoes every field would. They equal the loaded row, so the policy
        // passes, and the model drops them.
        let mut stale = renamed(&loaded, "edited");
        stale.ror_status = Some(RorStatus::Active);
        stale.ror_successor = juniper::Nullable::ExplicitNull;
        assert!(InstitutionPolicy::can_update(&unprivileged, &loaded, &stale, ()).is_ok());
        let after = loaded.update(&unprivileged, &stale).expect("stale update");

        assert_eq!(after.institution_name, stale.institution_name);
        assert_eq!(after.ror_status, RorStatus::Withdrawn);
        assert_eq!(after.ror_successor, Some(ror(ROR_B)));
    }

    // ------------------------------------------------------------------
    // Changes of `ror` (§4.6)
    // ------------------------------------------------------------------

    #[test]
    fn an_update_that_changes_ror_resets_status_and_successor() {
        let (_guard, pool) = setup_test_db();
        for (label, user) in [
            (
                "unprivileged",
                user_with_project_roles("ror-change-user", None),
            ),
            (
                "privileged, fields not supplied",
                ror_sync_user("ror-change-sync"),
            ),
        ] {
            let row = seed(pool.as_ref(), &format!("Ror change {label}"), Some(ROR_A));
            let row = set_ror_state(
                pool.as_ref(),
                row.institution_id,
                RorStatus::Withdrawn,
                Some(ROR_B),
            );
            let ctx = ctx(&pool, Some(user));
            let mut patch = patch_from(&row);
            patch.ror = Some(ror(ROR_C));
            let after = row.update(&ctx, &patch).expect("ror change");
            assert_eq!(after.ror, Some(ror(ROR_C)), "{label}");
            assert_eq!(after.ror_status, RorStatus::Active, "{label}");
            assert_eq!(after.ror_successor, None, "{label}");
        }
    }

    #[test]
    fn a_privileged_update_that_changes_ror_writes_what_it_supplies_and_resets_the_rest() {
        let (_guard, pool) = setup_test_db();
        let sync = ctx(&pool, Some(ror_sync_user("ror-change-sync")));

        // Both supplied: both written.
        let row = seed(pool.as_ref(), "Both supplied", Some(ROR_A));
        let mut patch = patch_from(&row);
        patch.ror = Some(ror(ROR_C));
        patch.ror_status = Some(RorStatus::Inactive);
        patch.ror_successor = juniper::Nullable::Some(ror(ROR_B));
        let after = row.update(&sync, &patch).expect("both supplied");
        assert_eq!(
            (after.ror, after.ror_status, after.ror_successor),
            (Some(ror(ROR_C)), RorStatus::Inactive, Some(ror(ROR_B)))
        );

        // Only the status supplied: the successor is reset.
        let row = seed(pool.as_ref(), "Status supplied", Some(ROR_A));
        let row = set_ror_state(
            pool.as_ref(),
            row.institution_id,
            RorStatus::Inactive,
            Some(ROR_B),
        );
        let mut patch = patch_from(&row);
        patch.ror = Some(ror(ROR_D));
        patch.ror_status = Some(RorStatus::Withdrawn);
        let after = row.update(&sync, &patch).expect("status supplied");
        assert_eq!(
            (after.ror, after.ror_status, after.ror_successor),
            (Some(ror(ROR_D)), RorStatus::Withdrawn, None)
        );

        // Only the successor supplied: the status is reset to ACTIVE, and an
        // ACTIVE row cannot have a successor.
        let row = seed(pool.as_ref(), "Successor supplied", Some(ROR_A));
        let row = set_ror_state(
            pool.as_ref(),
            row.institution_id,
            RorStatus::Withdrawn,
            Some(ROR_B),
        );
        let history_before = history(pool.as_ref(), row.institution_id).len();
        let mut patch = patch_from(&row);
        patch.ror = Some(ror(ROR_D));
        patch.ror_successor = juniper::Nullable::Some(ror(ROR_C));
        assert_constraint_error(row.update(&sync, &patch), ACTIVE_SUCCESSOR_MESSAGE);
        let unchanged = stored(pool.as_ref(), row.institution_id);
        assert_eq!(unchanged, row);
        assert_eq!(
            history(pool.as_ref(), row.institution_id).len(),
            history_before
        );
    }

    #[test]
    fn clearing_ror_on_a_withdrawn_row_leaves_it_active_with_no_successor() {
        let (_guard, pool) = setup_test_db();
        let row = seed(pool.as_ref(), "Clear ror", Some(ROR_A));
        let row = set_ror_state(
            pool.as_ref(),
            row.institution_id,
            RorStatus::Withdrawn,
            Some(ROR_B),
        );
        let ctx = ctx(&pool, Some(user_with_project_roles("clear-ror", None)));
        let mut patch = patch_from(&row);
        patch.ror = None;
        // Two statements would write an intermediate row violating
        // `institution_ror_state_check`; one statement succeeds.
        let after = row.update(&ctx, &patch).expect("clear ror");
        assert_eq!(
            (after.ror, after.ror_status, after.ror_successor),
            (None, RorStatus::Active, None)
        );
    }

    #[test]
    fn the_four_checks_return_their_messages() {
        let (_guard, pool) = setup_test_db();
        let superuser = ctx(&pool, Some(unscoped_user("checks", &[Role::Superuser])));

        // A status on a row without a ROR ID.
        let no_ror = seed(pool.as_ref(), "No ror", None);
        let mut patch = patch_from(&no_ror);
        patch.ror_status = Some(RorStatus::Inactive);
        assert_constraint_error(no_ror.update(&superuser, &patch), STATE_CHECK_MESSAGE);

        // A successor on an ACTIVE row.
        let active = seed(pool.as_ref(), "Active", Some(ROR_A));
        let mut patch = patch_from(&active);
        patch.ror_successor = juniper::Nullable::Some(ror(ROR_B));
        assert_constraint_error(active.update(&superuser, &patch), ACTIVE_SUCCESSOR_MESSAGE);

        // Its own ROR ID as successor.
        let mut patch = patch_from(&active);
        patch.ror_status = Some(RorStatus::Withdrawn);
        patch.ror_successor = juniper::Nullable::Some(ror(ROR_A));
        assert_constraint_error(active.update(&superuser, &patch), SELF_SUCCESSOR_MESSAGE);

        // A malformed successor (GraphQL's `Ror` scalar would refuse it first).
        let mut patch = patch_from(&active);
        patch.ror_status = Some(RorStatus::Withdrawn);
        patch.ror_successor = juniper::Nullable::Some(ror("https://ror.org/not-a-ror"));
        assert_constraint_error(active.update(&superuser, &patch), INVALID_SUCCESSOR_MESSAGE);

        assert_eq!(stored(pool.as_ref(), active.institution_id), active);
        assert_eq!(stored(pool.as_ref(), no_ror.institution_id), no_ror);
    }

    // ------------------------------------------------------------------
    // Locking (§4.6)
    // ------------------------------------------------------------------

    fn wait_for_lock_wait(observer: &mut PgConnection, relation: &str, mode: &str) {
        let sql = format!(
            "SELECT count(*) AS n FROM pg_locks \
             WHERE relation = '{relation}'::regclass AND mode = '{mode}' AND NOT granted"
        );
        let deadline = Instant::now() + Duration::from_secs(30);
        while count(observer, &sql) == 0 {
            assert!(
                Instant::now() < deadline,
                "no session started waiting for {mode} on {relation}"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn try_row_lock(connection: &mut PgConnection, institution_id: Uuid, mode: &str) -> bool {
        connection.batch_execute("BEGIN").expect("begin");
        let result = diesel::sql_query(format!(
            "SELECT institution_id FROM institution WHERE institution_id = '{institution_id}' \
             {mode} NOWAIT"
        ))
        .execute(connection);
        connection.batch_execute("ROLLBACK").expect("rollback");
        match result {
            Ok(rows) => {
                assert_eq!(rows, 1);
                true
            }
            Err(error) if error.to_string().contains("could not obtain lock") => false,
            Err(error) => panic!("unexpected probe error: {error}"),
        }
    }

    #[test]
    fn an_update_keeping_ror_locks_for_no_key_update_and_new_affiliations_do_not_wait() {
        assert_eq!(
            UPDATE_TABLE_LOCK_SQL,
            "LOCK TABLE institution IN ROW EXCLUSIVE MODE"
        );
        let (_guard, pool) = setup_test_db();
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        let contributor = create_contributor(pool.as_ref());
        let contribution = create_contribution(pool.as_ref(), &work, &contributor);
        let target = seed(pool.as_ref(), "Lock target", Some(ROR_A));

        // Stop the update after its row lock: the history insert waits here.
        let mut holder = connect();
        holder
            .batch_execute("BEGIN; LOCK TABLE institution_history IN ACCESS EXCLUSIVE MODE;")
            .expect("hold institution_history");

        let update_ctx = ctx(&pool, Some(user_with_project_roles("lock-user", None)));
        let patch = renamed(&target, "renamed");
        let current = target.clone();
        let update = thread::spawn(move || current.update(&update_ctx, &patch));

        let mut observer = connect();
        wait_for_lock_wait(&mut observer, "institution_history", "RowExclusiveLock");

        // The row is locked, and not FOR UPDATE: FOR KEY SHARE is compatible
        // only with FOR NO KEY UPDATE and weaker locks.
        let mut probe = connect();
        assert!(!try_row_lock(
            &mut probe,
            target.institution_id,
            "FOR UPDATE"
        ));
        assert!(try_row_lock(
            &mut probe,
            target.institution_id,
            "FOR KEY SHARE"
        ));

        // A new affiliation referencing the row, on a work not linked to it,
        // does not wait.
        let mut writer = connect();
        writer
            .batch_execute("SET lock_timeout = '2s'")
            .expect("lock timeout");
        let started = Instant::now();
        diesel::insert_into(affiliation::table)
            .values(&NewAffiliation {
                contribution_id: contribution.contribution_id,
                institution_id: target.institution_id,
                affiliation_ordinal: 1,
                position: None,
            })
            .execute(&mut writer)
            .expect("the affiliation insert must not wait on the update's row lock");
        assert!(started.elapsed() < Duration::from_secs(2));

        holder.batch_execute("ROLLBACK").expect("release");
        let updated = update.join().expect("update thread").expect("update");
        assert_eq!(
            updated.institution_name,
            format!("{} renamed", target.institution_name)
        );
        assert_eq!(
            Affiliation::all(
                pool.as_ref(),
                10,
                0,
                None,
                Default::default(),
                vec![],
                Some(target.institution_id),
                None,
                vec![],
                vec![],
                None,
                None,
            )
            .expect("affiliations")
            .len(),
            1
        );
    }

    #[test]
    fn an_update_waiting_behind_share_row_exclusive_holds_no_row_lock() {
        let (_guard, pool) = setup_test_db();
        let target = seed(pool.as_ref(), "Waiting target", Some(ROR_A));

        // As stage B's migration does.
        let mut holder = connect();
        holder
            .batch_execute("BEGIN; LOCK TABLE institution IN SHARE ROW EXCLUSIVE MODE;")
            .expect("hold SHARE ROW EXCLUSIVE");

        let update_ctx = ctx(&pool, Some(ror_sync_user("waiting-sync")));
        let patch = renamed(&target, "renamed");
        let current = target.clone();
        let update = thread::spawn(move || current.update(&update_ctx, &patch));

        let mut observer = connect();
        wait_for_lock_wait(&mut observer, "institution", "RowExclusiveLock");

        let mut probe = connect();
        assert!(
            try_row_lock(&mut probe, target.institution_id, "FOR UPDATE"),
            "the waiting update must hold no row lock"
        );

        // Control: the probe does see a row lock when one is held.
        let mut row_holder = connect();
        row_holder.batch_execute("BEGIN").expect("begin");
        diesel::sql_query(format!(
            "SELECT institution_id FROM institution WHERE institution_id = '{}' \
             FOR NO KEY UPDATE",
            target.institution_id
        ))
        .execute(&mut row_holder)
        .expect("row lock");
        assert!(!try_row_lock(
            &mut probe,
            target.institution_id,
            "FOR UPDATE"
        ));
        row_holder.batch_execute("ROLLBACK").expect("release row");

        holder.batch_execute("ROLLBACK").expect("release table");
        let updated = update.join().expect("update thread").expect("update");
        assert_eq!(
            updated.institution_name,
            format!("{} renamed", target.institution_name)
        );
    }

    // ------------------------------------------------------------------
    // `rorStatuses` and the order (§4.7)
    // ------------------------------------------------------------------

    fn list(pool: &PgPool, filter: Option<&str>, statuses: Vec<RorStatus>) -> Vec<Uuid> {
        Institution::all(
            pool,
            100,
            0,
            filter.map(str::to_string),
            InstitutionOrderBy {
                field: InstitutionField::InstitutionId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            statuses,
            vec![],
            None,
            None,
        )
        .expect("list")
        .into_iter()
        .map(|row| row.institution_id)
        .collect()
    }

    fn count_with(pool: &PgPool, filter: Option<&str>, statuses: Vec<RorStatus>) -> i32 {
        Institution::count(
            pool,
            filter.map(str::to_string),
            vec![],
            statuses,
            vec![],
            None,
            None,
        )
        .expect("count")
    }

    #[test]
    fn ror_statuses_filters_lists_and_counts_alike() {
        let (_guard, pool) = setup_test_db();
        let active = seed(pool.as_ref(), "Filter active", Some(ROR_A));
        let inactive = seed(pool.as_ref(), "Filter inactive", Some(ROR_B));
        set_ror_state(
            pool.as_ref(),
            inactive.institution_id,
            RorStatus::Inactive,
            None,
        );
        let withdrawn = seed(pool.as_ref(), "Filter withdrawn", Some(ROR_C));
        set_ror_state(
            pool.as_ref(),
            withdrawn.institution_id,
            RorStatus::Withdrawn,
            Some(ROR_A),
        );
        let no_ror = seed(pool.as_ref(), "Filter no ror", None);

        let sorted = |mut ids: Vec<Uuid>| {
            ids.sort();
            ids
        };
        let all = sorted(vec![
            active.institution_id,
            inactive.institution_id,
            withdrawn.institution_id,
            no_ror.institution_id,
        ]);
        let cases: Vec<(Option<&str>, Vec<RorStatus>, Vec<Uuid>)> = vec![
            (None, vec![], all.clone()),
            (
                None,
                vec![RorStatus::Active, RorStatus::Inactive, RorStatus::Withdrawn],
                all.clone(),
            ),
            (
                None,
                vec![RorStatus::Withdrawn],
                vec![withdrawn.institution_id],
            ),
            (
                None,
                vec![RorStatus::Active, RorStatus::Withdrawn],
                sorted(vec![
                    active.institution_id,
                    withdrawn.institution_id,
                    no_ror.institution_id,
                ]),
            ),
            (
                Some("Filter"),
                vec![RorStatus::Inactive],
                vec![inactive.institution_id],
            ),
            (
                Some("0aaaaaa"),
                vec![RorStatus::Active],
                vec![active.institution_id],
            ),
            (Some("0aaaaaa"), vec![RorStatus::Withdrawn], vec![]),
        ];
        for (filter, statuses, expected) in cases {
            assert_eq!(
                list(pool.as_ref(), filter, statuses.clone()),
                expected,
                "list {filter:?} {statuses:?}"
            );
            assert_eq!(
                count_with(pool.as_ref(), filter, statuses.clone()),
                expected.len() as i32,
                "count {filter:?} {statuses:?}"
            );
        }
    }

    #[test]
    fn ordering_by_ror_status_follows_the_enum_and_breaks_ties_on_the_id() {
        let (_guard, pool) = setup_test_db();
        let withdrawn = seed(pool.as_ref(), "Order withdrawn", Some(ROR_A));
        set_ror_state(
            pool.as_ref(),
            withdrawn.institution_id,
            RorStatus::Withdrawn,
            None,
        );
        let inactive = seed(pool.as_ref(), "Order inactive", Some(ROR_B));
        set_ror_state(
            pool.as_ref(),
            inactive.institution_id,
            RorStatus::Inactive,
            None,
        );
        let mut actives = vec![
            seed(pool.as_ref(), "Order active one", Some(ROR_C)).institution_id,
            seed(pool.as_ref(), "Order active two", None).institution_id,
            seed(pool.as_ref(), "Order active three", Some(ROR_D)).institution_id,
        ];
        actives.sort();

        let ordered = |direction| {
            Institution::all(
                pool.as_ref(),
                100,
                0,
                None,
                InstitutionOrderBy {
                    field: InstitutionField::RorStatus,
                    direction,
                },
                vec![],
                None,
                None,
                vec![],
                vec![],
                None,
                None,
            )
            .expect("ordered")
            .into_iter()
            .map(|row| row.institution_id)
            .collect::<Vec<_>>()
        };

        let mut asc = actives.clone();
        asc.extend([inactive.institution_id, withdrawn.institution_id]);
        let mut desc = vec![withdrawn.institution_id, inactive.institution_id];
        desc.extend(actives.iter().copied());
        for _ in 0..3 {
            assert_eq!(ordered(Direction::Asc), asc);
            assert_eq!(ordered(Direction::Desc), desc);
        }
    }

    // ------------------------------------------------------------------
    // The works-bump trigger (§4.8, decision P3)
    // ------------------------------------------------------------------

    struct BumpFixture {
        institution: Institution,
        funded: Work,
        affiliated: Work,
    }

    fn bump_fixture(pool: &PgPool) -> BumpFixture {
        let publisher: Publisher = create_publisher(pool);
        let imprint = create_imprint(pool, &publisher);
        let funded = create_work(pool, &imprint);
        let affiliated = create_work(pool, &imprint);
        let contributor = create_contributor(pool);
        let contribution = create_contribution(pool, &affiliated, &contributor);
        let institution = seed(pool, "Bump", Some(ROR_A));
        Funding::create(
            pool,
            &NewFunding {
                work_id: funded.work_id,
                institution_id: institution.institution_id,
                program: None,
                project_name: None,
                project_shortname: None,
                grant_number: None,
            },
        )
        .expect("funding");
        Affiliation::create(
            pool,
            &NewAffiliation {
                contribution_id: contribution.contribution_id,
                institution_id: institution.institution_id,
                affiliation_ordinal: 1,
                position: None,
            },
        )
        .expect("affiliation");
        BumpFixture {
            institution,
            funded,
            affiliated,
        }
    }

    /// Run `change` on the institution and report, for the funded and the
    /// affiliated work, whether the trigger bumped `updated_at_with_relations`.
    fn bumps(pool: &PgPool, fixture: &BumpFixture, change: &str) -> (bool, bool) {
        let mut connection = pool.get().expect("connection");
        let works = format!(
            "('{}', '{}')",
            fixture.funded.work_id, fixture.affiliated.work_id
        );
        connection
            .batch_execute(&format!(
                "UPDATE work SET updated_at_with_relations = '2000-01-01T00:00:00Z' \
                 WHERE work_id IN {works}"
            ))
            .expect("reset the bump marker");
        connection
            .batch_execute(&format!(
                "UPDATE institution SET {change} WHERE institution_id = '{}'",
                fixture.institution.institution_id
            ))
            .unwrap_or_else(|error| panic!("{change}: {error}"));
        let bumped = |connection: &mut PgConnection, work_id: Uuid| {
            flag(
                connection,
                &format!(
                    "SELECT updated_at_with_relations > '2000-01-02T00:00:00Z' AS flag \
                     FROM work WHERE work_id = '{work_id}'"
                ),
            )
        };
        (
            bumped(&mut connection, fixture.funded.work_id),
            bumped(&mut connection, fixture.affiliated.work_id),
        )
    }

    #[test]
    fn the_trigger_bumps_works_only_for_exported_fields() {
        let (_guard, pool) = setup_test_db();
        let fixture = bump_fixture(pool.as_ref());

        for change in [
            "ror_status = 'inactive'".to_string(),
            format!("ror_successor = '{ROR_B}'"),
            format!("ror_status = 'withdrawn', ror_successor = '{ROR_C}'"),
        ] {
            assert_eq!(
                bumps(pool.as_ref(), &fixture, &change),
                (false, false),
                "{change} must not bump"
            );
        }
        for change in [
            "institution_name = 'Bump renamed'".to_string(),
            "institution_doi = 'https://doi.org/10.13039/100000001'".to_string(),
            format!("ror = '{ROR_D}'"),
            "country_code = 'fra'".to_string(),
        ] {
            assert_eq!(
                bumps(pool.as_ref(), &fixture, &change),
                (true, true),
                "{change} must bump"
            );
        }

        // Through the model too: a status-only privileged update bumps nothing.
        let current = stored(pool.as_ref(), fixture.institution.institution_id);
        let sync = ctx(&pool, Some(ror_sync_user("bump-sync")));
        let mut connection = pool.get().expect("connection");
        connection
            .batch_execute(&format!(
                "UPDATE work SET updated_at_with_relations = '2000-01-01T00:00:00Z' \
                 WHERE work_id IN ('{}', '{}')",
                fixture.funded.work_id, fixture.affiliated.work_id
            ))
            .expect("reset");
        let mut patch = patch_from(&current);
        patch.ror_status = Some(RorStatus::Inactive);
        patch.ror_successor = juniper::Nullable::ExplicitNull;
        current.update(&sync, &patch).expect("status-only update");
        assert_eq!(
            count(
                &mut connection,
                &format!(
                    "SELECT count(*) AS n FROM work WHERE work_id IN ('{}', '{}') \
                     AND updated_at_with_relations > '2000-01-02T00:00:00Z'",
                    fixture.funded.work_id, fixture.affiliated.work_id
                )
            ),
            0
        );
    }

    /// The column names inside each `ROW(<prefix>…)` of a function definition.
    fn row_fields(definition: &str, prefix: &str) -> Vec<BTreeSet<String>> {
        definition
            .split("ROW(")
            .skip(1)
            .map(|rest| rest.split(')').next().expect("a closing parenthesis"))
            .filter(|items| items.trim_start().starts_with(prefix))
            .map(|items| {
                items
                    .split(',')
                    .map(|item| {
                        item.trim()
                            .strip_prefix(prefix)
                            .unwrap_or_else(|| panic!("`{item}` lacks `{prefix}`"))
                            .to_string()
                    })
                    .collect()
            })
            .collect()
    }

    #[derive(Debug)]
    enum Selection {
        Field {
            name: String,
            children: Vec<Selection>,
        },
        Spread(String),
        Inline(Vec<Selection>),
    }

    #[derive(Debug, PartialEq)]
    enum Token {
        Name(String),
        Colon,
        Spread,
        Open,
        Close,
    }

    /// Tokenise a GraphQL document, dropping comments, punctuation other than
    /// braces and spreads, and every parenthesised argument or variable list
    /// (whose braces are input objects, not selection sets).
    fn tokens(document: &str) -> Vec<Token> {
        let mut out = Vec::new();
        let chars: Vec<char> = document.chars().collect();
        let mut i = 0;
        let mut parens = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '#' {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if c == '"' {
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    i += 1;
                }
                i += 1;
                continue;
            }
            match c {
                '(' => parens += 1,
                ')' => parens -= 1,
                _ if parens > 0 => {}
                '{' => out.push(Token::Open),
                '}' => out.push(Token::Close),
                // Outside an argument or variable list, a colon follows an
                // alias.
                ':' => out.push(Token::Colon),
                '.' if chars[i..].starts_with(&['.', '.', '.']) => {
                    out.push(Token::Spread);
                    i += 3;
                    continue;
                }
                '@' => {
                    // A directive: skip its name; its arguments are skipped as
                    // a parenthesised list.
                    i += 1;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    continue;
                }
                _ if c.is_alphabetic() || c == '_' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    out.push(Token::Name(chars[start..i].iter().collect()));
                    continue;
                }
                _ => {}
            }
            i += 1;
        }
        assert_eq!(parens, 0, "unbalanced parentheses");
        out
    }

    fn selection_set(tokens: &[Token], position: &mut usize) -> Vec<Selection> {
        assert_eq!(tokens[*position], Token::Open, "a selection set");
        *position += 1;
        let mut out = Vec::new();
        loop {
            match &tokens[*position] {
                Token::Close => {
                    *position += 1;
                    return out;
                }
                Token::Spread => {
                    *position += 1;
                    match &tokens[*position] {
                        Token::Name(on) if on == "on" => {
                            *position += 2; // `on` and the type condition
                            out.push(Selection::Inline(selection_set(tokens, position)));
                        }
                        Token::Name(name) => {
                            *position += 1;
                            out.push(Selection::Spread(name.clone()));
                        }
                        other => panic!("unexpected {other:?} after a spread"),
                    }
                }
                Token::Name(name) => {
                    let mut name = name.clone();
                    *position += 1;
                    if tokens[*position] == Token::Colon {
                        // `alias: field`: the field's name follows the colon.
                        let Token::Name(field) = &tokens[*position + 1] else {
                            panic!("an alias without a field");
                        };
                        name = field.clone();
                        *position += 2;
                    }
                    let children = if tokens[*position] == Token::Open {
                        selection_set(tokens, position)
                    } else {
                        Vec::new()
                    };
                    out.push(Selection::Field { name, children });
                }
                Token::Open | Token::Colon => panic!("unexpected {:?}", tokens[*position]),
            }
        }
    }

    /// Every top-level definition of a document, by name.
    fn definitions(document: &str) -> HashMap<String, Vec<Selection>> {
        let tokens = tokens(document);
        let mut out = HashMap::new();
        let mut position = 0;
        while position < tokens.len() {
            match &tokens[position] {
                Token::Name(keyword) if keyword == "fragment" => {
                    let Token::Name(name) = &tokens[position + 1] else {
                        panic!("a fragment name");
                    };
                    position += 4; // `fragment Name on Type`
                    out.insert(name.clone(), selection_set(&tokens, &mut position));
                }
                Token::Name(keyword) if keyword == "query" => {
                    let Token::Name(name) = &tokens[position + 1] else {
                        panic!("a query name");
                    };
                    position += 2;
                    out.insert(name.clone(), selection_set(&tokens, &mut position));
                }
                other => panic!("unexpected top-level token {other:?}"),
            }
        }
        out
    }

    fn snake_case(name: &str) -> String {
        let mut out = String::new();
        for c in name.chars() {
            if c.is_uppercase() {
                out.push('_');
                out.extend(c.to_lowercase());
            } else {
                out.push(c);
            }
        }
        out
    }

    /// Walk a selection set, following fragment spreads, and collect the
    /// fields selected under every field that returns an institution
    /// (`institution`, or any name ending in `Institution`).
    fn collect_institution_fields(
        selections: &[Selection],
        definitions: &HashMap<String, Vec<Selection>>,
        fragments: &mut BTreeSet<String>,
        fields: &mut BTreeSet<String>,
    ) {
        for selection in selections {
            match selection {
                Selection::Field { name, children } => {
                    if name == "institution" || name.ends_with("Institution") {
                        for child in children {
                            match child {
                                Selection::Field { name, children } if children.is_empty() => {
                                    fields.insert(snake_case(name));
                                }
                                other => panic!(
                                    "an institution selection holds {other:?}; extend this \
                                     test and the trigger's field list"
                                ),
                            }
                        }
                    } else {
                        collect_institution_fields(children, definitions, fragments, fields);
                    }
                }
                Selection::Inline(children) => {
                    collect_institution_fields(children, definitions, fragments, fields)
                }
                Selection::Spread(name) => {
                    if fragments.insert(name.clone()) {
                        let fragment = definitions
                            .get(name)
                            .unwrap_or_else(|| panic!("fragment {name} is not defined"));
                        collect_institution_fields(fragment, definitions, fragments, fields);
                    }
                }
            }
        }
    }

    #[test]
    fn the_trigger_field_list_equals_the_institution_fields_the_exports_select() {
        let (_guard, pool) = setup_test_db();
        let mut connection = pool.get().expect("connection");
        let definition = text(
            &mut connection,
            "SELECT pg_get_functiondef(\
             'institution_work_updated_at_with_relations()'::regprocedure) AS value",
        );
        let new_rows = row_fields(&definition, "NEW.");
        let old_rows = row_fields(&definition, "OLD.");
        assert_eq!(new_rows.len(), 1, "{definition}");
        assert_eq!(new_rows, old_rows, "NEW and OLD compare the same fields");
        let trigger_fields = new_rows.into_iter().next().unwrap();

        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../thoth-client/assets/queries.graphql");
        let document = std::fs::read_to_string(&path).expect("the export documents");
        let definitions = definitions(&document);
        let mut fragments = BTreeSet::new();
        let mut exported = BTreeSet::new();
        for query in ["WorkQuery", "WorksQuery"] {
            let selections = definitions
                .get(query)
                .unwrap_or_else(|| panic!("{query} is not defined"));
            collect_institution_fields(selections, &definitions, &mut fragments, &mut exported);
        }

        // The fragments the two export queries use, named here so that a new
        // one is noticed.
        assert_eq!(
            fragments,
            BTreeSet::from(["Funding", "Reference", "Work"].map(String::from))
        );
        assert_eq!(
            exported,
            BTreeSet::from(
                ["country_code", "institution_doi", "institution_name", "ror"].map(String::from)
            )
        );
        assert_eq!(
            trigger_fields, exported,
            "the trigger's field list must equal the institution fields the exports select"
        );
    }

    // ------------------------------------------------------------------
    // History (§4.9)
    // ------------------------------------------------------------------

    #[test]
    fn create_and_update_by_the_sync_each_add_one_history_row_under_its_user_id() {
        let (_guard, pool) = setup_test_db();
        let sync = ctx(&pool, Some(ror_sync_user("history-sync")));

        let created =
            Institution::create_with_history(&sync, &new_institution("History", Some(ROR_A)))
                .expect("create");
        let rows = history(pool.as_ref(), created.institution_id);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].user_id, "history-sync");
        assert_eq!(
            history_data(&rows[0]),
            created,
            "a create holds the created row"
        );

        let mut patch = renamed(&created, "updated");
        patch.ror_status = Some(RorStatus::Inactive);
        let updated = created.update(&sync, &patch).expect("update");
        let rows = history(pool.as_ref(), created.institution_id);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].user_id, "history-sync");
        assert_eq!(
            history_data(&rows[1]),
            created,
            "an update holds the previous row"
        );
        assert_eq!(updated.ror_status, RorStatus::Inactive);
    }

    #[test]
    fn an_update_records_the_row_re_read_under_lock_not_the_loaded_one() {
        let (_guard, pool) = setup_test_db();
        let row = seed(pool.as_ref(), "Re-read", Some(ROR_A));
        let loaded = stored(pool.as_ref(), row.institution_id);
        let changed = set_ror_state(pool.as_ref(), row.institution_id, RorStatus::Inactive, None);
        let user = ctx(&pool, Some(user_with_project_roles("re-read", None)));
        loaded
            .update(&user, &renamed(&loaded, "edited"))
            .expect("update");
        let rows = history(pool.as_ref(), row.institution_id);
        assert_eq!(rows.len(), 1);
        assert_eq!(history_data(&rows[0]), changed);
    }

    #[test]
    fn failed_creates_and_updates_add_no_history() {
        let (_guard, pool) = setup_test_db();
        let sync = ctx(&pool, Some(ror_sync_user("history-failures")));
        let before = (total_institutions(&pool), total_history(&pool));

        // `institution_institution_name_check`: the insert fails.
        let failed = Institution::create_with_history(&sync, &new_institution("", None));
        assert!(matches!(
            failed,
            Err(ThothError::DatabaseConstraintError(_))
        ));
        assert_eq!((total_institutions(&pool), total_history(&pool)), before);

        let row = Institution::create_with_history(&sync, &new_institution("Fails", None))
            .expect("create");
        let after_create = total_history(&pool);
        let mut patch = patch_from(&row);
        patch.ror_status = Some(RorStatus::Withdrawn);
        assert_constraint_error(row.update(&sync, &patch), STATE_CHECK_MESSAGE);
        assert_eq!(total_history(&pool), after_create);
        assert_eq!(stored(pool.as_ref(), row.institution_id), row);
    }

    #[test]
    fn the_unit_test_create_writes_no_history_and_the_guard_names_create_with_history() {
        let (_guard, pool) = setup_test_db();
        let row = seed(pool.as_ref(), "Fixture", None);
        assert!(history(pool.as_ref(), row.institution_id).is_empty());
        assert!(CREATE_WITHOUT_HISTORY_ERROR.contains("create_with_history"));
    }

    // ------------------------------------------------------------------
    // The migration (§4.1, §4.2)
    // ------------------------------------------------------------------

    fn applied(connection: &mut PgConnection) -> Vec<String> {
        connection
            .applied_migrations()
            .expect("applied migrations")
            .iter()
            .map(|version| version.to_string())
            .collect()
    }

    /// Revert every migration newer than C's, leaving C's applied.
    fn revert_newer_than_c(connection: &mut PgConnection) {
        while applied(connection)
            .iter()
            .max()
            .is_some_and(|newest| newest.as_str() > C_MIGRATION_VERSION)
        {
            connection
                .revert_last_migration(MIGRATIONS)
                .expect("revert a newer migration");
        }
        assert!(applied(connection).contains(&C_MIGRATION_VERSION.to_string()));
    }

    fn revert_through_c(connection: &mut PgConnection) {
        revert_newer_than_c(connection);
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("revert C's migration");
        assert_eq!(reverted.to_string(), C_MIGRATION_VERSION);
    }

    fn assert_c_absent(connection: &mut PgConnection) {
        assert_eq!(
            count(
                connection,
                "SELECT count(*) AS n FROM information_schema.columns \
                 WHERE table_name = 'institution' AND column_name IN ('ror_status', 'ror_successor')"
            ),
            0
        );
        assert_eq!(
            count(
                connection,
                "SELECT count(*) AS n FROM pg_type WHERE typname = 'ror_status'"
            ),
            0
        );
        assert_eq!(
            count(
                connection,
                "SELECT count(*) AS n FROM pg_constraint WHERE conname IN (\
                 'institution_ror_successor_check', 'institution_ror_state_check', \
                 'institution_ror_successor_self_check', 'institution_ror_active_successor_check')"
            ),
            0
        );
        let body = text(
            connection,
            "SELECT pg_get_functiondef(\
             'institution_work_updated_at_with_relations()'::regprocedure) AS value",
        );
        assert!(body.contains("NEW IS DISTINCT FROM OLD"), "{body}");
        assert!(!body.contains("ROW("), "{body}");
    }

    fn assert_c_present(connection: &mut PgConnection) {
        assert_eq!(
            text(
                connection,
                "SELECT string_agg(column_name || ':' || data_type || ':' || is_nullable || ':' \
                 || coalesce(column_default, ''), ',' ORDER BY ordinal_position) AS value \
                 FROM information_schema.columns WHERE table_name = 'institution' \
                 AND column_name IN ('ror_status', 'ror_successor')"
            ),
            "ror_status:USER-DEFINED:NO:'active'::ror_status,ror_successor:text:YES:"
        );
        assert_eq!(
            text(
                connection,
                "SELECT string_agg(enumlabel, ',' ORDER BY enumsortorder) AS value \
                 FROM pg_enum WHERE enumtypid = 'ror_status'::regtype"
            ),
            "active,inactive,withdrawn"
        );
        assert_eq!(
            count(
                connection,
                "SELECT count(*) AS n FROM pg_constraint WHERE conrelid = 'institution'::regclass \
                 AND convalidated AND conname IN (\
                 'institution_ror_successor_check', 'institution_ror_state_check', \
                 'institution_ror_successor_self_check', 'institution_ror_active_successor_check')"
            ),
            4
        );
        let body = text(
            connection,
            "SELECT pg_get_functiondef(\
             'institution_work_updated_at_with_relations()'::regprocedure) AS value",
        );
        assert!(
            body.contains(
                "ROW(NEW.institution_name, NEW.institution_doi, NEW.ror, NEW.country_code)"
            ),
            "{body}"
        );
    }

    #[test]
    fn the_migration_applies_to_empty_and_populated_tables_and_reverts() {
        let (_guard, pool) = setup_test_db();
        // Fixtures that do not touch `institution`, created before the revert.
        let publisher = create_publisher(pool.as_ref());
        let imprint = create_imprint(pool.as_ref(), &publisher);
        let work = create_work(pool.as_ref(), &imprint);
        drop(pool);

        let mut connection = connect();

        // Down, then up, on an empty table.
        revert_through_c(&mut connection);
        assert_c_absent(&mut connection);
        connection
            .run_pending_migrations(MIGRATIONS)
            .expect("up on an empty table");
        assert_c_present(&mut connection);

        // Down, then up, on a populated table, with a linked work.
        revert_through_c(&mut connection);
        connection
            .batch_execute(&format!(
                "INSERT INTO institution (institution_name, ror, institution_doi) VALUES \
                 ('Populated one', '{ROR_A}', NULL), \
                 ('Populated two', NULL, 'https://doi.org/10.13039/100000002'), \
                 ('Populated three', '{ROR_B}', NULL); \
                 INSERT INTO funding (work_id, institution_id) \
                 SELECT '{}', institution_id FROM institution \
                 WHERE institution_name = 'Populated one'; \
                 UPDATE work SET updated_at_with_relations = '2000-01-01T00:00:00Z' \
                 WHERE work_id = '{}';",
                work.work_id, work.work_id
            ))
            .expect("populate");
        connection
            .run_pending_migrations(MIGRATIONS)
            .expect("up on a populated table");
        assert_c_present(&mut connection);
        assert_eq!(
            count(
                &mut connection,
                "SELECT count(*) AS n FROM institution \
                 WHERE ror_status = 'active' AND ror_successor IS NULL"
            ),
            3
        );
        assert!(
            flag(
                &mut connection,
                &format!(
                    "SELECT updated_at_with_relations = '2000-01-01T00:00:00Z' AS flag \
                     FROM work WHERE work_id = '{}'",
                    work.work_id
                )
            ),
            "the migration fires no works bump"
        );
    }

    #[test]
    fn the_manual_rollback_pair_then_the_runner_restores_the_schema() {
        let (_guard, pool) = setup_test_db();
        drop(pool);
        let mut connection = connect();
        revert_newer_than_c(&mut connection);

        connection
            .transaction::<_, diesel::result::Error, _>(|connection| {
                connection.batch_execute(C_DOWN_SQL)?;
                diesel::sql_query(format!(
                    "DELETE FROM __diesel_schema_migrations WHERE version = '{C_MIGRATION_VERSION}'"
                ))
                .execute(connection)?;
                Ok(())
            })
            .expect("the manual rollback pair");
        assert!(!applied(&mut connection).contains(&C_MIGRATION_VERSION.to_string()));
        assert_c_absent(&mut connection);

        connection
            .run_pending_migrations(MIGRATIONS)
            .expect("the forward runner");
        assert!(applied(&mut connection).contains(&C_MIGRATION_VERSION.to_string()));
        assert_c_present(&mut connection);
    }
}
