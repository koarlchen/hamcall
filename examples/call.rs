use std::env;

// Usage `call CALLSIGN`
pub fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage `call CALLSIGN`");
    } else {
        let call = &args[1];
        match callsign::analyze_callsign(call) {
            Ok(c) => println!("{} => {:?}", call, c),
            Err(e) => eprintln!("{} => {:?}", call, e),
        }
    }
}
