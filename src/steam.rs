use std::{
    collections::{HashMap, HashSet},
    env,
};

use serde::Deserialize;

#[derive(Default)]
pub struct SteamApi {
    ids: HashMap<u64, String>,
    api_key: String,
}

#[derive(Deserialize, Debug)]
pub struct Player {
    steamid: String,
    personaname: String,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    players: Vec<Player>,
}

#[derive(Deserialize, Debug)]
pub struct SteamApiResponse {
    response: Response,
}

impl SteamApi {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            ..Default::default()
        }
    }
    pub fn add_ids(&mut self, id_list: &[u64]) {
        for chunk in id_list.chunks(100) {
            let list = chunk
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(",");
            let url = format!(
                "http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
                &self.api_key, list
            );
            let Ok(mut response) = ureq::get(&url).call() else {
                //TODO let's figure out how to handle that later
                continue;
            };
            let Ok(response) = response.body_mut().read_json::<SteamApiResponse>() else {
                //TODO let's figure out how to handle that later
                continue;
            };
            let players = response.response.players;
            let mut recv_ids = HashSet::new();
            for player in players {
                if let Ok(id) = player.steamid.parse::<u64>() {
                    self.ids.insert(id, player.personaname);
                    recv_ids.insert(id);
                }
            }

            // Have to do this since the order of returned players isn't static
            let a: HashSet<u64> = HashSet::from_iter(chunk.iter().cloned());
            let difference = a.difference(&recv_ids).collect::<Vec<&u64>>();
            for diff in difference {
                self.ids.insert(*diff, String::from("Unknown/Spoofed"));
            }

            //TODO not urgent but some throttling measure here/in the struct?
        }
    }

    pub fn get(&self, id: u64) -> &str {
        if let Some(player) = self.ids.get(&id) {
            player
        } else {
            //TODO let's figure out how to handle that later
            "Unknown/Unparsed"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_api() {
        let api_key = env::var("APIKEY").unwrap(); //TODO changeme
        let mut s = SteamApi::new(api_key);
        s.add_ids(&[76561197960435530, 0101, 76561198001860904]);
    }
}
