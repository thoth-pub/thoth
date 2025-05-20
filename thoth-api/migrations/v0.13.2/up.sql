-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create locale enum type
CREATE TYPE locale_code AS ENUM (
    'af', 'af_NA', 'af_ZA', 'agq', 'agq_CM', 'ak', 'ak_GH', 'sq', 'sq_AL', 'am', 'am_ET',
    'ar', 'ar_DZ', 'ar_BH', 'ar_EG', 'ar_IQ', 'ar_JO', 'ar_KW', 'ar_LB', 'ar_LY', 'ar_MA',
    'ar_OM', 'ar_QA', 'ar_SA', 'ar_SD', 'ar_SY', 'ar_TN', 'ar_AE', 'ar_001', 'ar_YE',
    'hy', 'hy_AM', 'as', 'as_IN', 'ast', 'ast_ES', 'asa', 'asa_TZ', 'az', 'az_Cyrl',
    'az_Cyrl_AZ', 'az_Latn', 'az_Latn_AZ', 'ksf', 'ksf_CM', 'bm', 'bm_ML', 'bas', 'bas_CM',
    'eu', 'eu_ES', 'be', 'be_BY', 'bem', 'bem_ZM', 'bez', 'bez_TZ', 'bn', 'bn_BD', 'bn_IN',
    'brx', 'brx_IN', 'bs', 'bs_BA', 'br', 'br_FR', 'bg', 'bg_BG', 'my', 'my_MM', 'ca',
    'ca_ES', 'ckb', 'kmr', 'sdh', 'tzm', 'tzm_Latn', 'tzm_Latn_MA', 'chr', 'chr_US', 'cgg',
    'cgg_UG', 'zh', 'zh_Hans', 'zh_CN', 'zh_Hans_CN', 'zh_Hans_HK', 'zh_Hans_MO', 'zh_Hans_SG',
    'zh_Hant', 'zh_Hant_HK', 'zh_Hant_MO', 'zh_Hant_TW', 'swc', 'swc_CD', 'kw', 'kw_GB',
    'hr', 'hr_HR', 'cs', 'cs_CZ', 'da', 'da_DK', 'dua', 'dua_CM', 'dv', 'nl', 'nl_AW',
    'nl_BE', 'nl_CW', 'nl_NL', 'nl_SX', 'ebu', 'ebu_KE', 'en', 'en_AI', 'en_AS', 'en_AU',
    'en_AT', 'en_BB', 'en_BE', 'en_BZ', 'en_BM', 'en_BW', 'en_IO', 'en_BI', 'en_CM', 'en_CA',
    'en_KY', 'en_CX', 'en_CC', 'en_CK', 'en_CY', 'en_DK', 'en_DG', 'en_DM', 'en_EG', 'en_ER',
    'en_EU', 'en_FK', 'en_FJ', 'en_FI', 'en_GM', 'en_DE', 'en_GH', 'en_GI', 'en_GD', 'en_GU',
    'en_GG', 'en_GY', 'en_HK', 'en_IN', 'en_IE', 'en_IM', 'en_IL', 'en_JM', 'en_JE', 'en_KE',
    'en_KI', 'en_KW', 'en_LS', 'en_MO', 'en_MG', 'en_MW', 'en_MY', 'en_MT', 'en_MH', 'en_MU',
    'en_FM', 'en_MS', 'en_NA', 'en_NR', 'en_NL', 'en_NZ', 'en_NG', 'en_NU', 'en_NF', 'en_MP',
    'en_NO', 'en_PA', 'en_PK', 'en_PW', 'en_PG', 'en_PH', 'en_PN', 'en_PR', 'en_RW', 'en_WS',
    'en_SA', 'en_SC', 'en_SL', 'en_SG', 'en_SX', 'en_SI', 'en_SB', 'en_SS', 'en_SH', 'en_KN',
    'en_LC', 'en_SD', 'en_SZ', 'en_SE', 'en_CH', 'en_TZ', 'en_TK', 'en_TO', 'en_TT', 'en_TV',
    'en_ZA', 'en_AE', 'en_UM', 'en_VI', 'en_US_POSIX', 'en_UG', 'en_GB', 'en_US', 'en_VU',
    'en_ZM', 'en_ZW', 'eo', 'et', 'et_EE', 'ee', 'ee_GH', 'ee_TG', 'ewo', 'ewo_CM', 'fo',
    'fo_FO', 'fil', 'fil_PH', 'fi', 'fi_FI', 'fr', 'fr_BE', 'fr_BJ', 'fr_BF', 'fr_BI',
    'fr_CM', 'fr_CA', 'fr_CF', 'fr_TD', 'fr_KM', 'fr_CG', 'fr_CD', 'fr_CI', 'fr_DJ', 'fr_GQ',
    'fr_FR', 'fr_GF', 'fr_GA', 'fr_GP', 'fr_GN', 'fr_LU', 'fr_MG', 'fr_ML', 'fr_MQ', 'fr_YT',
    'fr_MC', 'fr_NE', 'fr_RW', 'fr_RE', 'fr_BL', 'fr_MF', 'fr_MU', 'fr_SN', 'fr_CH', 'fr_TG',
    'ff', 'ff_SN', 'gl', 'gl_ES', 'lg', 'lg_UG', 'ka', 'ka_GE', 'de', 'de_AT', 'de_BE',
    'de_DE', 'de_LI', 'de_LU', 'de_CH', 'el', 'el_CY', 'el_GR', 'gu', 'gu_IN', 'guz',
    'guz_KE', 'ha', 'ha_Latn', 'ha_Latn_GH', 'ha_Latn_NE', 'ha_Latn_NG', 'haw', 'haw_US',
    'he', 'he_IL', 'hi', 'hi_IN', 'hu', 'hu_HU', 'is', 'is_IS', 'ig', 'ig_NG', 'id', 'id_ID',
    'ga', 'ga_IE', 'it', 'it_IT', 'it_CH', 'ja', 'ja_JP', 'dyo', 'dyo_SN', 'kea', 'kea_CV',
    'kab', 'kab_DZ', 'kl', 'kl_GL', 'kln', 'kln_KE', 'kam', 'kam_KE', 'kn', 'kn_IN', 'kk',
    'kk_Cyrl', 'kk_Cyrl_KZ', 'km', 'km_KH', 'ki', 'ki_KE', 'rw', 'rw_RW', 'kok', 'kok_IN',
    'ko', 'ko_KR', 'khq', 'khq_ML', 'ses', 'ses_ML', 'nmg', 'nmg_CM', 'ky', 'lag', 'lag_TZ',
    'lv', 'lv_LV', 'ln', 'ln_CG', 'ln_CD', 'lt', 'lt_LT', 'lu', 'lu_CD', 'luo', 'luo_KE',
    'luy', 'luy_KE', 'mk', 'mk_MK', 'jmc', 'jmc_TZ', 'mgh', 'mgh_MZ', 'kde', 'kde_TZ', 'mg',
    'mg_MG', 'ms', 'ms_BN', 'ms_MY', 'ml', 'ml_IN', 'mt', 'mt_MT', 'gv', 'gv_GB', 'mr',
    'mr_IN', 'mas', 'mas_KE', 'mas_TZ', 'mer', 'mer_KE', 'mfe', 'mfe_MU', 'mua', 'mua_CM',
    'naq', 'naq_NA', 'ne', 'ne_IN', 'ne_NP', 'nd', 'nd_ZW', 'nb', 'nb_NO', 'nn', 'nn_NO',
    'nus', 'nus_SD', 'nyn', 'nyn_UG', 'or', 'or_IN', 'om', 'om_ET', 'om_KE', 'ps', 'ps_AF',
    'fa', 'fa_AF', 'fa_IR', 'pl', 'pl_PL', 'pt', 'pt_AO', 'pt_BR', 'pt_GW', 'pt_MZ', 'pt_PT',
    'pt_ST', 'pa', 'pa_Arab', 'pa_Arab_PK', 'pa_Guru', 'pa_Guru_IN', 'ro', 'ro_MD', 'ro_RO',
    'rm', 'rm_CH', 'rof', 'rof_TZ', 'rn', 'rn_BI', 'ru', 'ru_MD', 'ru_RU', 'ru_UA', 'rwk',
    'rwk_TZ', 'saq', 'saq_KE', 'sg', 'sg_CF', 'sbp', 'sbp_TZ', 'sa', 'gd', 'gd_GB', 'seh',
    'seh_MZ', 'sr', 'sr_Cyrl', 'sr_Cyrl_BA', 'sr_Cyrl_ME', 'sr_Cyrl_RS', 'sr_Latn', 'sr_Latn_BA',
    'sr_Latn_ME', 'sr_Latn_RS', 'ksb', 'ksb_TZ', 'sn', 'sn_ZW', 'ii', 'ii_CN', 'si', 'si_LK',
    'sk', 'sk_SK', 'sl', 'sl_SI', 'xog', 'xog_UG', 'so', 'so_DJ', 'so_ET', 'so_KE', 'so_SO',
    'es', 'es_AR', 'es_BO', 'es_CL', 'es_CO', 'es_CR', 'es_DO', 'es_EC', 'es_SV', 'es_GQ',
    'es_GT', 'es_HN', 'es_419', 'es_MX', 'es_NI', 'es_PA', 'es_PY', 'es_PE', 'es_PR', 'es_ES',
    'es_US', 'es_UY', 'es_VE', 'sw', 'sw_KE', 'sw_TZ', 'sv', 'sv_FI', 'sv_SE', 'gsw', 'gsw_CH',
    'shi', 'shi_Latn', 'shi_Latn_MA', 'shi_Tfng', 'shi_Tfng_MA', 'dav', 'dav_KE', 'tg', 'ta',
    'ta_IN', 'ta_LK', 'twq', 'twq_NE', 'te', 'te_IN', 'teo', 'teo_KE', 'teo_UG', 'th', 'th_TH',
    'bo', 'bo_CN', 'bo_IN', 'ti', 'ti_ER', 'ti_ET', 'to', 'to_TO', 'tr', 'tr_TR', 'tk', 'uk',
    'uk_UA', 'ur', 'ur_IN', 'ur_PK', 'ug', 'ug_CN', 'uz', 'uz_Arab', 'uz_Arab_AF', 'uz_Cyrl',
    'uz_Cyrl_UZ', 'uz_Latn', 'uz_Latn_UZ', 'vai', 'vai_Latn', 'vai_Latn_LR', 'vai_Vaii',
    'vai_Vaii_LR', 'val', 'val_ES', 'vi', 'vi_VN', 'vun', 'vun_TZ', 'cy', 'cy_GB', 'wo', 'xh',
    'yav', 'yav_CM', 'yo', 'yo_NG', 'dje', 'dje_NE', 'zu', 'zu_ZA'
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
    'en'::locale_code,
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
