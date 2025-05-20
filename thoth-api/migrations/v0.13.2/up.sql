-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create locale enum type
CREATE TYPE locale_code AS ENUM (
    'AF', 'AF_NA', 'AF_ZA', 'AGQ', 'AGQ_CM', 'AK', 'AK_GH', 'SQ', 'SQ_AL', 'AM', 'AM_ET',
    'AR', 'AR_DZ', 'AR_BH', 'AR_EG', 'AR_IQ', 'AR_JO', 'AR_KW', 'AR_LB', 'AR_LY', 'AR_MA',
    'AR_OM', 'AR_QA', 'AR_SA', 'AR_SD', 'AR_SY', 'AR_TN', 'AR_AE', 'AR_001', 'AR_YE',
    'HY', 'HY_AM', 'AS', 'AS_IN', 'AST', 'AST_ES', 'ASA', 'ASA_TZ', 'AZ', 'AZ_CYRL',
    'AZ_CYRL_AZ', 'AZ_LATN', 'AZ_LATN_AZ', 'KSF', 'KSF_CM', 'BM', 'BM_ML', 'BAS', 'BAS_CM',
    'EU', 'EU_ES', 'BE', 'BE_BY', 'BEM', 'BEM_ZM', 'BEZ', 'BEZ_TZ', 'BN', 'BN_BD', 'BN_IN',
    'BRX', 'BRX_IN', 'BS', 'BS_BA', 'BR', 'BR_FR', 'BG', 'BG_BG', 'MY', 'MY_MM', 'CA',
    'CA_ES', 'CKB', 'KMR', 'SDH', 'TZM', 'TZM_LATN', 'TZM_LATN_MA', 'CHR', 'CHR_US', 'CGG',
    'CGG_UG', 'ZH', 'ZH_HANS', 'ZH_CN', 'ZH_HANS_CN', 'ZH_HANS_HK', 'ZH_HANS_MO', 'ZH_HANS_SG',
    'ZH_HANT', 'ZH_HANT_HK', 'ZH_HANT_MO', 'ZH_HANT_TW', 'SWC', 'SWC_CD', 'KW', 'KW_GB',
    'HR', 'HR_HR', 'CS', 'CS_CZ', 'DA', 'DA_DK', 'DUA', 'DUA_CM', 'DV', 'NL', 'NL_AW',
    'NL_BE', 'NL_CW', 'NL_NL', 'NL_SX', 'EBU', 'EBU_KE', 'EN', 'EN_AI', 'EN_AS', 'EN_AU',
    'EN_AT', 'EN_BB', 'EN_BE', 'EN_BZ', 'EN_BM', 'EN_BW', 'EN_IO', 'EN_BI', 'EN_CM', 'EN_CA',
    'EN_KY', 'EN_CX', 'EN_CC', 'EN_CK', 'EN_CY', 'EN_DK', 'EN_DG', 'EN_DM', 'EN_EG', 'EN_ER',
    'EN_EU', 'EN_FK', 'EN_FJ', 'EN_FI', 'EN_GM', 'EN_DE', 'EN_GH', 'EN_GI', 'EN_GD', 'EN_GU',
    'EN_GG', 'EN_GY', 'EN_HK', 'EN_IN', 'EN_IE', 'EN_IM', 'EN_IL', 'EN_JM', 'EN_JE', 'EN_KE',
    'EN_KI', 'EN_KW', 'EN_LS', 'EN_MO', 'EN_MG', 'EN_MW', 'EN_MY', 'EN_MT', 'EN_MH', 'EN_MU',
    'EN_FM', 'EN_MS', 'EN_NA', 'EN_NR', 'EN_NL', 'EN_NZ', 'EN_NG', 'EN_NU', 'EN_NF', 'EN_MP',
    'EN_NO', 'EN_PA', 'EN_PK', 'EN_PW', 'EN_PG', 'EN_PH', 'EN_PN', 'EN_PR', 'EN_RW', 'EN_WS',
    'EN_SA', 'EN_SC', 'EN_SL', 'EN_SG', 'EN_SX', 'EN_SI', 'EN_SB', 'EN_SS', 'EN_SH', 'EN_KN',
    'EN_LC', 'EN_SD', 'EN_SZ', 'EN_SE', 'EN_CH', 'EN_TZ', 'EN_TK', 'EN_TO', 'EN_TT', 'EN_TV',
    'EN_ZA', 'EN_AE', 'EN_UM', 'EN_VI', 'EN_US_POSIX', 'EN_UG', 'EN_GB', 'EN_US', 'EN_VU',
    'EN_ZM', 'EN_ZW', 'EO', 'ET', 'ET_EE', 'EE', 'EE_GH', 'EE_TG', 'EWO', 'EWO_CM', 'FO',
    'FO_FO', 'FIL', 'FIL_PH', 'FI', 'FI_FI', 'FR', 'FR_BE', 'FR_BJ', 'FR_BF', 'FR_BI',
    'FR_CM', 'FR_CA', 'FR_CF', 'FR_TD', 'FR_KM', 'FR_CG', 'FR_CD', 'FR_CI', 'FR_DJ', 'FR_GQ',
    'FR_FR', 'FR_GF', 'FR_GA', 'FR_GP', 'FR_GN', 'FR_LU', 'FR_MG', 'FR_ML', 'FR_MQ', 'FR_YT',
    'FR_MC', 'FR_NE', 'FR_RW', 'FR_RE', 'FR_BL', 'FR_MF', 'FR_MU', 'FR_SN', 'FR_CH', 'FR_TG',
    'FF', 'FF_SN', 'GL', 'GL_ES', 'LG', 'LG_UG', 'KA', 'KA_GE', 'DE', 'DE_AT', 'DE_BE',
    'DE_DE', 'DE_LI', 'DE_LU', 'DE_CH', 'EL', 'EL_CY', 'EL_GR', 'GU', 'GU_IN', 'GUZ',
    'GUZ_KE', 'HA', 'HA_LATN', 'HA_LATN_GH', 'HA_LATN_NE', 'HA_LATN_NG', 'HAW', 'HAW_US',
    'HE', 'HE_IL', 'HI', 'HI_IN', 'HU', 'HU_HU', 'IS', 'IS_IS', 'IG', 'IG_NG', 'ID', 'ID_ID',
    'GA', 'GA_IE', 'IT', 'IT_IT', 'IT_CH', 'JA', 'JA_JP', 'DYO', 'DYO_SN', 'KEA', 'KEA_CV',
    'KAB', 'KAB_DZ', 'KL', 'KL_GL', 'KLN', 'KLN_KE', 'KAM', 'KAM_KE', 'KN', 'KN_IN', 'KK',
    'KK_CYRL', 'KK_CYRL_KZ', 'KM', 'KM_KH', 'KI', 'KI_KE', 'RW', 'RW_RW', 'KOK', 'KOK_IN',
    'KO', 'KO_KR', 'KHQ', 'KHQ_ML', 'SES', 'SES_ML', 'NMG', 'NMG_CM', 'KY', 'LAG', 'LAG_TZ',
    'LV', 'LV_LV', 'LN', 'LN_CG', 'LN_CD', 'LT', 'LT_LT', 'LU', 'LU_CD', 'LUO', 'LUO_KE',
    'LUY', 'LUY_KE', 'MK', 'MK_MK', 'JMC', 'JMC_TZ', 'MGH', 'MGH_MZ', 'KDE', 'KDE_TZ', 'MG',
    'MG_MG', 'MS', 'MS_BN', 'MS_MY', 'ML', 'ML_IN', 'MT', 'MT_MT', 'GV', 'GV_GB', 'MR',
    'MR_IN', 'MAS', 'MAS_KE', 'MAS_TZ', 'MER', 'MER_KE', 'MFE', 'MFE_MU', 'MUA', 'MUA_CM',
    'NAQ', 'NAQ_NA', 'NE', 'NE_IN', 'NE_NP', 'ND', 'ND_ZW', 'NB', 'NB_NO', 'NN', 'NN_NO',
    'NUS', 'NUS_SD', 'NYN', 'NYN_UG', 'OR', 'OR_IN', 'OM', 'OM_ET', 'OM_KE', 'PS', 'PS_AF',
    'FA', 'FA_AF', 'FA_IR', 'PL', 'PL_PL', 'PT', 'PT_AO', 'PT_BR', 'PT_GW', 'PT_MZ', 'PT_PT',
    'PT_ST', 'PA', 'PA_ARAB', 'PA_ARAB_PK', 'PA_GURU', 'PA_GURU_IN', 'RO', 'RO_MD', 'RO_RO',
    'RM', 'RM_CH', 'ROF', 'ROF_TZ', 'RN', 'RN_BI', 'RU', 'RU_MD', 'RU_RU', 'RU_UA', 'RWK',
    'RWK_TZ', 'SAQ', 'SAQ_KE', 'SG', 'SG_CF', 'SBP', 'SBP_TZ', 'SA', 'GD', 'GD_GB', 'SEH',
    'SEH_MZ', 'SR', 'SR_CYRL', 'SR_CYRL_BA', 'SR_CYRL_ME', 'SR_CYRL_RS', 'SR_LATN', 'SR_LATN_BA',
    'SR_LATN_ME', 'SR_LATN_RS', 'KSB', 'KSB_TZ', 'SN', 'SN_ZW', 'II', 'II_CN', 'SI', 'SI_LK',
    'SK', 'SK_SK', 'SL', 'SL_SI', 'XOG', 'XOG_UG', 'SO', 'SO_DJ', 'SO_ET', 'SO_KE', 'SO_SO',
    'ES', 'ES_AR', 'ES_BO', 'ES_CL', 'ES_CO', 'ES_CR', 'ES_DO', 'ES_EC', 'ES_SV', 'ES_GQ',
    'ES_GT', 'ES_HN', 'ES_419', 'ES_MX', 'ES_NI', 'ES_PA', 'ES_PY', 'ES_PE', 'ES_PR', 'ES_ES',
    'ES_US', 'ES_UY', 'ES_VE', 'SW', 'SW_KE', 'SW_TZ', 'SV', 'SV_FI', 'SV_SE', 'GSW', 'GSW_CH',
    'SHI', 'SHI_LATN', 'SHI_LATN_MA', 'SHI_TFNG', 'SHI_TFNG_MA', 'DAV', 'DAV_KE', 'TG', 'TA',
    'TA_IN', 'TA_LK', 'TWQ', 'TWQ_NE', 'TE', 'TE_IN', 'TEO', 'TEO_KE', 'TEO_UG', 'TH', 'TH_TH',
    'BO', 'BO_CN', 'BO_IN', 'TI', 'TI_ER', 'TI_ET', 'TO', 'TO_TO', 'TR', 'TR_TR', 'TK', 'UK',
    'UK_UA', 'UR', 'UR_IN', 'UR_PK', 'UG', 'UG_CN', 'UZ', 'UZ_ARAB', 'UZ_ARAB_AF', 'UZ_CYRL',
    'UZ_CYRL_UZ', 'UZ_LATN', 'UZ_LATN_UZ', 'VAI', 'VAI_LATN', 'VAI_LATN_LR', 'VAI_VAII',
    'VAI_VAII_LR', 'VAL', 'VAL_ES', 'VI', 'VI_VN', 'VUN', 'VUN_TZ', 'CY', 'CY_GB', 'WO', 'XH',
    'YAV', 'YAV_CM', 'YO', 'YO_NG', 'DJE', 'DJE_NE', 'ZU', 'ZU_ZA'
);

-- Commented out locale table creation and data insertion
/*
CREATE TABLE locale (
    locale_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code TEXT NOT NULL UNIQUE CHECK (octet_length(code) >= 1),
    name TEXT NOT NULL CHECK (octet_length(name) >= 1)
);

-- Populate locale table with JSON data and English
INSERT INTO locale (locale_id, code, name)
VALUES
    (uuid_generate_v4(), 'af', 'Afrikaans (af)'),
    (uuid_generate_v4(), 'af-NA', 'Afrikaans (Namibia) (af-NA)'),
    (uuid_generate_v4(), 'af-ZA', 'Afrikaans (South Africa) (af-ZA)'),
    -- ... rest of the locale data ...
ON CONFLICT (code) DO NOTHING;
*/

-- Create the title table
CREATE TABLE title (
    title_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id UUID NOT NULL REFERENCES work (work_id) ON DELETE CASCADE,
    locale_code locale_code NOT NULL,
    full_title TEXT NOT NULL CHECK (octet_length(full_title) >= 1),
    title TEXT NOT NULL CHECK (octet_length(title) >= 1),
    subtitle TEXT CHECK (octet_length(subtitle) >= 1),
    canonical BOOLEAN DEFAULT FALSE
);

-- Migrate existing work titles to the title table with English locale
INSERT INTO title (title_id, work_id, locale_code, full_title, title, subtitle, canonical)
SELECT
    uuid_generate_v4(),
    work_id,
    'EN'::locale_code,
    full_title,
    title,
    subtitle,
    TRUE
FROM work
WHERE full_title IS NOT NULL
    AND title IS NOT NULL;

-- Drop title-related columns from the work table
ALTER TABLE work
    DROP COLUMN full_title,
    DROP COLUMN title,
    DROP COLUMN subtitle;
