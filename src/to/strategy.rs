//! Strategy parameter handling for add.to()

use std::collections::HashMap;
use crate::core::types::StrategyValue;

pub struct Strategy {
    params: HashMap<String, StrategyValue>,
}

impl Strategy {
    pub fn new(params: HashMap<String, StrategyValue>) -> Self {
        Self { params }
    }
}
