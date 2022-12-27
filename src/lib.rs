pub mod gates;
pub mod primitive;
pub mod pyzx;
pub mod qasm;
pub mod util;

use gates::*;
use primitive::*;
use pyo3::prelude::*;
use pyzx::json::*;
use pyzx::to_json::*;
use qasm::to_qasm::*;
use util::*;

use crate::gates::dist_select_simple;

fn dist_select_simple_internal(n: i32, dist: i32) -> Vec<QubitCell> {
    assert!(n > 0);
    assert!(dist > 0);
    assert!(dist <= n);
    let mut qubits = Vec::new();
    let first_qubit = cellize(Qubit::new("first"));
    qubits.push(first_qubit.clone());
    let first_control = Qubit::control(first_qubit.clone());
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
        qubits.extend(datas.clone());
        qubits.extend(ancillas.clone());
        qubits.extend(targets.clone());
        qubits.push(first_qubit.clone());

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
        qubits.extend(datas.clone());
        qubits.extend(ancillas.clone());
        qubits.extend(targets.clone());
        qubits.extend(redundant_targets.clone());
        qubits.push(first_qubit.clone());

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

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: i32, b: i32) -> PyResult<String> {
    Ok((a + b + 1).to_string())
}

#[pyfunction]
fn uniform_layered(n: i32, count: i32) -> PyResult<Vec<String>> {
    let qubits_vec = uniform_layered_internal(n, count);

    let jsons = qubits_vec
        .iter()
        .map(|qubits| {
            let pyzx_json = to_pyzx_circuit(qubits.clone());
            let json = serde_json::to_string(&pyzx_json).unwrap();
            json
        })
        .collect::<Vec<_>>();

    Ok(jsons)
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

#[pyfunction]
#[pyo3(text_signature = "(json, /)")]
fn count_t_depth(json: String) -> PyResult<i32> {
    let pyzx: PyzxCircuitJson = serde_json::from_str(&json).unwrap();
    let plane = pyzx.produce_plane();
    let depth = PyzxCircuitJson::count_depth(&plane);
    Ok(depth)
}

#[pyfunction]
fn layered(n: i32) -> PyResult<String> {
    println!();
    let first_qubit = cellize(Qubit::new("first"));
    let first_control = Qubit::control(first_qubit.clone());
    let datas = (0..n)
        .map(|i| cellize(Qubit::new(format!("data_{}", i).as_str())))
        .collect::<Vec<_>>();
    let ancillas = (0..n)
        .map(|i| cellize(Qubit::new(format!("ancilla_{}", i).as_str())))
        .collect::<Vec<_>>();
    let controls = in_over_2n(n, &first_control, datas.clone(), ancillas.clone());
    let target_sample_1 = cellize(Qubit::new("target_s1"));
    let export_target_sample_1 = Qubit::export(target_sample_1.clone());
    export_target_sample_1.control_by(&controls[0]);
    let mut qubits = Vec::new();
    qubits.extend(datas.clone());
    qubits.extend(ancillas.clone());
    qubits.push(first_qubit.clone());
    qubits.push(target_sample_1.clone());

    let pyzx_json = to_pyzx_circuit(qubits);
    let json = serde_json::to_string(&pyzx_json).unwrap();
    Ok(json)
}

#[pyfunction]
fn test_gate() -> PyResult<String> {
    println!("");
    let q1 = cellize(Qubit::new("q1"));
    let q2 = cellize(Qubit::new("q2"));
    let q3 = cellize(Qubit::new("q3"));
    // let q4 = cellize(Qubit::new("q4"));
    // let control = Qubit::control(q1.clone());
    // let (leftc, _) = in_layer(&control, q2.clone(), q3.clone());
    // let leftc = in_layer(&control, q2.clone(), q3.clone());
    // let export = Qubit::export(q4.clone());
    // export.control_by(&leftc);
    toffoli(q1.clone(), q2.clone(), q3.clone());
    // let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone()]);
    // let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone(), q4.clone()]);
    let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone()]);
    let json = serde_json::to_string(&pyzx_json).unwrap();
    Ok(json)
}

#[pyfunction]
fn test_gate_qasm() -> PyResult<String> {
    println!("");
    let q1 = cellize(Qubit::new("q1"));
    let q2 = cellize(Qubit::new("q2"));
    let q3 = cellize(Qubit::new("q3"));
    // let q4 = cellize(Qubit::new("q4"));
    // let control = Qubit::control(q1.clone());
    // let (leftc, _) = in_layer(&control, q2.clone(), q3.clone());
    // let leftc = in_layer(&control, q2.clone(), q3.clone());
    // let export = Qubit::export(q4.clone());
    // export.control_by(&leftc);
    toffoli(q1.clone(), q2.clone(), q3.clone());
    // let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone()]);
    // let pyzx_json = to_pyzx_circuit(vec![q1.clone(), q2.clone(), q3.clone(), q4.clone()]);
    let qasm_file = to_qasm(vec![q1.clone(), q2.clone(), q3.clone()]);
    Ok(qasm_file.to_string())
}

#[pyfunction]
fn output_json() -> PyResult<String> {
    let mut wire_vertices = WireVertices::new();
    wire_vertices.insert(
        "b0".to_string(),
        WireVerticesValue {
            annotation: WireVerticesAnnotation {
                boundary: true,
                coord: vec![0.0, 0.0],
                input: true,
                output: false,
            },
        },
    );
    wire_vertices.insert(
        "b1".to_string(),
        WireVerticesValue {
            annotation: WireVerticesAnnotation {
                boundary: true,
                coord: vec![2.0, 0.0],
                input: false,
                output: true,
            },
        },
    );
    let mut node_vertices = NodeVertices::new();
    node_vertices.insert(
        "v0".to_string(),
        NodeVerticesValue {
            annotation: NodeVerticesAnnotation {
                coord: vec![1.0, 0.0],
            },
            data: NodeVerticesData {
                kind: "Z".to_string(),
                value: Some("\\pi".to_string()),
                is_edge: None,
            },
        },
    );
    let mut undir_edges = UndirEdges::new();
    undir_edges.insert(
        "e0".to_string(),
        UndirEdgesValue {
            src: "b0".to_string(),
            tgt: "v0".to_string(),
        },
    );
    undir_edges.insert(
        "e1".to_string(),
        UndirEdgesValue {
            src: "v0".to_string(),
            tgt: "b1".to_string(),
        },
    );
    let test_struct = PyzxCircuitJson {
        wire_vertices,
        node_vertices,
        undir_edges,
    };
    let serialized = serde_json::to_string(&test_struct).unwrap();
    Ok(serialized)
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn prepare_circuit(_py: Python, m: &PyModule) -> PyResult<()> {
    println!("prepare-circuit version 1.0.8");
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(output_json, m)?)?;
    m.add_function(wrap_pyfunction!(test_gate, m)?)?;
    m.add_function(wrap_pyfunction!(test_gate_qasm, m)?)?;
    m.add_function(wrap_pyfunction!(layered, m)?)?;
    m.add_function(wrap_pyfunction!(count_t_depth, m)?)?;
    m.add_function(wrap_pyfunction!(uniform_layered, m)?)?;
    m.add_function(wrap_pyfunction!(uniform_layered_qasm, m)?)?;
    m.add_function(wrap_pyfunction!(uniform_layered_redundant, m)?)?;
    m.add_function(wrap_pyfunction!(dist_select_simple_export, m)?)?;
    Ok(())
}
