use std::{cell::RefCell, rc::Rc};

pub trait GateFragmentLabel {}

pub trait GateFragment {}

// not just vec of gate fragments, but a vec without instruments
pub trait FragmentPartition {}

// not just vec of gate fragments, but a vec with instruments
pub trait FragmentInstruments {}

type Ref<T> = Rc<RefCell<T>>;

pub type SuceesOrMessage = Result<(), String>;
pub type GateFragmentRef = Ref<dyn GateFragment>;
pub type FragmentPartitionRef = Ref<dyn FragmentPartition>;
pub type FragmentInstrumentsRef = Ref<dyn FragmentInstruments>;

pub trait FragmentLike {
    fn add_gate(&mut self, gate: Box<dyn GateFragmentLabel>) -> Result<GateFragmentRef, String>;
    fn remove_gate(&mut self, gate: GateFragmentRef) -> SuceesOrMessage;
    fn qubit_connect(
        &mut self,
        src_gate: GateFragmentRef,
        dst_gate: GateFragmentRef,
    ) -> SuceesOrMessage;
    fn qubit_disconnect(
        &mut self,
        src_gate: GateFragmentRef,
        dst_gate: GateFragmentRef,
    ) -> SuceesOrMessage;
    fn control_connect(&mut self, target_partition: FragmentPartitionRef) -> SuceesOrMessage;
    fn control_disconnect(
        &mut self,
        partition_a: FragmentPartitionRef,
        partition_b: FragmentPartitionRef,
    ) -> SuceesOrMessage;
    fn feedback_connect(
        &mut self,
        src_instruments: FragmentInstrumentsRef,
        dst_partition: FragmentPartitionRef,
        bool_function_label: String,
    ) -> SuceesOrMessage;
    fn feedback_disconnect(
        &mut self,
        src_instruments: FragmentInstrumentsRef,
        dst_partition: FragmentPartitionRef,
    ) -> SuceesOrMessage;
}

pub trait Gate {}

pub struct Circuit {}

pub trait CircuitExtraction {
    fn extract_circuit(&self) -> Result<Circuit, String>;
}

pub trait CircuitLike: FragmentLike + CircuitExtraction {
    fn get_qubit_num(&self, fragment: GateFragmentRef) -> usize;
    fn apply_gate(&mut self, gate: Ref<dyn Gate>, target_qubits: Vec<usize>) -> SuceesOrMessage;
    fn apply_fragment(&mut self, fragment: GateFragmentRef) -> SuceesOrMessage;
}
