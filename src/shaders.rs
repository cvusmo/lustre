// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/shaders.rs

pub mod compute_mandelbrot {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/mandelbrot.comp",
    }
}

pub mod compute_border {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/border.comp",
    }
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/vertex.comp",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/fragment.comp",
    }
}

pub mod image {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/image.comp",
    }
}
