use super::SignalKey;
use super::SignalWidth;
use super::{Signal, SignalSource};
use std::collections::HashMap;

pub struct Module {
    input: Vec<SignalKey>,
    output: Vec<SignalKey>,
    signals: HashMap<SignalKey, Signal>,
}

#[allow(dead_code)]
impl Module {
    pub fn new() -> Module {
        return Module {
            input: Vec::new(),
            output: Vec::new(),
            signals: HashMap::new(),
        };
    }

    pub fn new_input(&mut self, name: String, length: SignalWidth) {
        let key = self.new_signal(name, length);
        self.input.push(key);
    }

    pub fn new_output(&mut self, name: String, length: SignalWidth) {
        let key = self.new_signal(name, length);
        self.output.push(key);
    }

    pub fn new_signal(&mut self, name: String, length: SignalWidth) -> SignalKey {
        let key: SignalKey = name.into();
        let signal = Signal::new(key.clone(), length, SignalSource::Unconnected);
        assert_eq!(false, self.signals.contains_key(&key));
        self.signals.insert(key.clone(), signal);
        return key;
    }

    pub fn get_signals(&self) -> &HashMap<SignalKey, Signal> {
        return &self.signals;
    }

    pub fn get_signals_mut(&mut self) -> &mut HashMap<SignalKey, Signal> {
        return &mut self.signals;
    }

    pub fn add_signal(&mut self, signal: Signal) {
        assert_eq!(false, self.signals.contains_key(&signal.key));
        self.signals.insert(signal.key.clone(), signal);
    }
}
