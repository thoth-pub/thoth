CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-------------------- Publisher
CREATE TABLE publisher (
    publisher_id        UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publisher_name      TEXT NOT NULL CHECK (octet_length(publisher_name) >= 1), -- UNIQ below
    publisher_shortname TEXT CHECK (octet_length(publisher_shortname) >= 1),
    publisher_url       TEXT CHECK (publisher_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)')
);
-- case-insensitive UNIQ index on publisher_name
CREATE UNIQUE INDEX publisher_uniq_idx on publisher(lower(publisher_name));

-------------------- Work

CREATE TYPE work_type AS ENUM (
  'book-chapter',
  'monograph',
  'edited-book',
  'textbook',
  'journal-issue'
);

CREATE TABLE work (
    work_id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_type           work_type NOT NULL,
    full_title          TEXT NOT NULL CHECK (octet_length(full_title) >= 1),
    title               TEXT NOT NULL CHECK (octet_length(title) >= 1),
    subtitle            TEXT CHECK (octet_length(subtitle) >= 1),
    publisher_id        UUID NOT NULL REFERENCES publisher(publisher_id),
    doi                 TEXT CHECK (doi ~* 'https:\/\/doi.org\/10.\d{4,9}\/[-._\;\(\)\/:a-zA-Z0-9]+$'), -- UNIQ below
    publication_date    DATE
);
-- case-insensitive UNIQ index on doi
CREATE UNIQUE INDEX doi_uniq_idx on work(lower(doi));

-------------------- Contributor

CREATE TABLE contributor (
    contributor_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    first_name          TEXT CHECK (octet_length(first_name) >= 1),
    last_name           TEXT NOT NULL CHECK (octet_length(last_name) >= 1),
    full_name           TEXT NOT NULL CHECK (octet_length(full_name) >= 1),
    orcid               TEXT CHECK (orcid ~* '0000-000(1-[5-9]|2-[0-9]|3-[0-4])\d{3}-\d{3}[\dX]'), -- UNIQ below
    website             TEXT CHECK (octet_length(website) >= 1)
);
-- case-insensitive UNIQ index on orcid
CREATE UNIQUE INDEX orcid_uniq_idx on contributor(lower(orcid));

CREATE TYPE contribution_type AS ENUM (
  'author',
  'editor',
  'translator',
  'photographer',
  'ilustrator',
  'foreword-by',
  'introduction-by',
  'afterword-by',
  'preface-by'
);

CREATE TABLE contribution (
    work_id             UUID NOT NULL REFERENCES work(work_id),
    contributor_id      UUID NOT NULL REFERENCES contributor(contributor_id),
    contribution_type   contribution_type NOT NULL,
    main_contribution   BOOLEAN NOT NULL DEFAULT False,
    biography           TEXT CHECK (octet_length(biography) >= 1),
    institution         TEXT CHECK (octet_length(institution) >= 1),
    PRIMARY KEY (work_id, contributor_id, contribution_type)
);

-------------------- Publication

CREATE TYPE publication_type AS ENUM (
  'Paperback',
  'Hardback',
  'PDF',
  'HTML',
  'XML',
  'Epub',
  'Mobi'
);

CREATE TABLE publication (
    publication_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publication_type    publication_type NOT NULL,
    work_id             UUID NOT NULL REFERENCES work(work_id),
    isbn                TEXT CHECK (octet_length(isbn) = 17),
    publication_url     TEXT CHECK (publication_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)')
);

---------------------------------------------------------------------------
---------------------------------------------------------------------------
---------------------------------------------------------------------------

INSERT INTO publisher VALUES
('00000000-0000-0000-DDDD-000000000001', 'Open Book Publishers', 'OBP', 'https://www.openbookpublishers.com'),
('00000000-0000-0000-DDDD-000000000002', 'punctum books', null, 'https://punctumbooks.com');

INSERT INTO work VALUES
('00000000-0000-0000-AAAA-000000000001', 'monograph', 'That Greece Might Still Be Free: The Philhellenes in the War of Independence', 'That Greece Might Still Be Free', 'The Philhellenes in the War of Independence', '00000000-0000-0000-DDDD-000000000001', 'https://doi.org/10.11647/obp.0001', '2008-11-01'),
('00000000-0000-0000-AAAA-000000000002', 'textbook', 'Conservation Biology in Sub-Saharan Africa', 'Conservation Biology in Sub-Saharan Africa', null, '00000000-0000-0000-DDDD-000000000001', 'https://doi.org/10.11647/obp.0177', '2019-09-09');

INSERT INTO contributor VALUES
('00000000-0000-0000-CCCC-000000000001', 'William', 'St Clair', 'William St Clair', null, 'https://research.sas.ac.uk/search/fellow/158'),
('00000000-0000-0000-CCCC-000000000002', 'Roderick', 'Beaton', 'Roderick Beaton', null, null),
('00000000-0000-0000-CCCC-000000000003', 'John W.', 'Wilson', 'John W. Wilson', '0000-0002-7230-1449', 'https://johnnybirder.com/index.html'),
('00000000-0000-0000-CCCC-000000000004', 'Richard B.', 'Primack', 'Richard B. Primack', '0000-0002-3748-9853', 'https://www.rprimacklab.com');

INSERT INTO contribution VALUES
('00000000-0000-0000-AAAA-000000000001', '00000000-0000-0000-CCCC-000000000001', 'author', True, 'William St Clair is a Senior Research Fellow at the Institute of English Studies, School of Advanced Study, University of London, and of the Centre for History and Economics, University of Cambridge. His works include <i>Lord Elgin and the Marbles</i> and <i>The Reading Nation in the Romantic Period</i>. He is a Fellow of the British Academy and of the Royal Society of Literature.', null),
('00000000-0000-0000-AAAA-000000000001', '00000000-0000-0000-CCCC-000000000002', 'introduction-by', False, null, null),
('00000000-0000-0000-AAAA-000000000002', '00000000-0000-0000-CCCC-000000000003', 'author', True, 'John W. Wilson is a conservation biologist interested in solving the dynamic challenges of a changing world. He received his BSc and MSc from Pretoria University, and his PhD from North Carolina State University. He has over 15 years of experience with conservation across Africa. As a NASA Earth and Space Science Fellow, he studied interactions between habitat loss and climate change in West Africa. He also spent 13 months on uninhabited Gough Island, a World Heritage Site in the South Atlantic, where he combatted invasive species. Beyond that, he has studied individual organisms, populations, and natural communities across Southern, East, Central, and West Africa. His work has covered pertinent topics such as conservation planning, population monitoring, protected areas management, translocations, ecological restoration, and movement ecology in savannahs, grasslands, forests, wetlands, and agricultural systems. His love for nature also dominates his free time; he has contributed over 50,000 observation records to the citizen science platforms eBird and iNaturalist, which he also helps curate.', null),
('00000000-0000-0000-AAAA-000000000002', '00000000-0000-0000-CCCC-000000000004', 'author', True, 'Richard B. Primack is a Professor of Biology, specializing in plant ecology, conservation biology, and tropical ecology. He is the author of three widely used conservation biology textbooks; local co-authors have helped to produce 36 translations of these books with local examples. He has been Editor-in-Chief of the journal <i>Biological Conservation</i>, and served as President of the <i>Association for Tropical Biology and Conservation</i>. His research documents the effects of climate change on plants and animals in the Eastern U.S.A., and is often featured in the popular press.', 'Boston University');

INSERT INTO publication VALUES
('00000000-0000-0000-BBBB-000000000001', 'Paperback', '00000000-0000-0000-AAAA-000000000001', '978-1-906924-00-3', null),
('00000000-0000-0000-BBBB-000000000002', 'Hardback', '00000000-0000-0000-AAAA-000000000001', '978-1-906924-01-0', null),
('00000000-0000-0000-BBBB-000000000003', 'PDF', '00000000-0000-0000-AAAA-000000000001', '978-1-906924-02-7', null),
('00000000-0000-0000-BBBB-000000000004', 'Paperback', '00000000-0000-0000-AAAA-000000000002', '978-1-78374-750-4', null),
('00000000-0000-0000-BBBB-000000000005', 'Hardback', '00000000-0000-0000-AAAA-000000000002', '978-1-78374-751-1', null),
('00000000-0000-0000-BBBB-000000000006', 'PDF', '00000000-0000-0000-AAAA-000000000002', '978-1-78374-752-8', null);
