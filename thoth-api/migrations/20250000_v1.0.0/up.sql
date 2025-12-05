--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: contribution_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.contribution_type AS ENUM (
    'author',
    'editor',
    'translator',
    'photographer',
    'illustrator',
    'music-editor',
    'foreword-by',
    'introduction-by',
    'afterword-by',
    'preface-by',
    'software-by',
    'research-by',
    'contributions-by',
    'indexer'
);


--
-- Name: country_code; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.country_code AS ENUM (
    'afg',
    'ala',
    'alb',
    'dza',
    'asm',
    'and',
    'ago',
    'aia',
    'ata',
    'atg',
    'arg',
    'arm',
    'abw',
    'aus',
    'aut',
    'aze',
    'bhs',
    'bhr',
    'bgd',
    'brb',
    'blr',
    'bel',
    'blz',
    'ben',
    'bmu',
    'btn',
    'bol',
    'bes',
    'bih',
    'bwa',
    'bvt',
    'bra',
    'iot',
    'brn',
    'bgr',
    'bfa',
    'bdi',
    'cpv',
    'khm',
    'cmr',
    'can',
    'cym',
    'caf',
    'tcd',
    'chl',
    'chn',
    'cxr',
    'cck',
    'col',
    'com',
    'cok',
    'cri',
    'civ',
    'hrv',
    'cub',
    'cuw',
    'cyp',
    'cze',
    'cod',
    'dnk',
    'dji',
    'dma',
    'dom',
    'ecu',
    'egy',
    'slv',
    'gnq',
    'eri',
    'est',
    'swz',
    'eth',
    'flk',
    'fro',
    'fji',
    'fin',
    'fra',
    'guf',
    'pyf',
    'atf',
    'gab',
    'gmb',
    'geo',
    'deu',
    'gha',
    'gib',
    'grc',
    'grl',
    'grd',
    'glp',
    'gum',
    'gtm',
    'ggy',
    'gin',
    'gnb',
    'guy',
    'hti',
    'hmd',
    'hnd',
    'hkg',
    'hun',
    'isl',
    'ind',
    'idn',
    'irn',
    'irq',
    'irl',
    'imn',
    'isr',
    'ita',
    'jam',
    'jpn',
    'jey',
    'jor',
    'kaz',
    'ken',
    'kir',
    'kwt',
    'kgz',
    'lao',
    'lva',
    'lbn',
    'lso',
    'lbr',
    'lby',
    'lie',
    'ltu',
    'lux',
    'mac',
    'mdg',
    'mwi',
    'mys',
    'mdv',
    'mli',
    'mlt',
    'mhl',
    'mtq',
    'mrt',
    'mus',
    'myt',
    'mex',
    'fsm',
    'mda',
    'mco',
    'mng',
    'mne',
    'msr',
    'mar',
    'moz',
    'mmr',
    'nam',
    'nru',
    'npl',
    'nld',
    'ncl',
    'nzl',
    'nic',
    'ner',
    'nga',
    'niu',
    'nfk',
    'prk',
    'mkd',
    'mnp',
    'nor',
    'omn',
    'pak',
    'plw',
    'pse',
    'pan',
    'png',
    'pry',
    'per',
    'phl',
    'pcn',
    'pol',
    'prt',
    'pri',
    'qat',
    'cog',
    'reu',
    'rou',
    'rus',
    'rwa',
    'blm',
    'shn',
    'kna',
    'lca',
    'maf',
    'spm',
    'vct',
    'wsm',
    'smr',
    'stp',
    'sau',
    'sen',
    'srb',
    'syc',
    'sle',
    'sgp',
    'sxm',
    'svk',
    'svn',
    'slb',
    'som',
    'zaf',
    'sgs',
    'kor',
    'ssd',
    'esp',
    'lka',
    'sdn',
    'sur',
    'sjm',
    'swe',
    'che',
    'syr',
    'twn',
    'tjk',
    'tza',
    'tha',
    'tls',
    'tgo',
    'tkl',
    'ton',
    'tto',
    'tun',
    'tur',
    'tkm',
    'tca',
    'tuv',
    'uga',
    'ukr',
    'are',
    'gbr',
    'umi',
    'usa',
    'ury',
    'uzb',
    'vut',
    'vat',
    'ven',
    'vnm',
    'vgb',
    'vir',
    'wlf',
    'esh',
    'yem',
    'zmb',
    'zwe'
);


--
-- Name: currency_code; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.currency_code AS ENUM (
    'adp',
    'aed',
    'afa',
    'afn',
    'alk',
    'all',
    'amd',
    'ang',
    'aoa',
    'aok',
    'aon',
    'aor',
    'ara',
    'arp',
    'ars',
    'ary',
    'ats',
    'aud',
    'awg',
    'aym',
    'azm',
    'azn',
    'bad',
    'bam',
    'bbd',
    'bdt',
    'bec',
    'bef',
    'bel',
    'bgj',
    'bgk',
    'bgl',
    'bgn',
    'bhd',
    'bif',
    'bmd',
    'bnd',
    'bob',
    'bop',
    'bov',
    'brb',
    'brc',
    'bre',
    'brl',
    'brn',
    'brr',
    'bsd',
    'btn',
    'buk',
    'bwp',
    'byb',
    'byn',
    'byr',
    'bzd',
    'cad',
    'cdf',
    'chc',
    'che',
    'chf',
    'chw',
    'clf',
    'clp',
    'cny',
    'cop',
    'cou',
    'crc',
    'csd',
    'csj',
    'csk',
    'cuc',
    'cup',
    'cve',
    'cyp',
    'czk',
    'ddm',
    'dem',
    'djf',
    'dkk',
    'dop',
    'dzd',
    'ecs',
    'ecv',
    'eek',
    'egp',
    'ern',
    'esa',
    'esb',
    'esp',
    'etb',
    'eur',
    'fim',
    'fjd',
    'fkp',
    'frf',
    'gbp',
    'gek',
    'gel',
    'ghc',
    'ghp',
    'ghs',
    'gip',
    'gmd',
    'gne',
    'gnf',
    'gns',
    'gqe',
    'grd',
    'gtq',
    'gwe',
    'gwp',
    'gyd',
    'hkd',
    'hnl',
    'hrd',
    'hrk',
    'htg',
    'huf',
    'idr',
    'iep',
    'ilp',
    'ilr',
    'ils',
    'inr',
    'iqd',
    'irr',
    'isj',
    'isk',
    'itl',
    'jmd',
    'jod',
    'jpy',
    'kes',
    'kgs',
    'khr',
    'kmf',
    'kpw',
    'krw',
    'kwd',
    'kyd',
    'kzt',
    'laj',
    'lak',
    'lbp',
    'lkr',
    'lrd',
    'lsl',
    'lsm',
    'ltl',
    'ltt',
    'luc',
    'luf',
    'lul',
    'lvl',
    'lvr',
    'lyd',
    'mad',
    'mdl',
    'mga',
    'mgf',
    'mkd',
    'mlf',
    'mmk',
    'mnt',
    'mop',
    'mro',
    'mru',
    'mtl',
    'mtp',
    'mur',
    'mvq',
    'mvr',
    'mwk',
    'mxn',
    'mxp',
    'mxv',
    'myr',
    'mze',
    'mzm',
    'mzn',
    'nad',
    'ngn',
    'nic',
    'nio',
    'nlg',
    'nok',
    'npr',
    'nzd',
    'omr',
    'pab',
    'peh',
    'pei',
    'pen',
    'pes',
    'pgk',
    'php',
    'pkr',
    'pln',
    'plz',
    'pte',
    'pyg',
    'qar',
    'rhd',
    'rok',
    'rol',
    'ron',
    'rsd',
    'rub',
    'rur',
    'rwf',
    'sar',
    'sbd',
    'scr',
    'sdd',
    'sdg',
    'sdp',
    'sek',
    'sgd',
    'shp',
    'sit',
    'skk',
    'sll',
    'sos',
    'srd',
    'srg',
    'ssp',
    'std',
    'stn',
    'sur',
    'svc',
    'syp',
    'szl',
    'thb',
    'tjr',
    'tjs',
    'tmm',
    'tmt',
    'tnd',
    'top',
    'tpe',
    'trl',
    'try',
    'ttd',
    'twd',
    'tzs',
    'uah',
    'uak',
    'ugs',
    'ugw',
    'ugx',
    'usd',
    'usn',
    'uss',
    'uyi',
    'uyn',
    'uyp',
    'uyu',
    'uyw',
    'uzs',
    'veb',
    'vef',
    'ves',
    'vnc',
    'vnd',
    'vuv',
    'wst',
    'xaf',
    'xag',
    'xau',
    'xba',
    'xbb',
    'xbc',
    'xbd',
    'xcd',
    'xdr',
    'xeu',
    'xfo',
    'xfu',
    'xof',
    'xpd',
    'xpf',
    'xpt',
    'xre',
    'xsu',
    'xts',
    'xua',
    'xxx',
    'ydd',
    'yer',
    'yud',
    'yum',
    'yun',
    'zal',
    'zar',
    'zmk',
    'zmw',
    'zrn',
    'zrz',
    'zwc',
    'zwd',
    'zwl',
    'zwn',
    'zwr'
);


--
-- Name: language_code; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.language_code AS ENUM (
    'aar',
    'abk',
    'ace',
    'ach',
    'ada',
    'ady',
    'afa',
    'afh',
    'afr',
    'ain',
    'aka',
    'akk',
    'alb',
    'ale',
    'alg',
    'alt',
    'amh',
    'ang',
    'anp',
    'apa',
    'ara',
    'arc',
    'arg',
    'arm',
    'arn',
    'arp',
    'art',
    'arw',
    'asm',
    'ast',
    'ath',
    'aus',
    'ava',
    'ave',
    'awa',
    'aym',
    'aze',
    'bad',
    'bai',
    'bak',
    'bal',
    'bam',
    'ban',
    'baq',
    'bas',
    'bat',
    'bej',
    'bel',
    'bem',
    'ben',
    'ber',
    'bho',
    'bih',
    'bik',
    'bin',
    'bis',
    'bla',
    'bnt',
    'bos',
    'bra',
    'bre',
    'btk',
    'bua',
    'bug',
    'bul',
    'bur',
    'byn',
    'cad',
    'cai',
    'car',
    'cat',
    'cau',
    'ceb',
    'cel',
    'cha',
    'chb',
    'che',
    'chg',
    'chi',
    'chk',
    'chm',
    'chn',
    'cho',
    'chp',
    'chr',
    'chu',
    'chv',
    'chy',
    'cmc',
    'cnr',
    'cop',
    'cor',
    'cos',
    'cpe',
    'cpf',
    'cpp',
    'cre',
    'crh',
    'crp',
    'csb',
    'cus',
    'cze',
    'dak',
    'dan',
    'dar',
    'day',
    'del',
    'den',
    'dgr',
    'din',
    'div',
    'doi',
    'dra',
    'dsb',
    'dua',
    'dum',
    'dut',
    'dyu',
    'dzo',
    'efi',
    'egy',
    'eka',
    'elx',
    'eng',
    'enm',
    'epo',
    'est',
    'ewe',
    'ewo',
    'fan',
    'fao',
    'fat',
    'fij',
    'fil',
    'fin',
    'fiu',
    'fon',
    'fre',
    'frm',
    'fro',
    'frr',
    'frs',
    'fry',
    'ful',
    'fur',
    'gaa',
    'gay',
    'gba',
    'gem',
    'geo',
    'ger',
    'gez',
    'gil',
    'gla',
    'gle',
    'glg',
    'glv',
    'gmh',
    'goh',
    'gon',
    'gor',
    'got',
    'grb',
    'grc',
    'gre',
    'grn',
    'gsw',
    'guj',
    'gwi',
    'hai',
    'hat',
    'hau',
    'haw',
    'heb',
    'her',
    'hil',
    'him',
    'hin',
    'hit',
    'hmn',
    'hmo',
    'hrv',
    'hsb',
    'hun',
    'hup',
    'iba',
    'ibo',
    'ice',
    'ido',
    'iii',
    'ijo',
    'iku',
    'ile',
    'ilo',
    'ina',
    'inc',
    'ind',
    'ine',
    'inh',
    'ipk',
    'ira',
    'iro',
    'ita',
    'jav',
    'jbo',
    'jpn',
    'jpr',
    'jrb',
    'kaa',
    'kab',
    'kac',
    'kal',
    'kam',
    'kan',
    'kar',
    'kas',
    'kau',
    'kaw',
    'kaz',
    'kbd',
    'kha',
    'khi',
    'khm',
    'kho',
    'kik',
    'kin',
    'kir',
    'kmb',
    'kok',
    'kom',
    'kon',
    'kor',
    'kos',
    'kpe',
    'krc',
    'krl',
    'kro',
    'kru',
    'kua',
    'kum',
    'kur',
    'kut',
    'lad',
    'lah',
    'lam',
    'lao',
    'lat',
    'lav',
    'lez',
    'lim',
    'lin',
    'lit',
    'lol',
    'loz',
    'ltz',
    'lua',
    'lub',
    'lug',
    'lui',
    'lun',
    'luo',
    'lus',
    'mac',
    'mad',
    'mag',
    'mah',
    'mai',
    'mak',
    'mal',
    'man',
    'mao',
    'map',
    'mar',
    'mas',
    'may',
    'mdf',
    'mdr',
    'men',
    'mga',
    'mic',
    'min',
    'mis',
    'mkh',
    'mlg',
    'mlt',
    'mnc',
    'mni',
    'mno',
    'moh',
    'mon',
    'mos',
    'mul',
    'mun',
    'mus',
    'mwl',
    'mwr',
    'myn',
    'myv',
    'nah',
    'nai',
    'nap',
    'nau',
    'nav',
    'nbl',
    'nde',
    'ndo',
    'nds',
    'nep',
    'new',
    'nia',
    'nic',
    'niu',
    'nno',
    'nob',
    'nog',
    'non',
    'nor',
    'nqo',
    'nso',
    'nub',
    'nwc',
    'nya',
    'nym',
    'nyn',
    'nyo',
    'nzi',
    'oci',
    'oji',
    'ori',
    'orm',
    'osa',
    'oss',
    'ota',
    'oto',
    'paa',
    'pag',
    'pal',
    'pam',
    'pan',
    'pap',
    'pau',
    'peo',
    'per',
    'phi',
    'phn',
    'pli',
    'pol',
    'pon',
    'por',
    'pra',
    'pro',
    'pus',
    'qaa',
    'que',
    'raj',
    'rap',
    'rar',
    'roa',
    'roh',
    'rom',
    'rum',
    'run',
    'rup',
    'rus',
    'sad',
    'sag',
    'sah',
    'sai',
    'sal',
    'sam',
    'san',
    'sas',
    'sat',
    'scn',
    'sco',
    'sel',
    'sem',
    'sga',
    'sgn',
    'shn',
    'sid',
    'sin',
    'sio',
    'sit',
    'sla',
    'slo',
    'slv',
    'sma',
    'sme',
    'smi',
    'smj',
    'smn',
    'smo',
    'sms',
    'sna',
    'snd',
    'snk',
    'sog',
    'som',
    'son',
    'sot',
    'spa',
    'srd',
    'srn',
    'srp',
    'srr',
    'ssa',
    'ssw',
    'suk',
    'sun',
    'sus',
    'sux',
    'swa',
    'swe',
    'syc',
    'syr',
    'tah',
    'tai',
    'tam',
    'tat',
    'tel',
    'tem',
    'ter',
    'tet',
    'tgk',
    'tgl',
    'tha',
    'tib',
    'tig',
    'tir',
    'tiv',
    'tkl',
    'tlh',
    'tli',
    'tmh',
    'tog',
    'ton',
    'tpi',
    'tsi',
    'tsn',
    'tso',
    'tuk',
    'tum',
    'tup',
    'tur',
    'tut',
    'tvl',
    'twi',
    'tyv',
    'udm',
    'uga',
    'uig',
    'ukr',
    'umb',
    'und',
    'urd',
    'uzb',
    'vai',
    'ven',
    'vie',
    'vol',
    'vot',
    'wak',
    'wal',
    'war',
    'was',
    'wel',
    'wen',
    'wln',
    'wol',
    'xal',
    'xho',
    'yao',
    'yap',
    'yid',
    'yor',
    'ypk',
    'zap',
    'zbl',
    'zen',
    'zgh',
    'zha',
    'znd',
    'zul',
    'zun',
    'zxx',
    'zza'
);


--
-- Name: language_relation; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.language_relation AS ENUM (
    'original',
    'translated-from',
    'translated-into'
);


--
-- Name: location_platform; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.location_platform AS ENUM (
    'Project MUSE',
    'OAPEN',
    'DOAB',
    'JSTOR',
    'EBSCO Host',
    'OCLC KB',
    'ProQuest KB',
    'ProQuest ExLibris',
    'EBSCO KB',
    'JISC KB',
    'Other',
    'Google Books',
    'Internet Archive',
    'ScienceOpen',
    'SciELO Books',
    'Publisher Website',
    'Zenodo',
    'Thoth'
);


--
-- Name: publication_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.publication_type AS ENUM (
    'Paperback',
    'Hardback',
    'PDF',
    'HTML',
    'XML',
    'Epub',
    'Mobi',
    'AZW3',
    'DOCX',
    'FictionBook',
    'MP3',
    'WAV'
);


--
-- Name: relation_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.relation_type AS ENUM (
    'replaces',
    'has-translation',
    'has-part',
    'has-child',
    'is-replaced-by',
    'is-translation-of',
    'is-part-of',
    'is-child-of'
);


--
-- Name: series_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.series_type AS ENUM (
    'journal',
    'book-series'
);


--
-- Name: subject_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.subject_type AS ENUM (
    'bic',
    'bisac',
    'thema',
    'lcc',
    'custom',
    'keyword'
);


--
-- Name: work_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.work_status AS ENUM (
    'cancelled',
    'forthcoming',
    'postponed-indefinitely',
    'active',
    'withdrawn',
    'superseded'
);


--
-- Name: work_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.work_type AS ENUM (
    'book-chapter',
    'monograph',
    'edited-book',
    'textbook',
    'journal-issue',
    'book-set'
);


--
-- Name: affiliation_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.affiliation_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM contribution
        WHERE work.work_id = contribution.work_id AND contribution.contribution_id = OLD.contribution_id
            OR work.work_id = contribution.work_id AND contribution.contribution_id = NEW.contribution_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: contributor_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.contributor_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM contribution
        -- No need to check OLD.contributor_id, as this will be the same as NEW.contributor_id in all relevant cases
        -- (contributor_id can't be changed on contributors which are referenced by existing contributions)
        WHERE work.work_id = contribution.work_id AND contribution.contributor_id = NEW.contributor_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: imprint_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.imprint_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE imprint_id = NEW.imprint_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: institution_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.institution_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        -- Same as contributor above (but can be connected to work via two different tables)
        -- Use two separate UPDATE statements as this is much faster than combining the WHERE clauses
        -- using OR (in tests, this caused several seconds' delay when saving institution updates)
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM funding
        WHERE work.work_id = funding.work_id AND funding.institution_id = NEW.institution_id;
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM affiliation, contribution
        WHERE work.work_id = contribution.work_id AND contribution.contribution_id = affiliation.contribution_id AND affiliation.institution_id = NEW.institution_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: location_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.location_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM publication
        WHERE work.work_id = publication.work_id AND publication.publication_id = OLD.publication_id
            OR work.work_id = publication.work_id AND publication.publication_id = NEW.publication_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: price_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.price_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM publication
        WHERE work.work_id = publication.work_id AND publication.publication_id = OLD.publication_id
            OR work.work_id = publication.work_id AND publication.publication_id = NEW.publication_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: publication_chapter_no_dimensions(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.publication_chapter_no_dimensions() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        (SELECT work_type FROM work WHERE work.work_id = NEW.work_id) = 'book-chapter' AND (
            NEW.width_mm IS NOT NULL OR
            NEW.width_in IS NOT NULL OR
            NEW.height_mm IS NOT NULL OR
            NEW.height_in IS NOT NULL OR
            NEW.depth_mm IS NOT NULL OR
            NEW.depth_in IS NOT NULL OR
            NEW.weight_g IS NOT NULL OR
            NEW.weight_oz IS NOT NULL
        )
    ) THEN
        RAISE EXCEPTION 'Chapters cannot have dimensions (Width/Height/Depth/Weight)';
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: publication_location_canonical_urls(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.publication_location_canonical_urls() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW.publication_type <> 'Hardback' AND
        NEW.publication_type <> 'Paperback' AND
        (SELECT COUNT(*) FROM location
            WHERE location.publication_id = NEW.publication_id
            AND location.canonical
            AND (location.landing_page IS NULL OR location.full_text_url IS NULL)
        ) > 0
    ) THEN
        RAISE EXCEPTION 'Digital publications must have both Landing Page and Full Text URL in all their canonical locations';
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: publisher_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.publisher_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM imprint
        -- Same as contributor above
        WHERE work.imprint_id = imprint.imprint_id AND imprint.publisher_id = NEW.publisher_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: series_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.series_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM issue
        -- Same as contributor above (note that although series is also connected to work
        -- via the imprint_id, changes to a series don't affect its imprint)
        WHERE work.work_id = issue.work_id AND issue.series_id = NEW.series_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: work_relation_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.work_relation_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE work_id = OLD.relator_work_id OR work_id = NEW.relator_work_id
            OR work_id = OLD.related_work_id OR work_id = NEW.related_work_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: work_set_updated_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.work_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at AND
        NEW.updated_at_with_relations IS NOT DISTINCT FROM OLD.updated_at_with_relations
    ) THEN
        NEW.updated_at := current_timestamp;
        NEW.updated_at_with_relations := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE work_id = OLD.work_id OR work_id = NEW.work_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: work_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.work_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM work_relation
        -- The positions of relator/related IDs in this statement don't matter, as
        -- every work_relation record has a mirrored record with relator/related IDs swapped
        WHERE work.work_id = work_relation.relator_work_id AND work_relation.related_work_id = NEW.work_id;
    END IF;
    RETURN NULL;
END;
$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: __diesel_schema_migrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.__diesel_schema_migrations (
    version character varying(50) NOT NULL,
    run_on timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: account; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.account (
    account_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name text NOT NULL,
    surname text NOT NULL,
    email text NOT NULL,
    hash bytea NOT NULL,
    salt text NOT NULL,
    is_superuser boolean DEFAULT false NOT NULL,
    is_bot boolean DEFAULT false NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    token text,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT account_email_check CHECK ((octet_length(email) >= 1)),
    CONSTRAINT account_name_check CHECK ((octet_length(name) >= 1)),
    CONSTRAINT account_salt_check CHECK ((octet_length(salt) >= 1)),
    CONSTRAINT account_surname_check CHECK ((octet_length(surname) >= 1)),
    CONSTRAINT account_token_check CHECK ((octet_length(token) >= 1))
);


--
-- Name: affiliation; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.affiliation (
    affiliation_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    contribution_id uuid NOT NULL,
    institution_id uuid NOT NULL,
    affiliation_ordinal integer NOT NULL,
    "position" text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT affiliation_affiliation_ordinal_check CHECK ((affiliation_ordinal > 0)),
    CONSTRAINT affiliation_position_check CHECK ((octet_length("position") >= 1))
);


--
-- Name: affiliation_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.affiliation_history (
    affiliation_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    affiliation_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: contribution; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.contribution (
    work_id uuid NOT NULL,
    contributor_id uuid NOT NULL,
    contribution_type public.contribution_type NOT NULL,
    main_contribution boolean DEFAULT true NOT NULL,
    biography text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    first_name text,
    last_name text NOT NULL,
    full_name text NOT NULL,
    contribution_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    contribution_ordinal integer NOT NULL,
    CONSTRAINT contribution_biography_check CHECK ((octet_length(biography) >= 1)),
    CONSTRAINT contribution_contribution_ordinal_check CHECK ((contribution_ordinal > 0)),
    CONSTRAINT contribution_first_name_check CHECK ((octet_length(first_name) >= 1)),
    CONSTRAINT contribution_full_name_check CHECK ((octet_length(full_name) >= 1)),
    CONSTRAINT contribution_last_name_check CHECK ((octet_length(last_name) >= 1))
);


--
-- Name: contribution_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.contribution_history (
    contribution_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    contribution_id uuid NOT NULL
);


--
-- Name: contributor; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.contributor (
    contributor_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    first_name text,
    last_name text NOT NULL,
    full_name text NOT NULL,
    orcid text,
    website text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT contributor_first_name_check CHECK ((octet_length(first_name) >= 1)),
    CONSTRAINT contributor_full_name_check CHECK ((octet_length(full_name) >= 1)),
    CONSTRAINT contributor_last_name_check CHECK ((octet_length(last_name) >= 1)),
    CONSTRAINT contributor_orcid_check CHECK ((orcid ~ '^https:\/\/orcid\.org\/\d{4}-\d{4}-\d{4}-\d{3}[\dX]$'::text)),
    CONSTRAINT contributor_website_check CHECK ((octet_length(website) >= 1))
);


--
-- Name: contributor_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.contributor_history (
    contributor_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    contributor_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: funding; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.funding (
    funding_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    institution_id uuid NOT NULL,
    program text,
    project_name text,
    project_shortname text,
    grant_number text,
    jurisdiction text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT funding_grant_number_check CHECK ((octet_length(grant_number) >= 1)),
    CONSTRAINT funding_jurisdiction_check CHECK ((octet_length(jurisdiction) >= 1)),
    CONSTRAINT funding_program_check CHECK ((octet_length(program) >= 1)),
    CONSTRAINT funding_project_name_check CHECK ((octet_length(project_name) >= 1)),
    CONSTRAINT funding_project_shortname_check CHECK ((octet_length(project_shortname) >= 1))
);


--
-- Name: funding_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.funding_history (
    funding_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    funding_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: imprint; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.imprint (
    imprint_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publisher_id uuid NOT NULL,
    imprint_name text NOT NULL,
    imprint_url text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    crossmark_doi text,
    CONSTRAINT imprint_crossmark_doi_check CHECK ((crossmark_doi ~* 'https:\/\/doi.org\/10.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$'::text)),
    CONSTRAINT imprint_imprint_name_check CHECK ((octet_length(imprint_name) >= 1)),
    CONSTRAINT imprint_imprint_url_check CHECK ((imprint_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text))
);


--
-- Name: imprint_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.imprint_history (
    imprint_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    imprint_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: institution; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.institution (
    institution_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    institution_name text NOT NULL,
    institution_doi text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    ror text,
    country_code public.country_code,
    CONSTRAINT institution_institution_doi_check CHECK ((institution_doi ~ '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._;()\/:a-zA-Z0-9<>+[\]]+$'::text)),
    CONSTRAINT institution_institution_name_check CHECK ((octet_length(institution_name) >= 1)),
    CONSTRAINT institution_ror_check CHECK ((ror ~ '^https:\/\/ror\.org\/0[a-hjkmnp-z0-9]{6}\d{2}$'::text))
);


--
-- Name: institution_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.institution_history (
    institution_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    institution_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: issue; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.issue (
    series_id uuid NOT NULL,
    work_id uuid NOT NULL,
    issue_ordinal integer NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    issue_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    CONSTRAINT issue_issue_ordinal_check CHECK ((issue_ordinal > 0))
);


--
-- Name: issue_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.issue_history (
    issue_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    issue_id uuid NOT NULL
);


--
-- Name: language; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.language (
    language_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    language_code public.language_code NOT NULL,
    language_relation public.language_relation NOT NULL,
    main_language boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: language_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.language_history (
    language_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    language_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: location; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.location (
    location_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publication_id uuid NOT NULL,
    landing_page text,
    full_text_url text,
    location_platform public.location_platform DEFAULT 'Other'::public.location_platform NOT NULL,
    canonical boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT location_full_text_url_check CHECK ((full_text_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT location_landing_page_check CHECK ((landing_page ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT location_url_check CHECK (((landing_page IS NOT NULL) OR (full_text_url IS NOT NULL)))
);


--
-- Name: location_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.location_history (
    location_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    location_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: price; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.price (
    price_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publication_id uuid NOT NULL,
    currency_code public.currency_code NOT NULL,
    unit_price double precision NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT price_unit_price_check CHECK ((unit_price > (0.0)::double precision))
);


--
-- Name: price_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.price_history (
    price_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    price_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: publication; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.publication (
    publication_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publication_type public.publication_type NOT NULL,
    work_id uuid NOT NULL,
    isbn text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    width_mm double precision,
    width_in double precision,
    height_mm double precision,
    height_in double precision,
    depth_mm double precision,
    depth_in double precision,
    weight_g double precision,
    weight_oz double precision,
    CONSTRAINT publication_depth_in_check CHECK ((depth_in > (0.0)::double precision)),
    CONSTRAINT publication_depth_in_not_missing CHECK (((depth_in IS NOT NULL) OR (depth_mm IS NULL))),
    CONSTRAINT publication_depth_mm_check CHECK ((depth_mm > (0.0)::double precision)),
    CONSTRAINT publication_depth_mm_not_missing CHECK (((depth_mm IS NOT NULL) OR (depth_in IS NULL))),
    CONSTRAINT publication_height_in_check CHECK ((height_in > (0.0)::double precision)),
    CONSTRAINT publication_height_in_not_missing CHECK (((height_in IS NOT NULL) OR (height_mm IS NULL))),
    CONSTRAINT publication_height_mm_check CHECK ((height_mm > (0.0)::double precision)),
    CONSTRAINT publication_height_mm_not_missing CHECK (((height_mm IS NOT NULL) OR (height_in IS NULL))),
    CONSTRAINT publication_isbn_check CHECK ((octet_length(isbn) = 17)),
    CONSTRAINT publication_non_physical_no_dimensions CHECK ((((width_mm IS NULL) AND (width_in IS NULL) AND (height_mm IS NULL) AND (height_in IS NULL) AND (depth_mm IS NULL) AND (depth_in IS NULL) AND (weight_g IS NULL) AND (weight_oz IS NULL)) OR (publication_type = 'Paperback'::public.publication_type) OR (publication_type = 'Hardback'::public.publication_type))),
    CONSTRAINT publication_weight_g_check CHECK ((weight_g > (0.0)::double precision)),
    CONSTRAINT publication_weight_g_not_missing CHECK (((weight_g IS NOT NULL) OR (weight_oz IS NULL))),
    CONSTRAINT publication_weight_oz_check CHECK ((weight_oz > (0.0)::double precision)),
    CONSTRAINT publication_weight_oz_not_missing CHECK (((weight_oz IS NOT NULL) OR (weight_g IS NULL))),
    CONSTRAINT publication_width_in_check CHECK ((width_in > (0.0)::double precision)),
    CONSTRAINT publication_width_in_not_missing CHECK (((width_in IS NOT NULL) OR (width_mm IS NULL))),
    CONSTRAINT publication_width_mm_check CHECK ((width_mm > (0.0)::double precision)),
    CONSTRAINT publication_width_mm_not_missing CHECK (((width_mm IS NOT NULL) OR (width_in IS NULL)))
);


--
-- Name: publication_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.publication_history (
    publication_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publication_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: publisher; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.publisher (
    publisher_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publisher_name text NOT NULL,
    publisher_shortname text,
    publisher_url text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT publisher_publisher_name_check CHECK ((octet_length(publisher_name) >= 1)),
    CONSTRAINT publisher_publisher_shortname_check CHECK ((octet_length(publisher_shortname) >= 1)),
    CONSTRAINT publisher_publisher_url_check CHECK ((publisher_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text))
);


--
-- Name: publisher_account; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.publisher_account (
    account_id uuid NOT NULL,
    publisher_id uuid NOT NULL,
    is_admin boolean DEFAULT false NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: publisher_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.publisher_history (
    publisher_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publisher_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: reference; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reference (
    reference_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    reference_ordinal integer NOT NULL,
    doi text,
    unstructured_citation text,
    issn text,
    isbn text,
    journal_title text,
    article_title text,
    series_title text,
    volume_title text,
    edition integer,
    author text,
    volume text,
    issue text,
    first_page text,
    component_number text,
    standard_designator text,
    standards_body_name text,
    standards_body_acronym text,
    url text,
    publication_date date,
    retrieval_date date,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT reference_article_title_check CHECK ((octet_length(article_title) >= 1)),
    CONSTRAINT reference_author_check CHECK ((octet_length(author) >= 1)),
    CONSTRAINT reference_component_number_check CHECK ((octet_length(component_number) >= 1)),
    CONSTRAINT reference_doi_andor_unstructured_citation CHECK (((doi IS NOT NULL) OR (unstructured_citation IS NOT NULL))),
    CONSTRAINT reference_doi_check CHECK ((doi ~ '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._;()\/:a-zA-Z0-9<>+[\]]+$'::text)),
    CONSTRAINT reference_edition_check CHECK ((edition > 0)),
    CONSTRAINT reference_first_page_check CHECK ((octet_length(first_page) >= 1)),
    CONSTRAINT reference_isbn_check CHECK ((octet_length(isbn) = 17)),
    CONSTRAINT reference_issn_check CHECK ((issn ~* '\d{4}\-\d{3}(\d|X)'::text)),
    CONSTRAINT reference_issue_check CHECK ((octet_length(issue) >= 1)),
    CONSTRAINT reference_journal_title_check CHECK ((octet_length(journal_title) >= 1)),
    CONSTRAINT reference_reference_ordinal_check CHECK ((reference_ordinal > 0)),
    CONSTRAINT reference_series_title_check CHECK ((octet_length(series_title) >= 1)),
    CONSTRAINT reference_standard_citation_required_fields CHECK ((((standard_designator IS NOT NULL) AND (standards_body_name IS NOT NULL) AND (standards_body_acronym IS NOT NULL)) OR ((standard_designator IS NULL) AND (standards_body_name IS NULL) AND (standards_body_acronym IS NULL)))),
    CONSTRAINT reference_standard_designator_check CHECK ((octet_length(standard_designator) >= 1)),
    CONSTRAINT reference_standards_body_acronym_check CHECK ((octet_length(standards_body_acronym) >= 1)),
    CONSTRAINT reference_standards_body_name_check CHECK ((octet_length(standards_body_name) >= 1)),
    CONSTRAINT reference_unstructured_citation_check CHECK ((octet_length(unstructured_citation) >= 1)),
    CONSTRAINT reference_url_check CHECK ((url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT reference_volume_check CHECK ((octet_length(volume) >= 1)),
    CONSTRAINT reference_volume_title_check CHECK ((octet_length(volume_title) >= 1))
);


--
-- Name: reference_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reference_history (
    reference_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    reference_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: series; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.series (
    series_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    series_type public.series_type NOT NULL,
    series_name text NOT NULL,
    issn_print text,
    issn_digital text,
    series_url text,
    imprint_id uuid NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    series_description text,
    series_cfp_url text,
    CONSTRAINT series_issn_digital_check CHECK ((issn_digital ~* '\d{4}\-\d{3}(\d|X)'::text)),
    CONSTRAINT series_issn_print_check CHECK ((issn_print ~* '\d{4}\-\d{3}(\d|X)'::text)),
    CONSTRAINT series_series_cfp_url_check CHECK ((series_cfp_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT series_series_description_check CHECK ((octet_length(series_description) >= 1)),
    CONSTRAINT series_series_name_check CHECK ((octet_length(series_name) >= 1)),
    CONSTRAINT series_series_url_check CHECK ((series_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text))
);


--
-- Name: series_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.series_history (
    series_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    series_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: subject; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.subject (
    subject_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    subject_type public.subject_type NOT NULL,
    subject_code text NOT NULL,
    subject_ordinal integer NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT subject_subject_code_check CHECK ((octet_length(subject_code) >= 1)),
    CONSTRAINT subject_subject_ordinal_check CHECK ((subject_ordinal > 0))
);


--
-- Name: subject_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.subject_history (
    subject_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    subject_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: work; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work (
    work_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_type public.work_type NOT NULL,
    work_status public.work_status NOT NULL,
    full_title text NOT NULL,
    title text NOT NULL,
    subtitle text,
    reference text,
    edition integer,
    imprint_id uuid NOT NULL,
    doi text,
    publication_date date,
    place text,
    page_count integer,
    page_breakdown text,
    image_count integer,
    table_count integer,
    audio_count integer,
    video_count integer,
    license text,
    copyright_holder text,
    landing_page text,
    lccn text,
    oclc text,
    short_abstract text,
    long_abstract text,
    general_note text,
    toc text,
    cover_url text,
    cover_caption text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    first_page text,
    last_page text,
    page_interval text,
    updated_at_with_relations timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    bibliography_note text,
    withdrawn_date date,
    CONSTRAINT work_active_publication_date_check CHECK ((((work_status = ANY (ARRAY['active'::public.work_status, 'withdrawn'::public.work_status, 'superseded'::public.work_status])) AND (publication_date IS NOT NULL)) OR (work_status <> ALL (ARRAY['active'::public.work_status, 'withdrawn'::public.work_status, 'superseded'::public.work_status])))),
    CONSTRAINT work_active_withdrawn_date_check CHECK (((work_status = 'withdrawn'::public.work_status) OR (work_status = 'superseded'::public.work_status) OR ((work_status <> ALL (ARRAY['withdrawn'::public.work_status, 'superseded'::public.work_status])) AND (withdrawn_date IS NULL)))),
    CONSTRAINT work_audio_count_check CHECK ((audio_count >= 0)),
    CONSTRAINT work_bibliography_note_check CHECK ((octet_length(bibliography_note) >= 1)),
    CONSTRAINT work_chapter_no_edition CHECK (((edition IS NULL) OR (work_type <> 'book-chapter'::public.work_type))),
    CONSTRAINT work_chapter_no_lccn CHECK (((lccn IS NULL) OR (work_type <> 'book-chapter'::public.work_type))),
    CONSTRAINT work_chapter_no_oclc CHECK (((oclc IS NULL) OR (work_type <> 'book-chapter'::public.work_type))),
    CONSTRAINT work_chapter_no_toc CHECK (((toc IS NULL) OR (work_type <> 'book-chapter'::public.work_type))),
    CONSTRAINT work_copyright_holder_check CHECK ((octet_length(copyright_holder) >= 1)),
    CONSTRAINT work_cover_caption_check CHECK ((octet_length(cover_caption) >= 1)),
    CONSTRAINT work_cover_url_check CHECK ((cover_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT work_doi_check CHECK ((doi ~ '^https:\/\/doi\.org\/10\.\d{4,9}\/[-._;()\/:a-zA-Z0-9<>+[\]]+$'::text)),
    CONSTRAINT work_edition_check CHECK ((edition > 0)),
    CONSTRAINT work_first_page_check CHECK ((octet_length(first_page) >= 1)),
    CONSTRAINT work_full_title_check CHECK ((octet_length(full_title) >= 1)),
    CONSTRAINT work_general_note_check CHECK ((octet_length(general_note) >= 1)),
    CONSTRAINT work_image_count_check CHECK ((image_count >= 0)),
    CONSTRAINT work_inactive_no_withdrawn_date_check CHECK (((((work_status = 'withdrawn'::public.work_status) OR (work_status = 'superseded'::public.work_status)) AND (withdrawn_date IS NOT NULL)) OR (work_status <> ALL (ARRAY['withdrawn'::public.work_status, 'superseded'::public.work_status])))),
    CONSTRAINT work_landing_page_check CHECK ((landing_page ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT work_last_page_check CHECK ((octet_length(last_page) >= 1)),
    CONSTRAINT work_lccn_check CHECK ((octet_length(lccn) >= 1)),
    CONSTRAINT work_license_check CHECK ((license ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT work_long_abstract_check CHECK ((octet_length(long_abstract) >= 1)),
    CONSTRAINT work_non_chapter_has_edition CHECK (((edition IS NOT NULL) OR (work_type = 'book-chapter'::public.work_type))),
    CONSTRAINT work_non_chapter_no_first_page CHECK (((first_page IS NULL) OR (work_type = 'book-chapter'::public.work_type))),
    CONSTRAINT work_non_chapter_no_last_page CHECK (((last_page IS NULL) OR (work_type = 'book-chapter'::public.work_type))),
    CONSTRAINT work_non_chapter_no_page_interval CHECK (((page_interval IS NULL) OR (work_type = 'book-chapter'::public.work_type))),
    CONSTRAINT work_oclc_check CHECK ((octet_length(oclc) >= 1)),
    CONSTRAINT work_page_breakdown_check CHECK ((octet_length(page_breakdown) >= 1)),
    CONSTRAINT work_page_count_check CHECK ((page_count > 0)),
    CONSTRAINT work_page_interval_check CHECK ((octet_length(page_interval) >= 1)),
    CONSTRAINT work_place_check CHECK ((octet_length(place) >= 1)),
    CONSTRAINT work_reference_check CHECK ((octet_length(reference) >= 1)),
    CONSTRAINT work_short_abstract_check CHECK ((octet_length(short_abstract) >= 1)),
    CONSTRAINT work_subtitle_check CHECK ((octet_length(subtitle) >= 1)),
    CONSTRAINT work_table_count_check CHECK ((table_count >= 0)),
    CONSTRAINT work_title_check CHECK ((octet_length(title) >= 1)),
    CONSTRAINT work_toc_check CHECK ((octet_length(toc) >= 1)),
    CONSTRAINT work_video_count_check CHECK ((video_count >= 0)),
    CONSTRAINT work_withdrawn_date_after_publication_date_check CHECK (((withdrawn_date IS NULL) OR (publication_date < withdrawn_date)))
);


--
-- Name: work_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_history (
    work_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: work_relation; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_relation (
    work_relation_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    relator_work_id uuid NOT NULL,
    related_work_id uuid NOT NULL,
    relation_type public.relation_type NOT NULL,
    relation_ordinal integer NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT work_relation_ids_check CHECK ((relator_work_id <> related_work_id)),
    CONSTRAINT work_relation_relation_ordinal_check CHECK ((relation_ordinal > 0))
);


--
-- Name: work_relation_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_relation_history (
    work_relation_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_relation_id uuid NOT NULL,
    account_id uuid NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: __diesel_schema_migrations __diesel_schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.__diesel_schema_migrations
    ADD CONSTRAINT __diesel_schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: account account_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.account
    ADD CONSTRAINT account_pkey PRIMARY KEY (account_id);


--
-- Name: affiliation_history affiliation_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.affiliation_history
    ADD CONSTRAINT affiliation_history_pkey PRIMARY KEY (affiliation_history_id);


--
-- Name: affiliation affiliation_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.affiliation
    ADD CONSTRAINT affiliation_pkey PRIMARY KEY (affiliation_id);


--
-- Name: contribution contribution_contribution_ordinal_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution
    ADD CONSTRAINT contribution_contribution_ordinal_work_id_uniq UNIQUE (contribution_ordinal, work_id);


--
-- Name: contribution_history contribution_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution_history
    ADD CONSTRAINT contribution_history_pkey PRIMARY KEY (contribution_history_id);


--
-- Name: contribution contribution_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution
    ADD CONSTRAINT contribution_pkey PRIMARY KEY (contribution_id);


--
-- Name: contribution contribution_work_id_contributor_id_contribution_type_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution
    ADD CONSTRAINT contribution_work_id_contributor_id_contribution_type_uniq UNIQUE (work_id, contributor_id, contribution_type);


--
-- Name: contributor_history contributor_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contributor_history
    ADD CONSTRAINT contributor_history_pkey PRIMARY KEY (contributor_history_id);


--
-- Name: contributor contributor_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contributor
    ADD CONSTRAINT contributor_pkey PRIMARY KEY (contributor_id);


--
-- Name: institution_history funder_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.institution_history
    ADD CONSTRAINT funder_history_pkey PRIMARY KEY (institution_history_id);


--
-- Name: funding_history funding_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funding_history
    ADD CONSTRAINT funding_history_pkey PRIMARY KEY (funding_history_id);


--
-- Name: funding funding_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funding
    ADD CONSTRAINT funding_pkey PRIMARY KEY (funding_id);


--
-- Name: imprint_history imprint_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.imprint_history
    ADD CONSTRAINT imprint_history_pkey PRIMARY KEY (imprint_history_id);


--
-- Name: imprint imprint_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.imprint
    ADD CONSTRAINT imprint_pkey PRIMARY KEY (imprint_id);


--
-- Name: institution institution_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.institution
    ADD CONSTRAINT institution_pkey PRIMARY KEY (institution_id);


--
-- Name: issue_history issue_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.issue_history
    ADD CONSTRAINT issue_history_pkey PRIMARY KEY (issue_history_id);


--
-- Name: issue issue_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.issue
    ADD CONSTRAINT issue_pkey PRIMARY KEY (issue_id);


--
-- Name: issue issue_series_id_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.issue
    ADD CONSTRAINT issue_series_id_work_id_uniq UNIQUE (series_id, work_id);


--
-- Name: language_history language_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.language_history
    ADD CONSTRAINT language_history_pkey PRIMARY KEY (language_history_id);


--
-- Name: language language_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.language
    ADD CONSTRAINT language_pkey PRIMARY KEY (language_id);


--
-- Name: location_history location_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.location_history
    ADD CONSTRAINT location_history_pkey PRIMARY KEY (location_history_id);


--
-- Name: location location_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.location
    ADD CONSTRAINT location_pkey PRIMARY KEY (location_id);


--
-- Name: price_history price_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.price_history
    ADD CONSTRAINT price_history_pkey PRIMARY KEY (price_history_id);


--
-- Name: price price_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.price
    ADD CONSTRAINT price_pkey PRIMARY KEY (price_id);


--
-- Name: price price_publication_id_currency_code_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.price
    ADD CONSTRAINT price_publication_id_currency_code_uniq UNIQUE (publication_id, currency_code);


--
-- Name: publication_history publication_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publication_history
    ADD CONSTRAINT publication_history_pkey PRIMARY KEY (publication_history_id);


--
-- Name: publication publication_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publication
    ADD CONSTRAINT publication_pkey PRIMARY KEY (publication_id);


--
-- Name: publication publication_publication_type_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publication
    ADD CONSTRAINT publication_publication_type_work_id_uniq UNIQUE (publication_type, work_id);


--
-- Name: publisher_account publisher_account_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publisher_account
    ADD CONSTRAINT publisher_account_pkey PRIMARY KEY (account_id, publisher_id);


--
-- Name: publisher_history publisher_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publisher_history
    ADD CONSTRAINT publisher_history_pkey PRIMARY KEY (publisher_history_id);


--
-- Name: publisher publisher_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publisher
    ADD CONSTRAINT publisher_pkey PRIMARY KEY (publisher_id);


--
-- Name: reference_history reference_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reference_history
    ADD CONSTRAINT reference_history_pkey PRIMARY KEY (reference_history_id);


--
-- Name: reference reference_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reference
    ADD CONSTRAINT reference_pkey PRIMARY KEY (reference_id);


--
-- Name: reference reference_reference_ordinal_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reference
    ADD CONSTRAINT reference_reference_ordinal_work_id_uniq UNIQUE (work_id, reference_ordinal);


--
-- Name: series_history series_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series_history
    ADD CONSTRAINT series_history_pkey PRIMARY KEY (series_history_id);


--
-- Name: series series_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series
    ADD CONSTRAINT series_pkey PRIMARY KEY (series_id);


--
-- Name: subject_history subject_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subject_history
    ADD CONSTRAINT subject_history_pkey PRIMARY KEY (subject_history_id);


--
-- Name: subject subject_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subject
    ADD CONSTRAINT subject_pkey PRIMARY KEY (subject_id);


--
-- Name: work_history work_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_history
    ADD CONSTRAINT work_history_pkey PRIMARY KEY (work_history_id);


--
-- Name: work work_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work
    ADD CONSTRAINT work_pkey PRIMARY KEY (work_id);


--
-- Name: work_relation_history work_relation_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation_history
    ADD CONSTRAINT work_relation_history_pkey PRIMARY KEY (work_relation_history_id);


--
-- Name: work_relation work_relation_ordinal_type_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation
    ADD CONSTRAINT work_relation_ordinal_type_uniq UNIQUE (relation_ordinal, relator_work_id, relation_type);


--
-- Name: work_relation work_relation_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation
    ADD CONSTRAINT work_relation_pkey PRIMARY KEY (work_relation_id);


--
-- Name: work_relation work_relation_relator_related_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation
    ADD CONSTRAINT work_relation_relator_related_uniq UNIQUE (relator_work_id, related_work_id);


--
-- Name: affiliation_uniq_ord_in_contribution_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX affiliation_uniq_ord_in_contribution_idx ON public.affiliation USING btree (contribution_id, affiliation_ordinal);


--
-- Name: doi_uniq_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX doi_uniq_idx ON public.work USING btree (lower(doi));


--
-- Name: email_uniq_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX email_uniq_idx ON public.account USING btree (lower(email));


--
-- Name: idx_account_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_account_email ON public.account USING btree (email);


--
-- Name: idx_affiliation_contribution_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_affiliation_contribution_id ON public.affiliation USING btree (contribution_id);


--
-- Name: idx_affiliation_ordinal_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_affiliation_ordinal_asc ON public.affiliation USING btree (affiliation_ordinal, contribution_id);


--
-- Name: idx_contribution_contributor_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_contribution_contributor_id ON public.contribution USING btree (contributor_id);


--
-- Name: idx_contribution_ordinal_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_contribution_ordinal_asc ON public.contribution USING btree (contribution_ordinal, work_id);


--
-- Name: idx_contribution_work_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_contribution_work_id ON public.contribution USING btree (work_id);


--
-- Name: idx_contributor_full_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_contributor_full_name ON public.contributor USING btree (full_name);


--
-- Name: idx_contributor_last_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_contributor_last_name ON public.contributor USING btree (last_name);


--
-- Name: idx_contributor_orcid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_contributor_orcid ON public.contributor USING btree (orcid);


--
-- Name: idx_funding_program; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_funding_program ON public.funding USING btree (program);


--
-- Name: idx_funding_work_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_funding_work_id ON public.funding USING btree (work_id);


--
-- Name: idx_imprint_imprint_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_imprint_imprint_name ON public.imprint USING btree (imprint_name);


--
-- Name: idx_imprint_imprint_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_imprint_imprint_url ON public.imprint USING btree (imprint_url);


--
-- Name: idx_imprint_publisher_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_imprint_publisher_id ON public.imprint USING btree (publisher_id);


--
-- Name: idx_institution_institution_doi; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_institution_institution_doi ON public.institution USING btree (institution_doi);


--
-- Name: idx_institution_institution_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_institution_institution_name ON public.institution USING btree (institution_name);


--
-- Name: idx_institution_ror; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_institution_ror ON public.institution USING btree (ror);


--
-- Name: idx_issue_ordinal_series_id_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_issue_ordinal_series_id_asc ON public.issue USING btree (issue_ordinal, series_id);


--
-- Name: idx_issue_ordinal_work_id_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_issue_ordinal_work_id_asc ON public.issue USING btree (issue_ordinal, work_id);


--
-- Name: idx_language_language_code_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_language_language_code_asc ON public.language USING btree (language_code, work_id);


--
-- Name: idx_location_location_platform_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_location_location_platform_asc ON public.location USING btree (location_platform, publication_id);


--
-- Name: idx_price_currency_code_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_price_currency_code_asc ON public.price USING btree (currency_code, publication_id);


--
-- Name: idx_publication_isbn; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_publication_isbn ON public.publication USING btree (isbn);


--
-- Name: idx_publication_publication_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_publication_publication_type ON public.publication USING btree (publication_type);


--
-- Name: idx_publication_work_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_publication_work_id ON public.publication USING btree (work_id);


--
-- Name: idx_publisher_account_account_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_publisher_account_account_id ON public.publisher_account USING btree (account_id);


--
-- Name: idx_publisher_publisher_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_publisher_publisher_name ON public.publisher USING btree (publisher_name);


--
-- Name: idx_publisher_publisher_shortname; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_publisher_publisher_shortname ON public.publisher USING btree (publisher_shortname);


--
-- Name: idx_reference_article_title; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_article_title ON public.reference USING btree (article_title);


--
-- Name: idx_reference_author_substr; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_author_substr ON public.reference USING btree ("substring"(author, 1, 255));


--
-- Name: idx_reference_doi; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_doi ON public.reference USING btree (doi);


--
-- Name: idx_reference_isbn; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_isbn ON public.reference USING btree (isbn);


--
-- Name: idx_reference_issn; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_issn ON public.reference USING btree (issn);


--
-- Name: idx_reference_journal_title; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_journal_title ON public.reference USING btree (journal_title);


--
-- Name: idx_reference_series_title; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_series_title ON public.reference USING btree (series_title);


--
-- Name: idx_reference_standard_designator; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_standard_designator ON public.reference USING btree (standard_designator);


--
-- Name: idx_reference_standards_body_acronym; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_standards_body_acronym ON public.reference USING btree (standards_body_acronym);


--
-- Name: idx_reference_standards_body_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_standards_body_name ON public.reference USING btree (standards_body_name);


--
-- Name: idx_reference_unstructured_citation; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_unstructured_citation ON public.reference USING btree (unstructured_citation);


--
-- Name: idx_reference_volume_title; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_volume_title ON public.reference USING btree (volume_title);


--
-- Name: idx_reference_work_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_work_id ON public.reference USING btree (work_id);


--
-- Name: idx_series_imprint_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_imprint_id ON public.series USING btree (imprint_id);


--
-- Name: idx_series_issn_digital; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_issn_digital ON public.series USING btree (issn_digital);


--
-- Name: idx_series_issn_print; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_issn_print ON public.series USING btree (issn_print);


--
-- Name: idx_series_series_description; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_series_description ON public.series USING btree (series_description);


--
-- Name: idx_series_series_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_series_name ON public.series USING btree (series_name);


--
-- Name: idx_series_series_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_series_series_url ON public.series USING btree (series_url);


--
-- Name: idx_subject_subject_code_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_subject_subject_code_asc ON public.subject USING btree (subject_code, work_id);


--
-- Name: idx_subject_subject_ordinal_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_subject_subject_ordinal_asc ON public.subject USING btree (subject_ordinal, work_id);


--
-- Name: idx_work_books_pub_date_desc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_books_pub_date_desc ON public.work USING btree (publication_date DESC) WHERE ((work_type = ANY (ARRAY['monograph'::public.work_type, 'edited-book'::public.work_type, 'textbook'::public.work_type])) AND (work_status = 'active'::public.work_status));


--
-- Name: idx_work_doi; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_doi ON public.work USING btree (doi);


--
-- Name: idx_work_full_title_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_full_title_asc ON public.work USING btree (full_title, work_id);


--
-- Name: idx_work_imprint_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_imprint_id ON public.work USING btree (imprint_id);


--
-- Name: idx_work_landing_page; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_landing_page ON public.work USING btree (landing_page);


--
-- Name: idx_work_long_abstract_substr; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_long_abstract_substr ON public.work USING btree ("substring"(long_abstract, 1, 255));


--
-- Name: idx_work_publication_date_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_publication_date_asc ON public.work USING btree (publication_date, work_id);


--
-- Name: idx_work_publication_date_desc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_publication_date_desc ON public.work USING btree (publication_date DESC, work_id);


--
-- Name: idx_work_reference; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_reference ON public.work USING btree (reference);


--
-- Name: idx_work_relation_relation_ordinal_related_relation_type_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_relation_relation_ordinal_related_relation_type_asc ON public.work_relation USING btree (relation_ordinal, related_work_id, relation_type);


--
-- Name: idx_work_relation_relation_ordinal_relator_relation_type_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_relation_relation_ordinal_relator_relation_type_asc ON public.work_relation USING btree (relation_ordinal, relator_work_id, relation_type);


--
-- Name: idx_work_short_abstract_substr; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_short_abstract_substr ON public.work USING btree ("substring"(short_abstract, 1, 255));


--
-- Name: idx_work_type_status_pub_date_desc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_type_status_pub_date_desc ON public.work USING btree (work_type, work_status, publication_date DESC);


--
-- Name: idx_work_updated_at_with_relations_desc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_updated_at_with_relations_desc ON public.work USING btree (updated_at_with_relations DESC, work_id);


--
-- Name: imprint_uniq_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX imprint_uniq_idx ON public.imprint USING btree (lower(imprint_name));


--
-- Name: institution_doi_uniq_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX institution_doi_uniq_idx ON public.institution USING btree (lower(institution_doi));


--
-- Name: issue_uniq_ord_in_series_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX issue_uniq_ord_in_series_idx ON public.issue USING btree (series_id, issue_ordinal);


--
-- Name: language_uniq_work_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX language_uniq_work_idx ON public.language USING btree (work_id, language_code);


--
-- Name: location_uniq_canonical_true_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX location_uniq_canonical_true_idx ON public.location USING btree (publication_id) WHERE canonical;


--
-- Name: location_uniq_platform_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX location_uniq_platform_idx ON public.location USING btree (publication_id, location_platform) WHERE (NOT (location_platform = 'Other'::public.location_platform));


--
-- Name: orcid_uniq_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX orcid_uniq_idx ON public.contributor USING btree (lower(orcid));


--
-- Name: publication_isbn_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX publication_isbn_idx ON public.publication USING btree (isbn);


--
-- Name: publisher_uniq_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX publisher_uniq_idx ON public.publisher USING btree (lower(publisher_name));


--
-- Name: series_issn_digital_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX series_issn_digital_idx ON public.series USING btree (issn_digital);


--
-- Name: series_issn_print_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX series_issn_print_idx ON public.series USING btree (issn_print);


--
-- Name: publication publication_chapter_no_dimensions_check; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER publication_chapter_no_dimensions_check BEFORE INSERT OR UPDATE ON public.publication FOR EACH ROW EXECUTE FUNCTION public.publication_chapter_no_dimensions();


--
-- Name: publication publication_location_canonical_urls_check; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER publication_location_canonical_urls_check BEFORE UPDATE ON public.publication FOR EACH ROW EXECUTE FUNCTION public.publication_location_canonical_urls();


--
-- Name: account set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.account FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: affiliation set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.affiliation FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: contribution set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.contribution FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: contributor set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.contributor FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: funding set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.funding FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: imprint set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.imprint FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: institution set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.institution FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: issue set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.issue FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: language set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.language FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: location set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.location FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: price set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.price FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: publication set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.publication FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: publisher set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.publisher FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: publisher_account set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.publisher_account FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: reference set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.reference FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: series set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.series FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: subject set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.subject FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: work set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.work FOR EACH ROW EXECUTE FUNCTION public.work_set_updated_at();


--
-- Name: work_relation set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.work_relation FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: affiliation set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.affiliation FOR EACH ROW EXECUTE FUNCTION public.affiliation_work_updated_at_with_relations();


--
-- Name: contribution set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.contribution FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: contributor set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER UPDATE ON public.contributor FOR EACH ROW EXECUTE FUNCTION public.contributor_work_updated_at_with_relations();


--
-- Name: funding set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.funding FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: imprint set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER UPDATE ON public.imprint FOR EACH ROW EXECUTE FUNCTION public.imprint_work_updated_at_with_relations();


--
-- Name: institution set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER UPDATE ON public.institution FOR EACH ROW EXECUTE FUNCTION public.institution_work_updated_at_with_relations();


--
-- Name: issue set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.issue FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: language set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.language FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: location set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.location FOR EACH ROW EXECUTE FUNCTION public.location_work_updated_at_with_relations();


--
-- Name: price set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.price FOR EACH ROW EXECUTE FUNCTION public.price_work_updated_at_with_relations();


--
-- Name: publication set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.publication FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: publisher set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER UPDATE ON public.publisher FOR EACH ROW EXECUTE FUNCTION public.publisher_work_updated_at_with_relations();


--
-- Name: reference set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.reference FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: series set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER UPDATE ON public.series FOR EACH ROW EXECUTE FUNCTION public.series_work_updated_at_with_relations();


--
-- Name: subject set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.subject FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: work set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER UPDATE ON public.work FOR EACH ROW EXECUTE FUNCTION public.work_work_updated_at_with_relations();


--
-- Name: work_relation set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.work_relation FOR EACH ROW EXECUTE FUNCTION public.work_relation_work_updated_at_with_relations();


--
-- Name: affiliation affiliation_contribution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.affiliation
    ADD CONSTRAINT affiliation_contribution_id_fkey FOREIGN KEY (contribution_id) REFERENCES public.contribution(contribution_id) ON DELETE CASCADE;


--
-- Name: affiliation_history affiliation_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.affiliation_history
    ADD CONSTRAINT affiliation_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: affiliation_history affiliation_history_affiliation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.affiliation_history
    ADD CONSTRAINT affiliation_history_affiliation_id_fkey FOREIGN KEY (affiliation_id) REFERENCES public.affiliation(affiliation_id) ON DELETE CASCADE;


--
-- Name: affiliation affiliation_institution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.affiliation
    ADD CONSTRAINT affiliation_institution_id_fkey FOREIGN KEY (institution_id) REFERENCES public.institution(institution_id) ON DELETE CASCADE;


--
-- Name: contribution contribution_contributor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution
    ADD CONSTRAINT contribution_contributor_id_fkey FOREIGN KEY (contributor_id) REFERENCES public.contributor(contributor_id) ON DELETE CASCADE;


--
-- Name: contribution_history contribution_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution_history
    ADD CONSTRAINT contribution_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: contribution_history contribution_history_contribution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution_history
    ADD CONSTRAINT contribution_history_contribution_id_fkey FOREIGN KEY (contribution_id) REFERENCES public.contribution(contribution_id) ON DELETE CASCADE;


--
-- Name: contribution contribution_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution
    ADD CONSTRAINT contribution_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: contributor_history contributor_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contributor_history
    ADD CONSTRAINT contributor_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: contributor_history contributor_history_contributor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contributor_history
    ADD CONSTRAINT contributor_history_contributor_id_fkey FOREIGN KEY (contributor_id) REFERENCES public.contributor(contributor_id) ON DELETE CASCADE;


--
-- Name: institution_history funder_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.institution_history
    ADD CONSTRAINT funder_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: institution_history funder_history_funder_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.institution_history
    ADD CONSTRAINT funder_history_funder_id_fkey FOREIGN KEY (institution_id) REFERENCES public.institution(institution_id) ON DELETE CASCADE;


--
-- Name: funding funding_funder_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funding
    ADD CONSTRAINT funding_funder_id_fkey FOREIGN KEY (institution_id) REFERENCES public.institution(institution_id) ON DELETE CASCADE;


--
-- Name: funding_history funding_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funding_history
    ADD CONSTRAINT funding_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: funding_history funding_history_funding_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funding_history
    ADD CONSTRAINT funding_history_funding_id_fkey FOREIGN KEY (funding_id) REFERENCES public.funding(funding_id) ON DELETE CASCADE;


--
-- Name: funding funding_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funding
    ADD CONSTRAINT funding_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: imprint_history imprint_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.imprint_history
    ADD CONSTRAINT imprint_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: imprint_history imprint_history_imprint_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.imprint_history
    ADD CONSTRAINT imprint_history_imprint_id_fkey FOREIGN KEY (imprint_id) REFERENCES public.imprint(imprint_id) ON DELETE CASCADE;


--
-- Name: imprint imprint_publisher_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.imprint
    ADD CONSTRAINT imprint_publisher_id_fkey FOREIGN KEY (publisher_id) REFERENCES public.publisher(publisher_id) ON DELETE CASCADE;


--
-- Name: issue_history issue_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.issue_history
    ADD CONSTRAINT issue_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: issue_history issue_history_issue_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.issue_history
    ADD CONSTRAINT issue_history_issue_id_fkey FOREIGN KEY (issue_id) REFERENCES public.issue(issue_id) ON DELETE CASCADE;


--
-- Name: issue issue_series_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.issue
    ADD CONSTRAINT issue_series_id_fkey FOREIGN KEY (series_id) REFERENCES public.series(series_id) ON DELETE CASCADE;


--
-- Name: issue issue_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.issue
    ADD CONSTRAINT issue_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: language_history language_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.language_history
    ADD CONSTRAINT language_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: language_history language_history_language_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.language_history
    ADD CONSTRAINT language_history_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(language_id) ON DELETE CASCADE;


--
-- Name: language language_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.language
    ADD CONSTRAINT language_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: location_history location_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.location_history
    ADD CONSTRAINT location_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: location_history location_history_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.location_history
    ADD CONSTRAINT location_history_location_id_fkey FOREIGN KEY (location_id) REFERENCES public.location(location_id) ON DELETE CASCADE;


--
-- Name: location location_publication_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.location
    ADD CONSTRAINT location_publication_id_fkey FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id) ON DELETE CASCADE;


--
-- Name: price_history price_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.price_history
    ADD CONSTRAINT price_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: price_history price_history_price_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.price_history
    ADD CONSTRAINT price_history_price_id_fkey FOREIGN KEY (price_id) REFERENCES public.price(price_id) ON DELETE CASCADE;


--
-- Name: price price_publication_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.price
    ADD CONSTRAINT price_publication_id_fkey FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id) ON DELETE CASCADE;


--
-- Name: publication_history publication_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publication_history
    ADD CONSTRAINT publication_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: publication_history publication_history_publication_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publication_history
    ADD CONSTRAINT publication_history_publication_id_fkey FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id) ON DELETE CASCADE;


--
-- Name: publication publication_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publication
    ADD CONSTRAINT publication_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: publisher_account publisher_account_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publisher_account
    ADD CONSTRAINT publisher_account_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id) ON DELETE CASCADE;


--
-- Name: publisher_account publisher_account_publisher_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publisher_account
    ADD CONSTRAINT publisher_account_publisher_id_fkey FOREIGN KEY (publisher_id) REFERENCES public.publisher(publisher_id) ON DELETE CASCADE;


--
-- Name: publisher_history publisher_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publisher_history
    ADD CONSTRAINT publisher_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: publisher_history publisher_history_publisher_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publisher_history
    ADD CONSTRAINT publisher_history_publisher_id_fkey FOREIGN KEY (publisher_id) REFERENCES public.publisher(publisher_id) ON DELETE CASCADE;


--
-- Name: reference_history reference_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reference_history
    ADD CONSTRAINT reference_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: reference_history reference_history_reference_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reference_history
    ADD CONSTRAINT reference_history_reference_id_fkey FOREIGN KEY (reference_id) REFERENCES public.reference(reference_id) ON DELETE CASCADE;


--
-- Name: reference reference_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reference
    ADD CONSTRAINT reference_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: series_history series_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series_history
    ADD CONSTRAINT series_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: series_history series_history_series_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series_history
    ADD CONSTRAINT series_history_series_id_fkey FOREIGN KEY (series_id) REFERENCES public.series(series_id) ON DELETE CASCADE;


--
-- Name: series series_imprint_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.series
    ADD CONSTRAINT series_imprint_id_fkey FOREIGN KEY (imprint_id) REFERENCES public.imprint(imprint_id) ON DELETE CASCADE;


--
-- Name: subject_history subject_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subject_history
    ADD CONSTRAINT subject_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: subject_history subject_history_subject_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subject_history
    ADD CONSTRAINT subject_history_subject_id_fkey FOREIGN KEY (subject_id) REFERENCES public.subject(subject_id) ON DELETE CASCADE;


--
-- Name: subject subject_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subject
    ADD CONSTRAINT subject_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: work_history work_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_history
    ADD CONSTRAINT work_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: work_history work_history_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_history
    ADD CONSTRAINT work_history_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: work work_imprint_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work
    ADD CONSTRAINT work_imprint_id_fkey FOREIGN KEY (imprint_id) REFERENCES public.imprint(imprint_id) ON DELETE CASCADE;


--
-- Name: work_relation work_relation_active_passive_pair; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation
    ADD CONSTRAINT work_relation_active_passive_pair FOREIGN KEY (relator_work_id, related_work_id) REFERENCES public.work_relation(related_work_id, relator_work_id) DEFERRABLE INITIALLY DEFERRED;


--
-- Name: work_relation_history work_relation_history_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation_history
    ADD CONSTRAINT work_relation_history_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.account(account_id);


--
-- Name: work_relation_history work_relation_history_work_relation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation_history
    ADD CONSTRAINT work_relation_history_work_relation_id_fkey FOREIGN KEY (work_relation_id) REFERENCES public.work_relation(work_relation_id) ON DELETE CASCADE;


--
-- Name: work_relation work_relation_related_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation
    ADD CONSTRAINT work_relation_related_work_id_fkey FOREIGN KEY (related_work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: work_relation work_relation_relator_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_relation
    ADD CONSTRAINT work_relation_relator_work_id_fkey FOREIGN KEY (relator_work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;

