use quick_xml::{self, events::Event, Reader};
use std::fs;
use std::io::{Cursor, Read};
use std::rc::Rc;
use zip::ZipArchive;

use super::{get_xml_attribute, Sheet};

#[allow(dead_code)]
pub struct Excel {
    shared_strings: Vec<Rc<String>>,
    archive: ZipArchive<Cursor<Vec<u8>>>,
}

#[allow(dead_code)]
impl Excel {
    pub fn open(file: &str) -> Excel {
        let reader = std::io::Cursor::new(fs::read(file).expect(file));
        let mut archive = ZipArchive::new(reader).expect(file);
        return Excel {
            shared_strings: Excel::shared_strings(&mut archive),
            archive,
        };
    }

    fn get_xml(archive: &mut ZipArchive<Cursor<Vec<u8>>>, path: &str) -> String {
        let mut doc = archive.by_name(path).expect(path);
        let mut content = String::new();
        doc.read_to_string(&mut content).unwrap();
        return content;
    }

    fn shared_strings(archive: &mut ZipArchive<Cursor<Vec<u8>>>) -> Vec<Rc<String>> {
        let content = Excel::get_xml(archive, "xl/sharedStrings.xml");
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);

        let mut buf = Vec::with_capacity(64);

        let vec_len = match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == b"sst" => get_xml_attribute(e, b"count"),
            _ => None,
        }
        .and_then(|attr| unsafe { std::str::from_utf8_unchecked(&attr).parse::<usize>().ok() });

        let mut shared_strings: Vec<Rc<String>> = Vec::with_capacity(match vec_len {
            Some(len) => len,
            None => 128,
        });

        let mut selected: bool = false;

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"t" => {
                        selected = true;
                    }
                    _ => {}
                },
                Ok(Event::End(ref e)) => match e.name() {
                    b"t" => {
                        selected = false;
                    }
                    _ => {}
                },
                Ok(Event::Text(e)) => {
                    if selected {
                        let value = Rc::new(String::from_utf8(e.to_vec()).unwrap());
                        shared_strings.push(value);
                    }
                }
                Ok(Event::Eof) => {
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    panic!("{} {:?}", reader.buffer_position(), e);
                }
            }
            buf.clear();
        }
        return shared_strings;
    }

    pub fn sheets(&mut self) -> Vec<String> {
        let mut sheets = Vec::new();
        let content = Excel::get_xml(&mut self.archive, "xl/workbook.xml");

        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);

        let mut buf = Vec::with_capacity(64);
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Empty(ref e)) => match e.name() {
                    b"sheet" => {
                        if let Some(a) = get_xml_attribute(e, b"name") {
                            sheets.push(String::from_utf8(a.to_vec()).unwrap().to_lowercase());
                        };
                    }
                    _ => {}
                },
                Ok(Event::Eof) => {
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    panic!("{} {:?}", reader.buffer_position(), e);
                }
            }
            buf.clear();
        }

        return sheets;
    }

    pub fn sheet(&mut self, sheet: &str) -> Sheet {
        let sheet_xml = Excel::get_xml(
            &mut self.archive,
            format!("xl/worksheets/{}.xml", sheet).as_str(),
        );
        return Sheet::from_xml(&sheet_xml, &self.shared_strings);
    }
}
