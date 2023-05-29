use lazy_static::lazy_static;
use regex::Regex;
use std::vec::Vec;

#[derive(Debug)]
pub struct Callsign {
    pub homecall: String,
    pub prefix: Option<String>,
    pub suffix: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CallsignError {
    ForbiddenCharacter,
    NoHomeCall,
    MultipleHomeCalls,
    MultiplePrefixes,
}

/// Analyze callsign and split it into its parts.
/// 
/// Pattern: `{prefix}/{homecall}/{suffix1}/{suffixN}`
/// 
/// # Arguments
/// 
/// `call`: Callsign to analyze. It is assumed, that the characters are all in uppercase.
/// 
/// # Returns
/// 
/// Returns the analyzed callsign separated into its parts.
/// If the callsign is invalid, an error stating the errors nature is returned.
pub fn analyze_callsign(call: &str) -> Result<Callsign, CallsignError> {
    lazy_static! {
        static ref RE_CALL: Regex = Regex::new(r"^[A-Z0-9]+[A-Z0-9/]*[A-Z0-9]+$").unwrap();
        static ref RE_HOME_CALL: Regex =
            Regex::new(r"((\d[A-Z])(\d+)([A-Z]+))|(([A-Z]+)(\d+)([A-Z]+))").unwrap();
        static ref RE_SUFFIX: Regex = Regex::new(r"/[A-Z0-9]+").unwrap();
    }

    // Check that only allowed characters are present
    if !RE_CALL.is_match(call) {
        return Err(CallsignError::ForbiddenCharacter);
    }

    // Search for home callsigns (= full callsigns)
    let mut calls = RE_HOME_CALL.find_iter(call);

    // Check if at least one home callsign was found
    if let Some(first_match) = calls.next() {
        // Check for multiple home callsigns
        if calls.count() != 0 {
            return Err(CallsignError::MultipleHomeCalls);
        }

        // Extract raw prefix
        let prefix_raw = &call[0..first_match.start()];

        // Check for multiple prefixes
        if prefix_raw.matches("/").count() > 1 {
            return Err(CallsignError::MultiplePrefixes);
        }

        // Get prefix (ignore trailing /)
        let prefix = match prefix_raw.len() {
            0 => None,
            len => Some(String::from(&prefix_raw[0..len - 1])),
        };

        // Split suffixes (ignore leading /)
        let suffixes: Vec<String> = RE_SUFFIX
            .find_iter(&call[first_match.end()..])
            .map(|suffix| String::from(&(suffix.as_str())[1..]))
            .collect();

        // Fill result struct
        Ok(Callsign {
            homecall: String::from(first_match.as_str()),
            prefix: prefix,
            suffix: suffixes,
        })
    } else {
        Err(CallsignError::NoHomeCall)
    }
}
