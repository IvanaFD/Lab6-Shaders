use std::fs;

use raylib::prelude::*;

use crate::vertex::Vertex;

pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<usize>,
}

impl Obj {
    // Vértices en el orden de las caras, de tres en tres
    pub fn get_vertex_array(&self, center: Vector3) -> Vec<Vertex> {
        self.indices
            .iter()
            .map(|&index| {
                let position = self.vertices[index];
                Vertex::new(position, (position - center).normalized())
            })
            .collect()
    }

    // Centro y tamaño de la caja que contiene al modelo
    pub fn bounds(&self) -> (Vector3, Vector3) {
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in &self.vertices {
            min = Vector3::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z));
            max = Vector3::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z));
        }
        ((min + max) / 2.0, max - min)
    }

    // Distancia del centro al vértice más lejano
    pub fn radius(&self, center: Vector3) -> f32 {
        self.vertices
            .iter()
            .map(|v| (*v - center).length())
            .fold(0.0, f32::max)
    }
}

pub fn load_obj(path: &str) -> Result<Obj, String> {
    // 1. Abrir y leer el archivo
    let content = fs::read_to_string(path).map_err(|e| format!("No se pudo leer {path}: {e}"))?;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // 2. Recorrer sus líneas
    for line in content.lines() {
        let mut parts = line.split_whitespace();

        match parts.next() {
            // Vértice: v x y z
            Some("v") => {
                let values: Vec<f32> = parts
                    .take(3)
                    .map(|value| value.parse::<f32>().map_err(|e| format!("Vértice inválido '{line}': {e}")))
                    .collect::<Result<_, _>>()?;

                if values.len() != 3 {
                    return Err(format!("Vértice incompleto: '{line}'"));
                }
                vertices.push(Vector3::new(values[0], values[1], values[2]));
            }
            // Cara: f 1 2 3  o  f 1/1/1 2/2/2 3/3/3
            Some("f") => {
                let face: Vec<usize> = parts
                    .map(|group| {
                        let first = group.split('/').next().unwrap_or("");
                        first
                            .parse::<usize>()
                            .map(|index| index - 1) // OBJ empieza en 1, Vec en 0
                            .map_err(|e| format!("Índice inválido '{group}': {e}"))
                    })
                    .collect::<Result<_, _>>()?;

                if face.len() != 3 {
                    return Err(format!("La cara no es un triángulo: '{line}'"));
                }
                indices.extend(face);
            }
            // Cualquier otra línea se ignora
            _ => {}
        }
    }

    Ok(Obj { vertices, indices })
}
