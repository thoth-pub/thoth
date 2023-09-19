use crate::marc21::{Marc21Entry, Marc21RecordThoth, Marc21Specification};
use marc::{MarcXml, Record};
use std::io::Write;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

#[derive(Copy, Clone)]
pub(crate) struct Marc21XmlThoth;

const MARC_ERROR: &str = "marc21xml::thoth";

impl Marc21Specification for Marc21XmlThoth {
    fn handle_event(w: &mut Vec<u8>, works: &[Work]) -> ThothResult<()> {
        match works {
            [] => Err(ThothError::IncompleteMetadataRecord(
                MARC_ERROR.to_string(),
                "Not enough data".to_string(),
            )),
            [work] => Marc21Entry::<Marc21RecordThoth>::marc21_xml(work, w),
            _ => {
                let xml = works
                    .iter()
                    .filter_map(|work| Marc21Entry::<Marc21RecordThoth>::to_record(work).ok())
                    .collect::<Vec<Record>>()
                    .xml_pretty()?;
                w.write_all(&xml)?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marc21::marc21record_thoth::tests::test_work;
    use chrono::Utc;

    #[test]
    fn test_generate_marc_xml_single_work() {
        let work = test_work();
        let current_date = Utc::now().format("%y%m%d").to_string();
        let expected = format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<marc:record xmlns:marc=\"http://www.loc.gov/MARC21/slim\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.loc.gov/MARC21/slim http://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd\">\n  <marc:leader>02336nam  22005292  4500</marc:leader>\n  <marc:controlfield tag=\"001\">00000000-0000-0000-aaaa-000000000001</marc:controlfield>\n  <marc:controlfield tag=\"006\">m     o  d        </marc:controlfield>\n  <marc:controlfield tag=\"007\">cr  n         </marc:controlfield>\n  <marc:controlfield tag=\"008\">{current_date}t20102010        ob    000 0 eng d</marc:controlfield>\n  <marc:datafield tag=\"010\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">LCCN010101</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">9783161484100</marc:subfield>\n    <marc:subfield code=\"q\">(PDF)</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">9789295055025</marc:subfield>\n    <marc:subfield code=\"q\">(XML)</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"z\">9781402894626</marc:subfield>\n    <marc:subfield code=\"q\">(Hardback)</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"024\" ind1=\"7\" ind2=\"\\\">\n    <marc:subfield code=\"a\">10.00001/BOOK.0001</marc:subfield>\n    <marc:subfield code=\"2\">doi</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"024\" ind1=\"7\" ind2=\"\\\">\n    <marc:subfield code=\"a\">OCLC010101</marc:subfield>\n    <marc:subfield code=\"2\">worldcat</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"040\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">UkCbTOM</marc:subfield>\n    <marc:subfield code=\"b\">eng</marc:subfield>\n    <marc:subfield code=\"e\">rda</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"041\" ind1=\"1\" ind2=\"\\\">\n    <marc:subfield code=\"a\">eng</marc:subfield>\n    <marc:subfield code=\"h\">spa</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"050\" ind1=\"0\" ind2=\"0\">\n    <marc:subfield code=\"a\">JA85</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n    <marc:subfield code=\"a\">AAB</marc:subfield>\n    <marc:subfield code=\"2\">bicssc</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n    <marc:subfield code=\"a\">AAA000000</marc:subfield>\n    <marc:subfield code=\"2\">bisacsh</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n    <marc:subfield code=\"a\">JWA</marc:subfield>\n    <marc:subfield code=\"2\">thema</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"100\" ind1=\"1\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Author, Sole,</marc:subfield>\n    <marc:subfield code=\"e\">author.</marc:subfield>\n    <marc:subfield code=\"u\">Thoth University.</marc:subfield>\n    <marc:subfield code=\"1\">https://orcid.org/0000-0002-0000-0001</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"245\" ind1=\"1\" ind2=\"0\">\n    <marc:subfield code=\"a\">Book Title :</marc:subfield>\n    <marc:subfield code=\"b\">Book Subtitle /</marc:subfield>\n    <marc:subfield code=\"c\">Sole Author; edited by Only Editor; translated by Translator.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"250\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">1st edition</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"264\" ind1=\"\\\" ind2=\"1\">\n    <marc:subfield code=\"a\">León, Spain :</marc:subfield>\n    <marc:subfield code=\"b\">OA Editions,</marc:subfield>\n    <marc:subfield code=\"c\">2010.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"264\" ind1=\"\\\" ind2=\"4\">\n    <marc:subfield code=\"c\">©2010</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"300\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">1 online resource.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"336\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">text</marc:subfield>\n    <marc:subfield code=\"b\">txt</marc:subfield>\n    <marc:subfield code=\"2\">rdacontent</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"337\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">computer</marc:subfield>\n    <marc:subfield code=\"b\">c</marc:subfield>\n    <marc:subfield code=\"2\">rdamedia</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"338\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">online resource</marc:subfield>\n    <marc:subfield code=\"b\">cr</marc:subfield>\n    <marc:subfield code=\"2\">rdacarrier</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"490\" ind1=\"1\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Name of series</marc:subfield>\n    <marc:subfield code=\"v\">vol. 11</marc:subfield>\n    <marc:subfield code=\"x\">8765-4321</marc:subfield>\n    <marc:subfield code=\"x\">1234-5678</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"500\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Please note that in this book the mathematical formulas are encoded in MathML.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"504\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Includes bibliography (pages 165-170) and index.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"505\" ind1=\"0\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Introduction; Chapter 1; Chapter 2; Bibliography; Index</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"506\" ind1=\"0\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Open Access</marc:subfield>\n    <marc:subfield code=\"f\">Unrestricted online access</marc:subfield>\n    <marc:subfield code=\"2\">star</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"520\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Lorem ipsum dolor sit amet</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"536\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Funding Institution</marc:subfield>\n    <marc:subfield code=\"c\">JA0001</marc:subfield>\n    <marc:subfield code=\"e\">Funding Programme</marc:subfield>\n    <marc:subfield code=\"f\">Funding Project</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"538\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Mode of access: World Wide Web.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"540\" ind1=\"\\\" ind2=\"\\\">\n    <marc:subfield code=\"a\">The text of this book is licensed under a Creative Commons Attribution 4.0 International license (CC BY 4.0). For more detailed information consult the publisher's website.</marc:subfield>\n    <marc:subfield code=\"u\">https://creativecommons.org/licenses/by/4.0/</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"588\" ind1=\"0\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Metadata licensed under CC0 Public Domain Dedication.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"700\" ind1=\"1\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Editor, Only,</marc:subfield>\n    <marc:subfield code=\"e\">editor.</marc:subfield>\n    <marc:subfield code=\"1\">https://orcid.org/0000-0002-0000-0004</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"700\" ind1=\"0\" ind2=\"\\\">\n    <marc:subfield code=\"a\">Translator,</marc:subfield>\n    <marc:subfield code=\"e\">translator.</marc:subfield>\n    <marc:subfield code=\"u\">COPIM.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"710\" ind1=\"2\" ind2=\"\\\">\n    <marc:subfield code=\"a\">OA Editions,</marc:subfield>\n    <marc:subfield code=\"e\">publisher.</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"830\" ind1=\"\\\" ind2=\"0\">\n    <marc:subfield code=\"a\">Name of series</marc:subfield>\n    <marc:subfield code=\"v\">vol. 11</marc:subfield>\n    <marc:subfield code=\"x\">8765-4321</marc:subfield>\n    <marc:subfield code=\"x\">1234-5678</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"0\">\n    <marc:subfield code=\"u\">https://doi.org/10.00001/book.0001</marc:subfield>\n    <marc:subfield code=\"z\">Connect to e-book</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"2\">\n    <marc:subfield code=\"u\">https://www.book.com/cover.jpg</marc:subfield>\n    <marc:subfield code=\"z\">Connect to cover image</marc:subfield>\n  </marc:datafield>\n  <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"2\">\n    <marc:subfield code=\"u\">https://creativecommons.org/publicdomain/zero/1.0/</marc:subfield>\n    <marc:subfield code=\"z\">CC0 Metadata License</marc:subfield>\n  </marc:datafield>\n</marc:record>");

        assert_eq!(Marc21XmlThoth {}.generate(&[work]), Ok(expected))
    }

    #[test]
    fn test_generate_marc_xml_multiple_works() {
        let works = &[test_work(), test_work()];
        let current_date = Utc::now().format("%y%m%d").to_string();
        let expected = format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<marc:collection xmlns:marc=\"http://www.loc.gov/MARC21/slim\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://www.loc.gov/MARC21/slim http://www.loc.gov/standards/marcxml/schema/MARC21slim.xsd\">\n  <marc:record>\n    <marc:leader>02336nam  22005292  4500</marc:leader>\n    <marc:controlfield tag=\"001\">00000000-0000-0000-aaaa-000000000001</marc:controlfield>\n    <marc:controlfield tag=\"006\">m     o  d        </marc:controlfield>\n    <marc:controlfield tag=\"007\">cr  n         </marc:controlfield>\n    <marc:controlfield tag=\"008\">{current_date}t20102010        ob    000 0 eng d</marc:controlfield>\n    <marc:datafield tag=\"010\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">LCCN010101</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">9783161484100</marc:subfield>\n      <marc:subfield code=\"q\">(PDF)</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">9789295055025</marc:subfield>\n      <marc:subfield code=\"q\">(XML)</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"z\">9781402894626</marc:subfield>\n      <marc:subfield code=\"q\">(Hardback)</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"024\" ind1=\"7\" ind2=\"\\\">\n      <marc:subfield code=\"a\">10.00001/BOOK.0001</marc:subfield>\n      <marc:subfield code=\"2\">doi</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"024\" ind1=\"7\" ind2=\"\\\">\n      <marc:subfield code=\"a\">OCLC010101</marc:subfield>\n      <marc:subfield code=\"2\">worldcat</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"040\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">UkCbTOM</marc:subfield>\n      <marc:subfield code=\"b\">eng</marc:subfield>\n      <marc:subfield code=\"e\">rda</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"041\" ind1=\"1\" ind2=\"\\\">\n      <marc:subfield code=\"a\">eng</marc:subfield>\n      <marc:subfield code=\"h\">spa</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"050\" ind1=\"0\" ind2=\"0\">\n      <marc:subfield code=\"a\">JA85</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n      <marc:subfield code=\"a\">AAB</marc:subfield>\n      <marc:subfield code=\"2\">bicssc</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n      <marc:subfield code=\"a\">AAA000000</marc:subfield>\n      <marc:subfield code=\"2\">bisacsh</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n      <marc:subfield code=\"a\">JWA</marc:subfield>\n      <marc:subfield code=\"2\">thema</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"100\" ind1=\"1\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Author, Sole,</marc:subfield>\n      <marc:subfield code=\"e\">author.</marc:subfield>\n      <marc:subfield code=\"u\">Thoth University.</marc:subfield>\n      <marc:subfield code=\"1\">https://orcid.org/0000-0002-0000-0001</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"245\" ind1=\"1\" ind2=\"0\">\n      <marc:subfield code=\"a\">Book Title :</marc:subfield>\n      <marc:subfield code=\"b\">Book Subtitle /</marc:subfield>\n      <marc:subfield code=\"c\">Sole Author; edited by Only Editor; translated by Translator.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"250\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">1st edition</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"264\" ind1=\"\\\" ind2=\"1\">\n      <marc:subfield code=\"a\">León, Spain :</marc:subfield>\n      <marc:subfield code=\"b\">OA Editions,</marc:subfield>\n      <marc:subfield code=\"c\">2010.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"264\" ind1=\"\\\" ind2=\"4\">\n      <marc:subfield code=\"c\">©2010</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"300\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">1 online resource.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"336\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">text</marc:subfield>\n      <marc:subfield code=\"b\">txt</marc:subfield>\n      <marc:subfield code=\"2\">rdacontent</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"337\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">computer</marc:subfield>\n      <marc:subfield code=\"b\">c</marc:subfield>\n      <marc:subfield code=\"2\">rdamedia</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"338\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">online resource</marc:subfield>\n      <marc:subfield code=\"b\">cr</marc:subfield>\n      <marc:subfield code=\"2\">rdacarrier</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"490\" ind1=\"1\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Name of series</marc:subfield>\n      <marc:subfield code=\"v\">vol. 11</marc:subfield>\n      <marc:subfield code=\"x\">8765-4321</marc:subfield>\n      <marc:subfield code=\"x\">1234-5678</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"500\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Please note that in this book the mathematical formulas are encoded in MathML.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"504\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Includes bibliography (pages 165-170) and index.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"505\" ind1=\"0\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Introduction; Chapter 1; Chapter 2; Bibliography; Index</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"506\" ind1=\"0\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Open Access</marc:subfield>\n      <marc:subfield code=\"f\">Unrestricted online access</marc:subfield>\n      <marc:subfield code=\"2\">star</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"520\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Lorem ipsum dolor sit amet</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"536\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Funding Institution</marc:subfield>\n      <marc:subfield code=\"c\">JA0001</marc:subfield>\n      <marc:subfield code=\"e\">Funding Programme</marc:subfield>\n      <marc:subfield code=\"f\">Funding Project</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"538\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Mode of access: World Wide Web.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"540\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">The text of this book is licensed under a Creative Commons Attribution 4.0 International license (CC BY 4.0). For more detailed information consult the publisher's website.</marc:subfield>\n      <marc:subfield code=\"u\">https://creativecommons.org/licenses/by/4.0/</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"588\" ind1=\"0\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Metadata licensed under CC0 Public Domain Dedication.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"700\" ind1=\"1\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Editor, Only,</marc:subfield>\n      <marc:subfield code=\"e\">editor.</marc:subfield>\n      <marc:subfield code=\"1\">https://orcid.org/0000-0002-0000-0004</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"700\" ind1=\"0\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Translator,</marc:subfield>\n      <marc:subfield code=\"e\">translator.</marc:subfield>\n      <marc:subfield code=\"u\">COPIM.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"710\" ind1=\"2\" ind2=\"\\\">\n      <marc:subfield code=\"a\">OA Editions,</marc:subfield>\n      <marc:subfield code=\"e\">publisher.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"830\" ind1=\"\\\" ind2=\"0\">\n      <marc:subfield code=\"a\">Name of series</marc:subfield>\n      <marc:subfield code=\"v\">vol. 11</marc:subfield>\n      <marc:subfield code=\"x\">8765-4321</marc:subfield>\n      <marc:subfield code=\"x\">1234-5678</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"0\">\n      <marc:subfield code=\"u\">https://doi.org/10.00001/book.0001</marc:subfield>\n      <marc:subfield code=\"z\">Connect to e-book</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"2\">\n      <marc:subfield code=\"u\">https://www.book.com/cover.jpg</marc:subfield>\n      <marc:subfield code=\"z\">Connect to cover image</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"2\">\n      <marc:subfield code=\"u\">https://creativecommons.org/publicdomain/zero/1.0/</marc:subfield>\n      <marc:subfield code=\"z\">CC0 Metadata License</marc:subfield>\n    </marc:datafield>\n  </marc:record>\n  <marc:record>\n    <marc:leader>02336nam  22005292  4500</marc:leader>\n    <marc:controlfield tag=\"001\">00000000-0000-0000-aaaa-000000000001</marc:controlfield>\n    <marc:controlfield tag=\"006\">m     o  d        </marc:controlfield>\n    <marc:controlfield tag=\"007\">cr  n         </marc:controlfield>\n    <marc:controlfield tag=\"008\">{current_date}t20102010        ob    000 0 eng d</marc:controlfield>\n    <marc:datafield tag=\"010\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">LCCN010101</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">9783161484100</marc:subfield>\n      <marc:subfield code=\"q\">(PDF)</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">9789295055025</marc:subfield>\n      <marc:subfield code=\"q\">(XML)</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"020\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"z\">9781402894626</marc:subfield>\n      <marc:subfield code=\"q\">(Hardback)</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"024\" ind1=\"7\" ind2=\"\\\">\n      <marc:subfield code=\"a\">10.00001/BOOK.0001</marc:subfield>\n      <marc:subfield code=\"2\">doi</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"024\" ind1=\"7\" ind2=\"\\\">\n      <marc:subfield code=\"a\">OCLC010101</marc:subfield>\n      <marc:subfield code=\"2\">worldcat</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"040\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">UkCbTOM</marc:subfield>\n      <marc:subfield code=\"b\">eng</marc:subfield>\n      <marc:subfield code=\"e\">rda</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"041\" ind1=\"1\" ind2=\"\\\">\n      <marc:subfield code=\"a\">eng</marc:subfield>\n      <marc:subfield code=\"h\">spa</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"050\" ind1=\"0\" ind2=\"0\">\n      <marc:subfield code=\"a\">JA85</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n      <marc:subfield code=\"a\">AAB</marc:subfield>\n      <marc:subfield code=\"2\">bicssc</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n      <marc:subfield code=\"a\">AAA000000</marc:subfield>\n      <marc:subfield code=\"2\">bisacsh</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"072\" ind1=\" \" ind2=\"7\">\n      <marc:subfield code=\"a\">JWA</marc:subfield>\n      <marc:subfield code=\"2\">thema</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"100\" ind1=\"1\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Author, Sole,</marc:subfield>\n      <marc:subfield code=\"e\">author.</marc:subfield>\n      <marc:subfield code=\"u\">Thoth University.</marc:subfield>\n      <marc:subfield code=\"1\">https://orcid.org/0000-0002-0000-0001</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"245\" ind1=\"1\" ind2=\"0\">\n      <marc:subfield code=\"a\">Book Title :</marc:subfield>\n      <marc:subfield code=\"b\">Book Subtitle /</marc:subfield>\n      <marc:subfield code=\"c\">Sole Author; edited by Only Editor; translated by Translator.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"250\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">1st edition</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"264\" ind1=\"\\\" ind2=\"1\">\n      <marc:subfield code=\"a\">León, Spain :</marc:subfield>\n      <marc:subfield code=\"b\">OA Editions,</marc:subfield>\n      <marc:subfield code=\"c\">2010.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"264\" ind1=\"\\\" ind2=\"4\">\n      <marc:subfield code=\"c\">©2010</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"300\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">1 online resource.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"336\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">text</marc:subfield>\n      <marc:subfield code=\"b\">txt</marc:subfield>\n      <marc:subfield code=\"2\">rdacontent</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"337\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">computer</marc:subfield>\n      <marc:subfield code=\"b\">c</marc:subfield>\n      <marc:subfield code=\"2\">rdamedia</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"338\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">online resource</marc:subfield>\n      <marc:subfield code=\"b\">cr</marc:subfield>\n      <marc:subfield code=\"2\">rdacarrier</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"490\" ind1=\"1\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Name of series</marc:subfield>\n      <marc:subfield code=\"v\">vol. 11</marc:subfield>\n      <marc:subfield code=\"x\">8765-4321</marc:subfield>\n      <marc:subfield code=\"x\">1234-5678</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"500\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Please note that in this book the mathematical formulas are encoded in MathML.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"504\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Includes bibliography (pages 165-170) and index.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"505\" ind1=\"0\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Introduction; Chapter 1; Chapter 2; Bibliography; Index</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"506\" ind1=\"0\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Open Access</marc:subfield>\n      <marc:subfield code=\"f\">Unrestricted online access</marc:subfield>\n      <marc:subfield code=\"2\">star</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"520\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Lorem ipsum dolor sit amet</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"536\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Funding Institution</marc:subfield>\n      <marc:subfield code=\"c\">JA0001</marc:subfield>\n      <marc:subfield code=\"e\">Funding Programme</marc:subfield>\n      <marc:subfield code=\"f\">Funding Project</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"538\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Mode of access: World Wide Web.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"540\" ind1=\"\\\" ind2=\"\\\">\n      <marc:subfield code=\"a\">The text of this book is licensed under a Creative Commons Attribution 4.0 International license (CC BY 4.0). For more detailed information consult the publisher's website.</marc:subfield>\n      <marc:subfield code=\"u\">https://creativecommons.org/licenses/by/4.0/</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"588\" ind1=\"0\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Metadata licensed under CC0 Public Domain Dedication.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"700\" ind1=\"1\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Editor, Only,</marc:subfield>\n      <marc:subfield code=\"e\">editor.</marc:subfield>\n      <marc:subfield code=\"1\">https://orcid.org/0000-0002-0000-0004</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"700\" ind1=\"0\" ind2=\"\\\">\n      <marc:subfield code=\"a\">Translator,</marc:subfield>\n      <marc:subfield code=\"e\">translator.</marc:subfield>\n      <marc:subfield code=\"u\">COPIM.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"710\" ind1=\"2\" ind2=\"\\\">\n      <marc:subfield code=\"a\">OA Editions,</marc:subfield>\n      <marc:subfield code=\"e\">publisher.</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"830\" ind1=\"\\\" ind2=\"0\">\n      <marc:subfield code=\"a\">Name of series</marc:subfield>\n      <marc:subfield code=\"v\">vol. 11</marc:subfield>\n      <marc:subfield code=\"x\">8765-4321</marc:subfield>\n      <marc:subfield code=\"x\">1234-5678</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"0\">\n      <marc:subfield code=\"u\">https://doi.org/10.00001/book.0001</marc:subfield>\n      <marc:subfield code=\"z\">Connect to e-book</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"2\">\n      <marc:subfield code=\"u\">https://www.book.com/cover.jpg</marc:subfield>\n      <marc:subfield code=\"z\">Connect to cover image</marc:subfield>\n    </marc:datafield>\n    <marc:datafield tag=\"856\" ind1=\"4\" ind2=\"2\">\n      <marc:subfield code=\"u\">https://creativecommons.org/publicdomain/zero/1.0/</marc:subfield>\n      <marc:subfield code=\"z\">CC0 Metadata License</marc:subfield>\n    </marc:datafield>\n  </marc:record>\n</marc:collection>");

        assert_eq!(Marc21XmlThoth {}.generate(works), Ok(expected))
    }
}
