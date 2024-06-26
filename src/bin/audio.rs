use bevy_kira_audio::prelude::*;
use bevy::prelude::*;

pub fn jump_audio(asset_server: &Res<AssetServer>, audio: &Res<Audio>){
    let music = asset_server.load("jump.ogg");
    audio.play(music);
}

pub fn fall_box_audio(asset_server: &Res<AssetServer>, audio: &Res<Audio>){
    let music = asset_server.load("box_hit_ground.ogg");
    audio.play(music);
}
pub fn death_audio(asset_server: &Res<AssetServer>, audio: &Res<Audio>){
    let music = asset_server.load("death_sound.ogg");
    audio.play(music);
}

pub fn main_music_audio(asset_server: Res<AssetServer>, audio: Res<Audio>){
    let music = asset_server.load("main-theme.ogg");
    audio.play(music).looped();
}

pub fn game_over_audio(asset_server: &Res<AssetServer>, audio: &Res<Audio>){
    let music = asset_server.load("game_over.ogg");
    audio.play(music);
}

#[allow(dead_code)]
fn main() {}