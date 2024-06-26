use std::{ time::Duration};

use bevy::prelude::*;
use bevy_renet::renet::{ChannelConfig, ClientId, ConnectionConfig, SendType};
use serde::{Deserialize, Serialize};

#[cfg(feature = "transport")]
pub const PRIVATE_KEY: &[u8; bevy_renet::renet::transport::NETCODE_KEY_BYTES] = b"an example very very secret key."; // 32-bytes
#[cfg(feature = "transport")]
pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Component)]
pub struct Player {
    pub id: ClientId,
    pub asset: String, // keep track of players asset to draw correct for every player
}
#[derive(Component)]
pub struct ReadyText;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub ready: bool,
}

pub enum ClientChannel {
    Input,
    Command,
}
pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
    NetworkedBoxes,
}

#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec3);


#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerCreate {
        entity: Entity,
        id: ClientId,
        translation: [f32; 3],
        asset: String, // to assign asset for new player connected
    },
    PlayerRemove {
        id: ClientId,
    },
    AllReady{},
    // SpawnFakeBox {
    //     entity: Entity,
    //     translation: [f32; 3],
    // },
    // nakoniec som sa rozhodol ze client bude dostavat box aj pre fake boxy lebo on neriesi kolizie
    SpawnBox {
        entity: Entity,
        translation: [f32; 3],
    },
    DespawnBox {
        entity_to_despawn: Entity,
    },
    SoundAction{
        sound: u8,
    },
    AnimatePlayer{
        id: ClientId,
        facing_right: bool
    },
    StopAnimate{
        id: ClientId,
    },
    ExitWindow{},
    YouReady{id: ClientId},
    YouDead{id: ClientId},
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    // tu potom pridaj entity type a tak bude vediet client ako scalovat obrazok podla toho type
    pub entities: Vec<Entity>,
    pub translations: Vec<[f32; 3]>,
    pub scales: Vec<f32>,
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedBoxes{
    pub entities: Vec<Entity>,
    pub translations: Vec<[f32; 3]>,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Command => 0,
            ClientChannel::Input => 1,

        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::Input.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::ZERO,
                },
            },
            ChannelConfig {
                channel_id: Self::Command.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::ZERO,
                },
            },
        ]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 1,
            ServerChannel::NetworkedBoxes => 2,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            // na sync pozicii hracov
            ChannelConfig {
                channel_id: Self::NetworkedEntities.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            // na sync pozicii boxov
            ChannelConfig {
                channel_id: Self::NetworkedBoxes.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            // server messages su napriklad spawni hraca spawni boxu je to enum
            ChannelConfig {
                channel_id: Self::ServerMessages.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
        ]
    }
}

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        server_channels_config: ServerChannel::channels_config(),
    }
}

/// set up a simple 3D scene
pub fn setup_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>, // Adds /assets dir
) {
    // SPAWN BACKGROUND
    commands.spawn(SpriteBundle{
        transform: Transform::from_translation(Vec3::new(0.0, 200.0, 0.0)),
        texture: asset_server.load("origbig.png"),
        ..default()
    });
    let texture_map = asset_server.load("map_version1.png");
    commands.spawn(
        SpriteBundle{
            transform: Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)) // transform move sprite
                * Transform::from_scale(Vec3::splat(4.0)), // scale sprite
            texture: texture_map,
            ..default()
        }
    );
    commands.spawn((TextBundle::from_sections([TextSection::new(
        "PRESS -R- TO GET READY",
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
    }),ReadyText));
}


