use std::env;
use std::io;
use std::io::Write;

use valence::*;
use valence::prelude::*;
// use valence::anvil::*;

use crate::login::LoginEvent;
// use std::path::PathBuf;
use valence::anvil::parsing::*;


#[derive(Debug, Resource)]
pub struct EventCtrResource {
    pub current_event: Option<u32>,
    pub event_table: Vecl
}
impl EventCtrResource {
    pub fn new() -> EventCtrResource{
        EventCtrResource{layer_id: None}
    }
}

pub struct SpawnPlugin;
impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        let login_asset = EventCtrResource::new();
        app
            .insert_resource(login_asset)
            // .add_systems(Startup, setup)
            // .add_systems(Update, handle_login_event);
            // .add_systems(Update, handle_chunk_loads);
    }
}
