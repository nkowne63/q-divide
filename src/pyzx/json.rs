use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Coord = Vec<f64>;

#[derive(Serialize, Deserialize, Debug)]
pub struct WireVerticesAnnotation {
    pub boundary: bool,
    pub coord: Coord,
    pub input: bool,
    pub output: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WireVerticesValue {
    pub annotation: WireVerticesAnnotation,
}
pub type WireVertices = HashMap<String, WireVerticesValue>;

impl WireVerticesValue {
    pub fn create(is_input: bool, coord: Coord) -> WireVerticesValue {
        WireVerticesValue {
            annotation: WireVerticesAnnotation {
                boundary: true,
                coord: coord,
                input: is_input,
                output: !is_input,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeVerticesData {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub kind: String,
    pub value: Option<String>,
    pub is_edge: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeVerticesAnnotation {
    pub coord: Coord,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeVerticesValue {
    pub annotation: NodeVerticesAnnotation,
    pub data: NodeVerticesData,
}
pub type NodeVertices = HashMap<String, NodeVerticesValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct UndirEdgesValue {
    pub src: String,
    pub tgt: String,
}
pub type UndirEdges = HashMap<String, UndirEdgesValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct PyzxCircuitJson {
    pub wire_vertices: WireVertices,
    pub node_vertices: NodeVertices,
    pub undir_edges: UndirEdges,
}
