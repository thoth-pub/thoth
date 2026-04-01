--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: abstract_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.abstract_type AS ENUM (
    'short',
    'long'
);


--
-- Name: accessibility_exception; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.accessibility_exception AS ENUM (
    'micro-enterprises',
    'disproportionate-burden',
    'fundamental-alteration'
);


--
-- Name: accessibility_standard; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.accessibility_standard AS ENUM (
    'wcag-21-aa',
    'wcag-21-aaa',
    'wcag-22-aa',
    'wcag-22-aaa',
    'epub-a11y-10-aa',
    'epub-a11y-10-aaa',
    'epub-a11y-11-aa',
    'epub-a11y-11-aaa',
    'pdf-ua-1',
    'pdf-ua-2'
);


--
-- Name: award_role; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.award_role AS ENUM (
    'SHORT_LISTED',
    'WINNER',
    'LONG_LISTED',
    'COMMENDED',
    'RUNNER_UP',
    'JOINT_WINNER',
    'NOMINATED'
);


--
-- Name: contact_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.contact_type AS ENUM (
    'Accessibility'
);


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
-- Name: file_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.file_type AS ENUM (
    'publication',
    'frontcover',
    'additional_resource',
    'work_featured_video'
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
-- Name: locale_code; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.locale_code AS ENUM (
    'af',
    'af_na',
    'af_za',
    'agq',
    'agq_cm',
    'ak',
    'ak_gh',
    'sq',
    'sq_al',
    'am',
    'am_et',
    'aig',
    'ar',
    'ar_dz',
    'ar_bh',
    'ar_eg',
    'ar_iq',
    'ar_jo',
    'ar_kw',
    'ar_lb',
    'ar_ly',
    'ar_ma',
    'ar_om',
    'ar_qa',
    'ar_sa',
    'ar_sd',
    'ar_sy',
    'ar_tn',
    'ar_ae',
    'ar_001',
    'ar_ye',
    'hy',
    'hy_am',
    'as',
    'as_in',
    'ast',
    'ast_es',
    'asa',
    'asa_tz',
    'az',
    'az_cyrl',
    'az_cyrl_az',
    'az_latn',
    'az_latn_az',
    'ksf',
    'ksf_cm',
    'bah',
    'bm',
    'bm_ml',
    'bas',
    'bas_cm',
    'eu',
    'eu_es',
    'be',
    'be_by',
    'bem',
    'bem_zm',
    'bez',
    'bez_tz',
    'bn',
    'bn_bd',
    'bn_in',
    'brx',
    'brx_in',
    'bs',
    'bs_ba',
    'br',
    'br_fr',
    'bg',
    'bg_bg',
    'my',
    'my_mm',
    'ca',
    'ca_es',
    'ckb',
    'kmr',
    'sdh',
    'tzm',
    'tzm_latn',
    'tzm_latn_ma',
    'chr',
    'chr_us',
    'cgg',
    'cgg_ug',
    'zh',
    'zh_hans',
    'zh_cn',
    'zh_hans_cn',
    'zh_hans_hk',
    'zh_hans_mo',
    'zh_hans_sg',
    'zh_hant',
    'zh_hant_hk',
    'zh_hant_mo',
    'zh_hant_tw',
    'swc',
    'swc_cd',
    'kw',
    'kw_gb',
    'hr',
    'hr_hr',
    'cs',
    'cs_cz',
    'da',
    'da_dk',
    'dua',
    'dua_cm',
    'dv',
    'nl',
    'nl_aw',
    'nl_be',
    'nl_cw',
    'nl_nl',
    'nl_sx',
    'ebu',
    'ebu_ke',
    'en',
    'en_ai',
    'en_as',
    'en_au',
    'en_at',
    'en_bb',
    'en_be',
    'en_bz',
    'en_bm',
    'en_bw',
    'en_io',
    'en_bi',
    'en_cm',
    'en_ca',
    'en_ky',
    'en_cx',
    'en_cc',
    'en_ck',
    'en_cy',
    'en_dk',
    'en_dg',
    'en_dm',
    'en_eg',
    'en_er',
    'en_eu',
    'en_fk',
    'en_fj',
    'en_fi',
    'en_gm',
    'en_de',
    'en_gh',
    'en_gi',
    'en_gd',
    'en_gu',
    'en_gg',
    'en_gy',
    'en_hk',
    'en_in',
    'en_ie',
    'en_im',
    'en_il',
    'en_jm',
    'en_je',
    'en_ke',
    'en_ki',
    'en_kw',
    'en_ls',
    'en_mo',
    'en_mg',
    'en_mw',
    'en_my',
    'en_mt',
    'en_mh',
    'en_mu',
    'en_fm',
    'en_ms',
    'en_na',
    'en_nr',
    'en_nl',
    'en_nz',
    'en_ng',
    'en_nu',
    'en_nf',
    'en_mp',
    'en_no',
    'en_pa',
    'en_pk',
    'en_pw',
    'en_pg',
    'en_ph',
    'en_pn',
    'en_pr',
    'en_rw',
    'en_ws',
    'en_sa',
    'en_sc',
    'en_sl',
    'en_sg',
    'en_sx',
    'en_si',
    'en_sb',
    'en_ss',
    'en_sh',
    'en_kn',
    'en_lc',
    'svc',
    'vic',
    'en_sd',
    'en_sz',
    'en_se',
    'en_ch',
    'en_tz',
    'en_tk',
    'en_to',
    'en_tt',
    'en_tv',
    'en_za',
    'en_ae',
    'en_um',
    'en_vi',
    'en_us_posix',
    'en_ug',
    'en_gb',
    'en_us',
    'en_vu',
    'en_zm',
    'en_zw',
    'eo',
    'et',
    'et_ee',
    'ee',
    'ee_gh',
    'ee_tg',
    'ewo',
    'ewo_cm',
    'fo',
    'fo_fo',
    'fil',
    'fil_ph',
    'fi',
    'fi_fi',
    'fr',
    'fr_be',
    'fr_bj',
    'fr_bf',
    'fr_bi',
    'fr_cm',
    'fr_ca',
    'fr_cf',
    'fr_td',
    'fr_km',
    'fr_cg',
    'fr_cd',
    'fr_ci',
    'fr_dj',
    'fr_gq',
    'fr_fr',
    'fr_gf',
    'fr_ga',
    'fr_gp',
    'fr_gn',
    'fr_lu',
    'fr_mg',
    'fr_ml',
    'fr_mq',
    'fr_yt',
    'fr_mc',
    'fr_ne',
    'fr_rw',
    'fr_re',
    'fr_bl',
    'fr_mf',
    'fr_mu',
    'fr_sn',
    'fr_ch',
    'fr_tg',
    'ff',
    'ff_sn',
    'gl',
    'gl_es',
    'lao',
    'lg',
    'lg_ug',
    'ka',
    'ka_ge',
    'de',
    'de_at',
    'de_be',
    'de_de',
    'de_li',
    'de_lu',
    'de_ch',
    'el',
    'el_cy',
    'el_gr',
    'gu',
    'gu_in',
    'guz',
    'guz_ke',
    'ha',
    'ha_latn',
    'ha_latn_gh',
    'ha_latn_ne',
    'ha_latn_ng',
    'haw',
    'haw_us',
    'he',
    'he_il',
    'hi',
    'hi_in',
    'hu',
    'hu_hu',
    'is',
    'is_is',
    'ig',
    'ig_ng',
    'smn',
    'smn_fi',
    'id',
    'id_id',
    'ga',
    'ga_ie',
    'it',
    'it_it',
    'it_ch',
    'ja',
    'ja_jp',
    'dyo',
    'dyo_sn',
    'kea',
    'kea_cv',
    'kab',
    'kab_dz',
    'kl',
    'kl_gl',
    'kln',
    'kln_ke',
    'kam',
    'kam_ke',
    'kn',
    'kn_in',
    'kaa',
    'kk',
    'kk_cyrl',
    'kk_cyrl_kz',
    'km',
    'km_kh',
    'ki',
    'ki_ke',
    'rw',
    'rw_rw',
    'kok',
    'kok_in',
    'ko',
    'ko_kr',
    'khq',
    'khq_ml',
    'ses',
    'ses_ml',
    'nmg',
    'nmg_cm',
    'ky',
    'lag',
    'lag_tz',
    'lv',
    'lv_lv',
    'lir',
    'ln',
    'ln_cg',
    'ln_cd',
    'lt',
    'lt_lt',
    'lu',
    'lu_cd',
    'luo',
    'luo_ke',
    'luy',
    'luy_ke',
    'mk',
    'mk_mk',
    'jmc',
    'jmc_tz',
    'mgh',
    'mgh_mz',
    'kde',
    'kde_tz',
    'mg',
    'mg_mg',
    'ms',
    'ms_bn',
    'ms_my',
    'ml',
    'ml_in',
    'mt',
    'mt_mt',
    'gv',
    'gv_gb',
    'mr',
    'mr_in',
    'mas',
    'mas_ke',
    'mas_tz',
    'mer',
    'mer_ke',
    'mn',
    'mfe',
    'mfe_mu',
    'mua',
    'mua_cm',
    'naq',
    'naq_na',
    'ne',
    'ne_in',
    'ne_np',
    'se',
    'se_fi',
    'se_no',
    'se_se',
    'nd',
    'nd_zw',
    'nb',
    'nb_no',
    'nn',
    'nn_no',
    'nus',
    'nus_sd',
    'nyn',
    'nyn_ug',
    'or',
    'or_in',
    'om',
    'om_et',
    'om_ke',
    'ps',
    'ps_af',
    'fa',
    'fa_af',
    'fa_ir',
    'pl',
    'pl_pl',
    'pt',
    'pt_ao',
    'pt_br',
    'pt_gw',
    'pt_mz',
    'pt_pt',
    'pt_st',
    'pa',
    'pa_arab',
    'pa_arab_pk',
    'pa_guru',
    'pa_guru_in',
    'ro',
    'ro_md',
    'ro_ro',
    'rm',
    'rm_ch',
    'rof',
    'rof_tz',
    'rn',
    'rn_bi',
    'ru',
    'ru_md',
    'ru_ru',
    'ru_ua',
    'rwk',
    'rwk_tz',
    'saq',
    'saq_ke',
    'sg',
    'sg_cf',
    'sbp',
    'sbp_tz',
    'sa',
    'gd',
    'gd_gb',
    'seh',
    'seh_mz',
    'sr',
    'sr_cyrl',
    'sr_cyrl_ba',
    'sr_cyrl_me',
    'sr_cyrl_rs',
    'sr_latn',
    'sr_latn_ba',
    'sr_latn_me',
    'sr_latn_rs',
    'ksb',
    'ksb_tz',
    'sn',
    'sn_zw',
    'ii',
    'ii_cn',
    'si',
    'si_lk',
    'sk',
    'sk_sk',
    'sl',
    'sl_si',
    'xog',
    'xog_ug',
    'so',
    'so_dj',
    'so_et',
    'so_ke',
    'so_so',
    'es',
    'es_ar',
    'es_bo',
    'es_cl',
    'es_co',
    'es_cr',
    'es_do',
    'es_ec',
    'es_sv',
    'es_gq',
    'es_gt',
    'es_hn',
    'es_419',
    'es_mx',
    'es_ni',
    'es_pa',
    'es_py',
    'es_pe',
    'es_pr',
    'es_es',
    'es_us',
    'es_uy',
    'es_ve',
    'sw',
    'sw_ke',
    'sw_tz',
    'sv',
    'sv_fi',
    'sv_se',
    'gsw',
    'gsw_ch',
    'shi',
    'shi_latn',
    'shi_latn_ma',
    'shi_tfng',
    'shi_tfng_ma',
    'dav',
    'dav_ke',
    'tg',
    'ta',
    'ta_in',
    'ta_lk',
    'twq',
    'twq_ne',
    'mi',
    'te',
    'te_in',
    'teo',
    'teo_ke',
    'teo_ug',
    'th',
    'th_th',
    'bo',
    'bo_cn',
    'bo_in',
    'ti',
    'ti_er',
    'ti_et',
    'to',
    'to_to',
    'tr',
    'tk',
    'tr_tr',
    'tch',
    'uk',
    'uk_ua',
    'ur',
    'ur_in',
    'ur_pk',
    'ug',
    'ug_cn',
    'uz',
    'uz_arab',
    'uz_arab_af',
    'uz_cyrl',
    'uz_cyrl_uz',
    'uz_latn',
    'uz_latn_uz',
    'vai',
    'vai_latn',
    'vai_latn_lr',
    'vai_vaii',
    'vai_vaii_lr',
    'val',
    'val_es',
    'ca_es_valencia',
    'vi',
    'vi_vn',
    'vun',
    'vun_tz',
    'cy',
    'cy_gb',
    'wo',
    'xh',
    'yav',
    'yav_cm',
    'yo',
    'yo_ng',
    'dje',
    'dje_ne',
    'zu',
    'zu_za'
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
-- Name: resource_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.resource_type AS ENUM (
    'AUDIO',
    'VIDEO',
    'IMAGE',
    'BLOG',
    'WEBSITE',
    'DOCUMENT',
    'BOOK',
    'ARTICLE',
    'MAP',
    'SOURCE',
    'DATASET',
    'SPREADSHEET',
    'OTHER'
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
-- Name: biography_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.biography_work_updated_at_with_relations() RETURNS trigger
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
-- Name: file_upload_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.file_upload_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE work_id = OLD.work_id OR work_id = NEW.work_id;

        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM publication
        WHERE work.work_id = publication.work_id
            AND (publication.publication_id = OLD.publication_id OR publication.publication_id = NEW.publication_id);
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: file_work_updated_at_with_relations(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.file_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE work_id = OLD.work_id OR work_id = NEW.work_id;

        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM publication
        WHERE work.work_id = publication.work_id
            AND (publication.publication_id = OLD.publication_id OR publication.publication_id = NEW.publication_id);
    END IF;
    RETURN NULL;
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
DECLARE
    w1 uuid;  -- smaller work_id
    w2 uuid;  -- larger work_id
BEGIN
    -- If nothing really changed, skip
    IF NEW IS NOT DISTINCT FROM OLD THEN
        RETURN NULL;
    END IF;

    -- Determine the two work IDs involved in this relation
    IF TG_OP = 'DELETE' THEN
        w1 := LEAST(OLD.relator_work_id, OLD.related_work_id);
        w2 := GREATEST(OLD.relator_work_id, OLD.related_work_id);
    ELSE
        w1 := LEAST(NEW.relator_work_id, NEW.related_work_id);
        w2 := GREATEST(NEW.relator_work_id, NEW.related_work_id);
    END IF;

    -- Always lock/update in deterministic order: smaller ID first, then larger
    UPDATE work
    SET updated_at_with_relations = current_timestamp
    WHERE work_id = w1;

    IF w2 IS DISTINCT FROM w1 THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE work_id = w2;
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


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: abstract; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.abstract (
    abstract_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    content text NOT NULL,
    locale_code public.locale_code NOT NULL,
    abstract_type public.abstract_type DEFAULT 'short'::public.abstract_type NOT NULL,
    canonical boolean DEFAULT false NOT NULL,
    CONSTRAINT abstract_content_check CHECK ((octet_length(content) >= 1))
);


--
-- Name: abstract_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.abstract_history (
    abstract_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    abstract_id uuid NOT NULL,
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: additional_resource; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.additional_resource (
    additional_resource_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    title text NOT NULL,
    description text,
    attribution text,
    resource_type public.resource_type NOT NULL,
    doi text,
    handle text,
    url text,
    resource_ordinal integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    date date,
    CONSTRAINT additional_resource_resource_ordinal_check CHECK ((resource_ordinal > 0)),
    CONSTRAINT additional_resource_title_check CHECK ((octet_length(title) >= 1))
);


--
-- Name: additional_resource_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.additional_resource_history (
    additional_resource_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    additional_resource_id uuid NOT NULL,
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
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
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: award; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.award (
    award_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    title text NOT NULL,
    url text,
    category text,
    prize_statement text,
    award_ordinal integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    role public.award_role,
    year text,
    jury text,
    country public.country_code,
    CONSTRAINT award_award_ordinal_check CHECK ((award_ordinal > 0)),
    CONSTRAINT award_title_check CHECK ((octet_length(title) >= 1))
);


--
-- Name: award_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.award_history (
    award_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    award_id uuid NOT NULL,
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: biography; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.biography (
    biography_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    contribution_id uuid NOT NULL,
    content text NOT NULL,
    canonical boolean DEFAULT false NOT NULL,
    locale_code public.locale_code NOT NULL,
    CONSTRAINT biography_content_check CHECK ((octet_length(content) >= 1))
);


--
-- Name: biography_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.biography_history (
    biography_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    biography_id uuid NOT NULL,
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: book_review; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.book_review (
    book_review_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    title text,
    author_name text,
    url text,
    doi text,
    review_date date,
    journal_name text,
    journal_volume text,
    journal_number text,
    journal_issn text,
    text text,
    review_ordinal integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    reviewer_orcid text,
    reviewer_institution_id uuid,
    page_range text,
    CONSTRAINT book_review_review_ordinal_check CHECK ((review_ordinal > 0))
);


--
-- Name: book_review_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.book_review_history (
    book_review_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    book_review_id uuid NOT NULL,
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: contact; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.contact (
    contact_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publisher_id uuid NOT NULL,
    contact_type public.contact_type DEFAULT 'Accessibility'::public.contact_type NOT NULL,
    email text NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT contact_email_check CHECK ((octet_length(email) >= 1))
);


--
-- Name: contact_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.contact_history (
    contact_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    contact_id uuid NOT NULL,
    user_id text NOT NULL,
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
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    first_name text,
    last_name text NOT NULL,
    full_name text NOT NULL,
    contribution_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    contribution_ordinal integer NOT NULL,
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
    user_id text NOT NULL,
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
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: endorsement; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.endorsement (
    endorsement_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    author_name text,
    author_role text,
    url text,
    text text,
    endorsement_ordinal integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    author_orcid text,
    author_institution_id uuid,
    CONSTRAINT endorsement_endorsement_ordinal_check CHECK ((endorsement_ordinal > 0))
);


--
-- Name: endorsement_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.endorsement_history (
    endorsement_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    endorsement_id uuid NOT NULL,
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: file; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.file (
    file_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    file_type public.file_type NOT NULL,
    work_id uuid,
    publication_id uuid,
    object_key text NOT NULL,
    cdn_url text NOT NULL,
    mime_type text NOT NULL,
    bytes bigint NOT NULL,
    sha256 text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    additional_resource_id uuid,
    work_featured_video_id uuid,
    CONSTRAINT file_type_check CHECK ((((file_type = 'frontcover'::public.file_type) AND (work_id IS NOT NULL) AND (publication_id IS NULL) AND (additional_resource_id IS NULL) AND (work_featured_video_id IS NULL)) OR ((file_type = 'publication'::public.file_type) AND (publication_id IS NOT NULL) AND (work_id IS NULL) AND (additional_resource_id IS NULL) AND (work_featured_video_id IS NULL)) OR ((file_type <> ALL (ARRAY['frontcover'::public.file_type, 'publication'::public.file_type])) AND (work_id IS NULL) AND (publication_id IS NULL) AND (((additional_resource_id IS NOT NULL) AND (work_featured_video_id IS NULL)) OR ((work_featured_video_id IS NOT NULL) AND (additional_resource_id IS NULL))))))
);


--
-- Name: file_upload; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.file_upload (
    file_upload_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    file_type public.file_type NOT NULL,
    work_id uuid,
    publication_id uuid,
    declared_mime_type text NOT NULL,
    declared_extension text NOT NULL,
    declared_sha256 text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    additional_resource_id uuid,
    work_featured_video_id uuid,
    CONSTRAINT file_upload_type_check CHECK ((((file_type = 'frontcover'::public.file_type) AND (work_id IS NOT NULL) AND (publication_id IS NULL) AND (additional_resource_id IS NULL) AND (work_featured_video_id IS NULL)) OR ((file_type = 'publication'::public.file_type) AND (publication_id IS NOT NULL) AND (work_id IS NULL) AND (additional_resource_id IS NULL) AND (work_featured_video_id IS NULL)) OR ((file_type <> ALL (ARRAY['frontcover'::public.file_type, 'publication'::public.file_type])) AND (work_id IS NULL) AND (publication_id IS NULL) AND (((additional_resource_id IS NOT NULL) AND (work_featured_video_id IS NULL)) OR ((work_featured_video_id IS NOT NULL) AND (additional_resource_id IS NULL))))))
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
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT funding_grant_number_check CHECK ((octet_length(grant_number) >= 1)),
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
    user_id text NOT NULL,
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
    s3_bucket text,
    cdn_domain text,
    cloudfront_dist_id text,
    default_currency public.currency_code,
    default_place text,
    default_locale public.locale_code,
    CONSTRAINT imprint_crossmark_doi_check CHECK ((crossmark_doi ~* 'https:\/\/doi.org\/10.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$'::text)),
    CONSTRAINT imprint_imprint_name_check CHECK ((octet_length(imprint_name) >= 1)),
    CONSTRAINT imprint_imprint_url_check CHECK ((imprint_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT imprint_storage_cfg_all_or_none CHECK ((((s3_bucket IS NULL) AND (cdn_domain IS NULL) AND (cloudfront_dist_id IS NULL)) OR ((s3_bucket IS NOT NULL) AND (cdn_domain IS NOT NULL) AND (cloudfront_dist_id IS NOT NULL))))
);


--
-- Name: imprint_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.imprint_history (
    imprint_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    imprint_id uuid NOT NULL,
    user_id text NOT NULL,
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
    user_id text NOT NULL,
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
    issue_number integer,
    CONSTRAINT issue_issue_ordinal_check CHECK ((issue_ordinal > 0))
);


--
-- Name: issue_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.issue_history (
    issue_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id text NOT NULL,
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
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: language_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.language_history (
    language_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    language_id uuid NOT NULL,
    user_id text NOT NULL,
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
    user_id text NOT NULL,
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
    user_id text NOT NULL,
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
    accessibility_standard public.accessibility_standard,
    accessibility_additional_standard public.accessibility_standard,
    accessibility_exception public.accessibility_exception,
    accessibility_report_url text,
    CONSTRAINT check_accessibility_standard_rules CHECK (
CASE publication_type
    WHEN 'Paperback'::public.publication_type THEN ((accessibility_standard IS NULL) AND (accessibility_additional_standard IS NULL) AND (accessibility_exception IS NULL))
    WHEN 'Hardback'::public.publication_type THEN ((accessibility_standard IS NULL) AND (accessibility_additional_standard IS NULL) AND (accessibility_exception IS NULL))
    WHEN 'MP3'::public.publication_type THEN ((accessibility_standard IS NULL) AND (accessibility_additional_standard IS NULL) AND (accessibility_exception IS NULL))
    WHEN 'WAV'::public.publication_type THEN ((accessibility_standard IS NULL) AND (accessibility_additional_standard IS NULL) AND (accessibility_exception IS NULL))
    WHEN 'PDF'::public.publication_type THEN (((accessibility_standard IS NULL) OR (accessibility_standard = ANY (ARRAY['wcag-21-aa'::public.accessibility_standard, 'wcag-21-aaa'::public.accessibility_standard, 'wcag-22-aa'::public.accessibility_standard, 'wcag-22-aaa'::public.accessibility_standard]))) AND ((accessibility_additional_standard IS NULL) OR (accessibility_additional_standard = ANY (ARRAY['pdf-ua-1'::public.accessibility_standard, 'pdf-ua-2'::public.accessibility_standard]))))
    WHEN 'Epub'::public.publication_type THEN (((accessibility_standard IS NULL) OR (accessibility_standard = ANY (ARRAY['wcag-21-aa'::public.accessibility_standard, 'wcag-21-aaa'::public.accessibility_standard, 'wcag-22-aa'::public.accessibility_standard, 'wcag-22-aaa'::public.accessibility_standard]))) AND ((accessibility_additional_standard IS NULL) OR (accessibility_additional_standard = ANY (ARRAY['epub-a11y-10-aa'::public.accessibility_standard, 'epub-a11y-10-aaa'::public.accessibility_standard, 'epub-a11y-11-aa'::public.accessibility_standard, 'epub-a11y-11-aaa'::public.accessibility_standard]))))
    ELSE (((accessibility_standard IS NULL) OR (accessibility_standard = ANY (ARRAY['wcag-21-aa'::public.accessibility_standard, 'wcag-21-aaa'::public.accessibility_standard, 'wcag-22-aa'::public.accessibility_standard, 'wcag-22-aaa'::public.accessibility_standard]))) AND (accessibility_additional_standard IS NULL))
END),
    CONSTRAINT check_additional_standard_pdf_epub CHECK (((accessibility_additional_standard IS NULL) OR (publication_type = ANY (ARRAY['PDF'::public.publication_type, 'Epub'::public.publication_type])))),
    CONSTRAINT check_standard_or_exception CHECK ((((accessibility_exception IS NULL) AND (accessibility_standard IS NOT NULL)) OR ((accessibility_exception IS NOT NULL) AND (accessibility_standard IS NULL) AND (accessibility_additional_standard IS NULL)) OR ((accessibility_exception IS NULL) AND (accessibility_standard IS NULL) AND (accessibility_additional_standard IS NULL)))),
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
    user_id text NOT NULL,
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
    accessibility_statement text,
    accessibility_report_url text,
    zitadel_id text,
    CONSTRAINT publisher_accessibility_report_url_check CHECK ((octet_length(accessibility_report_url) >= 1)),
    CONSTRAINT publisher_accessibility_statement_check CHECK ((octet_length(accessibility_statement) >= 1)),
    CONSTRAINT publisher_publisher_name_check CHECK ((octet_length(publisher_name) >= 1)),
    CONSTRAINT publisher_publisher_shortname_check CHECK ((octet_length(publisher_shortname) >= 1)),
    CONSTRAINT publisher_publisher_url_check CHECK ((publisher_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text))
);


--
-- Name: publisher_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.publisher_history (
    publisher_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    publisher_id uuid NOT NULL,
    user_id text NOT NULL,
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
    user_id text NOT NULL,
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
    user_id text NOT NULL,
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
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: title; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.title (
    title_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    locale_code public.locale_code NOT NULL,
    full_title text NOT NULL,
    title text NOT NULL,
    subtitle text,
    canonical boolean DEFAULT false NOT NULL,
    CONSTRAINT title_full_title_check CHECK ((octet_length(full_title) >= 1)),
    CONSTRAINT title_subtitle_check CHECK ((octet_length(subtitle) >= 1)),
    CONSTRAINT title_title_check CHECK ((octet_length(title) >= 1))
);


--
-- Name: title_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.title_history (
    title_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    title_id uuid NOT NULL,
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: work; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work (
    work_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_type public.work_type NOT NULL,
    work_status public.work_status NOT NULL,
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
    resources_description text,
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
    CONSTRAINT work_general_note_check CHECK ((octet_length(general_note) >= 1)),
    CONSTRAINT work_image_count_check CHECK ((image_count >= 0)),
    CONSTRAINT work_inactive_no_withdrawn_date_check CHECK (((((work_status = 'withdrawn'::public.work_status) OR (work_status = 'superseded'::public.work_status)) AND (withdrawn_date IS NOT NULL)) OR (work_status <> ALL (ARRAY['withdrawn'::public.work_status, 'superseded'::public.work_status])))),
    CONSTRAINT work_landing_page_check CHECK ((landing_page ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
    CONSTRAINT work_last_page_check CHECK ((octet_length(last_page) >= 1)),
    CONSTRAINT work_lccn_check CHECK ((octet_length(lccn) >= 1)),
    CONSTRAINT work_license_check CHECK ((license ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'::text)),
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
    CONSTRAINT work_table_count_check CHECK ((table_count >= 0)),
    CONSTRAINT work_toc_check CHECK ((octet_length(toc) >= 1)),
    CONSTRAINT work_video_count_check CHECK ((video_count >= 0)),
    CONSTRAINT work_withdrawn_date_after_publication_date_check CHECK (((withdrawn_date IS NULL) OR (publication_date < withdrawn_date)))
);


--
-- Name: work_featured_video; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_featured_video (
    work_featured_video_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    title text,
    url text,
    width integer DEFAULT 560 NOT NULL,
    height integer DEFAULT 315 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT work_featured_video_height_check CHECK ((height > 0)),
    CONSTRAINT work_featured_video_width_check CHECK ((width > 0))
);


--
-- Name: work_featured_video_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_featured_video_history (
    work_featured_video_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_featured_video_id uuid NOT NULL,
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: work_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.work_history (
    work_history_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    user_id text NOT NULL,
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
    user_id text NOT NULL,
    data jsonb NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: abstract_history abstract_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.abstract_history
    ADD CONSTRAINT abstract_history_pkey PRIMARY KEY (abstract_history_id);


--
-- Name: abstract abstract_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.abstract
    ADD CONSTRAINT abstract_pkey PRIMARY KEY (abstract_id);


--
-- Name: additional_resource_history additional_resource_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.additional_resource_history
    ADD CONSTRAINT additional_resource_history_pkey PRIMARY KEY (additional_resource_history_id);


--
-- Name: additional_resource additional_resource_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.additional_resource
    ADD CONSTRAINT additional_resource_pkey PRIMARY KEY (additional_resource_id);


--
-- Name: additional_resource additional_resource_resource_ordinal_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.additional_resource
    ADD CONSTRAINT additional_resource_resource_ordinal_work_id_uniq UNIQUE (work_id, resource_ordinal) DEFERRABLE;


--
-- Name: affiliation affiliation_affiliation_ordinal_contribution_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.affiliation
    ADD CONSTRAINT affiliation_affiliation_ordinal_contribution_id_uniq UNIQUE (contribution_id, affiliation_ordinal) DEFERRABLE;


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
-- Name: award award_award_ordinal_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.award
    ADD CONSTRAINT award_award_ordinal_work_id_uniq UNIQUE (work_id, award_ordinal) DEFERRABLE;


--
-- Name: award_history award_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.award_history
    ADD CONSTRAINT award_history_pkey PRIMARY KEY (award_history_id);


--
-- Name: award award_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.award
    ADD CONSTRAINT award_pkey PRIMARY KEY (award_id);


--
-- Name: biography_history biography_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.biography_history
    ADD CONSTRAINT biography_history_pkey PRIMARY KEY (biography_history_id);


--
-- Name: biography biography_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.biography
    ADD CONSTRAINT biography_pkey PRIMARY KEY (biography_id);


--
-- Name: book_review_history book_review_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.book_review_history
    ADD CONSTRAINT book_review_history_pkey PRIMARY KEY (book_review_history_id);


--
-- Name: book_review book_review_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.book_review
    ADD CONSTRAINT book_review_pkey PRIMARY KEY (book_review_id);


--
-- Name: book_review book_review_review_ordinal_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.book_review
    ADD CONSTRAINT book_review_review_ordinal_work_id_uniq UNIQUE (work_id, review_ordinal) DEFERRABLE;


--
-- Name: contact contact_contact_type_publisher_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact_contact_type_publisher_id_uniq UNIQUE (publisher_id, contact_type);


--
-- Name: contact_history contact_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contact_history
    ADD CONSTRAINT contact_history_pkey PRIMARY KEY (contact_history_id);


--
-- Name: contact contact_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact_pkey PRIMARY KEY (contact_id);


--
-- Name: contribution contribution_contribution_ordinal_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution
    ADD CONSTRAINT contribution_contribution_ordinal_work_id_uniq UNIQUE (work_id, contribution_ordinal) DEFERRABLE;


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
-- Name: endorsement endorsement_endorsement_ordinal_work_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.endorsement
    ADD CONSTRAINT endorsement_endorsement_ordinal_work_id_uniq UNIQUE (work_id, endorsement_ordinal) DEFERRABLE;


--
-- Name: endorsement_history endorsement_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.endorsement_history
    ADD CONSTRAINT endorsement_history_pkey PRIMARY KEY (endorsement_history_id);


--
-- Name: endorsement endorsement_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.endorsement
    ADD CONSTRAINT endorsement_pkey PRIMARY KEY (endorsement_id);


--
-- Name: file file_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file_pkey PRIMARY KEY (file_id);


--
-- Name: file_upload file_upload_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_upload
    ADD CONSTRAINT file_upload_pkey PRIMARY KEY (file_upload_id);


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
-- Name: issue issue_issue_ordinal_series_id_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.issue
    ADD CONSTRAINT issue_issue_ordinal_series_id_uniq UNIQUE (series_id, issue_ordinal) DEFERRABLE;


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
    ADD CONSTRAINT reference_reference_ordinal_work_id_uniq UNIQUE (work_id, reference_ordinal) DEFERRABLE;


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
-- Name: subject subject_ordinal_type_uniq; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subject
    ADD CONSTRAINT subject_ordinal_type_uniq UNIQUE (work_id, subject_ordinal, subject_type) DEFERRABLE;


--
-- Name: subject subject_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subject
    ADD CONSTRAINT subject_pkey PRIMARY KEY (subject_id);


--
-- Name: title_history title_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.title_history
    ADD CONSTRAINT title_history_pkey PRIMARY KEY (title_history_id);


--
-- Name: title title_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.title
    ADD CONSTRAINT title_pkey PRIMARY KEY (title_id);


--
-- Name: work_featured_video_history work_featured_video_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_featured_video_history
    ADD CONSTRAINT work_featured_video_history_pkey PRIMARY KEY (work_featured_video_history_id);


--
-- Name: work_featured_video work_featured_video_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_featured_video
    ADD CONSTRAINT work_featured_video_pkey PRIMARY KEY (work_featured_video_id);


--
-- Name: work_featured_video work_featured_video_work_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_featured_video
    ADD CONSTRAINT work_featured_video_work_id_key UNIQUE (work_id);


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
    ADD CONSTRAINT work_relation_ordinal_type_uniq UNIQUE (relator_work_id, relation_ordinal, relation_type) DEFERRABLE;


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
-- Name: abstract_uniq_locale_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX abstract_uniq_locale_idx ON public.abstract USING btree (work_id, locale_code, abstract_type);


--
-- Name: abstract_unique_canonical_true_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX abstract_unique_canonical_true_idx ON public.abstract USING btree (work_id, abstract_type) WHERE canonical;


--
-- Name: biography_uniq_locale_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX biography_uniq_locale_idx ON public.biography USING btree (contribution_id, locale_code);


--
-- Name: biography_unique_canonical_true_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX biography_unique_canonical_true_idx ON public.biography USING btree (contribution_id) WHERE canonical;


--
-- Name: book_review_reviewer_institution_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX book_review_reviewer_institution_idx ON public.book_review USING btree (reviewer_institution_id) WHERE (reviewer_institution_id IS NOT NULL);


--
-- Name: doi_uniq_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX doi_uniq_idx ON public.work USING btree (lower(doi));


--
-- Name: endorsement_author_institution_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX endorsement_author_institution_idx ON public.endorsement USING btree (author_institution_id) WHERE (author_institution_id IS NOT NULL);


--
-- Name: file_additional_resource_unique_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX file_additional_resource_unique_idx ON public.file USING btree (additional_resource_id) WHERE (additional_resource_id IS NOT NULL);


--
-- Name: file_frontcover_work_unique_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX file_frontcover_work_unique_idx ON public.file USING btree (work_id) WHERE (file_type = 'frontcover'::public.file_type);


--
-- Name: file_object_key_unique_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX file_object_key_unique_idx ON public.file USING btree (object_key);


--
-- Name: file_publication_unique_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX file_publication_unique_idx ON public.file USING btree (publication_id) WHERE (file_type = 'publication'::public.file_type);


--
-- Name: file_upload_additional_resource_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX file_upload_additional_resource_idx ON public.file_upload USING btree (additional_resource_id) WHERE (additional_resource_id IS NOT NULL);


--
-- Name: file_upload_publication_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX file_upload_publication_idx ON public.file_upload USING btree (publication_id) WHERE (file_type = 'publication'::public.file_type);


--
-- Name: file_upload_work_featured_video_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX file_upload_work_featured_video_idx ON public.file_upload USING btree (work_featured_video_id) WHERE (work_featured_video_id IS NOT NULL);


--
-- Name: file_upload_work_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX file_upload_work_idx ON public.file_upload USING btree (work_id) WHERE (file_type = 'frontcover'::public.file_type);


--
-- Name: file_work_featured_video_unique_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX file_work_featured_video_unique_idx ON public.file USING btree (work_featured_video_id) WHERE (work_featured_video_id IS NOT NULL);


--
-- Name: idx_affiliation_contribution_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_affiliation_contribution_id ON public.affiliation USING btree (contribution_id);


--
-- Name: idx_affiliation_ordinal_asc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_affiliation_ordinal_asc ON public.affiliation USING btree (affiliation_ordinal, contribution_id);


--
-- Name: idx_contact_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_contact_email ON public.contact USING btree (email);


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
-- Name: idx_work_imprint_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_imprint_id ON public.work USING btree (imprint_id);


--
-- Name: idx_work_landing_page; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_work_landing_page ON public.work USING btree (landing_page);


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
-- Name: publisher_zitadel_id_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX publisher_zitadel_id_key ON public.publisher USING btree (zitadel_id) WHERE (zitadel_id IS NOT NULL);


--
-- Name: series_issn_digital_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX series_issn_digital_idx ON public.series USING btree (issn_digital);


--
-- Name: series_issn_print_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX series_issn_print_idx ON public.series USING btree (issn_print);


--
-- Name: title_uniq_locale_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX title_uniq_locale_idx ON public.title USING btree (work_id, locale_code);


--
-- Name: title_unique_canonical_true_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX title_unique_canonical_true_idx ON public.title USING btree (work_id) WHERE canonical;


--
-- Name: publication publication_chapter_no_dimensions_check; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER publication_chapter_no_dimensions_check BEFORE INSERT OR UPDATE ON public.publication FOR EACH ROW EXECUTE FUNCTION public.publication_chapter_no_dimensions();


--
-- Name: publication publication_location_canonical_urls_check; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER publication_location_canonical_urls_check BEFORE UPDATE ON public.publication FOR EACH ROW EXECUTE FUNCTION public.publication_location_canonical_urls();


--
-- Name: additional_resource set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.additional_resource FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: affiliation set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.affiliation FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: award set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.award FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: book_review set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.book_review FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: contact set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.contact FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: contribution set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.contribution FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: contributor set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.contributor FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: endorsement set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.endorsement FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: file set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.file FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: file_upload set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.file_upload FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


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
-- Name: work_featured_video set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.work_featured_video FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: work_relation set_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.work_relation FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: work_relation set_work_relation_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_relation_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.work_relation FOR EACH ROW EXECUTE FUNCTION public.work_relation_work_updated_at_with_relations();


--
-- Name: abstract set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.abstract FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: additional_resource set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.additional_resource FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: affiliation set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.affiliation FOR EACH ROW EXECUTE FUNCTION public.affiliation_work_updated_at_with_relations();


--
-- Name: award set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.award FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: biography set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.biography FOR EACH ROW EXECUTE FUNCTION public.biography_work_updated_at_with_relations();


--
-- Name: book_review set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.book_review FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: contribution set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.contribution FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: contributor set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER UPDATE ON public.contributor FOR EACH ROW EXECUTE FUNCTION public.contributor_work_updated_at_with_relations();


--
-- Name: endorsement set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.endorsement FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: file set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.file FOR EACH ROW EXECUTE FUNCTION public.file_work_updated_at_with_relations();


--
-- Name: file_upload set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.file_upload FOR EACH ROW EXECUTE FUNCTION public.file_upload_work_updated_at_with_relations();


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
-- Name: title set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.title FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: work_featured_video set_work_updated_at_with_relations; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR DELETE OR UPDATE ON public.work_featured_video FOR EACH ROW EXECUTE FUNCTION public.work_updated_at_with_relations();


--
-- Name: abstract_history abstract_history_abstract_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.abstract_history
    ADD CONSTRAINT abstract_history_abstract_id_fkey FOREIGN KEY (abstract_id) REFERENCES public.abstract(abstract_id) ON DELETE CASCADE;


--
-- Name: abstract abstract_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.abstract
    ADD CONSTRAINT abstract_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: additional_resource_history additional_resource_history_additional_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.additional_resource_history
    ADD CONSTRAINT additional_resource_history_additional_resource_id_fkey FOREIGN KEY (additional_resource_id) REFERENCES public.additional_resource(additional_resource_id) ON DELETE CASCADE;


--
-- Name: additional_resource additional_resource_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.additional_resource
    ADD CONSTRAINT additional_resource_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: affiliation affiliation_contribution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.affiliation
    ADD CONSTRAINT affiliation_contribution_id_fkey FOREIGN KEY (contribution_id) REFERENCES public.contribution(contribution_id) ON DELETE CASCADE;


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
-- Name: award_history award_history_award_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.award_history
    ADD CONSTRAINT award_history_award_id_fkey FOREIGN KEY (award_id) REFERENCES public.award(award_id) ON DELETE CASCADE;


--
-- Name: award award_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.award
    ADD CONSTRAINT award_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: biography biography_contribution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.biography
    ADD CONSTRAINT biography_contribution_id_fkey FOREIGN KEY (contribution_id) REFERENCES public.contribution(contribution_id) ON DELETE CASCADE;


--
-- Name: biography_history biography_history_biography_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.biography_history
    ADD CONSTRAINT biography_history_biography_id_fkey FOREIGN KEY (biography_id) REFERENCES public.biography(biography_id) ON DELETE CASCADE;


--
-- Name: book_review_history book_review_history_book_review_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.book_review_history
    ADD CONSTRAINT book_review_history_book_review_id_fkey FOREIGN KEY (book_review_id) REFERENCES public.book_review(book_review_id) ON DELETE CASCADE;


--
-- Name: book_review book_review_reviewer_institution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.book_review
    ADD CONSTRAINT book_review_reviewer_institution_id_fkey FOREIGN KEY (reviewer_institution_id) REFERENCES public.institution(institution_id) ON DELETE SET NULL;


--
-- Name: book_review book_review_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.book_review
    ADD CONSTRAINT book_review_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: contact_history contact_history_contact_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contact_history
    ADD CONSTRAINT contact_history_contact_id_fkey FOREIGN KEY (contact_id) REFERENCES public.contact(contact_id) ON DELETE CASCADE;


--
-- Name: contact contact_publisher_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact_publisher_id_fkey FOREIGN KEY (publisher_id) REFERENCES public.publisher(publisher_id) ON DELETE CASCADE;


--
-- Name: contribution contribution_contributor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contribution
    ADD CONSTRAINT contribution_contributor_id_fkey FOREIGN KEY (contributor_id) REFERENCES public.contributor(contributor_id) ON DELETE CASCADE;


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
-- Name: contributor_history contributor_history_contributor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.contributor_history
    ADD CONSTRAINT contributor_history_contributor_id_fkey FOREIGN KEY (contributor_id) REFERENCES public.contributor(contributor_id) ON DELETE CASCADE;


--
-- Name: endorsement endorsement_author_institution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.endorsement
    ADD CONSTRAINT endorsement_author_institution_id_fkey FOREIGN KEY (author_institution_id) REFERENCES public.institution(institution_id) ON DELETE SET NULL;


--
-- Name: endorsement_history endorsement_history_endorsement_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.endorsement_history
    ADD CONSTRAINT endorsement_history_endorsement_id_fkey FOREIGN KEY (endorsement_id) REFERENCES public.endorsement(endorsement_id) ON DELETE CASCADE;


--
-- Name: endorsement endorsement_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.endorsement
    ADD CONSTRAINT endorsement_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: file file_additional_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file_additional_resource_id_fkey FOREIGN KEY (additional_resource_id) REFERENCES public.additional_resource(additional_resource_id) ON DELETE CASCADE;


--
-- Name: file file_publication_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file_publication_id_fkey FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id) ON DELETE CASCADE;


--
-- Name: file_upload file_upload_additional_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_upload
    ADD CONSTRAINT file_upload_additional_resource_id_fkey FOREIGN KEY (additional_resource_id) REFERENCES public.additional_resource(additional_resource_id) ON DELETE CASCADE;


--
-- Name: file_upload file_upload_publication_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_upload
    ADD CONSTRAINT file_upload_publication_id_fkey FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id) ON DELETE CASCADE;


--
-- Name: file_upload file_upload_work_featured_video_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_upload
    ADD CONSTRAINT file_upload_work_featured_video_id_fkey FOREIGN KEY (work_featured_video_id) REFERENCES public.work_featured_video(work_featured_video_id) ON DELETE CASCADE;


--
-- Name: file_upload file_upload_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file_upload
    ADD CONSTRAINT file_upload_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: file file_work_featured_video_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file_work_featured_video_id_fkey FOREIGN KEY (work_featured_video_id) REFERENCES public.work_featured_video(work_featured_video_id) ON DELETE CASCADE;


--
-- Name: file file_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.file
    ADD CONSTRAINT file_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


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
-- Name: publisher_history publisher_history_publisher_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.publisher_history
    ADD CONSTRAINT publisher_history_publisher_id_fkey FOREIGN KEY (publisher_id) REFERENCES public.publisher(publisher_id) ON DELETE CASCADE;


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
-- Name: title_history title_history_title_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.title_history
    ADD CONSTRAINT title_history_title_id_fkey FOREIGN KEY (title_id) REFERENCES public.title(title_id) ON DELETE CASCADE;


--
-- Name: title title_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.title
    ADD CONSTRAINT title_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


--
-- Name: work_featured_video_history work_featured_video_history_work_featured_video_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_featured_video_history
    ADD CONSTRAINT work_featured_video_history_work_featured_video_id_fkey FOREIGN KEY (work_featured_video_id) REFERENCES public.work_featured_video(work_featured_video_id) ON DELETE CASCADE;


--
-- Name: work_featured_video work_featured_video_work_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.work_featured_video
    ADD CONSTRAINT work_featured_video_work_id_fkey FOREIGN KEY (work_id) REFERENCES public.work(work_id) ON DELETE CASCADE;


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


--
