use bevy::prelude::{Camera2dBundle, Commands, default, Transform};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
            ..default()
        },
        // tu bol marker (struct) aby sa dala vyhladavat ale nehybeme nou tak netreba
    ));
}

#[allow(dead_code)]
fn main() {}