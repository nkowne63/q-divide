pub mod gates;
pub mod primitive;
pub mod pyfunctions;
pub mod pyzx;
pub mod qasm;
pub mod util;

use crate::gates::*;
use crate::primitive::*;
use crate::pyzx::json::*;
use crate::pyzx::to_json::*;
use crate::qasm::to_qasm::*;
use crate::util::*;

use pyo3::prelude::*;

use pyfunctions::{
    json_based::{count_t_depth, layered, uniform_layered},
    tests::{output_json, sum_as_string, test_gate, test_gate_qasm},
};

fn dist_select_simple_internal(n: i32, dist: i32) -> Vec<QubitCell> {
    assert!(n > 0);
    assert!(dist > 0);
    assert!(dist <= n);
    let mut qubits = Vec::new();
    let first_qubit = cellize(Qubit::new("first"));
    qubits.push(first_qubit.clone());
    let first_control = Qubit::control(first_qubit);
    let datas = (0..n)
        .map(|i| cellize(Qubit::new(format!("data_{}", i).as_str())))
        .collect::<Vec<_>>();
    let (datas, ancillas, controls) = dist_select_simple(
        dist,
        n - dist,
        first_control,
        datas,
        "dist-select".to_string(),
    );
    qubits.extend(datas);
    qubits.extend(ancillas);
    let targets = (0..n)
        .map(|i| cellize(Qubit::new(format!("target_{}", i).as_str())))
        .collect::<Vec<_>>();
    // この時点でcontrols, targetsが存在している
    let random_data = generate_random_datas(controls.len(), 1);
    inject_qrom_datas(targets.clone(), controls, random_data);
    qubits.extend(targets);

    qubits
}

fn uniform_layered_internal(n: i32, count: i32) -> Vec<Vec<QubitCell>> {
    let mut qubits_vec = Vec::new();
    (0..count).for_each(|_| {
        let first_qubit = cellize(Qubit::new("first"));
        let first_control = Qubit::control(first_qubit.clone());
        let datas = (0..n)
            .map(|i| cellize(Qubit::new(format!("data_{}", i).as_str())))
            .collect::<Vec<_>>();
        let ancillas = (0..n)
            .map(|i| cellize(Qubit::new(format!("ancilla_{}", i).as_str())))
            .collect::<Vec<_>>();
        let controls = in_over_2n(n, &first_control, datas.clone(), ancillas.clone());
        let data_length = controls.len();
        let targets = (0..n)
            .map(|i| cellize(Qubit::new(format!("target_{}", i).as_str())))
            .collect::<Vec<_>>();
        // let target_length = targets.len();
        let random_data = generate_random_datas(data_length, 1);
        inject_qrom_datas(targets.clone(), controls, random_data);

        let mut qubits = Vec::new();
        qubits.extend(datas);
        qubits.extend(ancillas);
        qubits.extend(targets);
        qubits.push(first_qubit);

        qubits_vec.push(qubits);
    });
    qubits_vec
}

// rだけ余分なものがある
fn uniform_layered_internal_redundant(n: i32, count: i32, r: i32) -> Vec<Vec<QubitCell>> {
    let mut qubits_vec = Vec::new();
    (0..count).for_each(|_| {
        let first_qubit = cellize(Qubit::new("first"));
        let first_control = Qubit::control(first_qubit.clone());
        let datas = (0..n)
            .map(|i| cellize(Qubit::new(format!("data_{}", i).as_str())))
            .collect::<Vec<_>>();
        let ancillas = (0..n)
            .map(|i| cellize(Qubit::new(format!("ancilla_{}", i).as_str())))
            .collect::<Vec<_>>();
        let controls = in_over_2n(n, &first_control, datas.clone(), ancillas.clone());
        let data_length = controls.len();
        let targets = (0..(n))
            .map(|i| cellize(Qubit::new(format!("target_{}", i).as_str())))
            .collect::<Vec<_>>();
        println!("redundant_targets count: {}", r);
        let redundant_targets = (n..(n + r))
            .map(|i| cellize(Qubit::new(format!("redundant_target_{}", i).as_str())))
            .map(|qc| {
                Qubit::gate(qc.clone(), PrimitiveGate::X);
                Qubit::gate(qc.clone(), PrimitiveGate::X);
                qc
            })
            .collect::<Vec<_>>();
        // let target_length = targets.len();
        let random_data = generate_random_datas(data_length, 1);
        inject_qrom_datas(targets.clone(), controls, random_data);

        let mut qubits = Vec::new();
        qubits.extend(datas);
        qubits.extend(ancillas);
        qubits.extend(targets);
        qubits.extend(redundant_targets);
        qubits.push(first_qubit);

        qubits_vec.push(qubits);
    });
    qubits_vec
}

#[pyfunction]
fn dist_select_simple_export(n: i32, dist: i32) -> PyResult<String> {
    println!("n: {}, dist: {}", n, dist);
    let qubits = dist_select_simple_internal(n, dist);
    println!("qubits.len(): {}", qubits.len());
    let qasm_file = to_qasm(qubits);
    Ok(qasm_file.to_string())
}

/// generates uniform layered qrom in qasm format
#[pyfunction]
#[pyo3(text_signature = "(n, count, /)")]
fn uniform_layered_qasm(n: i32, count: i32) -> PyResult<Vec<String>> {
    let qubits_vec = uniform_layered_internal(n, count);

    let qasms = qubits_vec
        .iter()
        .map(|qubits| {
            let qasm_file = to_qasm(qubits.clone());
            qasm_file.to_string()
        })
        .collect::<Vec<_>>();

    Ok(qasms)
}

#[pyfunction]
#[pyo3(text_signature = "(n, count, r, /)")]
fn uniform_layered_redundant(n: i32, count: i32, r: i32) -> PyResult<Vec<String>> {
    let qubits_vec = uniform_layered_internal_redundant(n, count, r);

    let qasms = qubits_vec
        .iter()
        .map(|qubits| {
            let qasm_file = to_qasm(qubits.clone());
            qasm_file.to_string()
        })
        .collect::<Vec<_>>();

    Ok(qasms)
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn prepare_circuit(_py: Python, m: &PyModule) -> PyResult<()> {
    println!("prepare-circuit version 1.0.8");
    // tests
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(output_json, m)?)?;
    m.add_function(wrap_pyfunction!(test_gate, m)?)?;
    m.add_function(wrap_pyfunction!(test_gate_qasm, m)?)?;
    // json_based
    m.add_function(wrap_pyfunction!(layered, m)?)?;
    m.add_function(wrap_pyfunction!(count_t_depth, m)?)?;
    m.add_function(wrap_pyfunction!(uniform_layered, m)?)?;
    // qasm_layerd
    m.add_function(wrap_pyfunction!(uniform_layered_qasm, m)?)?;
    m.add_function(wrap_pyfunction!(uniform_layered_redundant, m)?)?;
    // dist_select_simple
    m.add_function(wrap_pyfunction!(dist_select_simple_export, m)?)?;
    // m_body
    Ok(())
}
