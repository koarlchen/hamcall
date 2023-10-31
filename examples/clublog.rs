use chrono::Utc;
use hamcall::clublogquery::ClubLogQuery;
use std::env;
use std::fs;

/// Example on how to work with the parsed ClubLog XML data.
///
/// Usage: `clublog <CLUBLOGXML> <PREFIX>`
pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: `clublog <CLUBLOGXML> <PREFIX>`");
    } else {
        // Read contents of the ClubLog XML file
        let raw = fs::read_to_string(&args[1]).unwrap();
        // Parse the contents into an object
        let clublog = hamcall::clublog::ClubLog::parse(&raw).unwrap();

        println!("Query information for prefix '{}'", args[2]);

        // Query information for a prefix
        let info = clublog.get_prefix(&args[2], &Utc::now().into()).unwrap();
        println!("Prefix information:\n{:?}", info);

        // Query information for the entity of the prefix
        let entity = clublog.get_entity(info.adif, &Utc::now().into());
        println!("Entity information:\n{:?}", entity);
    }
}
