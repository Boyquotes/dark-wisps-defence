use serde::{Deserialize, Serialize};
use crate::prelude::*;

pub struct ResourcesPlugin;
impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stock>();
    }
}

pub const MAX_DARK_ORE_STOCK: i32 = 9999;
pub const MAX_ESSENCE_STOCK: i32 = 999;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    DarkOre,
    Essence(EssenceType),
}
impl From<EssenceType> for ResourceType {
    fn from(essence_type: EssenceType) -> Self {
        Self::Essence(essence_type)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EssenceType {
    Fire,
    Water,
    Light,
    Electric,
}
impl EssenceType {
    pub const VARIANTS: [EssenceType; 4] = [EssenceType::Fire, EssenceType::Water, EssenceType::Light, EssenceType::Electric];
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EssenceContainer {
    pub essence_type: EssenceType,
    pub amount: i32,
}
impl EssenceContainer {
    pub fn new(essence_type: EssenceType, amount: i32) -> Self {
        Self { essence_type, amount }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cost {
    pub resource_type: ResourceType,
    pub amount: i32,
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
    pub fn can_cover(&self, cost: &Cost) -> bool {
        self.has(cost.resource_type, cost.amount)
    }
    pub fn can_cover_all(&self, costs: &[Cost]) -> bool {
        costs.iter().all(|c| self.has(c.resource_type, c.amount))
    }
    pub fn has(&self, resource_type: ResourceType, amount: i32) -> bool {
        self.get_info(resource_type).amount >= amount
    }
    pub fn add(&mut self, resource_type: ResourceType, amount: i32) {
        let info = self.get_info_mut(resource_type);
        info.amount = std::cmp::min(info.max_amount, info.amount + amount);
    }
    pub fn try_pay_cost(&mut self, cost: Cost) -> bool {
        self.try_remove(cost.resource_type, cost.amount)
    }
    pub fn try_pay_costs(&mut self, costs: &[Cost]) -> bool {
        if !self.can_cover_all(costs) { return false; }
        for cost in costs {
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
        stock.insert(ResourceType::Essence(EssenceType::Fire), StockInfo { amount: 0, max_amount: MAX_ESSENCE_STOCK });
        stock.insert(ResourceType::Essence(EssenceType::Water), StockInfo { amount: 0, max_amount: MAX_ESSENCE_STOCK });
        stock.insert(ResourceType::Essence(EssenceType::Light), StockInfo { amount: 0, max_amount: MAX_ESSENCE_STOCK });
        stock.insert(ResourceType::Essence(EssenceType::Electric), StockInfo { amount: 0, max_amount: MAX_ESSENCE_STOCK });
        Self(stock)
    }
}