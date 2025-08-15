use bevy::ecs::entity_disabling::Disabled;

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

/// When attaching triggers to the parent for change detection, we need to know which Identicator entity is interested in that action
/// Should be attached to the observer entity.
#[derive(Component)]
pub struct IndicatorObserverForChanges(Entity);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
#[component(immutable)]
#[require(IndicatorSpriteHandle, Disabled)]
pub enum IndicatorType {
    NoPower,
    OreDepleted,
    Disabled,
}
impl IndicatorType {
    fn on_insert_update_sprite_handle(
        trigger: Trigger<OnInsert, IndicatorType>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut indicators: Query<(&IndicatorType, &mut IndicatorSpriteHandle, &IndicatorOf, Has<Disabled>)>,
        parents_with_no_power: Query<(), With<NoPower>>,
    ) {
        let entity = trigger.target();
        let (indicator_type, mut sprite_handle, indicator_of, _) = indicators.get_mut(entity).unwrap();
        let path = match indicator_type {
            IndicatorType::NoPower => "indicators/no_power.png",
            IndicatorType::OreDepleted => panic!("No asset yet!"),
            IndicatorType::Disabled => panic!("No asset yet!")
        };
        sprite_handle.0 = asset_server.load(path);

        match indicator_type {
            IndicatorType::NoPower => {
                // Add Power State observers to the parent
                commands.spawn((Observer::new(Self::on_parent_gains_power).with_entity(indicator_of.0), IndicatorObserverForChanges(entity)));
                commands.spawn((Observer::new(Self::on_parent_looses_power).with_entity(indicator_of.0), IndicatorObserverForChanges(entity)));
                // Configure initial state
                if parents_with_no_power.contains(indicator_of.0) {
                    commands.entity(entity).remove::<Disabled>();
                }
            }
            _ => {}
        }
    }

    fn on_parent_looses_power(
        trigger: Trigger<OnInsert, NoPower>,
        mut commands: Commands,
        observers_for_changes: Query<&IndicatorObserverForChanges>,
        indicators: Query<&IndicatorType, With<Disabled>>,
    ) {
        println!("Here in Looses Power1!");
        let observer_entity = trigger.observer();
        let indicator_entity = observers_for_changes.get(observer_entity).unwrap();
        println!("Here in Looses Power!2");
        let Ok(indicator_type) = indicators.get(indicator_entity.0) else { 
            commands.entity(indicator_entity.0).despawn(); // Indicator no longer exist, remove the observer
            return;
        };
        println!("Here in Looses Power!3");
        if !matches!(indicator_type, IndicatorType::NoPower) { return; };
        println!("Here in Looses Power!4");
        commands.entity(indicator_entity.0).remove::<Disabled>();
    }

    fn on_parent_gains_power(
        trigger: Trigger<OnInsert, HasPower>,
        mut commands: Commands,
        observers_for_changes: Query<&IndicatorObserverForChanges>,
        indicators: Query<&IndicatorType>,
    ) {
        let observer_entity = trigger.observer();
        let indicator_entity = observers_for_changes.get(observer_entity).unwrap();
        let Ok(indicator_type) = indicators.get(indicator_entity.0) else { 
            commands.entity(indicator_entity.0).despawn(); // Indicator no longer exist, remove the observer
            return;
        };
        if !matches!(indicator_type, IndicatorType::NoPower) { return; };
        commands.entity(indicator_entity.0).insert(Disabled);
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
        let Ok(indicators) = parents.get(child_of.parent()) else {
            // No active indicators, hide display
            *visibility = Visibility::Hidden;
            continue; 
        };
        let indicator_count: usize = indicators.0.len();
        *visibility = Visibility::Inherited;
        
        // Update cycle time
        display.cycle_time += time.delta_secs();
        if display.cycle_time >= IndicatorDisplay::PERIOD_SECONDS {
            display.cycle_time = 0.;
            display.active_index = (display.active_index + 1) % indicator_count;
        }

        // Get active indicator and update sprite
        let Ok(sprite_handle) = indicators_sprites.get(indicators.0[display.active_index]) else { 
            // Indicator Disabled, cycle
            *visibility = Visibility::Hidden;
            display.active_index = (display.active_index + 1) % indicator_count;
            continue; 
        };
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