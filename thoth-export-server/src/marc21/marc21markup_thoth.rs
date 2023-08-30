use crate::marc21::Marc21RecordThoth;
use thoth_client::Work;
use thoth_errors::{ThothError, ThothResult};

use super::{Marc21Entry, Marc21Specification};

#[derive(Copy, Clone)]
pub(crate) struct Marc21MarkupThoth;

const MARC_ERROR: &str = "marc21markup::thoth";

impl Marc21Specification for Marc21MarkupThoth {
    fn generate(&self, works: &[Work]) -> ThothResult<String> {
        match works {
            [] => Err(ThothError::IncompleteMetadataRecord(
                MARC_ERROR.to_string(),
                "Not enough data".to_string(),
            )),
            [work] => Marc21Entry::<Marc21RecordThoth>::marc21_markup(work),
            _ => Ok(works
                .iter()
                .flat_map(Marc21Entry::<Marc21RecordThoth>::marc21_markup)
                .collect()),
        }
    }

    fn handle_event(_: &mut Vec<u8>, _: &[Work]) -> ThothResult<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marc21::marc21record_thoth::tests::test_work;
    use chrono::Utc;

    #[test]
    fn test_generate_marc_markup() {
        let work = test_work();
        let current_date = Utc::now().format("%y%m%d").to_string();
        let expected = format!("=LDR  02304nam  2200529 i 4500\n=001  00000000-0000-0000-aaaa-000000000001\n=006  m\\\\\\\\\\o\\\\d\\\\\\\\\\\\\\\\\n=007  cr\\\\n\\\\\\\\\\\\\\\\\\\n=008  {current_date}t20102010\\\\\\\\\\\\\\\\ob\\\\\\\\000\\0\\eng\\d\n=010  \\\\$aLCCN010101\n=020  \\\\$a9783161484100$q(PDF)\n=020  \\\\$a9789295055025$q(XML)\n=020  \\\\$z9781402894626$q(Hardback)\n=024  7\\$a10.00001/BOOK.0001$2doi\n=024  7\\$aOCLC010101$2worldcat\n=040  \\\\$aUkCbTOM$beng$erda\n=041  1\\$aeng$hspa\n=050  00$aJA85\n=072   7$aAAB$2bicssc\n=072   7$aAAA000000$2bisacsh\n=072   7$aJWA$2thema\n=100  1\\$aSole Author$eauthor$uThoth University$1https://orcid.org/0000-0002-0000-0001\n=245  00$aBook Title :$bBook Subtitle /$cSole Author; edited by Only Editor; translated by Translator.\n=250  \\\\$a1st edition\n=264  \\1$aLeón, Spain :$bOA Editions,$c2010.\n=264  \\4$c©2010\n=300  \\\\$a1 online resource.\n=336  \\\\$atext$btxt$2rdacontent\n=337  \\\\$acomputer$bc$2rdamedia\n=338  \\\\$aonline resource$bcr$2rdacarrier\n=490  1\\$aName of series$vvol. 11$x8765-4321\n=500  \\\\$aPlease note that in this book the mathematical formulas are encoded in MathML.\n=504  \\\\$aIncludes bibliography (pages 165-170) and index.\n=505  0\\$aIntroduction; Chapter 1; Chapter 2; Bibliography; Index\n=506  0\\$aOpen Access$fUnrestricted online access$2star\n=520  \\\\$aLorem ipsum dolor sit amet\n=536  \\\\$aFunding Institution$cJA0001$eFunding Programme$fFunding Project\n=538  \\\\$aMode of access: World Wide Web.\n=540  \\\\$aThe text of this book is licensed under a Creative Commons Attribution 4.0 International license (CC BY 4.0). For more detailed information consult the publisher's website.$uhttps://creativecommons.org/licenses/by/4.0/\n=588  0\\$aMetadata licensed under CC0 Public Domain Dedication.\n=700  1\\$aOnly Editor$eeditor$1https://orcid.org/0000-0002-0000-0004\n=700  1\\$aTranslator$etranslator$uCOPIM\n=710  2\\$aOA Editions,$epublisher.\n=830  \\0$aName of series$vvol. 11$x8765-4321\n=856  40$uhttps://doi.org/10.00001/book.0001$zConnect to e-book\n=856  42$uhttps://www.book.com/cover.jpg$zConnect to cover image\n=856  42$uhttps://creativecommons.org/publicdomain/zero/1.0/$zCC0 Metadata License\n");

        assert_eq!(Marc21MarkupThoth {}.generate(&[work]), Ok(expected))
    }
}
