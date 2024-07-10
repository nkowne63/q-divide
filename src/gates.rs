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

// cyclomatic complexity: 1
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
