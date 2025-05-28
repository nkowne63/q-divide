use crate::utils::set_partition::SetPartition;
use crate::utils::counters::Counter;
use super::gate_fragments::GateFragment;

static GLOBAL_CIRCUIT_FRAGMENT_ID: Counter = Counter::new();

use std::collections::HashMap;

pub struct BacktrackableEdges {
    pub edges: HashMap<usize, usize>,
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
    pub edges: HashMap<Vec<usize>, Vec<(usize, String)>>,
    pub labels: HashMap<String, Vec<usize>>,
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
    pub control_edges: SetPartition,
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
            control_edges: {
                let mut partition = SetPartition::new();
                for i in 0..len {
                    partition.add_element(i);
                }
                partition
            },
            feedback_edges: LabeledFeedbackEdges::new(),
        }
    }

    pub fn add_gate_fragment(&mut self, gate_fragment: GateFragment) -> Result<(), String> {
        self.gate_fragments.push(gate_fragment);
        // Ensure we have enough elements in the partition
        let current_len = self.control_edges.count_elements();
        let needed_len = self.gate_fragments.len();
        if needed_len > current_len {
            for i in current_len..needed_len {
                self.control_edges.add_element(i);
            }
        }

        Ok(())
    }

    pub fn remove_gate_fragment(&mut self, id: usize) -> Result<(), String> {
        let fragment_index = self.gate_fragments
            .iter()
            .position(|fragment| fragment.id == id)
            .ok_or(format!("Gate fragment with id {} not found", id))?;
        
        // Store the current groups before removing the fragment
        let groups = self.control_edges.get_groups();
        
        // Remove the fragment
        self.gate_fragments.remove(fragment_index);
        
        // Rebuild control_edges from scratch
        let mut new_control_edges = SetPartition::new();
        for i in 0..self.gate_fragments.len() {
            new_control_edges.add_element(i);
        }
        
        // Restore the groups, adjusting indices
        for (_representative, elements) in groups {
            // Filter out the removed element and adjust indices
            let adjusted_elements: Vec<usize> = elements
                .into_iter()
                .filter_map(|old_idx| {
                    if old_idx < fragment_index {
                        Some(old_idx)
                    } else if old_idx > fragment_index {
                        Some(old_idx - 1)
                    } else {
                        None // This was the removed element
                    }
                })
                .collect();
            
            // Union all elements in this group
            if adjusted_elements.len() > 1 {
                let first = adjusted_elements[0];
                for &element in &adjusted_elements[1..] {
                    new_control_edges.union_elements(first, element);
                }
            }
        }
        
        self.control_edges = new_control_edges;
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
    pub fn connect_control_edges(&mut self, a: usize, b: usize) -> Result<(), String> {
        self.control_edges.union_elements(a, b);
        Ok(())
    }
    pub fn divide_control_edges(&mut self, a: usize) -> Result<(), String> {
        self.control_edges.remove_element(a);
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

    pub fn join_fragments(_a: &Self, _b: &Self) -> Self {
        todo!() // join circuit fragments
    }

    pub fn divide(&self, predicate: Vec<bool>) -> Result<(Self, Self), String> {
        if predicate.len() != self.gate_fragments.len() {
            return Err(format!("Predicate length {} does not match circuit fragment length {}", predicate.len(), self.gate_fragments.len()));
        }
        todo!() // divide circuit fragments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn new() {
        let c = CircuitFragment::new(vec![]);
        assert_eq!(c.gate_fragments.len(), 0);
        assert_eq!(c.qubit_edges.edges.len(), 0);
        assert_eq!(c.qubit_edges.backtrack.len(), 0);
        assert_eq!(c.control_edges.count_elements(), 0);
        assert_eq!(c.feedback_edges.edges.len(), 0);
    }

    #[test]
    fn add_gate_fragment() {
        use super::super::gate_fragments::{GateFragment, GateFragmentLabel, Unitary};
        let mut c = CircuitFragment::new(vec![]);
        c.add_gate_fragment(GateFragment::new(GateFragmentLabel::Unitary(Unitary::H))).unwrap();
        assert_eq!(c.gate_fragments.len(), 1);
        assert_eq!(c.control_edges.count_elements(), 1);
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
        assert_eq!(c.control_edges.count_elements(), 0);
    }

    #[test]
    fn connect_control_edges() {
        use super::super::gate_fragments::{GateFragment, GateFragmentLabel, Unitary};
        let mut c = CircuitFragment::new(vec![]);
        
        // Add two gate fragments
        c.add_gate_fragment(GateFragment::new(GateFragmentLabel::Unitary(Unitary::H))).unwrap();
        c.add_gate_fragment(GateFragment::new(GateFragmentLabel::Unitary(Unitary::X))).unwrap();
        
        // Initially they should be in separate groups
        assert!(!c.control_edges.is_same_set(0, 1));
        
        // Connect them
        c.connect_control_edges(0, 1).unwrap();
        
        // Now they should be in the same group
        assert!(c.control_edges.is_same_set(0, 1));
        
        // Test get_groups method
        let groups = c.control_edges.get_groups();
        assert_eq!(groups.len(), 1); // Only one group now
        assert!(groups.values().any(|group| group.contains(&0) && group.contains(&1)));
    }

    #[test]
    fn divide_control_edges() {
        use super::super::gate_fragments::{GateFragment, GateFragmentLabel, Unitary};
        let mut c = CircuitFragment::new(vec![]);
        
        // Add three gate fragments
        c.add_gate_fragment(GateFragment::new(GateFragmentLabel::Unitary(Unitary::H))).unwrap();
        c.add_gate_fragment(GateFragment::new(GateFragmentLabel::Unitary(Unitary::X))).unwrap();
        c.add_gate_fragment(GateFragment::new(GateFragmentLabel::Unitary(Unitary::Y))).unwrap();
        
        // Connect 0 and 1
        c.connect_control_edges(0, 1).unwrap();
        assert!(c.control_edges.is_same_set(0, 1));
        assert!(!c.control_edges.is_same_set(0, 2));
        
        // Divide element 0 from its group
        c.divide_control_edges(0).unwrap();
        
        // Now 0 should be isolated (removed), 1 and 2 should still be separate
        assert!(!c.control_edges.is_same_set(0, 1));
        assert!(!c.control_edges.is_same_set(1, 2));
        
        // Element 0 should not be in any group anymore
        let same_group = c.control_edges.get_same_group(0);
        assert_eq!(same_group, vec![0]); // Returns itself when not found
    }

    #[test]
    fn set_partition_compatibility_methods() {
        use super::super::gate_fragments::{GateFragment, GateFragmentLabel, Unitary};
        let mut c = CircuitFragment::new(vec![]);
        
        // Add multiple gate fragments
        for _ in 0..5 {
            c.add_gate_fragment(GateFragment::new(GateFragmentLabel::Unitary(Unitary::H))).unwrap();
        }
        
        // Create some groups: {0,1}, {2,3}, {4}
        c.connect_control_edges(0, 1).unwrap();
        c.connect_control_edges(2, 3).unwrap();
        
        // Test find method - should return the smallest element in each group
        assert_eq!(c.control_edges.find(0), 0); // 0 is representative of {0,1}
        assert_eq!(c.control_edges.find(1), 0); // 0 is representative of {0,1}
        assert_eq!(c.control_edges.find(2), 2); // 2 is representative of {2,3}
        assert_eq!(c.control_edges.find(3), 2); // 2 is representative of {2,3}
        assert_eq!(c.control_edges.find(4), 4); // 4 is representative of {4}
        
        // Test get_groups
        let groups = c.control_edges.get_groups();
        assert_eq!(groups.len(), 3); // Three groups
        assert_eq!(groups.get(&0), Some(&vec![0, 1]));
        assert_eq!(groups.get(&2), Some(&vec![2, 3]));
        assert_eq!(groups.get(&4), Some(&vec![4]));
        
        // Test get_same_group
        assert_eq!(c.control_edges.get_same_group(0), vec![0, 1]);
        assert_eq!(c.control_edges.get_same_group(1), vec![0, 1]);
        assert_eq!(c.control_edges.get_same_group(2), vec![2, 3]);
        assert_eq!(c.control_edges.get_same_group(3), vec![2, 3]);
        assert_eq!(c.control_edges.get_same_group(4), vec![4]);
    }
}