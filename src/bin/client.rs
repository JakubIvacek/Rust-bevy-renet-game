use std::collections::HashMap;
use bevy_kira_audio::prelude::*;
use bevy::{
    prelude::*,
};
use bevy::app::AppExit;
use bevy::ecs::query::QueryData;
use bevy_renet::{
    client_connected,
    renet::{ClientId, RenetClient},
    RenetClientPlugin,
};
use sprite_animation::{AnimationIndices, AnimationTimer, Player};
use demo_bevy::{connection_config, setup_level, NetworkedEntities, NetworkedBoxes, PlayerInput, ServerChannel, ServerMessages, ReadyText};


// MODS
mod audio;
mod game_over;
mod sprite_animation;
mod camera_setup;
mod player_input;

// CONST
const FLOOR_HEIGHT: f32 = 35.0;

// COMPONENTS
#[derive(Component)]
struct ControlledPlayer;
#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

#[derive(Component)]
struct ExitTimer {
    timer: Timer,
}
#[derive(Debug)]
struct PlayerInfo {
    client_entity: Entity,
    server_entity: Entity,
}

#[derive(Debug, Default, Resource)]
struct ClientLobby {
    players: HashMap<ClientId, PlayerInfo>,
}

#[derive(Debug, Resource)]
struct CurrentClientId(u64);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;



#[derive(Component)]
struct Ready;

// CONNECT TO SERVER
#[cfg(feature = "transport")]
fn add_netcode_network(app: &mut App) {
    use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError};
    use demo_bevy::PROTOCOL_ID;
    use std::{net::UdpSocket, time::SystemTime};

    app.add_plugins(bevy_renet::transport::NetcodeClientPlugin);

    app.configure_sets(Update, Connected.run_if(client_connected));

    let client = RenetClient::new(connection_config());
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    app.insert_resource(client);
    app.insert_resource(transport);
    app.insert_resource(CurrentClientId(client_id));

    // If any error is found we just panic
    #[allow(clippy::never_loop)]
    fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
        for e in renet_error.read() {
            panic!("{}", e);
        }
    }

    app.add_systems(Update, panic_on_error_system);
}

// GET SERVER MESSAGES AND DO STUFF WITH THEM ALSO GET ALL THE ENTITIES
fn client_sync_players(
    mut exit: EventWriter<AppExit>,
    audio: Res<Audio>,
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    client_id: Res<CurrentClientId>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut query: Query<(&mut Transform,&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas, &mut Player)>,
    ready_text: Query<Entity, With<ReadyText>>,
    ready: Query<Entity, With<Ready>>,
    dead: Query<Entity, With<game_over::Dead>>,
    //mut query_score: Query<TextBundle, With<ScoreText>>,
) {
    let client_id = client_id.0;
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            // pridaj hraca
            ServerMessages::PlayerCreate { id, translation: _, entity, asset } => {
                println!("Player {} connected.", id);
                let texture = asset_server.load(asset);
                let layout = TextureAtlasLayout::from_grid(Vec2::new(24.0,24.0), 7, 1, None, None);
                let texture_atlas_layout = texture_atlas_layouts.add(layout);
                let animation_indices = AnimationIndices { first: 0, last: 6 };
                // Spawn Player

                let mut client_entity = commands.spawn((
                    SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(100.0, FLOOR_HEIGHT, 2.0)) // transform move sprite
                            * Transform::from_scale(Vec3::splat(4.0)), // scale sprite
                        texture,
                        ..default()
                    },
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: animation_indices.first,
                    },
                    Player{
                        id,
                        should_animate: false,
                        should_invert: false
                    },
                    animation_indices,
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                    // PlayerState::default(),
                    // JumpState::default(), // Add JumpState component with default values
                ));
                if client_id == id.raw() {
                    client_entity.insert(ControlledPlayer);
                }
                let player_info = PlayerInfo {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };
                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
            }
            // odstran hraca
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerInfo {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn();
                    network_mapping.0.remove(&server_entity);
                }
            }
            ServerMessages::SpawnBox { entity,translation } => {
                let novy_box = commands.spawn(SpriteBundle{
                    transform: Transform::from_translation(translation.into()),
                    texture: asset_server.load("box2.png"),
                    ..default()
                });
                network_mapping.0.insert(entity, novy_box.id());
                //update_score(&score);
                //update_score_text(&score, &query_score);
                }

            ServerMessages::DespawnBox{entity_to_despawn} => {
                // bez toho Some to robilo zle veci
                if let Some(entity) = network_mapping.0.remove(&entity_to_despawn) {
                    commands.entity(entity).despawn();
                }

            }
            // start animating player
            ServerMessages::AnimatePlayer{ id, facing_right: _} => {
                sprite_animation::turn_on_animate(id,&mut query);
                // flip_player(id, facing_right, &mut query);
            }
            // stop animating player
            ServerMessages::StopAnimate{ id} => {
                sprite_animation::turn_off_animate(id,&mut query);
            }
            ServerMessages::ExitWindow{} => {
                game_over::exit_game(&mut exit)
            }
            ServerMessages::YouReady{id } => {
               if id.raw() == client_id{
                    spawn_ready(&mut commands)
               }
            }
            ServerMessages::AllReady{} => {
                for entity in ready_text.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in ready.iter() {
                    commands.entity(entity).despawn();
                }
            }
            ServerMessages::YouDead{id } => {
                if id.raw() == client_id{
                    spawn_dead(&mut commands)
                }
            }
            // to play sound
            ServerMessages::SoundAction{ sound } => {
                match sound {
                    1 => audio::jump_audio(&asset_server,&audio),
                    2 => audio::fall_box_audio(&asset_server,&audio),
                    3 => {audio::game_over_audio(&asset_server,&audio);
                        game_over::game_over_spawn(&mut commands, &asset_server, &dead);
                    },
                    4 => audio::death_audio(&asset_server,&audio),
                    _ => { },
                }
            }
            }
        }

    // berie od serveru pozicie entit
    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                let translation = networked_entities.translations[i].into();
                let scale = Vec3::new(networked_entities.scales[i], 4.0, 4.0);
                let transform = Transform {
                    translation,
                    rotation: Default::default(),
                    scale
                };

                commands.entity(*entity).insert(transform);
            }
        }
    }
    // cita channel networkedBoxes
    while let Some(message) = client.receive_message(ServerChannel::NetworkedBoxes) {
        let networked_entities: NetworkedBoxes = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                let translation = networked_entities.translations[i].into();
                let k = Vec3::splat(1.0);
                let transform = Transform {
                    translation,
                    rotation: Default::default(),
                    scale: k,
                };

                commands.entity(*entity).insert(transform);
            }
        }
    }
}
pub fn spawn_ready(mut commands: &mut Commands){
    commands.spawn((TextBundle::from_sections([TextSection::new(
        "Ready",
        TextStyle {
            font_size: 40.0,
            color: Color::rgb(0.0, 0.0, 0.0),
            ..default()
        },
    )]).with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(350.0),
        left: Val::Px(520.0),
        ..default()
    }),Ready));
}
pub fn spawn_dead(mut commands: &mut Commands){
    commands.spawn((TextBundle::from_sections([TextSection::new(
        "YOU ARE DEAD",
        TextStyle {
            font_size: 40.0,
            color: Color::rgb(0.0, 0.0, 0.0),
            ..default()
        },
    )]).with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(350.0),
        left: Val::Px(480.0),
        ..default()
    }),game_over::Dead));
}
#[derive(Debug, Clone, PartialEq, Eq, Resource)]
pub struct Score {
    pub score: u32,
}



#[derive(Component, QueryData)]
pub struct ScoreText;
fn spawn_score(mut commands: &mut Commands){
    commands.spawn((TextBundle::from_sections([TextSection::new(
        "Score : 0",
        TextStyle {
            font_size: 40.0,
            color: Color::rgb(0.0, 0.0, 0.0),
            ..default()
        },
    )]).with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(600.0),
        left: Val::Px(100.0),
        ..default()
    }),ScoreText));
}
// Main WINDOW CREATION AND APP
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game".to_string(),
                resolution: bevy::window::WindowResolution::new(1200., 700.),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }));
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(AudioPlugin);
    //app.init_resource::<Score>();
    #[cfg(feature = "transport")]
    add_netcode_network(&mut app);


    app.insert_resource(ClientLobby::default());
    app.insert_resource(PlayerInput::default());
    app.insert_resource(NetworkMapping::default());
    app.add_systems(Update, player_input::player_input);
    app.add_systems(
        Update,
        (player_input::client_send_input, client_sync_players).in_set(Connected),
    );
    app.add_systems(FixedUpdate, sprite_animation::animate_sprite);

    app.add_systems(Startup, (setup_level, camera_setup::setup_camera, audio::main_music_audio));

    app.run();
}
