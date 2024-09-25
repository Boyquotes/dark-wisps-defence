use serde::{Deserialize, Serialize};
use crate::prelude::*;

pub struct ResourcesPlugin;
impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stock>();
    }
}

pub const MAX_DARK_ORE_STOCK: i32 = 9999;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    DarkOre,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cost {
    resource_type: ResourceType,
    amount: i32,
}

struct StockInfo {
    amount: i32,
    max_amount: i32,
}

#[derive(Resource)]
pub struct Stock(HashMap<ResourceType, StockInfo>);
impl Stock {
    pub fn get(&self, resource_type: ResourceType) -> i32 {
        self.get_info(resource_type).amount
    }
    pub fn can_cover(&self, cost: &Vec<Cost>) -> bool {
        cost.iter().all(|c| self.has(c.resource_type, c.amount))
    }
    pub fn has(&self, resource_type: ResourceType, amount: i32) -> bool {
        self.get_info(resource_type).amount >= amount
    }
    pub fn add(&mut self, resource_type: ResourceType, amount: i32) {
        let info = self.get_info_mut(resource_type);
        info.amount = std::cmp::min(info.max_amount, info.amount + amount);
    }
    pub fn try_pay_costs(&mut self, cost: &Vec<Cost>) -> bool {
        if !self.can_cover(cost) { return false; }
        for cost in cost {
            self.try_remove(cost.resource_type, cost.amount);
        }
        true
    }
    pub fn try_remove(&mut self, resource_type: ResourceType, amount: i32) -> bool {
        let info = self.get_info_mut(resource_type);
        if info.amount < amount { return false; }
        info.amount = info.amount - amount;
        true
    }
    fn get_info(&self, resource_type: ResourceType) -> &StockInfo {
        self.0.get(&resource_type).expect(format!("Resource type {resource_type:?} not found in stock").as_str())
    }
    fn get_info_mut(&mut self, resource_type: ResourceType) -> &mut StockInfo {
        self.0.get_mut(&resource_type).expect(format!("Resource type {resource_type:?} not found in stock").as_str())
    }
}
impl Default for Stock {
    fn default() -> Self {
        let mut stock = HashMap::new();
        stock.insert(ResourceType::DarkOre, StockInfo { amount: 5555, max_amount: MAX_DARK_ORE_STOCK });
        Self(stock)
    }
}