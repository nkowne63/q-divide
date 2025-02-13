use derive_more::Display;
use serde::{Serialize, Deserialize};


#[derive(Hash, Eq, PartialEq, Display, Clone, Copy, Serialize, Deserialize)]
pub enum Unitary {
    X,
    Y,
    Z,
    H,
    T,
    S,
    Tdag,
    Sdag,
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


use crate::utils::counters::Counter;

static GLOBAL_GATE_FRAGMENT_ID: Counter = Counter::new();

#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct GateFragment {
    id: usize,
    label: GateFragmentLabel,
    in_symbol: (),
    out_symbol: (),
}

impl GateFragment {
    pub fn new(label: GateFragmentLabel) -> Self {
        let id = GLOBAL_GATE_FRAGMENT_ID.increment();
        GateFragment {
            id,
            label,
            in_symbol: (),
            out_symbol: (),
        }
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
