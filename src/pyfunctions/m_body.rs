use pyo3::prelude::*;

use crate::primitive::Qubit;
use crate::qasm::to_qasm::to_qasm;
use crate::select_gates::data_combine::combine_random_cnots_m_interaction;
use crate::select_gates::simple_select_controls::in_over_2n;
use crate::util::cellize;

// n: number of input qubits, m: interaction count
#[pyfunction]
pub fn uniform_layered_m_body(n: i32, m: i32) -> PyResult<String> {
    // prepare qubits
    let first_qubit = cellize(Qubit::new("first"));
    let inputs = (0..n)
        .map(|i| cellize(Qubit::new(&format!("input_{}", i))))
        .collect::<Vec<_>>();
    let ancillas = (0..m)
        .map(|i| cellize(Qubit::new(&format!("ancilla_{}", i))))
        .collect::<Vec<_>>();
    let targets = (0..m)
        .map(|i| cellize(Qubit::new(&format!("target_{}", i))))
        .collect::<Vec<_>>();
    // get controls
    let first_control_from = Qubit::control(first_qubit.clone());
    let controls = in_over_2n(n, &first_control_from, inputs.clone(), ancillas.clone());
    // inject data
    let applied_targets = combine_random_cnots_m_interaction(m, controls, targets);
    // collect qubits
    let mut qubits = Vec::new();
    qubits.push(first_qubit);
    qubits.extend(inputs);
    qubits.extend(ancillas);
    qubits.extend(applied_targets);
    // get qasm string
    let qasm_file = to_qasm(qubits);
    Ok(qasm_file.to_string())
}
