DO $$
BEGIN
    IF EXISTS (
        SELECT
            1
        FROM
            (
                SELECT
                    locale_code::text AS locale_code
                FROM
                    public.abstract
                UNION ALL
                SELECT
                    locale_code::text
                FROM
                    public.biography
                UNION ALL
                SELECT
                    default_locale::text
                FROM
                    public.imprint
                WHERE
                    default_locale IS NOT NULL
                UNION ALL
                SELECT
                    locale_code::text
                FROM
                    public.title
            ) AS locale_codes
        WHERE
            locale_code IN ('ve', 've_za')
    ) THEN
        RAISE EXCEPTION 'Cannot revert locale_code enum while ve/ve_za values are in use.';
    END IF;
END
$$;

ALTER TYPE public.locale_code
    RENAME TO locale_code_old;

DO $$
DECLARE
    locale_enum_labels TEXT;
BEGIN
    SELECT
        string_agg(quote_literal(enumlabel), ', ' ORDER BY enumsortorder)
    INTO locale_enum_labels
    FROM
        pg_enum
    WHERE
        enumtypid = 'public.locale_code_old'::regtype
        AND enumlabel NOT IN ('ve', 've_za');

    EXECUTE 'CREATE TYPE public.locale_code AS ENUM (' || locale_enum_labels || ')';
END
$$;

ALTER TABLE public.abstract
    ALTER COLUMN locale_code TYPE public.locale_code
    USING locale_code::text::public.locale_code;

ALTER TABLE public.biography
    ALTER COLUMN locale_code TYPE public.locale_code
    USING locale_code::text::public.locale_code;

ALTER TABLE public.imprint
    ALTER COLUMN default_locale TYPE public.locale_code
    USING default_locale::text::public.locale_code;

ALTER TABLE public.title
    ALTER COLUMN locale_code TYPE public.locale_code
    USING locale_code::text::public.locale_code;

DROP TYPE public.locale_code_old;
