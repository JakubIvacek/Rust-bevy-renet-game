use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use demo_bevy::{ServerChannel, ServerMessages};

#[derive(Resource)]
pub struct GameOverTimer{
    pub timer: Timer,
    pub second_timer: Timer
}
#[derive(Component)]
pub struct Dead;
pub fn game_over_spawn(mut commands: &mut Commands,  asset_server: &Res<AssetServer>,dead: &Query<Entity, With<Dead>>){
    for entity in dead.iter() {
        commands.entity(entity).despawn();
    }
    commands.spawn(SpriteBundle{
        transform: Transform::from_translation(Vec3::new(0.0, 200.0, 5.0)),
        texture: asset_server.load("origbig.png"),
        ..default()
    });
    commands.spawn(SpriteBundle{
        transform: Transform::from_translation(Vec3::new(100.0, 250.0, 6.0)),
        texture: asset_server.load("game_over.png"),
        ..default()
    });
    commands.spawn((TextBundle::from_sections([TextSection::new(
        "all of the players died",
        TextStyle {
            font_size: 40.0,
            color: Color::rgb(0.0, 0.0, 0.0),
            ..default()
        },
    )]).with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(250.0),
        left: Val::Px(380.0),
        ..default()
    }),));
    commands.spawn((TextBundle::from_sections([TextSection::new(
        "exiting ...",
        TextStyle {
            font_size: 40.0,
            color: Color::rgb(0.0, 0.0, 0.0),
            ..default()
        },
    )]).with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(210.0),
        left: Val::Px(490.0),
        ..default()
    }),));

}
pub fn exit_game(mut exit: &mut EventWriter<AppExit>) {
    exit.send(AppExit);
}

pub fn exit_app_timer(mut over_timer: ResMut<GameOverTimer>,
                      mut server: ResMut<RenetServer>,
                      time: Res<Time>,
                      mut exit: EventWriter<AppExit>) {
    over_timer.timer.tick(time.delta());
    over_timer.second_timer.tick(time.delta());
    if over_timer.timer.finished(){
        let message = bincode::serialize(&ServerMessages::ExitWindow {}).unwrap();
        server.broadcast_message(ServerChannel::ServerMessages, message);
    }
    if over_timer.second_timer.finished(){
        exit.send(AppExit);
    }

}

#[allow(dead_code)]
fn main() {}