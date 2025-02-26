use crate::utils::union_find::UnionFind;
use crate::utils::counters::Counter;
use super::gate_fragments::GateFragment;

static GLOBAL_CIRCUIT_FRAGMENT_ID: Counter = Counter::new();

use std::collections::HashMap;

pub struct BacktrackableEdges {
    edges: HashMap<usize, usize>,
    backtrack: HashMap<usize, usize>,
}

impl BacktrackableEdges {
    fn new() -> Self {
        BacktrackableEdges {
            edges: HashMap::new(),
            backtrack: HashMap::new(),
        }
    }
    fn insert(&mut self, a: usize, b: usize) {
        self.edges.insert(a, b);
        self.backtrack.insert(b, a);
    }
    fn remove(&mut self, a: usize) -> Option<usize>{
        let b = self.edges.remove(&a)?;
        self.backtrack.remove(&b)
    }
}

pub struct LabeledFeedbackEdges {
    edges: HashMap<Vec<usize>, Vec<(usize, String)>>,
    labels: HashMap<String, Vec<usize>>,
}

impl LabeledFeedbackEdges {
    fn new() -> Self {
        LabeledFeedbackEdges {
            edges: HashMap::new(),
            labels: HashMap::new(),
        }
    }
    fn insert(&mut self, measurements: Vec<usize>, target: usize, label: String) {
        self.edges.entry(measurements).or_insert(Vec::new()).push((target, label.clone()));
        self.labels.entry(label).or_insert(Vec::new()).push(target);
    }
    fn remove(&mut self, label: String) -> Option<()>{
        let measurements = self.labels.remove(&label)?;
        let edges = self.edges.get_mut(&measurements)?;
        edges.retain(|(_, l)| l != &label);
        if edges.is_empty() {
            self.edges.remove(&measurements)?;
        }
        return Some(());
    }
    fn has_label(&self, label: &str) -> bool {
        self.labels.contains_key(label)
    }
}

pub struct CircuitFragment {
    pub id: usize,
    pub gate_fragments: Vec<GateFragment>,
    pub qubit_edges: BacktrackableEdges,
    pub control_edges: UnionFind,
    pub feedback_edges: LabeledFeedbackEdges,
}

impl CircuitFragment {
    pub fn new(gate_fragments: Vec<GateFragment>) -> Self {
        let id = GLOBAL_CIRCUIT_FRAGMENT_ID.increment();
        let len = gate_fragments.len();
        CircuitFragment {
            id,
            gate_fragments: gate_fragments,
            qubit_edges: BacktrackableEdges::new(),
            control_edges: UnionFind::new(len),
            feedback_edges: LabeledFeedbackEdges::new(),
        }
    }

    pub fn add_gate_fragment(&mut self, gate_fragment: GateFragment) -> Result<(), String> {
        self.gate_fragments.push(gate_fragment);
        self.control_edges.ensure(self.gate_fragments.len());

        Ok(())
    }

    pub fn remove_gate_fragment(&mut self, id: usize) -> Result<(), String> {
        self.gate_fragments.retain(|fragment| fragment.id != id as usize);
        self.control_edges.remove(id);
        Ok(())
    }

    pub fn connect_qubit_edges(&mut self, source: usize, destination: usize) -> Result<(), String> {
        if self.qubit_edges.edges.contains_key(&source) {
            return Err(format!("GateFragment {} is already connected as start", source));
        };
        if self.qubit_edges.backtrack.contains_key(&destination) {
            return Err(format!("GateFragment {} is already connected as end", destination));
        };
        self.qubit_edges.insert(source, destination);
        Ok(())
    }
    pub fn disconnect_qubit_edges(&mut self, source: usize) -> Result<(), String> {
        self.qubit_edges.remove(source).ok_or(format!("GateFragment {} is not connected as start", source))?;
        Ok(())
    }
    pub fn unite_control_edges(&mut self, a: usize, b: usize) -> Result<(), String> {
        self.control_edges.union(a, b);
        Ok(())
    }
    pub fn eject_control_edges(&mut self, a: usize) -> Result<(), String> {
        self.control_edges.remove(a);
        Ok(())
    }
    pub fn connect_feedback_edges(&mut self, measurements: Vec<usize>, target: usize, label: String) -> Result<(), String> {
        if self.feedback_edges.has_label(&label) {
            return Err(format!("Label {} is already in use", label));
        }
        self.feedback_edges.insert(measurements, target, label);
        Ok(())
    }
    pub fn remove_feedback_edges(&mut self, label: String) -> Result<(), String> {
        self.feedback_edges.remove(label).ok_or("Label not found")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn new() {
        let c = CircuitFragment::new(vec![]);
        assert_eq!(c.id, 0);
        assert_eq!(c.gate_fragments.len(), 0);
        assert_eq!(c.qubit_edges.edges.len(), 0);
        assert_eq!(c.qubit_edges.backtrack.len(), 0);
        assert_eq!(c.control_edges.len(), 0);
        assert_eq!(c.feedback_edges.edges.len(), 0);
    }

    #[test]
    fn add_gate_fragment() {
        use super::super::gate_fragments::{GateFragment, GateFragmentLabel, Unitary};
        let mut c = CircuitFragment::new(vec![]);
        c.add_gate_fragment(GateFragment::new(GateFragmentLabel::Unitary(Unitary::H))).unwrap();
        assert_eq!(c.gate_fragments.len(), 1);
        assert_eq!(c.control_edges.len(), 1);
    }

    #[test]
    fn remove_gate_fragment() {
        use super::super::gate_fragments::{GateFragment, GateFragmentLabel, Unitary};
        let mut c = CircuitFragment::new(vec![]);
        let g = GateFragment::new(GateFragmentLabel::Unitary(Unitary::H));
        let g_id = g.id;
        c.add_gate_fragment(g).unwrap();
        c.remove_gate_fragment(g_id).unwrap();
        assert_eq!(c.gate_fragments.len(), 0);
        assert_eq!(c.control_edges.len(), 0);
    }
}