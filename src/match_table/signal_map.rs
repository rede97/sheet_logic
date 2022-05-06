use crate::verilog_model::SignalKey;
use std::collections::HashMap;
use std::ops::Range;
use std::vec::Vec;

#[derive(Debug)]
pub struct SignalMapSlot {
    pub ranges: Vec<(u16, u16)>,
    pub segs: Range<usize>,
}

#[derive(Debug)]
pub struct SignalMapSlots {
    pub slots: Vec<SignalMapSlot>,
    pub slot_case: HashMap<usize, Vec<usize>>,
}

impl SignalMapSlots {
    pub fn new() -> Self {
        return {
            SignalMapSlots {
                slots: Vec::new(),
                slot_case: HashMap::new(),
            }
        };
    }

    pub fn insert(&mut self, case_idx: usize, ranges: Vec<(u16, u16)>, segs: Range<usize>) {
        let slot_idx = match self
            .slots
            .iter()
            .position(|s| s.ranges == ranges && s.segs == segs)
        {
            Some(slot_idx) => slot_idx,
            None => {
                let slot_idx = self.slots.len();
                self.slots.push(SignalMapSlot {
                    ranges: ranges,
                    segs: segs,
                });
                slot_idx
            }
        };

        match self.slot_case.get_mut(&case_idx) {
            Some(v) => {
                v.push(slot_idx);
            }
            None => {
                let v = vec![slot_idx];
                self.slot_case.insert(case_idx, v);
            }
        }
    }
}

#[derive(Debug)]
pub struct SignalMapCase(pub HashMap<SignalKey, SignalMapSlots>);

impl SignalMapCase {
    pub fn new() -> Self {
        return SignalMapCase(HashMap::new());
    }

    pub fn insert(
        &mut self,
        signal_key: &SignalKey,
        case_idx: usize,
        ranges: Vec<(u16, u16)>,
        segs: Range<usize>,
    ) {
        if !self.0.contains_key(signal_key) {
            self.0.insert(signal_key.clone(), SignalMapSlots::new());
        }
        self.0
            .get_mut(signal_key)
            .unwrap()
            .insert(case_idx, ranges, segs);
    }
}
