use std::collections::HashMap;

use crate::models::match_history_model::MatchHistory;
use crate::models::profile_model::Profile;
use crate::settings::Settings;

use ureq::serde_json;

// (champion_name: win, loses)
fn get_highest(map: &HashMap<String, (i64, i64)>) -> (String, i64, i64) {
    let mut result: (String, i64, i64) = ("".into(), -1, -1);

    for (entry, value) in map {
        if result.1 == -1 {
            result.0 = entry.clone();
            result.1 = value.0;
            result.2 = value.1;
            continue;
        }
        if value.0 + value.1 > result.1 + result.2 {
            result.0 = entry.clone();
            result.1 = value.0;
            result.2 = value.1;
        }
    }

    result
}

pub fn get_most_played_champs(data: &Vec<MatchHistory>) -> Vec<(String, String, String)> {
    // (champion_name, number of samples, winrate)
    let mut champions: Vec<(String, String, String)> = Vec::new();

    // (champion_name: wins, loses)
    let mut tmp: HashMap<String, (i64, i64)> = HashMap::new();

    for element in data {
        let value: &(i64, i64) = tmp
            .get(&element.player_champion.champion_name)
            .get_or_insert(&(0, 0));
        let mut result: (i64, i64) = *value;
        if element.player_champion.win {
            result.0 = result.0 + 1;
        } else {
            result.1 = result.1 + 1;
        }
        tmp.insert(element.player_champion.champion_name.clone(), result);
    }

    let mut i = 3;

    while i > 0 && tmp.len() > 0 {
        let highest = get_highest(&tmp);
        let winrate = (((highest.1 as f32) / ((highest.1 + highest.2) as f32)) * 100.0) as u32;
        champions.push((
            highest.0.clone(),
            (highest.1 + highest.2).to_string(),
            winrate.to_string(),
        ));
        tmp.remove(&highest.0);
        i -= 1;
    }

    champions
}

pub fn get_latest_icon(data: &Vec<MatchHistory>) -> String {
    let mut icon: String = "4603".into();

    if data.len() > 0 {
        icon = data
            .get(0)
            .unwrap()
            .player_champion
            .profile_icon
            .to_string();
    }

    icon
}

pub fn get_player_profile(
    settings: &Settings,
    summoner_name: &String,
) -> Result<Profile, ureq::Error> {
    let mut profile = Profile::default();

    let request = format!(
        "https://euw1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{summoner_name}",
        summoner_name = summoner_name
    );

    let response: serde_json::Value = ureq::get(&request)
        .set("X-Riot-Token", &settings.riot_api)
        .call()?
        .into_json()?;

    if response.is_object() {
        let map = response.as_object().unwrap();

        if map.contains_key("name") && map.contains_key("puuid") {
            let name = map.get("name").unwrap();
            let puuid = map.get("puuid").unwrap();

            if name.is_string() && puuid.is_string() {
                profile.name = name.as_str().unwrap().into();
                profile.puuid = puuid.as_str().unwrap().into();
            }
        }
    }

    Ok(profile)
}

pub fn get_champ_title(champ: &String) -> Result<String, ureq::Error> {
    let mut title: String = "Default title".into();

    let request =
        format!("http://ddragon.leagueoflegends.com/cdn/12.12.1/data/en_US/champion.json");

    let response: serde_json::Value = ureq::get(&request).call()?.into_json()?;

    if response.as_object().is_some() {
        let map = response.as_object().unwrap();

        if map.contains_key("data") {
            let data = map.get("data").unwrap();

            if data.is_object() {
                let data = data.as_object().unwrap();
                if data.contains_key(champ) {
                    let data = data.get(champ).unwrap();
                    if data.is_object() && data.as_object().unwrap().contains_key("title") {
                        let data = data.as_object().unwrap().get("title").unwrap();
                        if data.is_string() {
                            title = data.as_str().unwrap().to_string();
                        }
                    }
                }
            }
        }
    }
    Ok(title)
}

fn get_spell(id: usize, spells: &Vec<ureq::serde_json::Value>) -> String {
    let mut spell: String = "".into();

    if spells.len() > id {
        let tmp = spells.get(id).unwrap();

        if tmp.is_object() && tmp.as_object().unwrap().contains_key("id") {
            let tmp = tmp.as_object().unwrap().get("id").unwrap();

            if tmp.is_string() {
                spell = tmp.as_str().unwrap().to_string();
            }
        }
    }

    spell
}

// HashMap = Spell name {Q, W, E, R}, spell image name {*.png}
pub fn get_champ_spells(champ: &String) -> Result<HashMap<String, String>, ureq::Error> {
    let mut spells: HashMap<String, String> = HashMap::from([
        ("Q".into(), "ZoeQ".into()),
        ("W".into(), "ZoeW".into()),
        ("E".into(), "ZoeE".into()),
        ("R".into(), "ZoeR".into()),
    ]);

    let request =
        format!("http://ddragon.leagueoflegends.com/cdn/12.12.1/data/en_US/championFull.json");

    let response: serde_json::Value = ureq::get(&request).call()?.into_json()?;

    if response.as_object().is_some() {
        let map = response.as_object().unwrap();

        if map.contains_key("data") {
            let data = map.get("data").unwrap();

            if data.is_object() {
                let data = data.as_object().unwrap();
                if data.contains_key(champ) {
                    let data = data.get(champ).unwrap();
                    if data.is_object() && data.as_object().unwrap().contains_key("spells") {
                        let data = data.as_object().unwrap().get("spells").unwrap();
                        if data.is_array() {
                            let data = data.as_array().unwrap();
                            spells.insert("Q".into(), get_spell(0, &data));
                            spells.insert("W".into(), get_spell(1, &data));
                            spells.insert("E".into(), get_spell(2, &data));
                            spells.insert("R".into(), get_spell(3, &data));
                        }
                    }
                }
            }
        }
    }
    Ok(spells)
}
