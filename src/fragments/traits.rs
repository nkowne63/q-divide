use std::{cell::RefCell, rc::Rc};

pub trait GateFragmentLabel {}

pub trait GateFragment {}

// not just vec of gate fragments, but a vec without instruments
pub trait FragmentPartition {}

// not just vec of gate fragments, but a vec with instruments
pub trait FragmentInstruments {}

type Ref<T> = Rc<RefCell<T>>;

type SuceesOrMessage = Result<(), String>;

pub trait FragmentLike {
    fn add_gate(&mut self, gate: impl GateFragmentLabel) -> Result<Ref<impl GateFragment>, String>;
    fn remove_gate(&mut self, gate: Ref<impl GateFragment>) -> SuceesOrMessage;
    fn qubit_connect(&mut self, src_gate: Ref<impl GateFragment>, dst_gate: Ref<impl GateFragment>) -> SuceesOrMessage;
    fn qubit_disconnect(&mut self, src_gate: Ref<impl GateFragment>, dst_gate: Ref<impl GateFragment>) -> SuceesOrMessage;
    fn control_connect(&mut self, target_partition: Ref<impl FragmentPartition>) -> SuceesOrMessage;
    fn control_disconnect(&mut self, partition_a: Ref<impl FragmentPartition>, partition_b: Ref<impl FragmentPartition>) -> SuceesOrMessage;
    fn feedback_connect(&mut self, src_instruments: Ref<impl FragmentInstruments>, dst_partition: Ref<impl FragmentPartition>, bool_function_label: String) -> SuceesOrMessage;
    fn feedback_disconnect(&mut self, src_instruments: Ref<impl FragmentInstruments>, dst_partition: Ref<impl FragmentPartition>) -> SuceesOrMessage;
}

pub trait Gate {}

pub struct Circuit {}

pub trait CircuitExtraction {
    fn extract_circuit(&self) -> Result<Circuit, String>;
}

pub trait CircuitLike: FragmentLike + CircuitExtraction {
    fn get_qubit_num(&self, fragment: Ref<impl GateFragment>) -> usize;
    fn apply_gate(&mut self, gate: Ref<impl Gate>, target_qubits: Vec<usize>) -> SuceesOrMessage;
    fn apply_fragment(&mut self, fragment: Ref<impl GateFragment>) -> SuceesOrMessage;
}
