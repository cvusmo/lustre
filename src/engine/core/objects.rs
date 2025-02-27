// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/core/objects.rs

use crate::engine::render::MainVertex;
use gltf::import;
use rand::prelude::*;
use std::fs::File;
use std::io::Read;

pub fn load_mesh(path: &str) -> Result<(Vec<MainVertex>, Vec<u32>), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let (document, buffers, _) = import(path)?;

    let mesh = document.meshes().next().ok_or("No mesh found")?;
    let primitive = mesh.primitives().next().ok_or("No primitive found")?;

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or("No positions found")?
        .collect();

    let normals: Vec<[f32; 3]> = if let Some(iter) = reader.read_normals() {
        iter.collect()
    } else {
        vec![[0.0, 0.0, 1.0]; positions.len()]
    };

    let colors: Vec<[f32; 3]> = if let Some(color_iter) = reader.read_colors(0) {
        color_iter.into_rgb_f32().collect()
    } else {
        let mut rng = rand::rng();
        (0..positions.len())
            .map(|_| {
                [
                    rng.random_range(0.0..1.0),
                    rng.random_range(0.0..1.0),
                    rng.random_range(0.0..1.0),
                ]
            })
            .collect()
    };

    let indices: Vec<u32> = if let Some(index_iter) = reader.read_indices() {
        index_iter.into_u32().collect()
    } else {
        (0..positions.len() as u32).collect() // Fallback: assumes triangles or points
    };

    let vertices: Vec<MainVertex> = positions
        .into_iter()
        .zip(normals.into_iter())
        .zip(colors.into_iter())
        .map(|((position, normal), color)| MainVertex {
            position,
            normal,
            color,
        })
        .collect();

    Ok((vertices, indices))
}

