use callsign::call;
use callsign::clublog::Adif;
use chrono::{DateTime, Utc};
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
        let clublog = callsign::clublog::ClubLog::parse(&raw).unwrap();

        let csv = read_csv(fname);

        for entry in csv {
            match call::analyze_callsign(&clublog, &entry.0, entry.2) {
                Ok(c) => println!("{} => {:?}", entry.0, c),
                Err(e) => eprintln!("{} => {:?}", entry.0, e),
            }
        }
    }
}

/// Read csv file with test data.
///
/// The csv file is assumed to have the following column names where each column name contains the data of the named ADIF field.
/// <CALL>,<ADIF>,<QSO_DATE>,<TIME_ON>
fn read_csv(fname: &str) -> Vec<(String, Adif, DateTime<Utc>)> {
    let mut result: Vec<(String, Adif, DateTime<Utc>)> = Vec::new();

    let file = File::open(fname).unwrap();
    let lines = BufReader::new(file).lines();

    for line in lines {
        let line = line.unwrap();

        let splits: Vec<&str> = line.split(',').collect();

        let call = String::from(splits[0]);
        let adif = splits[1].parse::<Adif>().unwrap();
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
