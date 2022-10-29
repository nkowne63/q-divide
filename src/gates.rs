use std::convert::TryInto;

use crate::{util::cellize, pyzx::serialize_utils::QubitSerializeUtil};
use super::primitive::*;

pub fn cnot(q1: QubitCell, q2: QubitCell) {
    let control_from = Qubit::control(q1);
    let target = Qubit::export(q2);
    target.control_by(&control_from);
}

pub fn toffoli(q1: QubitCell, q2: QubitCell, q3: QubitCell) {
    Qubit::gate(q3.clone(), PrimitiveGate::H);
    Qubit::gate(q3.clone(), PrimitiveGate::T);

    let control_from_2 = Qubit::control(q2);
    let export_3_1 = Qubit::export(q3.clone());
    export_3_1.control_by(&control_from_2);

    Qubit::gate(q3.clone(), PrimitiveGate::TDag);

    let control_from_1 = Qubit::control(q1);
    let export_3_2 = Qubit::export(q3.clone());
    export_3_2.control_by(&control_from_1);

    Qubit::gate(q3.clone(), PrimitiveGate::T);

    let export_3_3 = Qubit::export(q3.clone());
    export_3_3.control_by(&control_from_2);

    Qubit::gate(q3.clone(), PrimitiveGate::TDag);

    Qubit::gate(q3.clone(), PrimitiveGate::H);

    Qubit::gate(q3, PrimitiveGate::SDag);
}

pub fn toffoli_first_control(q1c: &ControlFrom, q2: QubitCell, q3: QubitCell) {
    Qubit::gate(q3.clone(), PrimitiveGate::T);

    let control_from_2 = Qubit::control(q2);
    let export_3_1 = Qubit::export(q3.clone());
    export_3_1.control_by(&control_from_2);

    Qubit::gate(q3.clone(), PrimitiveGate::TDag);

    let export_3_2 = Qubit::export(q3.clone());
    export_3_2.control_by(q1c);

    Qubit::gate(q3.clone(), PrimitiveGate::T);

    let export_3_3 = Qubit::export(q3.clone());
    export_3_3.control_by(&control_from_2);

    Qubit::gate(q3.clone(), PrimitiveGate::TDag);

    Qubit::gate(q3.clone(), PrimitiveGate::H);

    Qubit::gate(q3, PrimitiveGate::SDag);
}

pub fn in_layer(
    q1c: &ControlFrom,
    data: QubitCell,
    output: QubitCell,
) -> (ControlFrom, ControlFrom) {
    toffoli_first_control(q1c, data.clone(), output.clone());

    let control_left = Qubit::control(output.clone());

    let export_1 = Qubit::export(output.clone());
    export_1.control_by(q1c);

    let control_right = Qubit::control(output.clone());

    let export_2 = Qubit::export(output.clone());
    export_2.control_by(q1c);

    Qubit::gate(data.clone(), PrimitiveGate::X);
    toffoli_first_control(q1c, data.clone(), output);
    Qubit::gate(data, PrimitiveGate::X);

    (control_left, control_right)
}

pub fn in_over_2n(
    n: i32,
    control: &ControlFrom,
    datas: Vec<QubitCell>,
    ancillas: Vec<QubitCell>,
) -> Vec<ControlFrom> {
    if datas.len() != ancillas.len() {
        panic!("datas and ancillas must be the same length");
    }
    if datas.len() != n as usize {
        panic!("datas length and depth must be the same");
    }
    if n <= 0 {
        panic!("n must be greater than 0");
    }
    if n == 1 {
        let data = datas[0].clone();
        let ancilla = ancillas[0].clone();
        let controls_tuple = in_layer(control, data, ancilla);
        return vec![controls_tuple.0, controls_tuple.1];
    } else {
        let first_data = datas[0].clone();
        let first_ancilla = ancillas[0].clone();
        let remaining_datas = datas[1..].to_vec();
        let remaining_ancillas = ancillas[1..].to_vec();
        let controls_tuple = in_layer(control, first_data, first_ancilla);
        let mut controls = Vec::new();
        let left_controls = in_over_2n(
            n - 1,
            &controls_tuple.0,
            remaining_datas.clone(),
            remaining_ancillas.clone(),
        );
        let right_controls = in_over_2n(
            n - 1,
            &controls_tuple.1,
            remaining_datas,
            remaining_ancillas,
        );
        controls.extend(left_controls);
        controls.extend(right_controls);
        controls
    }
}

// data_listの1段目はctsのlengthと同じ
// 2段目はqcsのlengthと同じ
pub fn inject_qrom_datas(qcs: Vec<QubitCell>, cts: Vec<ControlFrom>, data_list: Vec<Vec<bool>>) {
    data_list.iter().enumerate().for_each(|(index, data)| {
        let control = &cts[index];
        data.iter().enumerate().for_each(|(target, value)| {
            if *value {
                let export = Qubit::export(qcs[target].clone());
                export.control_by(control);
            }
        });
    });
}

pub fn generate_random_datas(count: usize, length: usize) -> Vec<Vec<bool>> {
    // use rand::Rng;
    // let mut rng = rand::thread_rng();
    let mut ret = Vec::new();
    for _ in 0..count {
        let mut inner_vec = Vec::new();
        for _ in 0..length {
            // let rng_value = rng.gen::<bool>();
            // inner_vec.push(rng_value);
            inner_vec = vec![true;length]
        }
        ret.push(inner_vec)
    }
    ret
}

//// dist-select

pub fn divide_qubits(count: i32, original: Vec<QubitCell>) -> (Vec<QubitCell>, Vec<QubitCell>) {
    (original[..count as usize].to_vec(), original[count as usize..].to_vec())
}

// [a,b] -> [[a,b],[a',b']] -> [[a,b],[a',b'], [a'', b''], [a''', b''']] -> ...
pub fn cnot_copy_n(n: i32, original: Vec<QubitCell>) -> Vec<Vec<QubitCell>> {
    if n <= 0 {
        panic!("n must be greater than 0");
    }
    if n == 0 {
        return vec![original];
    }
    let target = original.iter().enumerate().map(|(idx, q)| {
        let target = cellize(Qubit::new(format!("{}-copy-{}-layer-{}", q.qubit_id(), idx, n).as_str()));
        cnot(q.clone(), target.clone());
        target
    }).collect::<Vec<QubitCell>>();
    let vec_first = cnot_copy_n(n-1, original);
    let vec_second = cnot_copy_n(n-1, target);
    [vec_first, vec_second].concat()
}

// return value: (original, ancilla)
pub fn cnot_uncopy_n(n: i32, doubled: Vec<QubitCell>) -> (Vec<QubitCell>, Vec<QubitCell>) {
    if n <= 0 {
        panic!("n must be greater than 0");
    }
    if n % 2 != 0 {
        panic!("n must be even");
    }
    if n == 0 {
        return (doubled, vec![]);
    }
    let half: usize = (n / 2).try_into().unwrap();
    let (higher, lower) = divide_qubits(half as i32, doubled);
    // この時点で(original, ancilla)になっている
    let uncopied_higher = cnot_uncopy_n(n-1, higher);
    let uncopied_lower = cnot_uncopy_n(n-1, lower);
    let original_len = uncopied_higher.0.len();
    (0..original_len).for_each(|idx| {
        cnot(uncopied_higher.0[idx].clone(), uncopied_lower.0[idx].clone());
    });
    // higherのoriginalだけをoriginalとして返す
    (uncopied_higher.0, [uncopied_lower.0, uncopied_higher.1, uncopied_lower.1].concat())
}

// return value: (original, ancilla, control)
pub fn eq_ladder(value: Vec<bool>, control_from: ControlFrom, original: Vec<QubitCell>, ancillas: Vec<QubitCell>) -> (Vec<QubitCell>,Vec<QubitCell>,ControlFrom) {
    if value.len() != original.len() {
        panic!("value and original must be the same length");
    }
    if value.len() != ancillas.len() {
        panic!("value and ancillas must be the same length");
    }
    let mut control = control_from;
    value.iter().enumerate().for_each(|(idx, bit)| {
        if *bit {
            Qubit::gate(original[idx].clone(), PrimitiveGate::X);
        }
        toffoli_first_control(&control, original[idx].clone(), ancillas[idx].clone());
        if *bit {
            Qubit::gate(original[idx].clone(), PrimitiveGate::X);
        }
        control = Qubit::control(ancillas[idx].clone());
    });
    // controlは最後のancillaのcontrolになっている
    (original, ancillas, control)
}

fn num_to_vec_bool(num: usize) -> Vec<bool> {
    let mut ret = Vec::new();
    let mut num = num;
    while num > 0 {
        ret.push(num % 2 == 1);
        num /= 2;
    }
    ret.reverse();
    ret
}

// return value: (original, ancilla, control)
pub fn dist_select_simple(high_count: i32, low_count: i32, control_from: ControlFrom, original: Vec<QubitCell>, name: String) -> (Vec<QubitCell>, Vec<QubitCell>, Vec<ControlFrom>) {
    let all_copied_blocks = cnot_copy_n(high_count, original.clone());
    let mut all_ancillas = vec![];
    let mut all_controls = vec![];
    for (block_idx, copied) in all_copied_blocks.iter().enumerate() {
        // prepare qubits
        let ancillas = (0..high_count+low_count).map(|idx| cellize(Qubit::new(format!("{}-block-{}-ancilla-{}", name, block_idx,idx).as_str()))).collect::<Vec<QubitCell>>();
        let (higher_ancillas, lower_ancillas) = divide_qubits(high_count as i32, ancillas.clone());
        let (higher_copied, lower_copied) = divide_qubits(high_count as i32, copied.to_vec());
        // initialization
        let (higher_copied, higher_ancillas, higher_carry) = eq_ladder(num_to_vec_bool(block_idx), control_from.clone(), higher_copied, higher_ancillas);
        // ladder
        let controls = in_over_2n(low_count, &higher_carry, lower_copied, lower_ancillas);
        all_controls.extend(controls);
        // uncomputation
        eq_ladder(num_to_vec_bool(block_idx), control_from.clone(), higher_copied, higher_ancillas);
        all_ancillas.extend(ancillas)
    }
    let (original, copied) = cnot_uncopy_n(high_count, original);
    all_ancillas.extend(copied);
    (original, all_ancillas, all_controls)
}

#[cfg(test)]
mod tests {
    use crate::{pyzx::to_json::to_pyzx_circuit, util::cellize};

    use super::*;
    #[test]
    fn test_rand() {
        let rand = generate_random_datas(2, 2);
        println!("{:?}", rand);
    }
    #[test]
    fn generate_random() {
        let n = 5;
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
        let target_length = targets.len();
        let random_data = generate_random_datas(data_length, target_length);
        inject_qrom_datas(targets.clone(), controls, random_data);

        let mut qubits = Vec::new();
        qubits.extend(datas);
        qubits.extend(ancillas);
        qubits.extend(targets);
        qubits.push(first_qubit);

        let pyzx_json = to_pyzx_circuit(qubits);

        println!("json {:?}", pyzx_json);
    }
}
