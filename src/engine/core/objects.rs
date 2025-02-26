// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/core/objects.rs

use crate::engine::render::MainVertex;

// use fbx object for vertices

// Hardcoded cubey boi
pub fn get_cube_vertices() -> Vec<MainVertex> {
    vec![
        // Front face (normal: [0, 0, 1])
        MainVertex {
            position: [-0.5, -0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        MainVertex {
            position: [0.5, -0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        MainVertex {
            position: [0.5, 0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        MainVertex {
            position: [-0.5, -0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        MainVertex {
            position: [0.5, 0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        MainVertex {
            position: [-0.5, 0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        // Back face (normal: [0, 0, -1])
        MainVertex {
            position: [0.5, -0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        MainVertex {
            position: [-0.5, -0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        MainVertex {
            position: [-0.5, 0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        MainVertex {
            position: [0.5, -0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        MainVertex {
            position: [-0.5, 0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        MainVertex {
            position: [0.5, 0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        // Left face (normal: [-1, 0, 0])
        MainVertex {
            position: [-0.5, -0.5, -0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [-0.5, -0.5, 0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [-0.5, 0.5, 0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [-0.5, -0.5, -0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [-0.5, 0.5, 0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [-0.5, 0.5, -0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        // Right face (normal: [1, 0, 0])
        MainVertex {
            position: [0.5, -0.5, 0.5],
            normal: [1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [0.5, -0.5, -0.5],
            normal: [1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [0.5, 0.5, -0.5],
            normal: [1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [0.5, -0.5, 0.5],
            normal: [1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [0.5, 0.5, -0.5],
            normal: [1.0, 0.0, 0.0],
        },
        MainVertex {
            position: [0.5, 0.5, 0.5],
            normal: [1.0, 0.0, 0.0],
        },
        // Top face (normal: [0, 1, 0])
        MainVertex {
            position: [-0.5, 0.5, 0.5],
            normal: [0.0, 1.0, 0.0],
        },
        MainVertex {
            position: [0.5, 0.5, 0.5],
            normal: [0.0, 1.0, 0.0],
        },
        MainVertex {
            position: [0.5, 0.5, -0.5],
            normal: [0.0, 1.0, 0.0],
        },
        MainVertex {
            position: [-0.5, 0.5, 0.5],
            normal: [0.0, 1.0, 0.0],
        },
        MainVertex {
            position: [0.5, 0.5, -0.5],
            normal: [0.0, 1.0, 0.0],
        },
        MainVertex {
            position: [-0.5, 0.5, -0.5],
            normal: [0.0, 1.0, 0.0],
        },
        // Bottom face (normal: [0, -1, 0])
        MainVertex {
            position: [-0.5, -0.5, -0.5],
            normal: [0.0, -1.0, 0.0],
        },
        MainVertex {
            position: [0.5, -0.5, -0.5],
            normal: [0.0, -1.0, 0.0],
        },
        MainVertex {
            position: [0.5, -0.5, 0.5],
            normal: [0.0, -1.0, 0.0],
        },
        MainVertex {
            position: [-0.5, -0.5, -0.5],
            normal: [0.0, -1.0, 0.0],
        },
        MainVertex {
            position: [0.5, -0.5, 0.5],
            normal: [0.0, -1.0, 0.0],
        },
        MainVertex {
            position: [-0.5, -0.5, 0.5],
            normal: [0.0, -1.0, 0.0],
        },
    ]
}
