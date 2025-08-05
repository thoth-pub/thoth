-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create locale enum type
CREATE TYPE locale_code AS ENUM (
    'af', 'af_na', 'af_za', 'agq', 'agq_cm', 'ak', 'ak_gh', 'sq', 'sq_al', 'am', 'am_et', 'aig',
    'ar', 'ar_dz', 'ar_bh', 'ar_eg', 'ar_iq', 'ar_jo', 'ar_kw', 'ar_lb', 'ar_ly', 'ar_ma', 'ar_om',
    'ar_qa', 'ar_sa', 'ar_sd', 'ar_sy', 'ar_tn', 'ar_ae', 'ar_001', 'ar_ye', 'hy', 'hy_am', 'as',
    'as_in', 'ast', 'ast_es', 'asa', 'asa_tz', 'az', 'az_cyrl', 'az_cyrl_az', 'az_latn', 
    'az_latn_az', 'ksf', 'ksf_cm', 'bah', 'bm', 'bm_ml', 'bas', 'bas_cm', 'eu', 'eu_es', 'be', 'be_by',
    'bem', 'bem_zm', 'bez', 'bez_tz', 'bn', 'bn_bd', 'bn_in', 'brx', 'brx_in', 'bs', 'bs_ba', 'br',
    'br_fr', 'bg', 'bg_bg', 'my', 'my_mm', 'ca', 'ca_es', 'ckb', 'kmr', 'sdh', 'tzm', 'tzm_latn',
    'tzm_latn_ma', 'chr', 'chr_us', 'cgg', 'cgg_ug', 'zh', 'zh_hans', 'zh_cn', 'zh_hans_cn',
    'zh_hans_hk', 'zh_hans_mo', 'zh_hans_sg', 'zh_hant', 'zh_hant_hk', 'zh_hant_mo', 'zh_hant_tw',
    'swc', 'swc_cd', 'kw', 'kw_gb', 'hr', 'hr_hr', 'cs', 'cs_cz', 'da', 'da_dk', 'dua', 'dua_cm',
    'dv', 'nl', 'nl_aw', 'nl_be', 'nl_cw', 'nl_nl', 'nl_sx', 'ebu', 'ebu_ke', 'en', 'en_ai',
    'en_as', 'en_au', 'en_at', 'en_bb', 'en_be', 'en_bz', 'en_bm', 'en_bw', 'en_io', 'en_bi', 'en_cm', 
    'en_ca', 'en_ky', 'en_cx', 'en_cc', 'en_ck', 'en_cy', 'en_dk', 'en_dg', 'en_dm', 'en_eg', 'en_er', 
    'en_eu', 'en_fk', 'en_fj', 'en_fi', 'en_gm', 'en_de', 'en_gh', 'en_gi', 'en_gd', 'en_gu', 'en_gg', 
    'en_gy', 'en_hk', 'en_in', 'en_ie', 'en_im', 'en_il', 'en_jm', 'en_je', 'en_ke', 'en_ki', 'en_kw', 
    'en_ls', 'en_mo', 'en_mg', 'en_mw', 'en_my', 'en_mt', 'en_mh', 'en_mu', 'en_fm', 'en_ms', 'en_na', 
    'en_nr', 'en_nl', 'en_nz', 'en_ng', 'en_nu', 'en_nf', 'en_mp', 'en_no', 'en_pa', 'en_pk', 'en_pw', 
    'en_pg', 'en_ph', 'en_pn', 'en_pr', 'en_rw', 'en_ws', 'en_sa', 'en_sc', 'en_sl', 'en_sg', 'en_sx', 
    'en_si', 'en_sb', 'en_ss', 'en_sh', 'en_kn', 'en_lc', 'svc', 'vic', 'en_sd', 'en_sz', 'en_se', 
    'en_ch', 'en_tz', 'en_tk', 'en_to', 'en_tt', 'en_tv', 'en_za', 'en_ae', 'en_um', 'en_vi', 
    'en_us_posix', 'en_ug', 'en_gb', 'en_us', 'en_vu', 'en_zm', 'en_zw', 'eo', 'et', 'et_ee', 
    'ee', 'ee_gh', 'ee_tg', 'ewo', 'ewo_cm', 'fo', 'fo_fo', 'fil', 'fil_ph', 'fi', 'fi_fi', 'fr', 
    'fr_be', 'fr_bj', 'fr_bf', 'fr_bi', 'fr_cm', 'fr_ca', 'fr_cf', 'fr_td', 'fr_km', 'fr_cg', 'fr_cd', 
    'fr_ci', 'fr_dj', 'fr_gq', 'fr_fr', 'fr_gf', 'fr_ga', 'fr_gp', 'fr_gn', 'fr_lu', 'fr_mg', 'fr_ml', 
    'fr_mq', 'fr_yt', 'fr_mc', 'fr_ne', 'fr_rw', 'fr_re', 'fr_bl', 'fr_mf', 'fr_mu', 'fr_sn', 'fr_ch', 
    'fr_tg', 'ff', 'ff_sn', 'gl', 'gl_es', 'lao', 'lg', 'lg_ug', 'ka', 'ka_ge', 'de', 'de_at', 'de_be', 
    'de_de', 'de_li', 'de_lu', 'de_ch', 'el', 'el_cy', 'el_gr', 'gu', 'gu_in', 'guz', 'guz_ke', 'ha', 
    'ha_latn', 'ha_latn_gh', 'ha_latn_ne', 'ha_latn_ng', 'haw', 'haw_us', 'he', 'he_il', 'hi', 'hi_in', 
    'hu', 'hu_hu', 'is', 'is_is', 'ig', 'ig_ng', 'smn', 'smn_fi', 'id', 'id_id', 'ga', 'ga_ie', 'it', 
    'it_it', 'it_ch', 'ja', 'ja_jp', 'dyo', 'dyo_sn', 'kea', 'kea_cv', 'kab', 'kab_dz', 'kl', 'kl_gl', 
    'kln', 'kln_ke', 'kam', 'kam_ke', 'kn', 'kn_in', 'kaa', 'kk', 'kk_cyrl', 'kk_cyrl_kz', 'km', 'km_kh', 
    'ki', 'ki_ke', 'rw', 'rw_rw', 'kok', 'kok_in', 'ko', 'ko_kr', 'khq', 'khq_ml', 'ses', 'ses_ml', 'nmg', 
    'nmg_cm', 'ky', 'lag', 'lag_tz', 'lv', 'lv_lv', 'lir', 'ln', 'ln_cg', 'ln_cd', 'lt', 'lt_lt', 'lu', 
    'lu_cd', 'luo', 'luo_ke', 'luy', 'luy_ke', 'mk', 'mk_mk', 'jmc', 'jmc_tz', 'mgh', 'mgh_mz', 'kde', 
    'kde_tz', 'mg', 'mg_mg', 'ms', 'ms_bn', 'ms_my', 'ml', 'ml_in', 'mt', 'mt_mt', 'gv', 'gv_gb', 'mr', 
    'mr_in', 'mas', 'mas_ke', 'mas_tz', 'mer', 'mer_ke', 'mn', 'mfe', 'mfe_mu', 'mua', 'mua_cm', 'naq', 
    'naq_na', 'ne', 'ne_in', 'ne_np', 'se', 'se_fi', 'se_no', 'se_se', 'nd', 'nd_zw', 'nb', 'nb_no', 'nn', 
    'nn_no', 'nus', 'nus_sd', 'nyn', 'nyn_ug', 'or', 'or_in', 'om', 'om_et', 'om_ke', 'ps', 'ps_af', 'fa', 
    'fa_af', 'fa_ir', 'pl', 'pl_pl', 'pt', 'pt_ao', 'pt_br', 'pt_gw', 'pt_mz', 'pt_pt', 'pt_st', 'pa', 
    'pa_arab', 'pa_arab_pk', 'pa_guru', 'pa_guru_in', 'ro', 'ro_md', 'ro_ro', 'rm', 'rm_ch', 'rof', 
    'rof_tz', 'rn', 'rn_bi', 'ru', 'ru_md', 'ru_ru', 'ru_ua', 'rwk', 'rwk_tz', 'saq', 'saq_ke', 'sg', 
    'sg_cf', 'sbp', 'sbp_tz', 'sa', 'gd', 'gd_gb', 'seh', 'seh_mz', 'sr', 'sr_cyrl', 'sr_cyrl_ba', 
    'sr_cyrl_me', 'sr_cyrl_rs', 'sr_latn', 'sr_latn_ba', 'sr_latn_me', 'sr_latn_rs', 'ksb', 'ksb_tz', 
    'sn', 'sn_zw', 'ii', 'ii_cn', 'si', 'si_lk', 'sk', 'sk_sk', 'sl', 'sl_si', 'xog', 'xog_ug', 'so', 
    'so_dj', 'so_et', 'so_ke', 'so_so', 'es', 'es_ar', 'es_bo', 'es_cl', 'es_co', 'es_cr', 'es_do', 'es_ec', 
    'es_sv', 'es_gq', 'es_gt', 'es_hn', 'es_419', 'es_mx', 'es_ni', 'es_pa', 'es_py', 'es_pe', 'es_pr', 
    'es_es', 'es_us', 'es_uy', 'es_ve', 'sw', 'sw_ke', 'sw_tz', 'sv', 'sv_fi', 'sv_se', 'gsw', 'gsw_ch', 
    'shi', 'shi_latn', 'shi_latn_ma', 'shi_tfng', 'shi_tfng_ma', 'dav', 'dav_ke', 'tg', 'ta', 'ta_in', 
    'ta_lk', 'twq', 'twq_ne', 'mi', 'te', 'te_in', 'teo', 'teo_ke', 'teo_ug', 'th', 'th_th', 'bo', 'bo_cn', 
    'bo_in', 'ti', 'ti_er', 'ti_et', 'to', 'to_to', 'tr', 'tk', 'tr_tr', 'tch', 'uk', 'uk_ua', 'ur', 'ur_in', 
    'ur_pk', 'ug', 'ug_cn', 'uz', 'uz_arab', 'uz_arab_af', 'uz_cyrl', 'uz_cyrl_uz', 'uz_latn', 'uz_latn_uz', 
    'vai', 'vai_latn', 'vai_latn_lr', 'vai_vaii', 'vai_vaii_lr', 'val', 'val_es', 'ca_es_valencia', 'vi', 
    'vi_vn', 'vun', 'vun_tz', 'cy', 'cy_gb', 'wo', 'xh', 'yav', 'yav_cm', 'yo', 'yo_ng', 'dje', 'dje_ne', 
    'zu', 'zu_za'
);

-- Create the title table
CREATE TABLE IF NOT EXISTS title (
    title_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id UUID NOT NULL REFERENCES work (work_id) ON DELETE CASCADE,
    locale_code locale_code NOT NULL,
    full_title TEXT NOT NULL CHECK (octet_length(full_title) >= 1),
    title TEXT NOT NULL CHECK (octet_length(title) >= 1),
    subtitle TEXT CHECK (octet_length(subtitle) >= 1),
    canonical BOOLEAN NOT NULL DEFAULT FALSE
);

-- Create the title_history table
CREATE TABLE IF NOT EXISTS title_history (
    title_history_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title_id UUID NOT NULL REFERENCES title (title_id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES account (account_id) ON DELETE CASCADE,
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Migrate existing work titles to the title table with English locale
INSERT INTO title (title_id, work_id, locale_code, full_title, title, subtitle, canonical)
SELECT
    uuid_generate_v4(),
    work_id,
    'en'::locale_code,
    '<full_title>' || full_title || '</full_title>',
    '<title>' || title || '</title>',
    CASE WHEN subtitle IS NOT NULL THEN '<subtitle>' || subtitle || '</subtitle>' ELSE NULL END,
    TRUE
FROM work
WHERE full_title IS NOT NULL
    AND title IS NOT NULL;

-- Only allow one canonical title per work                                         
CREATE UNIQUE INDEX IF NOT EXISTS title_unique_canonical_true_idx ON title(work_id)
    WHERE canonical;

-- Only allow one instance of each locale per work
CREATE UNIQUE INDEX IF NOT EXISTS title_uniq_locale_idx ON title(work_id, locale_code);

-- Drop title-related columns from the work table
ALTER TABLE work
    DROP COLUMN full_title,
    DROP COLUMN title,
    DROP COLUMN subtitle;

-- Create AbstractType enum
CREATE TYPE abstract_type AS ENUM (
    'short',
    'long'
);

-- Create the abstract table
CREATE TABLE IF NOT EXISTS abstract (
    abstract_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id UUID NOT NULL REFERENCES work (work_id) ON DELETE CASCADE,
    content TEXT NOT NULL CHECK (octet_length(content) >= 1),
    locale_code locale_code NOT NULL,
    abstract_type abstract_type NOT NULL DEFAULT 'short',
    canonical BOOLEAN NOT NULL DEFAULT FALSE
);

-- -----------------------------------------------------------------------------
-- Conversion Function
-- -----------------------------------------------------------------------------
-- This function attempts to detect the format of the input text (HTML, Markdown,
-- or Plaintext) and converts it into a basic JATS XML structure.
-- NOTE: This function uses heuristics and regular expressions for conversion. It
-- covers common cases but is not a full-fledged parser. It is designed to be
-- sufficient for this one-time data migration.
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION convert_to_jats(content_in TEXT)
RETURNS TEXT AS $$
DECLARE
    processed_content TEXT := content_in;
BEGIN
    -- Return NULL immediately if input is NULL or empty.
    IF processed_content IS NULL OR processed_content = '' THEN
        RETURN NULL;
    END IF;

    -- The CASE statement detects the format and applies conversion rules.
    CASE
        -- A) HTML Detection: Looks for common HTML tags. Now includes <sc>.
        WHEN processed_content ~* '<(p|em|i|strong|b|sup|sub|sc|code|a|br)\b' THEN
            -- Convert HTML tags to their JATS equivalents.
            processed_content := regexp_replace(processed_content, '<a\s+href="([^"]+)"[^>]*>(.*?)</a>', '<ext-link xlink:href="\1">\2</ext-link>', 'gi');
            processed_content := regexp_replace(processed_content, '<(strong|b)>(.*?)</\1>', '<bold>\2</bold>', 'gi');
            processed_content := regexp_replace(processed_content, '<(em|i)>(.*?)</\1>', '<italic>\2</italic>', 'gi');
            processed_content := regexp_replace(processed_content, '<code>(.*?)</code>', '<monospace>\1</monospace>', 'gi');
            processed_content := regexp_replace(processed_content, '<br\s*/?>', '<break/>', 'gi');
            -- <sup>, <sub>, and <sc> are valid in JATS, so they are left as is.

        -- B) Markdown Detection: Looks for Markdown syntax like **, *, ``, etc.
        WHEN processed_content ~ '(\*\*|__).+?\1' OR
             processed_content ~ '(?<![a-zA-Z0-9])(\*|_).+?\1(?![a-zA-Z0-9])' OR
             processed_content ~ '`[^`]+`' OR
             processed_content ~ '\[[^\]]+\]\([^)]+\)' THEN
            -- Convert Markdown to JATS. Order of replacement is important.
            processed_content := regexp_replace(processed_content, '\[([^\]]+)\]\(([^)]+)\)', '<ext-link xlink:href="\2">\1</ext-link>', 'g');
            processed_content := regexp_replace(processed_content, '\*\*(.+?)\*\*', '<bold>\1</bold>', 'g');
            processed_content := regexp_replace(processed_content, '__(.+?)__', '<bold>\1</bold>', 'g');
            processed_content := regexp_replace(processed_content, '\*(.+?)\*', '<italic>\1</italic>', 'g');
            processed_content := regexp_replace(processed_content, '_(.+?)_', '<italic>\1</italic>', 'g');
            processed_content := regexp_replace(processed_content, '`([^`]+)`', '<monospace>\1</monospace>', 'g');
            processed_content := regexp_replace(processed_content, '  \n', '<break/>\n', 'g');

            -- Wrap the result in <p> tags as Markdown is just a fragment.
            processed_content := '<p>' || processed_content || '</p>';
            -- Convert double newlines to paragraph breaks.
            processed_content := regexp_replace(processed_content, '\n\n', '</p><p>', 'g');

        -- C) Plaintext (Default Case)
        ELSE
            -- For plaintext, convert all-caps words to <sc> tags, then wrap in <p> tags and handle newlines.
            -- This rule assumes that words in all caps (e.g., "NASA") should be rendered in small-caps.
            processed_content := regexp_replace(processed_content, '\b([A-Z]{2,})\b', '<sc>\1</sc>', 'g');

            -- Wrap the content in <p> tags and convert newlines.
            processed_content := '<p>' || processed_content || '</p>';
            processed_content := regexp_replace(processed_content, E'\n\n', '</p><p>', 'g');
            processed_content := regexp_replace(processed_content, E'\n', '<break/>', 'g');
    END CASE;

    -- Return the processed content without the <abstract> wrapper.
    RETURN processed_content;

END;
$$ LANGUAGE plpgsql;

-- Insert short abstracts into the abstract table using the conversion function
INSERT INTO abstract (abstract_id, work_id, content, locale_code, abstract_type, canonical)
SELECT
    uuid_generate_v4() AS abstract_id,
    work_id,
    convert_to_jats(short_abstract) AS content,
    'en'::locale_code, -- Assuming 'en' as the default locale code
    'short'::abstract_type,
    TRUE
FROM
    work
WHERE
    short_abstract IS NOT NULL AND short_abstract != '';

-- Insert long abstracts into the abstract table using the conversion function
INSERT INTO abstract (abstract_id, work_id, content, locale_code, abstract_type, canonical)
SELECT
    uuid_generate_v4() AS abstract_id,
    work_id,
    convert_to_jats(long_abstract) AS content,
    'en'::locale_code, -- Assuming 'en' as the default locale code
    'long'::abstract_type,
    TRUE
FROM
    work
WHERE
    long_abstract IS NOT NULL AND long_abstract != '';

-- Clean up the conversion function after the migration is complete
DROP FUNCTION convert_to_jats(TEXT);

-- Only allow one canonical abstract per work
CREATE UNIQUE INDEX IF NOT EXISTS abstract_unique_canonical_true_idx
ON abstract(work_id, abstract_type)
WHERE canonical;

-- Only allow one instance of each locale per work
CREATE UNIQUE INDEX IF NOT EXISTS abstract_uniq_locale_idx
ON abstract(work_id, locale_code, abstract_type);

-- Drop title-related columns from the work table
ALTER TABLE work
    DROP COLUMN short_abstract,
    DROP COLUMN long_abstract;
