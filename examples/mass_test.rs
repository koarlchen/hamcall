use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use callsign::call;

// Usage `call FNAME`
pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage `call FNAME`");
    } else {
        let fname = &args[1];
        let file = File::open(fname).unwrap();
        let calls = BufReader::new(file).lines();

        for call in calls {
            let call = call.unwrap();
            match call::analyze_callsign(&call) {
                Ok(c) => println!("{} => {:?}", call, c),
                Err(e) if e == call::CallsignError::InternalError => {
                    panic!("Internal error occurred for '{}'", call);
                }
                Err(e) => eprintln!("{} => {:?}", call, e),
            }
        }
    }
}
