use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup_system.system())
        .run();
}

fn startup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // UI camera
    commands.spawn_bundle(UiCameraBundle::default());

    // root node
    /*commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            justify_content: JustifyContent::SpaceBetween,
            ..Default::default()
        },
        material: materials.add(Color::NONE.into()),
        ..Default::default()
    });*/
}
