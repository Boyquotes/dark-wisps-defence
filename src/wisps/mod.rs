pub mod components;
mod materials;
pub mod spawning;
pub mod systems;

use bevy::sprite::Material2dPlugin;

use crate::prelude::*;

pub struct WispsPlugin;
impl Plugin for WispsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                Material2dPlugin::<materials::WispFireMaterial>::default(),
                Material2dPlugin::<materials::WispWaterMaterial>::default(),
                Material2dPlugin::<materials::WispLightMaterial>::default(),
                Material2dPlugin::<materials::WispElectricMaterial>::default(),
                
            ))
            .add_event::<spawning::BuilderWisp>()
            .add_systems(PostUpdate, (
                spawning::BuilderWisp::spawn_system,
            ))
            .add_systems(Update, (
                systems::move_wisps,
                systems::target_wisps,
                systems::wisp_charge_attack,
                systems::collide_wisps,
                systems::remove_dead_wisps,
                systems::spawn_wisps.run_if(is_game_mode),
            ));
        app.add_observer(spawning::on_wisp_spawn_attach_material::<components::WispFireType, materials::WispFireMaterial>);
        app.add_observer(spawning::on_wisp_spawn_attach_material::<components::WispWaterType, materials::WispWaterMaterial>);
        app.add_observer(spawning::on_wisp_spawn_attach_material::<components::WispLightType, materials::WispLightMaterial>);
        app.add_observer(spawning::on_wisp_spawn_attach_material::<components::WispElectricType, materials::WispElectricMaterial>);
    }
}