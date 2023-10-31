use chrono::Utc;
use std::env;
use std::fs;

/// Example on how to analyze a callsign.
///
/// Usage: `call <CALLSIGN>`
pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: `call <CLUBLOGXML> <CALLSIGN>`");
    } else {
        let file = &args[1];
        let call = &args[2];

        // Read contents of the ClubLog XML file
        let raw = fs::read_to_string(file).unwrap();
        // Parse the contents into an object
        let clublog = hamcall::clublog::ClubLog::parse(&raw).unwrap();
        // Convert the object for faster access times
        let clublogmap = hamcall::clublogmap::ClubLogMap::from(clublog);

        // Timestamp used together with the call for analyzer (e.g. some entities are only valid for a certain time)
        let timestamp = Utc::now();

        // Analyze the call to geht the entity, the ADIF identifier and a few more things
        match hamcall::call::analyze_callsign(&clublogmap, call, &timestamp) {
            Ok(c) => {
                // Check if the entity is whitelisted and if so, if the callsign is part of the whitelist
                if hamcall::call::check_whitelist(&clublogmap, &c, &timestamp) {
                    println!("{} => {:?}", call, c)
                } else {
                    println!(
                        "Callsign matches to entity {} but is not whitelisted",
                        c.dxcc.unwrap()
                    )
                }
            }
            Err(e) => eprintln!("{} => {:?}", call, e),
        }
    }
}
