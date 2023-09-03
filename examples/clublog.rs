use callsign::clublog;
use chrono::Utc;
use std::env;
use std::fs;

pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage `call CLUBLOGXML PREFIX`");
    } else {
        let raw = fs::read_to_string(&args[1]).unwrap();
        let clublog = clublog::ClubLog::parse(&raw).unwrap();

        println!("Query information for prefix '{}'", args[2]);

        let info = clublog.get_prefix(&args[2], &Utc::now().into()).unwrap();
        println!("Prefix information:\n{:?}", info);

        if let Some(adif) = info.adif {
            let entity = clublog.get_entity(adif, &Utc::now().into());
            println!("Entity information:\n{:?}", entity);
        }
    }
}
