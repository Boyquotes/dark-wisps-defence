use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Health(pub i32);
impl Health {
    pub fn decrease(&mut self, amount: i32) {
        self.0 = std::cmp::max(self.0 - amount, 0);
    }
    pub fn is_dead(&self) -> bool {
        self.0 <= 0
    }
}