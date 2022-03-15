mod excel;
mod sheet;
mod position;

pub use excel::*;
pub use sheet::*;
pub use position::*;

use quick_xml::events::BytesStart;
use quick_xml::{self};
use std::borrow::Cow;

fn get_xml_attribute<'a>(e: &'a BytesStart, key: &[u8]) -> Option<Cow<'a, [u8]>> {
    for attr in e.attributes() {
        match attr {
            Ok(ref attr) if attr.key == key => {
                return Some(attr.value.clone());
            }
            _ => {}
        }
    }
    return None;
}
