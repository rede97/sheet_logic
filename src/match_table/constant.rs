use super::MatchTableColumn;
use std::collections::HashMap;

// number, case index
pub struct SegConstantCase(pub HashMap<u128, Vec<usize>>);

pub struct SegsConstantCase {
    // segment index, case
    pub segs_set: HashMap<usize, SegConstantCase>,
}

pub struct ConstantCasePrinter<'a> {
    pub cc: &'a SegsConstantCase,
    pub header: &'a Vec<MatchTableColumn>,
}

impl<'a> ConstantCasePrinter<'a> {
    pub fn new(cc: &'a SegsConstantCase, header: &'a Vec<MatchTableColumn>) -> Self {
        return ConstantCasePrinter { cc, header };
    }
}

impl<'a> std::fmt::Display for ConstantCasePrinter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys: Vec<&usize> = self.cc.segs_set.keys().collect();
        keys.sort();
        for k in keys {
            let signal = match &self.header[*k] {
                MatchTableColumn::Segment(signal) => signal,
                _ => unreachable!(),
            };
            let mut constants: Vec<(&u128, &Vec<usize>)> = self.cc.segs_set[k].0.iter().collect();
            constants.sort_by(|a, b| a.0.cmp(b.0));
            writeln!(f, "{}: {:?}", signal.as_str(), constants)?;
        }
        return Ok(());
    }
}

impl SegsConstantCase {
    pub fn new() -> Self {
        return SegsConstantCase {
            segs_set: HashMap::new(),
        };
    }

    pub fn insert(&mut self, case_idx: usize, seg_idx: usize, constant: u128) {
        if !self.segs_set.contains_key(&seg_idx) {
            self.segs_set
                .insert(seg_idx, SegConstantCase(HashMap::new()));
        }

        let case = self.segs_set.get_mut(&seg_idx).unwrap();
        match case.0.get_mut(&constant) {
            Some(v) => {
                v.push(case_idx);
            }
            None => {
                let v = vec![case_idx];
                case.0.insert(constant, v);
            }
        }

        // .insert(constant as u64);
    }
}
