// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/core/voxel.rs

use crate::engine::render::MainVertex;
use rand::prelude::*;

pub fn generate_voxel_mesh(voxels: &Vec<Vec<Vec<bool>>>) -> (Vec<MainVertex>, Vec<u32>) {
    let mut vertices: Vec<MainVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut rng = rand::rng();

    let width = voxels.len();
    if width == 0 {
        return (vertices, indices);
    }
    let height = voxels[0].len();
    if height == 0 {
        return (vertices, indices);
    }
    let depth = voxels[0][0].len();
    if depth == 0 {
        return (vertices, indices);
    }

    fn add_face(
        vertices: &mut Vec<MainVertex>,
        indices: &mut Vec<u32>,
        positions: [[f32; 3]; 4],
        normal: [f32; 3],
        color: [f32; 3],
    ) {
        let start = vertices.len() as u32;
        for &pos in positions.iter() {
            vertices.push(MainVertex {
                position: pos,
                normal,
                color,
            });
        }
        // Two triangles: 0, 1, 2 and 0, 2, 3
        indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
    }

    // Loop over the voxel grid
    for x in 0..width {
        for y in 0..height {
            for z in 0..depth {
                if voxels[x][y][z] {
                    let x_f = x as f32;
                    let y_f = y as f32;
                    let z_f = z as f32;

                    let p000 = [x_f, y_f, z_f];
                    let p100 = [x_f + 1.0, y_f, z_f];
                    let p110 = [x_f + 1.0, y_f + 1.0, z_f];
                    let p010 = [x_f, y_f + 1.0, z_f];
                    let p001 = [x_f, y_f, z_f + 1.0];
                    let p101 = [x_f + 1.0, y_f, z_f + 1.0];
                    let p111 = [x_f + 1.0, y_f + 1.0, z_f + 1.0];
                    let p011 = [x_f, y_f + 1.0, z_f + 1.0];

                    // Random color per voxel
                    let color = [
                        rng.random_range(0.0..1.0),
                        rng.random_range(0.0..1.0),
                        rng.random_range(0.0..1.0),
                    ];

                    // Front face: z+ direction
                    if z == depth - 1 || !voxels[x][y][z + 1] {
                        add_face(
                            &mut vertices,
                            &mut indices,
                            [p001, p101, p111, p011],
                            [0.0, 0.0, 1.0],
                            color,
                        );
                    }
                    // Back face: z- direction
                    if z == 0 || !voxels[x][y][z - 1] {
                        add_face(
                            &mut vertices,
                            &mut indices,
                            [p100, p000, p010, p110],
                            [0.0, 0.0, -1.0],
                            color,
                        );
                    }
                    // Right face: x+ direction
                    if x == width - 1 || !voxels[x + 1][y][z] {
                        add_face(
                            &mut vertices,
                            &mut indices,
                            [p101, p100, p110, p111],
                            [1.0, 0.0, 0.0],
                            color,
                        );
                    }
                    // Left face: x- direction
                    if x == 0 || !voxels[x - 1][y][z] {
                        add_face(
                            &mut vertices,
                            &mut indices,
                            [p000, p001, p011, p010],
                            [-1.0, 0.0, 0.0],
                            color,
                        );
                    }
                    // Top face: y+ direction
                    if y == height - 1 || !voxels[x][y + 1][z] {
                        add_face(
                            &mut vertices,
                            &mut indices,
                            [p010, p011, p111, p110],
                            [0.0, 1.0, 0.0],
                            color,
                        );
                    }
                    // Bottom face: y- direction
                    if y == 0 || !voxels[x][y - 1][z] {
                        add_face(
                            &mut vertices,
                            &mut indices,
                            [p001, p000, p100, p101],
                            [0.0, -1.0, 0.0],
                            color,
                        );
                    }
                }
            }
        }
    }

    (vertices, indices)
}

pub fn get_voxel_mesh(size: usize) -> (Vec<MainVertex>, Vec<u32>) {
    let mut voxel_grid = vec![vec![vec![false; size]; size]; size];
    let center = size / 2;
    voxel_grid[center][center][center] = true;
    generate_voxel_mesh(&voxel_grid)
}

pub fn get_64_voxel_mesh() -> (Vec<MainVertex>, Vec<u32>) {
    let grid_width = 64;
    let grid_height = 1;
    let grid_depth = 64;
    let voxel_grid = vec![vec![vec![false; grid_depth]; grid_height]; grid_width];
    generate_voxel_mesh(&voxel_grid)
}
