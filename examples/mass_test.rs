use chrono::Utc;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use callsign::call;

// Usage `call FNAME`
pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage `call CLUBLOGXML FNAME`");
    } else {
        let xml = &args[1];
        let fname = &args[2];

        let raw = fs::read_to_string(xml).unwrap();
        let clublog = callsign::clublog::ClubLog::parse(&raw).unwrap();

        let file = File::open(fname).unwrap();
        let calls = BufReader::new(file).lines();

        for call in calls {
            let call = call.unwrap();
            match call::analyze_callsign(&clublog, &call, Utc::now()) {
                Ok(c) => println!("{} => {:?}", call, c),
                Err(e) if e == call::CallsignError::InternalError => {
                    panic!("Internal error occurred for '{}'", call);
                }
                Err(e) => eprintln!("{} => {:?}", call, e),
            }
        }
    }
}
