use chrono::{Date, DateTime, NaiveDate};
use chrono::{NaiveDateTime, TimeDelta, Duration};


use std::collections::HashMap;
use std::{env,fs,cmp};
use serde_json::Value;

#[derive(Debug)]
enum JsonValue {
    String(String),
    Number(i64),
    List(Vec<i64>),
    DateTime(NaiveDateTime)
}


pub fn read_file<'a>() -> Vec<String> {
    println!("reading file");
    let path = "raidlog.txt";
    let contents = fs::read_to_string(path).expect("Failed to read the file");
    let log_in_lines = contents.split("\n");
    let log_vectored: Vec<&str> = log_in_lines.collect();
    let mut new_vector: Vec<String> = Vec::new();
    for item in log_vectored {
        let item_string = item.to_string();
        new_vector.push(item_string);
    }
    new_vector
}

fn extract_data_from_line (line_: &String) -> (String, String, String)  {
    //splitting date, time and name of player
    let line_split = line_.split(" ");
    let line_split: Vec<&str> = line_split.collect();
    let date = line_split.get(0);
    let time = line_split.get(1);
    let player_name = line_split.get(3);
    //converting the options to str
    let date = match date {
        Some(x) => x,
        None => {
            println!("Error on line, cannot parse date:");
            println!("{}", line_);
            ""
        },
    };
    let time = match time {
        Some(x) => x,
        None => {
            println!("Error on line, cannot parse time:");
            println!("{}", line_);
            ""
        },
    };
    let player_name = match player_name {
        Some(x) => x,
        None => {
            println!("Error on line, cannot parse player name:");
            println!("{}", line_);
            ""
        },
    };
    (date.to_string(), time.to_string(), player_name.to_string())
}

fn parse_time (date:&String, time:&String) -> Result<NaiveDateTime, chrono::ParseError> {
    //unioning date and time to parse into Datetime Struct
    let mut datetime = String::from("2025/");
    datetime.push_str(&date);
    datetime.push('-');
    datetime.push_str(&time);
    datetime.truncate(datetime.len().saturating_sub(4));
    let datetime = datetime.as_str();
    let datetime: Result<NaiveDateTime, chrono::ParseError> = NaiveDateTime::parse_from_str(datetime, "%Y/%m/%d-%H:%M:%S");
    datetime
}



/// converts the log to players hash : {"Player1"}
pub fn log_to_hash(log_vectored: Vec<String>, player_list: Vec<String>, afk_timer:i64) -> HashMap<String, HashMap<String, JsonValue>> {
    let mut players_hash: HashMap<String, HashMap<String, JsonValue>> = HashMap::new();
    
    for line_ in log_vectored {
        //parse and extract date, time and player name 
        let (date, time, player_name) = extract_data_from_line(&line_);
        if date.len() < 2 || player_name.len() < 2 || time.len() < 2 {
            continue
        };
        let datetime = parse_time(&date, &time);
        let datetime = match datetime {
            Ok(x) => x,
            Err(_) => {
                println!("error on datetime on line:");
                println!("{}", line_);
                continue
            }
        };

        
        // start pushing data into HashMap
        if player_list.contains(&player_name) {
            println!("Date: {}, time: {}, name:{}", date, time, player_name);
            let individual_player_hash = players_hash.get_mut(&player_name);
            // This match of option is also covering the insert of new player in hashmap
            match individual_player_hash {
                Some(extracted_player_hash) => {
                    let last_seen = extracted_player_hash.get("end_time").expect("Error grabbing last end time");
                    let last_seen = match *last_seen {
                        JsonValue::DateTime(x) => x,
                        _ => panic!(),
                    };

                    let afk_time = datetime - last_seen;
                    if afk_time.num_seconds() > afk_timer {
                        let afk_list = extracted_player_hash.get("afk_list").expect("failed to load afk list");
                        match afk_list {
                            JsonValue::List(x) => {
                                let mut new_vec = x.clone();
                                new_vec.push(afk_time.num_seconds());
                                extracted_player_hash.insert(String::from("afk_list"), JsonValue::List(new_vec));
                            },
                            _ => panic!(),
                        };  
                    };
                    extracted_player_hash.insert(String::from("end_time"), JsonValue::DateTime(datetime));
                },
                None => {
                    let mut new_player_hash: HashMap<String, JsonValue> = HashMap::new();
                    
                    new_player_hash.insert(String::from("start_time"), JsonValue::DateTime(datetime));
                    new_player_hash.insert(String::from("end_time"), JsonValue::DateTime(datetime));
                    let afk_list:Vec<i64> = Vec::new();
                    new_player_hash.insert(String::from("afk_list"), JsonValue::List(afk_list));
                    
                    
                    players_hash.insert(player_name, new_player_hash);
                }
            }
            
        } else {
            continue
        };
    }
    players_hash
}

///adding additional numbers to player json
fn transform_players_hashes(mut players_hash:HashMap<String, HashMap<String, JsonValue>>) -> (i64, HashMap<String, HashMap<String, JsonValue>>) {
    //this will be max duration, basically duration of raid
    // will be used mostly for checks on small raids
    let mut max_duration:i64 = 0;
    for (_, mut player_values) in players_hash.iter_mut() {
        // extract data
        let start_time = player_values.get("start_time").expect("error extracting start time for player");
        let start_time = match  start_time{
            JsonValue::DateTime(x) => x,
            _ => panic!(),            
        };
        let end_time = player_values.get("end_time").expect("error extracting end time for player");
        let end_time = match  end_time { 
            JsonValue::DateTime(x) => x,
            _ => panic!(),            
        };
        

        // calculate time played and insert that into json
        let time_played = (*end_time - *start_time).num_seconds();
        if time_played > max_duration {
            max_duration = time_played;
        }
        player_values.insert(String::from("time_played"), JsonValue::Number(time_played));

        // calculate total time spent afk and insert that into json
        let player_afk_list = player_values.get("afk_list").expect("failed to load afk list");
        let player_afk_list = match player_afk_list {
            JsonValue::List(x) => x,
            _ => panic!("failed to load afk list"),
        };
        let mut total_afk_time = 0;
        for afk_time in player_afk_list {
            total_afk_time += afk_time;
        };
        let times_went_afk = player_afk_list.len() as i64;
        player_values.insert(String::from("afk_time"), JsonValue::Number(total_afk_time));
        player_values.insert(String::from("times_went_afk"), JsonValue::Number(times_went_afk));
    };   
    (max_duration, players_hash)

}

fn player_atendance_list(players_hash:HashMap<String, HashMap<String, JsonValue>>, required_time:i32) -> Vec<String> {
    let mut player_attendance_list:Vec<String> = Vec::new();
    for (player_name, player_values) in players_hash.iter() {
        let player_played_time = match player_values.get("time_played").expect("failed unpacking play time") {
            JsonValue::Number(x) => x,
            _ => panic!("failed unpacking play time"),
        };
        if player_played_time >= 
        
    }
    player_attendance_list
}

fn create_warnings(players_hash:HashMap<String, HashMap<String, JsonValue>>){
    let mut warning_list:Vec<String> = Vec::new();
}


fn main() {
    let afk_timer = 15*60 as i64;
    let required_raid_time = 90 * 60 as i64;
    let player_list = vec![String::from("Freddobene"), String::from("Sporedni"), String::from("Djurdjica"), String::from("Sygga"), String::from("Meacvlpa")];


    let log_vectored = read_file();
    let mut players_hash: HashMap<String, HashMap<String, JsonValue>> = log_to_hash(log_vectored, player_list, afk_timer);
    let (max_duration, players_hash) = transform_players_hashes(players_hash);
    
    let required_raid_time = cmp::min(max_duration,required_raid_time);

    dbg!("{:?}",players_hash);
    
    
}
