use crate::{gates::*, primitive::*};
use crate::{pyzx::serialize_utils::QubitSerializeUtil, util::cellize};

use super::simple_select_controls::*;

//// dist-select

pub fn divide_qubits(count: i32, original: Vec<QubitCell>) -> (Vec<QubitCell>, Vec<QubitCell>) {
    (
        original[..count as usize].to_vec(),
        original[count as usize..].to_vec(),
    )
}

// [a,b] -> [[a,b],[a',b']] -> [[a,b],[a',b'], [a'', b''], [a''', b''']] -> ...
pub fn cnot_copy_n(n: i32, original: Vec<QubitCell>) -> Vec<Vec<QubitCell>> {
    if n < 0 {
        panic!("n must be greater than 0");
    }
    if n == 0 {
        return vec![original];
    }
    let target = original
        .iter()
        .enumerate()
        .map(|(idx, q)| {
            let target = cellize(Qubit::new(
                format!("{}-copy-{}-layer-{}", q.qubit_id(), idx, n).as_str(),
            ));
            cnot(q.clone(), target.clone());
            target
        })
        .collect::<Vec<QubitCell>>();
    let vec_first = cnot_copy_n(n - 1, original);
    let vec_second = cnot_copy_n(n - 1, target);
    [vec_first, vec_second].concat()
}

// return value: (original, ancilla)
pub fn cnot_uncopy_n(n: i32, doubled: Vec<QubitCell>) -> (Vec<QubitCell>, Vec<QubitCell>) {
    if n < 0 {
        panic!("n must be greater than or equals to 0");
    }
    if n == 0 {
        return (doubled, vec![]);
    }
    if doubled.len() % 2 != 0 {
        panic!("doubled.len() must be even, but it is {}", doubled.len());
    }
    let half: usize = doubled.len() / 2;
    let (higher, lower) = divide_qubits(half as i32, doubled);
    // この時点で(original, ancilla)になっている
    let uncopied_higher = cnot_uncopy_n(n - 1, higher);
    let uncopied_lower = cnot_uncopy_n(n - 1, lower);
    let original_len = uncopied_higher.0.len();
    (0..original_len).for_each(|idx| {
        cnot(
            uncopied_higher.0[idx].clone(),
            uncopied_lower.0[idx].clone(),
        );
    });
    // higherのoriginalだけをoriginalとして返す
    (
        uncopied_higher.0,
        [uncopied_lower.0, uncopied_higher.1, uncopied_lower.1].concat(),
    )
}

// return value: (original, ancilla, control)
pub fn eq_ladder(
    value: Vec<bool>,
    control_from: ControlFrom,
    original: Vec<QubitCell>,
    ancillas: Vec<QubitCell>,
) -> (Vec<QubitCell>, Vec<QubitCell>, ControlFrom) {
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

fn num2boolvec_fixed_length(num: usize, length: usize) -> Vec<bool> {
    let mut ret = num_to_vec_bool(num);
    while ret.len() < length {
        ret.insert(0, false);
    }
    ret
}

// return value: (original, ancilla, control)
pub fn dist_select_simple(
    high_count: i32,
    low_count: i32,
    control_from: ControlFrom,
    original: Vec<QubitCell>,
    name: String,
) -> (Vec<QubitCell>, Vec<QubitCell>, Vec<ControlFrom>) {
    // clone datas
    let all_copied_blocks = cnot_copy_n(high_count, original);
    let mut all_ancillas = vec![];
    let mut all_controls = vec![];
    for (block_idx, copied) in all_copied_blocks.iter().enumerate() {
        // prepare qubits
        let ancillas = (0..high_count + low_count)
            .map(|idx| {
                cellize(Qubit::new(
                    format!("{}-block-{}-ancilla-{}", name, block_idx, idx).as_str(),
                ))
            })
            .collect::<Vec<QubitCell>>();
        let (higher_ancillas, lower_ancillas) = divide_qubits(high_count as i32, ancillas.clone());
        let (higher_copied, lower_copied) = divide_qubits(high_count as i32, copied.to_vec());
        // initialization
        let (higher_copied, higher_ancillas, higher_carry) = eq_ladder(
            num2boolvec_fixed_length(block_idx, higher_copied.len()),
            control_from.clone(),
            higher_copied,
            higher_ancillas,
        );
        // ladder
        let controls = in_over_2n(low_count, &higher_carry, lower_copied, lower_ancillas);
        all_controls.extend(controls);
        // uncomputation
        eq_ladder(
            num2boolvec_fixed_length(block_idx, higher_copied.len()),
            control_from.clone(),
            higher_copied,
            higher_ancillas,
        );
        all_ancillas.extend(ancillas);
    }
    let all_copied_qubits = all_copied_blocks.concat();
    let (original, copied) = cnot_uncopy_n(high_count, all_copied_qubits);
    all_ancillas.extend(copied);
    (original, all_ancillas, all_controls)
}
