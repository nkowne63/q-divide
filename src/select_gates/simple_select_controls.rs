use crate::gates::*;
use crate::primitive::*;

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
        vec![controls_tuple.0, controls_tuple.1]
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
