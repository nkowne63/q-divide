use crate::primitive::*;
use crate::select_gates::data_combine::*;
use crate::select_gates::simple_select_controls::*;
use crate::util::*;

pub fn uniform_layered_internal(n: i32, count: i32) -> Vec<Vec<QubitCell>> {
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
pub fn uniform_layered_internal_redundant(n: i32, count: i32, r: i32) -> Vec<Vec<QubitCell>> {
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
