use bevy::prelude::{Component, Deref, DerefMut, Query, Res, TextureAtlas, Time, Timer, Transform};
use bevy_renet::renet::ClientId;
#[derive(Component)]
pub struct Player{
    pub id: ClientId,
    pub should_animate: bool,
    pub should_invert: bool
}
#[derive(Component)]
pub(crate) struct AnimationIndices { // TO SAVE ANIMATION SLICES COUNT
    pub first: usize,
    pub last: usize,
}
#[derive(Component, Deref, DerefMut)]
pub  struct AnimationTimer(pub Timer);
pub fn  turn_on_animate(
    id: ClientId,
    query: &mut Query<(&mut Transform,&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas, &mut Player)>
){
    for (_transform, _indices, _timer, _atlas,mut player) in query.iter_mut() {
        if player.id == id{
            player.should_animate = true;
        }
    }
}
pub fn  turn_off_animate(
    id: ClientId,
    query: &mut Query<(&mut Transform,&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas, &mut Player)>
){
    for (_transform,_indices, _timer, _atlas,mut player) in query.iter_mut() {
        if player.id == id{
            player.should_animate = false;
        }
    }
}
pub fn  animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas, &Player)>,  // Query finds gets our Player
) {
    // Animate sprite
    for (indices, mut timer, mut atlas, player) in &mut query {
        if player.should_animate{
            timer.tick(time.delta());
            if timer.just_finished() {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        } else {
            // If no movement buttons are pressed, reset the animation
            timer.reset();
            atlas.index = indices.first;
        }
    }
}

#[allow(dead_code)]
fn main() {}