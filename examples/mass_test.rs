use chrono::{DateTime, Utc};
use hamcall::{call, clublog};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

// Usage `call CLUBLOGXML FNAME`
pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage `call CLUBLOGXML FNAME`");
    } else {
        let xml = &args[1];
        let fname = &args[2];

        let raw = fs::read_to_string(xml).unwrap();
        let clublog = clublog::ClubLog::parse(&raw).unwrap();

        let csv = read_csv(fname);

        for entry in csv {
            match call::analyze_callsign(&clublog, &entry.0, &entry.2) {
                Ok(c) => {
                    if entry.1 != c.adif {
                        let entity_theirs = &clublog
                            .get_entity(entry.1, &entry.2)
                            .map_or("", |e| &e.name);
                        let entity_mine =
                            &clublog.get_entity(c.adif, &entry.2).map_or("", |e| &e.name);
                        eprintln!(
                            "{} => ADIF mismatch (theirs={} ({:?}) != mine={} ({:?}))",
                            entry.0, entry.1, entity_theirs, c.adif, entity_mine
                        );
                        continue;
                    }
                    if !call::check_whitelist(&clublog, &entry.0, c.adif, &entry.2) {
                        eprintln!(
                            "{} => Callsign matches to entity {} but is not whitelisted",
                            &entry.0,
                            c.dxcc.unwrap()
                        );
                        continue;
                    }
                    println!("{} => {:?}", entry.0, c);
                }
                Err(e) => eprintln!("{} => {:?}", entry.0, e),
            }
        }
    }
}

/// Read csv file with test data.
///
/// The csv file is assumed to have the following columns where the column names refer to ADIF fields
/// <CALL>,<ADIF>,<QSO_DATE>,<TIME_ON>
fn read_csv(fname: &str) -> Vec<(String, clublog::Adif, DateTime<Utc>)> {
    let mut result: Vec<(String, clublog::Adif, DateTime<Utc>)> = Vec::new();

    let file = File::open(fname).unwrap();
    let lines = BufReader::new(file).lines();

    for line in lines {
        let line = line.unwrap();

        let splits: Vec<&str> = line.split(',').collect();

        let call = String::from(splits[0]);
        let adif = splits[1].parse::<clublog::Adif>().unwrap();
        let timestamp: DateTime<Utc> = DateTime::parse_from_str(
            &format!("{} {} +0000", splits[2], splits[3]),
            "%Y%m%d %H%M %z",
        )
        .unwrap()
        .into();

        result.push((call, adif, timestamp));
    }

    result
}
