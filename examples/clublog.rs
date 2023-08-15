use std::fs;
use std::env;
use callsign::clublog;


pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage `call CLUBLOGXML PREFIX`");
    } else {
        let raw = fs::read_to_string(&args[1]).unwrap();
        let clublog = clublog::ClubLog::parse(&raw).unwrap();

        let info = clublog.lookup_prefix(&args[2]).unwrap();
        println!("CallInfo for {}:\n{:?}", args[2], info);
    }
}
