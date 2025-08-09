use crate::prelude::*;

pub struct IndicatorsPlugin;
impl Plugin for IndicatorsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                cycle_indicators_system,
            ))
            .add_observer(IndicatorType::on_insert_update_sprite_handle);
    }
}

#[derive(Component)]
#[relationship(relationship_target = Indicators)]
pub struct IndicatorOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = IndicatorOf, linked_spawn)]
pub struct Indicators(Vec<Entity>);

#[derive(Component)]
#[require(Transform, Visibility, Sprite, ZDepth = Z_MAP_UI)]
pub struct IndicatorDisplay {
    pub active_index: usize,
    pub cycle_time: f32,
}
impl IndicatorDisplay {
    const PERIOD_SECONDS: f32 = 3.;
    const MIN_ALPHA: f32 = 0.;
    const MAX_ALPHA: f32 = 1.;
}
impl Default for IndicatorDisplay {
    fn default() -> Self {
        Self {
            active_index: 0,
            cycle_time: 0.0,
        }
    }
}

#[derive(Component, Default)]
struct IndicatorSpriteHandle(Handle<Image>);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
#[component(immutable)]
#[require(IndicatorSpriteHandle)]
pub enum IndicatorType {
    NoPower,
    OreDepleted,
    Disabled,
}
impl IndicatorType {
    fn on_insert_update_sprite_handle(
        trigger: Trigger<OnInsert, IndicatorType>,
        asset_server: Res<AssetServer>,
        mut indicators: Query<(&IndicatorType, &mut IndicatorSpriteHandle)>,
    ) {
        let entity = trigger.target();
        let Ok((indicator_type, mut sprite_handle)) = indicators.get_mut(entity) else { return; };
        let path = match indicator_type {
            IndicatorType::NoPower => "indicators/no_power.png",
            IndicatorType::OreDepleted => panic!("No asset yet!"),
            IndicatorType::Disabled => panic!("No asset yet!")
        };
        sprite_handle.0 = asset_server.load(path);
    }
}

// Cycle through indicators and animate fade in/out.
fn cycle_indicators_system(
    time: Res<Time>,
    parents: Query<&Indicators>,
    indicators_sprites: Query<&IndicatorSpriteHandle>,
    mut displays: Query<(&mut IndicatorDisplay, &mut Sprite, &mut Visibility, &ChildOf)>,
) {
    for (mut display, mut sprite, mut visibility, child_of) in displays.iter_mut() {
        let Ok(indicators) = parents.get(child_of.parent()) else { continue; };
        let indicator_count: usize = indicators.0.len();
        if indicator_count == 0 {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Inherited;
        // Update cycle time
        display.cycle_time += time.delta_secs();
        if display.cycle_time >= IndicatorDisplay::PERIOD_SECONDS {
            display.cycle_time = 0.;
            display.active_index = (display.active_index + 1) % indicator_count;
        }

        // Get active indicator and update sprite
        let Ok(sprite_handle) = indicators_sprites.get(indicators.0[display.active_index]) else { continue; };
        sprite.image = sprite_handle.0.clone();
                
        // Calculate fade alpha based on cycle time
        let progress = display.cycle_time / IndicatorDisplay::PERIOD_SECONDS;
        let alpha = if progress < 0.5 {
            IndicatorDisplay::MIN_ALPHA + (IndicatorDisplay::MAX_ALPHA - IndicatorDisplay::MIN_ALPHA) * (progress * 2.0)
        } else {
            IndicatorDisplay::MAX_ALPHA - (IndicatorDisplay::MAX_ALPHA - IndicatorDisplay::MIN_ALPHA) * ((progress - 0.5) * 2.0)
        };
        sprite.color.set_alpha(alpha);
    }
}