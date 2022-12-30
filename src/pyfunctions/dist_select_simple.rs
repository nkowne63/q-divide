use crate::primitive::*;
use crate::qasm::to_qasm::*;
use crate::select_gates::data_combine::*;
use crate::select_gates::simple_dist_select::*;
use crate::util::*;

use pyo3::prelude::*;

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

#[pyfunction]
pub fn dist_select_simple_export(n: i32, dist: i32) -> PyResult<String> {
    println!("n: {}, dist: {}", n, dist);
    let qubits = dist_select_simple_internal(n, dist);
    println!("qubits.len(): {}", qubits.len());
    let qasm_file = to_qasm(qubits);
    Ok(qasm_file.to_string())
}
