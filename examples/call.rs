use chrono::Utc;
use std::env;
use std::fs;

// Usage `call CALLSIGN`
pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage `call CLUBLOGXML CALLSIGN`");
    } else {
        let file = &args[1];
        let call = &args[2];

        let raw = fs::read_to_string(file).unwrap();
        let clublog = hamcall::clublog::ClubLog::parse(&raw).unwrap();

        match hamcall::call::analyze_callsign(&clublog, call, &Utc::now()) {
            Ok(c) => println!("{} => {:?}", call, c),
            Err(e) => eprintln!("{} => {:?}", call, e),
        }
    }
}
