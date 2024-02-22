#![allow(clippy::type_complexity)]
use valence::*; use valence::client::Client;
use valence::prelude::*;
use command::handler::CommandResultEvent;
use command_macros::Command;

use std::collections::hash_map::HashMap;
use std::fs::read_to_string;
use std::env;
pub struct LoginInfo {
    map: HashMap<String, String>
}
impl LoginInfo {
    pub fn serialize_from_file () -> std::io::Result<LoginInfo>{
        let login_file = env::var("LOGIN_FILE").unwrap_or("logins".to_string());
        let mut map = HashMap::<String, String>::new();
        for line in read_to_string(login_file).unwrap_or("admin admin".to_string()).lines() {
            let mut itr = line.split(" ");
            let login = itr.next().unwrap_or("none");
            let password = itr.next().unwrap_or("none");
            let _ = map.insert(login.to_string(), password.to_string());
            println!("login: {} password: {}", login, password);
        }
        Ok(LoginInfo{map})
         
    }
    pub fn verify(self: &LoginInfo,login: String, passwod: String) -> bool {
        if let Some(password_map) = self.map.get(&login) {
            if *password_map == passwod { return true; }
        }
        false
    }
}

#[derive(Command, Debug, Clone)]
#[paths("login {login} {password}")]
#[scopes("all.login")]
pub struct LoginCommand {
    login: String, 
    password: String
}

pub fn handle_test_command(
    mut events: EventReader<CommandResultEvent<LoginCommand>>,
    mut clients: Query<&mut Client>,
    // mut logins: Query<&LoginInfo>,
) {
    for event in events.read() {
        let client = &mut clients.get_mut(event.executor).unwrap();
        println!(
            "Login command executed"
            );
        client.send_chat_message(format!(
            "Login command executed"
        ));
    }
}
