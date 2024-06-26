use std::{collections::HashMap};
use std::time::Duration;
use bevy::{
    prelude::*, };
use bevy_renet::{
    renet::{ClientId, RenetServer, ServerEvent},
    RenetServerPlugin, };
use rand::{thread_rng, Rng};
use demo_bevy::{
    ClientChannel, NetworkedEntities, NetworkedBoxes, Player, PlayerInput, ServerChannel,
                ServerMessages};
use demo_bevy::{setup_level};
// Mutable global variable to keep track of players connected
use std::sync::Mutex;

mod game_over;
mod collision_detection;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref PLAYER_COUNT: Mutex<i32> = Mutex::new(0); // players connected
    static ref BOX: Mutex<i32> = Mutex::new(0); // boxes spawned for testing
}
fn increment_player_count() {
    let mut var = PLAYER_COUNT.lock().unwrap();
    *var += 1;
}
fn increment_box() {
    let mut var = BOX.lock().unwrap();
    *var += 1;
}


// GAME CONSTS
const BOX_SPEED: f32 = 100.0;
const BOX_SIZE: f32 = 72.0;
const PUSH_SPEED: f32 = 2.0;
const FLOOR_HEIGHT: f32 = 35.0;
const PLAYER_SPEED: f32 = 400.0;
const LEFT_WALL: f32 = -400.0;
const RIGHT_WALL:  f32 = 600.0;
const JUMP_VELOCITY: f32 = 500.0;
const NUM_OF_BOXES: usize =14;
const MOST_LEFT_BOX: f32 = -360.0;
const MOST_RIGHT_BOX: f32 = 550.0;
const BOX_SPAWNS: [f32; NUM_OF_BOXES] = [-360.0,-290.0,-220.0,-150.0,-80.0,-10.0,60.0,130.0,200.0,270.0,340.0,410.0,480.0,550.0];

#[derive(States,Debug, Default, Hash, Clone, Eq, PartialEq, Copy)]
enum RunState {
    #[default]
    Waiting,
    Playing,
    GameOver,
}


#[derive(Resource)] // will spawn boxes, resource means it can be acessed like ResMut<BoxSpawner>
struct BoxSpawner{
    timer: Timer,
}
#[derive(Component)]
struct FakeboxState{
    smer_doprava: bool, // state for FakeBox
    index_padnutia: usize,
}

#[derive(Component)]
struct FakeBox;  // box when it's still on the crane

#[derive(Component)]
struct Box;  // box in the game

#[derive(Component)]
struct BoxState{
    oprety_zdola: bool, // kedy gravitacia netaha dole
    _oprety_sprava: bool,
    _oprety_zlava: bool,
    oprety_zhora: bool,
    bot_zmena: bool // zapamatat ci doslo k bot kolizii pri check ak nie tak reset oprety zdola
}
// player animation
#[derive(Component)]
struct AnimationIndices { // TO SAVE ANIMATION SLICES COUNT
    first: usize,
    last: usize,
}
#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

// player movement
#[derive(Component)]
struct JumpState {
    is_jumping: bool,
    jump_timer: Timer,
    fall_timer: Timer,
    can_jump: bool,
    floor_reset: bool // nech moze skocit iba ked sa dotkne zeme od posledneho skoku
}
#[derive(Component)]
struct PlayerState{
    oprety_zprava: bool,
    oprety_zdola: bool,
    oprety_zlava: bool,
    oprety_zhora: bool,
    ready: bool,
    dead: bool,
}



#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<ClientId, Entity>,
}


impl Default for JumpState {
    fn default() -> Self {
        JumpState {
            is_jumping: false,
            floor_reset: true,
            can_jump: true,
            jump_timer: Timer::from_seconds(0.2, TimerMode::Once),
            fall_timer: Timer::from_seconds(0.4, TimerMode::Once),
        }
    }
}
impl Default for PlayerState {
    fn default() -> Self {
        PlayerState{
            oprety_zprava: false,
            oprety_zdola: true,
            oprety_zlava: false,
            oprety_zhora: false,
            ready: false,
            dead: false
        }
    }
}

impl Default for BoxState {
    fn default() -> Self {
        BoxState {
            oprety_zdola: false,
            _oprety_sprava: false,
            _oprety_zlava: false,
            oprety_zhora: false,
            bot_zmena: false
        }
    }
}

// Server config
#[cfg(feature = "transport")]
fn add_netcode_network(app: &mut App) {
    use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
    use bevy_renet::transport::NetcodeServerPlugin;
    use demo_bevy::{connection_config, PROTOCOL_ID};
    use std::{net::UdpSocket, time::SystemTime};

    app.add_plugins(NetcodeServerPlugin);

    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time: Duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    app.insert_resource(server);
    app.insert_resource(transport);
}

// Main app setup game
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game".to_string(),
                resolution: bevy::window::WindowResolution::new(1200., 700.),
                resizable: false,
                visible: false,
                ..Default::default()
            }),
            ..Default::default()
        }));
    app.add_plugins(RenetServerPlugin);
    app.insert_resource(ServerLobby::default());
    app.init_state::<RunState>();
    #[cfg(feature = "transport")]
    add_netcode_network(&mut app);
    app.add_systems(Update, (check_if_all_ready).run_if(in_state(RunState::Waiting)));
    app.add_systems(Update, (server_update_system, server_network_sync));
    app.add_systems(
        Update,
        (
            craning,
            spawn_fake_box,
            fall_boxes,
            handle_collisions,
            check_all_dead,
            //despawn_boxes,
        ).run_if(in_state(RunState::Playing)),
    );

    app.add_systems(FixedUpdate, move_players_system.run_if(in_state(RunState::Playing)));
    app.add_systems(Update, game_over::exit_app_timer.run_if(in_state(RunState::GameOver)));
    app.add_systems(Startup, (setup_level, setup_timer_and_spawner));

    app.run();


}
// GET PLAYER ASSET BASED ON PLAYER COUNT
fn get_asset() -> String {
    return if *PLAYER_COUNT.lock().unwrap() == 0 {
        "running_animation.png".to_string()
    } else if *PLAYER_COUNT.lock().unwrap() == 1 {
        "running_animation2.png".to_string()
    } else if *PLAYER_COUNT.lock().unwrap() == 2 {
        "running_animation3.png".to_string()
    } else {
        "running_animation.png".to_string()
    }
}

// player ready check
fn check_if_all_ready(mut server: ResMut<RenetServer>,
                      mut commands: Commands,
                      mut players: Query<(Entity, &Player, &Transform,&mut PlayerState)>){
    let mut count = 0;
    let mut length = 0;
    for (_entity, _player, _transform, player_state) in players.iter() {
        if player_state.ready != true{
            count = count + 1;
        }
        length += 1;
    }
    if count == 0 && length > 0 {
        commands.insert_resource(NextState(Some(RunState::Playing)));
        let message = bincode::serialize(&ServerMessages::AllReady{}).unwrap();
        server.broadcast_message(ServerChannel::ServerMessages, message);
    }
}
fn check_if_should_ready(
    mut commands: &Commands,
    mut players: &Query<(Entity, &Player, &Transform,&mut PlayerState)>) -> bool {
    let mut count = 0;
    let mut length = 0;
    for (_entity, _player, _transform, player_state) in players.iter() {
        if player_state.ready != true{
            count = count + 1;
        }
        length += 1;
    }
    if count == 0 {
       return false
    }
    return true
}

#[allow(clippy::too_many_arguments)]
// prijma network spravy a kona na zaklade nich
// take client messages and do stuff based on them
fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    mut players: Query<(Entity, &Player, &Transform,&mut PlayerState)>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);
                // Initialize other players for this new client
                for (entity, player, transform, _player_state) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        id: player.id,
                        entity,
                        translation,
                        asset: player.asset.clone(),
                    })
                        .unwrap();
                    server.send_message(*client_id, ServerChannel::ServerMessages, message);
                }

                // Spawn new player
                let transform = Transform::from_translation(Vec3::new(100.0, FLOOR_HEIGHT, 2.0))* Transform::from_scale(Vec3::splat(4.0));
                let texture = asset_server.load("running_animation.png");
                let layout = TextureAtlasLayout::from_grid(Vec2::new(24.0,24.0), 7, 1, None, None);
                let texture_atlas_layout = texture_atlas_layouts.add(layout);
                let animation_indices = AnimationIndices { first: 0, last: 6 };
                let player_entity = commands.spawn((
                    SpriteBundle {
                        transform, // transform move sprite
                        // scale sprite
                        texture,
                        ..default()
                    },
                    TextureAtlas {
                        layout: texture_atlas_layout,
                        index: animation_indices.first,
                    },
                    // animation_indices,
                    // AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                    // Player, // Player tag for queries
                    PlayerState::default(),
                    JumpState::default(), // Add JumpState component with default values
                )).insert(PlayerInput::default())
                    .insert(Player {
                        id: *client_id,
                        asset: get_asset()
                    })
                    .id();

                lobby.players.insert(*client_id, player_entity);

                let translation: [f32; 3] = transform.translation.into();
                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *client_id,
                    entity: player_entity,
                    translation,
                    asset: get_asset()
                })
                    .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
                {
                    let mut count = PLAYER_COUNT.lock().unwrap();
                    *count += 1;
                }
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {} disconnected: {}", client_id, reason);

                if let Some(player_entity) = lobby.players.remove(client_id) {
                    commands.entity(player_entity).despawn();
                }

                let message = bincode::serialize(&ServerMessages::PlayerRemove { id: *client_id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }
    // movement update cita input kanal tam sa posielaju keypress spravy
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            if input.right{
                let message = bincode::serialize(&ServerMessages::AnimatePlayer{
                    id: client_id,
                    facing_right: true
                }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            else if input.left{
                let message = bincode::serialize(&ServerMessages::AnimatePlayer{
                    id: client_id,
                    facing_right: false
                }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }else{
                let message = bincode::serialize(&ServerMessages::StopAnimate{
                    id: client_id
                }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            if input.ready{
                for (_entity, player, _transform, mut player_state) in players.iter_mut(){
                    if player.id == client_id{
                        //println!("player_entered ready");
                        player_state.ready = true;
                    }
                }
                if check_if_should_ready(&commands, &players){
                    let message = bincode::serialize(&ServerMessages::YouReady{
                        id: client_id
                    }).unwrap();
                    server.broadcast_message(ServerChannel::ServerMessages, message);
                }
            }
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
//posiela info o poziciach networked_entities vsetkym hracom
fn server_network_sync(mut server: ResMut<RenetServer>, players_query: Query<(Entity, &Transform),With<Player>>, boxes_query: Query<(Entity, &Transform),Without<Player>>){ // dangerous second query
    // najskor hraci
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform) in players_query.iter() {
        networked_entities.entities.push(entity);
        networked_entities.translations.push(transform.translation.into());
        networked_entities.scales.push(transform.scale.x);
    }
    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
    // potom boxy
    let mut networked_entities = NetworkedBoxes::default();
    for (entity, transform) in boxes_query.iter() {
        networked_entities.entities.push(entity);
        networked_entities.translations.push(transform.translation.into());
    }
    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedBoxes, sync_message);


}

fn move_players_system(mut server: ResMut<RenetServer>, mut query: Query<(&mut Transform, &PlayerInput,&mut JumpState,&mut PlayerState),With<Player>>, time: Res<Time>) {
    //println!("dlzka qveriny { }",query.iter().len());
    for (mut transform, input,mut jump_state,player_state) in query.iter_mut() {
        //println!("iteracia { }",ff);
        //ff+= 1;
        let x = (input.right as i8 - input.left as i8) as f32;

        // To flip player
        if x > 0.0{
            transform.scale.x = 4.0;
        }else if x < 0.0{
            transform.scale.x = -4.0;
        }
        let mut new_player_position_x = transform.translation.x + x * PLAYER_SPEED * time.delta_seconds();
        if new_player_position_x < LEFT_WALL {
            new_player_position_x = LEFT_WALL;
        }
        else if new_player_position_x > RIGHT_WALL{
            new_player_position_x = RIGHT_WALL;
        }

        if new_player_position_x > transform.translation.x{
            if player_state.oprety_zprava == true{
                new_player_position_x = transform.translation.x;
            }
        }
        if new_player_position_x < transform.translation.x{
            if player_state.oprety_zlava == true{
                new_player_position_x = transform.translation.x;
            }
        }

        // toto je nechutne prepojene s collision checkingom ale co uz
        if player_state.oprety_zdola == true{  // is not standing on something

            jump_state.is_jumping = false;
            jump_state.floor_reset = true;
        }
        else if jump_state.is_jumping == false{ // is not in the jump phase
            transform.translation.y -= time.delta_seconds()*PLAYER_SPEED;
        }

        transform.translation.x = new_player_position_x;
        // teraz y suradnicu riesime:
        if input.up && !jump_state.is_jumping &&  jump_state.can_jump && jump_state.floor_reset {
            //START JUMP WHEN PRESSED checking jump_state so doesnt jump again when pressing
            if player_state.dead == false{
                let message = bincode::serialize(&ServerMessages::SoundAction{
                    sound: 1
                }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            jump_state.is_jumping = true;
            jump_state.floor_reset = false;
            jump_state.can_jump = false;
            jump_state.jump_timer.reset();
            jump_state.fall_timer.reset();
        }
        if jump_state.is_jumping {
            // Jumping up
            if !jump_state.jump_timer.finished(){
                transform.translation.y += JUMP_VELOCITY * time.delta_seconds();
            }
            //Jumping down when up is finished
            if jump_state.jump_timer.finished() {
                transform.translation.y -= JUMP_VELOCITY * time.delta_seconds();
            }
            //Tick timers and check finish
            jump_state.jump_timer.tick(time.delta());
            jump_state.fall_timer.tick(time.delta());
            if jump_state.fall_timer.finished(){
                jump_state.is_jumping = false;
            }
        }
    }
    for (_, _, mut jump_state, _) in query.iter_mut() {
        if jump_state.floor_reset {
            jump_state.can_jump = true;
        }
    }
}



pub fn setup_timer_and_spawner(mut commands: Commands) {
    commands.spawn(( // Treba tam kameru lebo potom hadze warningy aj ked nema okno
        Camera2dBundle {
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
            ..default()
        },
    ));
    commands.insert_resource(BoxSpawner {
        // create the repeating timer
        timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
    });
    commands.insert_resource(game_over::GameOverTimer {
        // create the repeating timer
        timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
        second_timer:  Timer::new(Duration::from_secs(10), TimerMode::Repeating),
    });
}

// ovlada boxy ked su este fake boxy (cestuju ma zeriave do stran)
fn craning(mut query: Query<(&mut Transform,Entity,&mut FakeboxState), With<FakeBox>>,time: Res<Time>,mut commands: Commands,
           asset_server: Res<AssetServer>, mut server: ResMut<RenetServer>,){
    for (mut transform,entity, fakebox_state) in &mut query.iter_mut(){  // pre kazdu
        if fakebox_state.smer_doprava{
            transform.translation.x =  transform.translation.x + BOX_SPEED * time.delta_seconds();
        }
        else{
            transform.translation.x =  transform.translation.x - BOX_SPEED * time.delta_seconds();
        }


        // spravi realny box ak je na spawn pozicii

        if ((BOX_SPAWNS[fakebox_state.index_padnutia] - transform.translation.x.round()).abs()) < 10.0 { // tricky porovnavanie floatov
            let novy_box_transform: Transform;
            novy_box_transform = Transform::from_translation(Vec3::new(BOX_SPAWNS[fakebox_state.index_padnutia], 520.0, 2.0)) // transform move sprite
                * Transform::from_scale(Vec3::splat(3.0));
            let novy_translation: [f32; 3] = novy_box_transform.translation.into();
            let novy_box = commands.spawn((SpriteBundle{
                transform: novy_box_transform,
                texture: asset_server.load("box.png"),
                ..default()
            },
                                               Box,
                                               BoxState::default())).id();
            // odstrani fakovy box
            let despawn_message = ServerMessages::DespawnBox {entity_to_despawn:entity};
            let despawn_message = bincode::serialize(&despawn_message).unwrap();
            server.broadcast_message(ServerChannel::ServerMessages,despawn_message);
            commands.entity(entity).despawn();

            // aby vsetci clienti spravili tuto entitu u seba
            let message = ServerMessages::SpawnBox {
                entity: novy_box,
                translation: novy_translation,
            };
            let message = bincode::serialize(&message).unwrap();
            server.broadcast_message(ServerChannel::ServerMessages,message);

        }
    }
}



fn spawn_fake_box(mut server: ResMut<RenetServer>, mut commands: Commands, time: Res<Time>, mut spawn_timer: ResMut<BoxSpawner>, asset_server: Res<AssetServer>, ){


    spawn_timer.timer.tick(time.delta());
                                                                 // TOTO NA TESTING ABY SA NESPAWNOVALI STALE BOXI
    if spawn_timer.timer.finished()   { // To start spawning only when player is connected  mozme nastavit asi potom na 2-3 ako chceme
        let num = thread_rng().gen_range(1..8);
        let mut rng = thread_rng();
        let bul =  rng.gen_bool(1.0 / 2.0);  // sanca 1/2

        spawn_timer.timer.set_duration(Duration::from_secs(num));
        let random_index = thread_rng().gen_range(0..NUM_OF_BOXES-1);
        println!("{} spawned", random_index);

        let novy_box;
        let novy_translation;
        if bul == false{
            novy_translation = [-500.0, 500.0, 2.0];
            novy_box = commands.spawn((SpriteBundle{
                transform: Transform::from_translation(Vec3::new(-500.0, 500.0, 2.0)) // transform move sprite
                ,
                texture: asset_server.load("box2.png"),
                ..default()
            },
                                       FakeBox,FakeboxState{smer_doprava:true,index_padnutia:random_index},
            )).id();
        }
        else{
            novy_translation = [600.0, 500.0, 2.0];
            novy_box = commands.spawn((SpriteBundle{
                transform: Transform::from_translation(Vec3::new(600.0, 500.0, 2.0)) // transform move sprite
                ,
                texture: asset_server.load("box2.png"),
                ..default()
            },
                                       FakeBox,FakeboxState{smer_doprava:false,index_padnutia:random_index},
            )).id();
        }
        let message = ServerMessages::SpawnBox {
            entity: novy_box,
            translation: novy_translation,
        };
        let message = bincode::serialize(&message).unwrap();
        server.broadcast_message(ServerChannel::ServerMessages,message);
        increment_box(); // TESTING
    }
}

fn fall_boxes(_server: ResMut<RenetServer>, mut query: Query<(&mut Transform, &mut BoxState), With<Box>>,time: Res<Time>,){
    for (mut transform, box_state) in &mut query.iter_mut(){
        if box_state.oprety_zdola == false{
            transform.translation.y =  transform.translation.y - BOX_SPEED * time.delta_seconds();
        }
    }
}
pub fn despawn_boxes(mut server: ResMut<RenetServer>, mut commands: Commands, mut boxy: Query<(&mut Transform,&mut BoxState, Entity), With<Box>>){
    //checkovat v loop ci je neajky riadok zaplneny cely ten ze sa tam uz nemesti dalsi box
    // potom len posielat spravy o despawnovani boxov aj tu despawnut
    struct Point {
        x: f32,
        y: f32,
    }
    let mut all_points: Vec<Point> = Vec::with_capacity(100);
    let mut levels: Vec<f32> = Vec::with_capacity(10);
    // Iterate through boxes to collect their positions
    for (transform, box_state, _) in boxy.iter_mut() {
        if box_state.oprety_zdola == true{
            all_points.push(Point {
                x: transform.translation.x,
                y: transform.translation.y
            });
        }
    }
    //Get all levels boxes
    for point in &all_points {
        if !levels.contains(&point.y) {
            levels.push(point.y);
        }
    }
    //println!("{}", levels.len());
    let left: f32 = -360.0;
    let right: f32 = 550.0;
    for level in levels{
        let mut space_available = true;

        // Sort boxes on this level by their x position
        let mut boxes_on_level: Vec<&Point> = all_points.iter()
            .filter(|point| point.y == level)
            .collect();
        boxes_on_level.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        let mut string = String::new();
        for i in 0..(boxes_on_level.len() - 1) {
            string.push_str(&format!("{} ", i));
        }
        println!("{}", string);
        for i in 0..(boxes_on_level.len() - 1) {
            let box_right_edge = boxes_on_level[i].x + 71.0;
            let next_box_left_edge = boxes_on_level[i + 1].x;
            if next_box_left_edge - box_right_edge < 71.0 {
                // There's not enough space between boxes on this level to fit another box
                //space_available = false;
                break;
            }
        }
        if space_available == false{
            for (_, _, mut entity) in boxy.iter_mut() {
                let message = ServerMessages::DespawnBox {
                    entity_to_despawn: entity
                };
                let message = bincode::serialize(&message).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages,message);
                commands.entity(entity).despawn();
            }
        }
    }

    /*let message = ServerMessages::DespawnBox {
        entity_to_despawn:
    }
    let message = bincode::serialize(&message).unwrap();
    server.broadcast_message(ServerChannel::ServerMessages,message);*/
}

fn handle_collisions(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut player: Query<(&mut Transform, &mut PlayerState, &mut JumpState, &mut Player, Entity)>,
    mut boxy: Query<(&mut Transform,&mut BoxState), (With<Box>,Without<Player>)>,
    //mut run_state: ResMut<State<RunState>>,
)
{
    // toto su iba kolizie medzi boxami
    // reset bot zmena a oprety zhora
    for ( _transform,mut box_state)in &mut boxy.iter_mut(){
        box_state.bot_zmena = false;
        box_state.oprety_zhora = false;
    }
    // check collisions
    let mut combos = boxy.iter_combinations_mut();
    while let Some([( mut trans_1,mut box_state_1), (mut trans_2, mut box_state_2)]) = combos.fetch_next(){
        let a = collision_detection::AABB{
            min: (trans_1.translation.x + 1.0, trans_1.translation.y),
            max: (trans_1.translation.x + BOX_SIZE - 1.0, trans_1.translation.y + BOX_SIZE)
        };
        let b = collision_detection::AABB{
            min: (trans_2.translation.x + 1.0, trans_2.translation.y),
            max: (trans_2.translation.x + BOX_SIZE - 1.0, trans_2.translation.y + BOX_SIZE)
        };
        let result = collision_detection::test_aabb_overlap(a, b); // Returns depth_y depth_x
        // CHECK ON WHICH SIDE THE COLLISION HAPPENED
        match result{
            Some((depth_x, depth_y)) => {
                if depth_x.abs() < depth_y.abs() {
                    // Collision along the X axis. React accordingly
                    if depth_x > 0.0 {
                        box_state_1._oprety_zlava = true;
                        box_state_2._oprety_sprava = true;
                        //trans_1.translation.x = trans_2.translation.x + BOX_SIZE;
                    } else {
                        //PRAVO
                        box_state_2._oprety_zlava = true;
                        box_state_1._oprety_sprava = true;
                        //trans_2.translation.x = trans_1.translation.x + BOX_SIZE;
                    }
                } else {
                    // Collision along the Y axis.
                    if depth_y > 0.0 {
                        // Top side collision
                    } else  if depth_y < 0.0{
                        // Box 2 fell on Box 1 send message with audio and set state
                        box_state_2.bot_zmena = true;
                        if box_state_2.oprety_zdola == false{
                            let message = bincode::serialize(&ServerMessages::SoundAction{
                                sound: 2
                            }).unwrap();
                            server.broadcast_message(ServerChannel::ServerMessages, message);
                        }
                        trans_2.translation.y = trans_1.translation.y + BOX_SIZE - 1.0;
                        box_state_2.oprety_zdola = true;
                        box_state_1.oprety_zhora = true;
                    }
                }
            }, _ => {}
        }
    }
    // Check if some box is in collision with ground send audio message if yes amd set state
    for ( mut transform,mut box_state)in &mut boxy.iter_mut(){
        if transform.translation.y <= FLOOR_HEIGHT - 3.0{
            box_state.bot_zmena = true;
            if box_state.oprety_zdola == false{
                let message = bincode::serialize(&ServerMessages::SoundAction{
                    sound: 2
                }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            box_state.oprety_zdola = true;
            transform.translation.y = FLOOR_HEIGHT - 3.0;
        }
    }
    // Check if bot collision with box or ground if not reset oprety_zdola
    for ( _transform,mut box_state)in &mut boxy.iter_mut(){
        if box_state.bot_zmena == false{
            box_state.oprety_zdola = false;
        }
    }
    // Collisions player boxes
    for (mut player_transform, mut player_state, jump_state,mut  player, entity) in &mut player.iter_mut(){
        player_state.oprety_zprava  = false;
        player_state.oprety_zlava = false;
        let mut bot = false;
        if player_state.dead != true {
            for (mut box_transform, mut _box_state) in boxy.iter_mut() {
                let a = collision_detection::AABB {
                    min: (player_transform.translation.x + 5.0, player_transform.translation.y),
                    max: (player_transform.translation.x + 75.0, player_transform.translation.y + 70.0)
                };
                let b = collision_detection::AABB {
                    min: (box_transform.translation.x + 1.0, box_transform.translation.y),
                    max: (box_transform.translation.x + BOX_SIZE - 1.0, box_transform.translation.y + BOX_SIZE)
                };
                let result = collision_detection::test_aabb_overlap(a, b); // Returns depth_y depth_x
                // CHECK ON WHICH SIDE THE COLLISION HAPPENED
                match result {
                    Some((depth_x, depth_y)) => {
                        if depth_x.abs() < depth_y.abs() {
                            // Collision along the X axis. React accordingly
                            if depth_x > 0.0 {
                                //  Player collision left -> pushing box left
                                player_state.oprety_zlava = true;
                                if _box_state._oprety_zlava != true && jump_state.is_jumping == false && _box_state.oprety_zhora != true {
                                    let mut new_x = box_transform.translation.x - PUSH_SPEED;
                                    if new_x <= MOST_LEFT_BOX { //check most_left spawn
                                        new_x = MOST_LEFT_BOX
                                    }
                                    box_transform.translation.x = new_x;
                                }
                            } else {
                                //  Player collision right -> pushing box right
                                player_state.oprety_zprava = true;
                                if _box_state._oprety_sprava != true && jump_state.is_jumping == false && _box_state.oprety_zhora != true {
                                    let mut new_x = box_transform.translation.x + PUSH_SPEED;
                                    if new_x >= MOST_RIGHT_BOX { // check most_right spawn
                                        new_x = MOST_RIGHT_BOX
                                    }
                                    box_transform.translation.x = new_x;
                                }
                            }
                        } else {
                            // Collision along the Y axis.
                            if depth_y > 0.0 {
                                bot = true;
                                // Top side collision
                                // Player bot collision with top of box so jump on box
                                if player_state.oprety_zdola == false {
                                    player_transform.translation.y = box_transform.translation.y + BOX_SIZE - 1.0;
                                }
                                player_state.oprety_zdola = true;
                            } else {
                                if _box_state.oprety_zdola == false {
                                    // PLAYER DEAD
                                    let message = bincode::serialize(&ServerMessages::SoundAction{
                                        sound: 4
                                    }).unwrap();
                                    server.broadcast_message(ServerChannel::ServerMessages, message);
                                    let message = bincode::serialize(&ServerMessages::YouDead {
                                        id: player.id
                                    }).unwrap();
                                    server.broadcast_message(ServerChannel::ServerMessages, message);
                                    player_state.dead = true;
                                    let message = bincode::serialize(&ServerMessages::PlayerRemove { id: player.id }).unwrap();
                                    server.broadcast_message(ServerChannel::ServerMessages, message);
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        // Check ground collision
        if player_transform.translation.y <= FLOOR_HEIGHT{
            bot = true;
            player_state.oprety_zdola = true;
            player_transform.translation.y = FLOOR_HEIGHT;
        }
        // If collision with ground or box not detected reset oprety_zdola
        if bot == false{
            player_state.oprety_zdola = false
        }
    }
}

fn check_all_dead(mut server: ResMut<RenetServer>,
                  mut commands: Commands,
                  mut players: Query<(Entity, &Player, &Transform,&mut PlayerState)>){
    let mut count = 0;
    let mut length = 0;
    for (_entity, _player, _transform, player_state) in players.iter() {
        if player_state.dead != true{
            count = count + 1;
        }
        length += 1;
    }
    if count == 0 && length > 0 {
        commands.insert_resource(NextState(Some(RunState::GameOver)));
        let message = bincode::serialize(&ServerMessages::SoundAction{
            sound: 3
        }).unwrap();
        server.broadcast_message(ServerChannel::ServerMessages, message);
    }
}