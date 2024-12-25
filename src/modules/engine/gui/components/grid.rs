// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/modules/engine/gui/components/grid.rs

use gtk::{prelude::*, Grid};
use gtk4 as gtk;

pub fn create_grid() -> Grid {
    let grid = Grid::builder().row_spacing(10).column_spacing(10).build();

    // Set grid to expand
    grid.set_vexpand(true);
    grid.set_hexpand(true);

    // Grid alignment
    grid.set_halign(gtk::Align::Fill);
    grid.set_valign(gtk::Align::Fill);

    grid
}
