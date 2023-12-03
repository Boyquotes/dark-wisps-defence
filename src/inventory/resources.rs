use bevy::prelude::*;

pub const MAX_DARK_ORE_STOCK: i32 = 9999;

#[derive(Resource, Default)]
pub struct DarkOreStock {
    pub amount: i32
}
impl DarkOreStock {
    pub fn add(&mut self, amount: i32) {
        self.amount += amount;
        if self.amount > MAX_DARK_ORE_STOCK { self.amount = MAX_DARK_ORE_STOCK; }
    }
}