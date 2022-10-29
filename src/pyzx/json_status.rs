use regex::Regex;
use std::collections::HashMap;

use super::json::*;

#[derive(Debug, Clone)]
pub enum PlaneElement {
    T,
    Ordinal,
    Cross(i32),
}

impl PyzxCircuitJson {
    pub fn find_vertical_node(&self, node: String) -> Option<String> {
        let graph = self;
        let target = graph
            .undir_edges
            .iter()
            .filter_map(|(_, pair)| {
                let from = pair.src.clone();
                let to = pair.tgt.clone();
                let contains = from == node || to == node;
                if !contains {
                    return None;
                }
                let from_node = graph.node_vertices.get(&from);
                let to_node = graph.node_vertices.get(&to);
                let from_x = match from_node {
                    Some(from_node) => *from_node.annotation.coord.get(0).unwrap(),
                    None => -1.0,
                };
                let to_x = match to_node {
                    Some(to_node) => *to_node.annotation.coord.get(0).unwrap(),
                    None => -1.0,
                };
                if from_x != to_x {
                    return None;
                }
                Some(if from == node { to } else { from })
            })
            .collect::<Vec<_>>()
            .get(0).cloned();
        target
    }
    pub fn get_node_coord(&self, node: String) -> (i32, i32) {
        let graph = self;
        let node = graph.node_vertices.get(&node).unwrap();
        // x2しないと整数にならない
        let coord_x = (node.annotation.coord.get(0).unwrap() * 2.0) as i32;
        let coord_y = *node.annotation.coord.get(1).unwrap() as i32;
        (coord_x, coord_y)
    }
    pub fn produce_plane(&self) -> HashMap<i32, HashMap<i32, PlaneElement>> {
        let graph = self;
        let nodes = graph.node_vertices.clone();
        // 横方向はx2して整数的にしてから入れる
        let mut map: HashMap<i32, HashMap<i32, PlaneElement>> = HashMap::new();
        for (node, node_data) in nodes {
            let coord = self.get_node_coord(node.clone());
            // 行に当たるもの
            let new_row = HashMap::new();
            let row = match map.get_mut(&coord.1) {
                None => {
                    map.insert(coord.1, new_row);
                    map.get_mut(&coord.1).unwrap()
                }
                Some(row) => row,
            };
            // PlaneElementを錬成する
            let cross_target_node = self.find_vertical_node(node.clone());
            let cross_target_y = cross_target_node.map(|s| self.get_node_coord(s).1);
            let node_value = node_data.data.value.clone();
            let is_t_value = match node_value {
                Some(str) => {
                    let re = Regex::new("pi/4").unwrap();
                    let is_match = re.is_match(str.as_str());
                    is_match
                }
                None => false,
            };
            let plane_elem = match (is_t_value, cross_target_y) {
                (true, None) => PlaneElement::T,
                (false, Some(y)) => PlaneElement::Cross(y),
                (false, None) => PlaneElement::Ordinal,
                (true, Some(_)) => panic!("T value cannot have vertical cross"),
            };
            row.insert(coord.0, plane_elem);
        }
        map
    }
    pub fn count_depth(plane: &HashMap<i32, HashMap<i32, PlaneElement>>) -> i32 {
        let mut count_map: HashMap<i32, HashMap<i32, i32>> = HashMap::new();
        let row_count = plane.len();
        let column_count = plane.iter().map(|(_, row)| row.keys().max().unwrap()).max().unwrap() * 2;
        for x in (0..column_count).map(|x| x as i32) {
            for y in (0..row_count).map(|y| -(y as i32)) {
                // left countを計算する
                let row = match count_map.get(&y) {
                    None => {
                        let new_row = HashMap::new();
                        count_map.insert(y, new_row);
                        count_map.get(&y).unwrap()
                    }
                    Some(row) => row,
                };
                // 今までのleftは必ず埋まっている想定
                // zero start
                let left_count = *row.get(&(x - 1)).unwrap_or(&0);
                // vertical countを計算する
                let plane_row = plane.get(&y);
                let plane_current = plane_row.map(|row| row.get(&x)).unwrap_or(None);
                // verticalがない場合にnoneになる
                let plane_vertical_row = plane_current
                    .map(|p| match p {
                        PlaneElement::Cross(y) => Some(y),
                        _ => None,
                    })
                    .unwrap_or(None);
                // count_mapのrow
                // verticalがある場合にsomeになる
                let vertical_target_count_row = match plane_vertical_row {
                    None => None,
                    Some(y) => match count_map.get(y) {
                        None => {
                            let new_row = HashMap::new();
                            count_map.insert(*y, new_row);
                            Some(count_map.get(y).unwrap())
                        }
                        Some(row) => Some(row),
                    },
                };
                // countがある場合にはsomeになる
                let vertical_target_count = match vertical_target_count_row {
                    None => None,
                    Some(row) => row.get(&(x - 1)),
                };
                // 今の場所がTかどうかを調べる
                let is_current_t = matches!(plane_current, Some(PlaneElement::T));
                // 新しいcountを計算する
                let new_count = match vertical_target_count {
                    None => left_count,
                    Some(count) => left_count.max(*count),
                } + if is_current_t { 1 } else { 0 };
                // count_mapに書き込む
                // 上で作ってるのでunwrapできる
                let row = count_map.get_mut(&y).unwrap();
                row.insert(x, new_count);
                if let Some(y) = plane_vertical_row {
                    // これも上で作ってるのでunwrapできる
                    let vertical_row: &mut HashMap<i32, i32> = count_map.get_mut(y).unwrap();
                    vertical_row.insert(x, new_count);
                }
                // debug zone
                // println!("x,y,is_current_t {} {} {:?}", x, y, new_count);
                // println!("{:?}", count_map);
            }
        }
        
        (0..row_count)
            .map(|y| -(y as i32))
            .map(|y| {
                let y = y as i32;
                let row = count_map.get(&y).unwrap();
                let last_value = row.get(&((column_count as i32) - 1)).unwrap();
                *last_value
            })
            .max()
            .unwrap()
    }
}

// pub fn find_next_node(graph: PyzxCircuitJson, node: String) -> Option<String> {}

// pub fn t_depth(graph: PyzxCircuitJson) {}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_json(path: &str) -> String {
        std::fs::read_to_string(path).unwrap()
    }
    #[test]
    fn real_world_test() {
        let json = &sample_json("./test/depth-20.json");
        let pyzx = serde_json::from_str::<PyzxCircuitJson>(json).unwrap();
        let plane = pyzx.produce_plane();
        let depth = PyzxCircuitJson::count_depth(&plane);
        println!("{:#?}", depth);
        assert_eq!(depth, 20)
    }
}
