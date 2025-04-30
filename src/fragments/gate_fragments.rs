use derive_more::Display;
use serde::{Serialize, Deserialize};


#[derive(Hash, Eq, PartialEq, Display, Clone, Serialize, Deserialize)]
pub enum Unitary {
    X,
    Y,
    Z,
    H,
    T,
    S,
    Tdag,
    Sdag,
    Other(String)
}

#[derive(Hash, Eq, PartialEq, Display, Clone, Serialize, Deserialize)]
pub enum Control {
    Truthy,
    Falsy,
}

#[derive(Hash, Eq, PartialEq, Display, Clone, Serialize, Deserialize)]
pub enum GateFragmentLabel {
    Unitary(Unitary),
    Instrument,
    Control(Control),
}

impl GateFragmentLabel {
    pub fn is_measurement(&self) -> bool {
        match self {
            GateFragmentLabel::Instrument => true,
            _ => false,
        }
    }
    pub fn is_unitary(&self) -> bool {
        match self {
            GateFragmentLabel::Unitary(_) => true,
            _ => false,
        }
    }
}

use crate::utils::counters::Counter;

pub static GLOBAL_GATE_FRAGMENT_ID: Counter = Counter::new();

#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct GateFragment {
    pub id: usize,
    pub label: GateFragmentLabel
}

impl GateFragment {
    pub fn new(label: GateFragmentLabel) -> Self {
        let id = GLOBAL_GATE_FRAGMENT_ID.increment();
        GateFragment {
            id,
            label
        }
    }
    pub fn is_measurement(&self) -> bool {
        self.label.is_measurement()
    }
}

impl From<GateFragmentLabel> for GateFragment {
    fn from(label: GateFragmentLabel) -> Self {
        GateFragment::new(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_fragment_creation() {
        let fragment1: GateFragment = GateFragmentLabel::Unitary(Unitary::X).into();
        let fragment2: GateFragment = GateFragmentLabel::Unitary(Unitary::Y).into();
        assert_ne!(fragment1.id, fragment2.id);
    }
}
