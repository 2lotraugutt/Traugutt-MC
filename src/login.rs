use std::collections::HashMap;
use std::fs::read_to_string; use std::env;

use valence::*;
use valence::prelude::*;

use valence::entity::living::Health;

use valence::command::handler::CommandResultEvent;
use valence::command::scopes::CommandScopes;
use valence::command::AddCommand;
use valence::command_macros::Command;

use valence::client::ViewDistance;

use crate::open_world::OpenWorldResource;
// use crate::spawn::SpawnResource;

const LOGIN_SPAWN_POS: [f64; 3] = [
    1.5 as f64,
    128 as f64,
    1.5 as f64,
];

pub struct LoginPlugin;
impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        let login_asset = LoginResource::serialize_from_file().unwrap();
        app
            .insert_resource(login_asset)
            .add_systems(Startup, setup)
            .add_command::<LoginCommand>()
            .add_command::<SpectateCommand>()
            .add_event::<LoginEvent>()
            .add_systems(Update, (handle_login_command,handle_spectate_command,init_clients));
    }
}

#[derive(Debug, Resource)]
pub struct LoginResource {
    pub layer_id: Option<Entity>,
    map: HashMap<String, (String, Vec<String>)>
}
impl LoginResource {
    pub fn serialize_from_file () -> std::io::Result<LoginResource>{
        let login_file = env::var("LOGIN_FILE").unwrap_or("creds.txt".to_string());
        let mut map = HashMap::<String, (String, Vec<String>)>::new();
        for line in read_to_string(login_file).unwrap_or("admin admin".to_string()).lines() {
            let mut itr = line.split(" ");
            let login = itr.next().unwrap_or("none");
            let password = itr.next().unwrap_or("none");
            let _ = map.insert(login.to_string(), (password.to_string(),itr.map(|x| x.to_string()).collect()));
        }
        Ok(LoginResource{map, layer_id: None})
         
    }
    pub fn verify(self: &LoginResource,login: &String, passwod: &String) -> bool {
        if let Some((password_map,_)) = self.map.get(login) {
            if *password_map == *passwod { return true; }
        }
        false
    }
    pub fn roles(self: &LoginResource,login: &String) -> Vec<String> {
        if let Some((_,vector)) = self.map.get(login) {
            return vector.to_vec();
        }
        unreachable!();
    }
}

#[derive(Command, Debug, Clone)]
#[paths("spectate")]
#[scopes("notloged.spectate")]
pub(crate) struct SpectateCommand {
}

#[derive(Command, Debug, Clone)]
#[paths("login {password}")]
#[scopes("notloged.login")]
pub(crate) struct LoginCommand {
    password: String
}

#[derive(Event)]
pub struct LoginEvent{
    pub player: Entity
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    mut dimensions: ResMut<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>, 
    mut login_resource: ResMut<LoginResource>
) {
    let mut dim = DimensionType::default();
    dim.has_skylight = false;
    dimensions.insert(Ident::new("login").unwrap(), dim);    
    let mut login_layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);
    worldgen(&mut login_layer);

    let spawned = commands.spawn(login_layer);
    login_resource.layer_id =  Some(spawned.id());
}

fn worldgen(
    layer: &mut LayerBundle,
) {
    layer.chunk.insert_chunk([0, 0], UnloadedChunk::new());
    let block = BlockState::BARRIER;
    // let block = BlockState::STONE;
    layer.chunk.set_block([1, 127, 1], block);
    layer.chunk.set_block([1, 130, 1], block);
    for y in 128..=129 {
            layer.chunk.set_block([2, y, 1], block);
            layer.chunk.set_block([0, y, 1], block);
            layer.chunk.set_block([1, y, 2], block);
            layer.chunk.set_block([1, y, 0], block);
    }
}




fn handle_login_command(
    mut events: EventReader<CommandResultEvent<LoginCommand>>,
    mut clients: Query<(
            &mut Client, // client
            &mut CommandScopes,
            &Username
        )>,
    login_resource: Res<LoginResource>,
    mut ev_login: EventWriter<LoginEvent>,
    // spawn_resource: Res<SpawnResource>,
    // open_world_resource: Res<OpenWorldResource>, 
) {
    for event in events.read() {
        let compiled_command = &event.result;
        let (
            ref mut client,
            ref mut permissions,
            ref username,
        ) = &mut clients.get_mut(event.executor).unwrap();
        let username = (**username).to_string();
        let password = &compiled_command.password;
        if login_resource.verify(&username, &password) {
            client.send_chat_message(
                "[Login] ".into_text().color(Color::GOLD) +
                "You loged in".into_text().color(Color::GREEN).bold().not_italic()
            );
            permissions.add("loged");
            permissions.remove("notloged");
            for role in login_resource.roles(&username) {
                client.send_chat_message(
                "[Login] ".into_text().color(Color::GOLD) +
                "You have role: ".into_text().color(Color::WHITE) +
                (&role).into_text().color(Color::GREEN).not_italic()
                );
                permissions.add(&role);
            }
            ev_login.send(LoginEvent{player: event.executor});
        }
        else {
            client.send_chat_message(
                "[Login] ".into_text().color(Color::GOLD) +
                "LOGIN FAILED".into_text().color(Color::RED).bold().not_italic()
            );
        }
    }
}
fn handle_spectate_command(
    mut events: EventReader<CommandResultEvent<SpectateCommand>>,
    mut clients: Query<(
            &mut Client, // client
            &mut CommandScopes,
            &mut EntityLayerId, // layer_id
            &mut VisibleChunkLayer, // visable_chunk_layer
            &mut VisibleEntityLayers, // visable_chunk_layer
            &mut GameMode,
        )>,
    open_world_resource: Res<OpenWorldResource>, 
    // mut ev_login: EventWriter<LoginEvent>,
) {
    for event in events.read() {
        let (
            ref mut client,
            ref mut permissions,
            ref mut layer_id,
            ref mut visible_chunk_layer,
            ref mut visible_entity_layer,
            ref mut gamemode,
        ) = &mut clients.get_mut(event.executor).unwrap();
        client.send_chat_message(
            "[Login] ".into_text().color(Color::GOLD) +
            "You are a spectator of the world".into_text().color(Color::GREEN).bold().not_italic()
        );
        permissions.add("spectator");
        permissions.remove("notloged");
        **gamemode = GameMode::Spectator;
        layer_id.0 = open_world_resource.layer_id.unwrap();
        visible_chunk_layer.0 = open_world_resource.layer_id.unwrap();
        visible_entity_layer.0.insert(open_world_resource.layer_id.unwrap());
        // ev_login.send(SpectateEvent{player: event.executor});
    }
}

fn init_clients(
    mut clients: Query<
        (
            &mut Client, // client
            &mut EntityLayerId, // layer_id
            &mut VisibleChunkLayer, // visable_chunk_layer
            &mut Position, // pos
            &mut GameMode, // game_mode
            &mut Health, //health
            &mut CommandScopes, //permisssions
            &mut ViewDistance
        ),
        Added<Client>,
    >,
    login_resource: Res<LoginResource>
) {
    for (
        mut client,
        mut layer_id,
        mut visible_chunk_layer,
        mut pos,
        mut game_mode,
        mut health,
        mut permissions,
        mut distance
    ) in &mut clients
    {
        distance.set(10);
        let layer = login_resource.layer_id.unwrap();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        pos.set(LOGIN_SPAWN_POS);
        *game_mode = GameMode::Adventure;
        health.0 = 20.0;

        client.send_chat_message(
            "Welcome to ".into_text() +
            "2LO Traugutt".into_text().color(Color::GREEN).bold() +
            " Minecraft server".into_text()
        );
        client.send_chat_message(
            "Type ".into_text() +
            "/login <password>".into_text().color(Color::YELLOW).bold().not_italic() +
            " to login".into_text()
        );
        client.set_title("LOG IN");

        permissions.add("everyone");
        permissions.add("notloged");
    }
}
