use std::io::Write;
use thoth_client::Work;
use xml::writer::{EventWriter, Result};

use super::XmlSpecification;

pub struct Onix3Oapen {}

impl XmlSpecification for Onix3Oapen {
    fn handle_event<W: Write>(_w: &mut EventWriter<W>, _work: &[Work]) -> Result<()> {
        todo!()
    }
}
