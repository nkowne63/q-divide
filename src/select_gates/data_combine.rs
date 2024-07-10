use crate::primitive::*;
use rand::Rng;

// cyclomatic complexity: 1 + 3(loop) = 4
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

// cyclomatic complexity: 1 + 2(loop) = 3
pub fn generate_datas(count: usize, length: usize) -> Vec<Vec<bool>> {
    // use rand::Rng;
    // let mut rng = rand::thread_rng();
    let mut ret = Vec::new();
    for _ in 0..count {
        let mut inner_vec = Vec::new();
        for _ in 0..length {
            // let rng_value = rng.gen::<bool>();
            // inner_vec.push(rng_value);
            inner_vec = vec![true; length]
        }
        ret.push(inner_vec)
    }
    ret
}

pub fn combine_random_cnots_m_interaction(
    m: i32,
    control_froms: Vec<ControlFrom>,
    qubits: Vec<QubitCell>,
) -> Vec<QubitCell> {
    assert!(m > 0, "m must be positive");
    assert!(
        m <= control_froms.len() as i32,
        "m must be less than control_froms length"
    );
    // for each qubits iterate...
    qubits
        .iter()
        .map(|qubit| {
            // pick up m random control_from index from control_froms
            let mut picked_control_froms_idx = vec![];
            while picked_control_froms_idx.len() < m as usize {
                let idx = rand::thread_rng().gen_range(0..control_froms.len());
                if !picked_control_froms_idx.contains(&idx) {
                    picked_control_froms_idx.push(idx);
                }
            }
            // sort picked_control_froms_idx
            picked_control_froms_idx.sort();
            // pick up control_from from control_froms by index in picked_control_froms_idx
            let picked_control_froms = picked_control_froms_idx
                .iter()
                .map(|idx| control_froms[*idx].clone())
                .collect::<Vec<_>>();
            // apply control_from to qubit
            picked_control_froms.iter().for_each(|control_from| {
                let control_target = Qubit::export(qubit.clone());
                control_target.control_by(control_from);
            });
            qubit.clone()
        })
        .collect::<Vec<_>>()
}
