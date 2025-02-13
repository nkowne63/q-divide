#[derive(Hash, Eq, PartialEq, derive_more::Display, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

#[derive(Hash, Eq, PartialEq, derive_more::Display, Clone, serde::Serialize, serde::Deserialize)]
pub enum Control {
    Truthy,
    Falsy,
}

#[derive(Hash, Eq, PartialEq, derive_more::Display, Clone, serde::Serialize, serde::Deserialize)]
pub enum GateFragmentLabel {
    Unitary(Unitary),
    Instrument,
    Control(Control),
}


use std::sync::atomic::{AtomicUsize, Ordering};

static GLOBAL_GATE_FRAGMENT_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Hash, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct GateFragment {
    id: usize,
    label: GateFragmentLabel,
}

impl GateFragment {
    pub fn new(label: GateFragmentLabel) -> Self {
        let id = GLOBAL_GATE_FRAGMENT_ID.fetch_add(1, Ordering::Relaxed);
        GateFragment {
            id,
            label,
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
