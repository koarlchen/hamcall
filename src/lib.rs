use lazy_static::lazy_static;
use regex::Regex;
use std::vec::Vec;
use thiserror::Error;

/// Callsign
#[derive(Debug)]
pub struct Callsign {
    // Prefix of home callsign
    pub prefix: String,
    // Number of home callsign
    pub number: u16,
    // Suffix of home callsing
    pub suffix: String,
    // Additional prefix
    pub add_prefix: Option<String>,
    // Additional suffixes
    pub add_suffix: Vec<String>,
}

impl Callsign {
    // Get complete callsign
    pub fn call(&self) -> String {
        let str_prefix = if self.add_prefix.is_some() {
            format!("{}/", self.prefix)
        } else {
            String::new()
        };

        let str_suffix = if self.add_suffix.len() > 0 {
            format!("/{}", self.add_suffix.join("/"))
        } else {
            String::new()
        };

        format!("{}{}{}", str_prefix, self.number, str_suffix)
    }

    // Get home callsign
    pub fn homecall(&self) -> String {
        format!("{}{}{}", self.prefix, self.number, self.suffix)
    }
}

/// Possible reasons for an invalid callsign
#[derive(Error, Debug, PartialEq, Eq)]
pub enum CallsignError {
    #[error("Callsign is of invalid format or include invalid characters")]
    InvalidFormat,

    #[error("Failed to find a home callsign")]
    NoHomeCall,

    #[error("Found multiple home callsigns")]
    MultipleHomeCalls,

    #[error("Found multiple additional prefixes")]
    MultipleAdditionalPrefixes,
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
            Regex::new(r"((\d[A-Z])(\d{1,4})([A-Z]+))|(([A-Z]+)(\d{1,4})([A-Z]+))").unwrap();
        static ref RE_SUFFIX: Regex = Regex::new(r"/[A-Z0-9]+").unwrap();
    }

    // Check that only allowed characters are present and the callsign does not begin or end with a /
    if !RE_CALL.is_match(call) {
        return Err(CallsignError::InvalidFormat);
    }

    // Search for home callsigns (= complete callsigns)
    let mut calls = RE_HOME_CALL.captures_iter(call);

    // Check if at least one home callsign is present
    if let Some(first_match) = calls.next() {
        // Check for multiple home callsigns
        if calls.count() != 0 {
            return Err(CallsignError::MultipleHomeCalls);
        }

        // Get regex groups offsets
        // Required to check which of both major regex groups to access later on
        let (group_offset, homecall) = if first_match.get(1).is_some() {
            (1, first_match.get(1).unwrap().as_str())
        } else {
            (5, first_match.get(5).unwrap().as_str())
        };

        // Extract homecalls prefix, number and suffix (unwraps are safe due to pre-calculated offset and used regex)
        let prefix = first_match.get(group_offset + 1).unwrap().as_str().into();
        let number = first_match
            .get(group_offset + 2)
            .unwrap()
            .as_str()
            .parse::<u16>()
            .unwrap();
        let suffix = first_match.get(group_offset + 3).unwrap().as_str().into();

        // Get offset of homecall within complete callsign (unwrap is safe since homecall is part of complete callsign)
        let call_offset = call.find(homecall).unwrap();

        // Extract raw additional prefix
        let add_prefix_raw = &call[0..call_offset];

        // Check for multiple additional prefixes
        if add_prefix_raw.matches("/").count() > 1 {
            return Err(CallsignError::MultipleAdditionalPrefixes);
        }

        // Get prefix (ignore trailing /)
        let add_prefix = match add_prefix_raw.len() {
            0 => None,
            len => Some(String::from(&add_prefix_raw[0..len - 1])),
        };

        // Split suffixes (ignore leading /)
        let add_suffixes: Vec<String> = RE_SUFFIX
            .find_iter(&call[call_offset + homecall.len()..])
            .map(|suffix| String::from(&(suffix.as_str())[1..]))
            .collect();

        // Fill result struct
        Ok(Callsign {
            prefix,
            number,
            suffix,
            add_prefix,
            add_suffix: add_suffixes,
        })
    } else {
        Err(CallsignError::NoHomeCall)
    }
}
