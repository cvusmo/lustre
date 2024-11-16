// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre

// src/configuration/animation.rs
// github.com/cvusmo/gameengine

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnimationConfig {
    pub blur: bool,  
    pub fade_in: bool, 
}

impl AnimationConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        AnimationConfig {
            blur: true,  
            fade_in: true, 
        }
    }

    pub fn animation_default_settings(&self) -> (bool, bool) {
        (self.blur, self.fade_in)
    }
}
