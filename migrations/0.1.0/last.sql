CREATE TABLE price (
    price_id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publication_id      UUID NOT NULL REFERENCES publication(publication_id),
    currency_code       currency_code NOT NULL,
    price               decimal NOT NULL
);


---------------------------------------------------------------------------
---------------------------------------------------------------------------
---------------------------------------------------------------------------

INSERT INTO publisher VALUES
('00000000-0000-0000-DDDD-000000000001', 'Open Book Publishers', 'OBP', 'https://www.openbookpublishers.com'),
('00000000-0000-0000-DDDD-000000000002', 'punctum books', null, 'https://punctumbooks.com');

INSERT INTO work VALUES
('00000000-0000-0000-AAAA-000000000001', 'monograph', 'active', 'That Greece Might Still Be Free: The Philhellenes in the War of Independence', 'That Greece Might Still Be Free', 'The Philhellenes in the War of Independence', null, 1, '00000000-0000-0000-DDDD-000000000001', 'https://doi.org/10.11647/obp.0001', '2008-11-01', 'Cambridge, UK', 156, 234, 440, 'xxi + 419', 41, 2, 0, 0, 'http://creativecommons.org/licenses/by-nc-nd/2.0/', 'William St Clair', 'https://www.openbookpublishers.com/product/3', null, 849917874, 'When in 1821, the Greeks rose in violent revolution against Ottoman rule, waves of sympathy spread across western Europe and the USA. Inspired by a belief that Greece had a unique claim on the sympathy of the world, more than a thousand Philhellenes set out to fight for the cause. This meticulously researched and highly readable account of their aspirations and experiences has long been the standard account of the Philhellenic movement and essential reading for students of the Greek War of Independence, Byron and European Romanticism. Its relevance to more modern conflicts is also becoming increasingly appreciated.', 'When in 1821, the Greeks rose in violent revolution against Ottoman rule, waves of sympathy spread across western Europe and the USA. Inspired by a belief that Greece had a unique claim on the sympathy of the world, more than a thousand Philhellenes set out to fight for the cause. This meticulously researched and highly readable account of their aspirations and experiences has long been the standard account of the Philhellenic movement and essential reading for students of the Greek War of Independence, Byron and European Romanticism. Its relevance to more modern conflicts is also becoming increasingly appreciated.', null, null, 'https://www.openbookpublishers.com/shopimages/products/cover/3.jpg', null),
('00000000-0000-0000-AAAA-000000000002', 'textbook', 'active', 'Conservation Biology in Sub-Saharan Africa', 'Conservation Biology in Sub-Saharan Africa', null, null, 1, '00000000-0000-0000-DDDD-000000000001', 'https://doi.org/10.11647/obp.0177', '2019-09-09', 'Cambridge, UK', 156, 234, 320, 'vi + 314', 32, 5, 0, 0, 'http://creativecommons.org/licenses/by/4.0/', 'John W. Wilson, Richard B. Primack', 'https://www.openbookpublishers.com/product/1013', null, null, null, null, null, null, 'https://www.openbookpublishers.com/shopimages/products/cover/1013.jpg', null),
('00000000-0000-0000-AAAA-000000000003', 'journal-issue', 'out-of-print', 'What Works in Conservation 2015', 'What Works in Conservation', '2015', null, 1, '00000000-0000-0000-DDDD-000000000001', 'https://doi.org/10.11647/obp.0060', '2015-07-01', 'Cambridge, UK', 156, 234, 198, 'xii + 176', 23, 46, 0, 0, 'http://creativecommons.org/licenses/by/4.0/', 'William J. Sutherland', 'https://www.openbookpublishers.com/product/347', null, null, 'What Works in Conservation has been created to provide practitioners with answers to these and many other questions about practical conservation. This book provides an assessment of the effectiveness of 648 conservation interventions based on summarized scientific evidence relevant to the practical global conservation of amphibians, reducing the risk of predation for birds, conservation of European farmland biodiversity and some aspects of enhancing natural pest control and soil fertility.', 'Is planting grass margins around fields beneficial for wildlife? Which management interventions increase bee numbers in farmland? Does helping migrating toads across roads increase populations? How do you reduce predation on bird populations? What Works in Conservation has been created to provide practitioners with answers to these and many other questions about practical conservation.
This book provides an assessment of the effectiveness of over 200 conservation interventions based on summarized scientific evidence relevant to the practical global conservation of amphibians, reducing the risk of predation for birds, conservation of European farmland biodiversity and some aspects of enhancing natural pest control and soil fertility. It contains key results from the summarized evidence for each conservation intervention and an assessment of the effectiveness of each by international expert panels. The volume is published in partnership with the Conservation Evidence project and is fully linked to the project''s website where background papers such as abstracts and published journal articles can be freely accessed.', null, null, 'http://www.openbookpublishers.com/shopimages/products/cover/347.jpg', null);

INSERT INTO language VALUES
('00000000-0000-0000-FFFF-000000000001', '00000000-0000-0000-AAAA-000000000001', 'eng', 'original', True),
('00000000-0000-0000-FFFF-000000000002', '00000000-0000-0000-AAAA-000000000002', 'eng', 'original', True),
('00000000-0000-0000-FFFF-000000000003', '00000000-0000-0000-AAAA-000000000003', 'eng', 'original', True);

INSERT INTO contributor VALUES
('00000000-0000-0000-CCCC-000000000001', 'William', 'St Clair', 'William St Clair', null, 'https://research.sas.ac.uk/search/fellow/158'),
('00000000-0000-0000-CCCC-000000000002', 'Roderick', 'Beaton', 'Roderick Beaton', null, null),
('00000000-0000-0000-CCCC-000000000003', 'John W.', 'Wilson', 'John W. Wilson', '0000-0002-7230-1449', 'https://johnnybirder.com/index.html'),
('00000000-0000-0000-CCCC-000000000004', 'Richard B.', 'Primack', 'Richard B. Primack', '0000-0002-3748-9853', 'https://www.rprimacklab.com'),
('00000000-0000-0000-CCCC-000000000005', 'William J.', 'Sutherland', 'William J. Sutherland', null, null);


INSERT INTO contribution VALUES
('00000000-0000-0000-AAAA-000000000001', '00000000-0000-0000-CCCC-000000000001', 'author', True, 'William St Clair is a Senior Research Fellow at the Institute of English Studies, School of Advanced Study, University of London, and of the Centre for History and Economics, University of Cambridge. His works include <i>Lord Elgin and the Marbles</i> and <i>The Reading Nation in the Romantic Period</i>. He is a Fellow of the British Academy and of the Royal Society of Literature.', null),
('00000000-0000-0000-AAAA-000000000001', '00000000-0000-0000-CCCC-000000000002', 'introduction-by', False, null, null),
('00000000-0000-0000-AAAA-000000000002', '00000000-0000-0000-CCCC-000000000003', 'author', True, 'John W. Wilson is a conservation biologist interested in solving the dynamic challenges of a changing world. He received his BSc and MSc from Pretoria University, and his PhD from North Carolina State University. He has over 15 years of experience with conservation across Africa. As a NASA Earth and Space Science Fellow, he studied interactions between habitat loss and climate change in West Africa. He also spent 13 months on uninhabited Gough Island, a World Heritage Site in the South Atlantic, where he combatted invasive species. Beyond that, he has studied individual organisms, populations, and natural communities across Southern, East, Central, and West Africa. His work has covered pertinent topics such as conservation planning, population monitoring, protected areas management, translocations, ecological restoration, and movement ecology in savannahs, grasslands, forests, wetlands, and agricultural systems. His love for nature also dominates his free time; he has contributed over 50,000 observation records to the citizen science platforms eBird and iNaturalist, which he also helps curate.', null),
('00000000-0000-0000-AAAA-000000000002', '00000000-0000-0000-CCCC-000000000004', 'author', True, 'Richard B. Primack is a Professor of Biology, specializing in plant ecology, conservation biology, and tropical ecology. He is the author of three widely used conservation biology textbooks; local co-authors have helped to produce 36 translations of these books with local examples. He has been Editor-in-Chief of the journal <i>Biological Conservation</i>, and served as President of the <i>Association for Tropical Biology and Conservation</i>. His research documents the effects of climate change on plants and animals in the Eastern U.S.A., and is often featured in the popular press.', 'Boston University'),
('00000000-0000-0000-AAAA-000000000003', '00000000-0000-0000-CCCC-000000000005', 'editor', True, null, null);

INSERT INTO publication VALUES
('00000000-0000-0000-BBBB-000000000001', 'Paperback', '00000000-0000-0000-AAAA-000000000001', '978-1-906924-00-3', null),
('00000000-0000-0000-BBBB-000000000002', 'Hardback', '00000000-0000-0000-AAAA-000000000001', '978-1-906924-01-0', null),
('00000000-0000-0000-BBBB-000000000003', 'PDF', '00000000-0000-0000-AAAA-000000000001', '978-1-906924-02-7', null),
('00000000-0000-0000-BBBB-000000000004', 'Paperback', '00000000-0000-0000-AAAA-000000000002', '978-1-78374-750-4', null),
('00000000-0000-0000-BBBB-000000000005', 'Hardback', '00000000-0000-0000-AAAA-000000000002', '978-1-78374-751-1', null),
('00000000-0000-0000-BBBB-000000000006', 'PDF', '00000000-0000-0000-AAAA-000000000002', '978-1-78374-752-8', null);

INSERT INTO series VALUES
('00000000-0000-0000-EEEE-000000000001', 'journal', 'What Works in Conservation', '2059-4232', '2059-4240', null, '00000000-0000-0000-DDDD-000000000001');

INSERT INTO issue VALUES
('00000000-0000-0000-EEEE-000000000001', '00000000-0000-0000-AAAA-000000000003', 1);
