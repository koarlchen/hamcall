// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Analyzer for callsigns based on the data from the ClubLog XML to get further information like the callsigns entity.
//!
//! The example `call.rs` shows the basic usage of this module.

use crate::clublog::{Adif, CallsignException, CqZone, Prefix, ADIF_ID_NO_DXCC};
use crate::clublogquery::ClubLogQuery;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

/// Representation of a callsign together with detailed information like the name of the entity or the ADIF DXCC identifier.
#[derive(Debug, PartialEq)]
pub struct Callsign {
    /// Complete callsign
    pub call: String,
    /// ADIF DXCC identifier
    pub adif: Adif,
    /// Name of entity
    pub dxcc: Option<String>,
    /// CQ zone
    pub cqzone: Option<CqZone>,
    /// Continent
    pub continent: Option<String>,
    /// Longitude
    pub longitude: Option<f32>,
    /// Latitude
    pub latitude: Option<f32>,
}

impl Callsign {
    /// Check if callsign is assigned to no DXCC (like for /AM, /MM or /SAT)
    ///
    /// # Arguments
    ///
    /// (None)
    ///
    /// # Returns
    ///
    /// True if the callsign is assigned to no DXCC
    pub fn is_special_entity(&self) -> bool {
        self.adif == ADIF_ID_NO_DXCC
    }

    /// Instantiate a new maritime mobile callsign
    ///
    /// # Arguments
    ///
    /// - `call`: Callsign
    ///
    /// # Returns
    ///
    /// Callsign struct
    fn new_maritime_mobile(call: &str) -> Callsign {
        Callsign {
            call: String::from(call),
            adif: ADIF_ID_NO_DXCC,
            dxcc: None,
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
        }
    }

    /// Instantiate a new aeronautical mobile callsign
    ///
    /// # Arguments
    ///
    /// - `call`: Callsign
    ///
    /// # Returns
    ///
    /// Callsign struct
    fn new_aeronautical_mobile(call: &str) -> Callsign {
        Callsign {
            call: String::from(call),
            adif: ADIF_ID_NO_DXCC,
            dxcc: None,
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
        }
    }

    /// Instantiate a new satellite callsign
    ///
    /// # Arguments
    ///
    /// - `call`: Callsign
    ///
    /// # Returns
    ///
    /// Callsign struct
    fn new_satellite(call: &str) -> Callsign {
        Callsign {
            call: String::from(call),
            adif: ADIF_ID_NO_DXCC,
            dxcc: None,
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
        }
    }

    /// Instantiate a new callsign from a ClubLog prefix
    ///
    /// # Arguments
    ///
    /// - `call`: Callsign
    /// - `prefix`: Callsign exception entry
    ///
    /// # Returns
    ///
    /// Callsign struct
    fn from_prefix(call: &str, prefix: &Prefix) -> Callsign {
        Callsign {
            call: String::from(call),
            adif: prefix.adif,
            dxcc: Some(prefix.entity.clone()),
            cqzone: prefix.cqz,
            continent: prefix.cont.clone(),
            longitude: prefix.long,
            latitude: prefix.lat,
        }
    }

    /// Instantiate a new callsign from a ClubLog callsign exception
    ///
    /// # Arguments
    ///
    /// - `call`: Callsign
    /// - `exc`: Callsign exception entry
    ///
    /// # Returns
    ///
    /// Callsign struct
    fn from_exception(call: &str, exc: &CallsignException) -> Callsign {
        Callsign {
            call: String::from(call),
            adif: exc.adif,
            dxcc: Some(exc.entity.clone()),
            cqzone: exc.cqz,
            continent: exc.cont.clone(),
            longitude: exc.long,
            latitude: exc.lat,
        }
    }
}

/// Possible reasons for an invalid callsign
#[derive(Error, Debug, PartialEq)]
pub enum CallsignError {
    /// Callsign is of invalid format or includes invalid characters
    #[error("Callsign is of invalid format or includes invalid characters")]
    BasicFormat,

    /// Callsign was used in an invalid operation
    #[error("Callsign was used in an invalid operation")]
    InvalidOperation,

    /// Callsign does not begin with a valid prefix
    #[error("Callsign does not begin with a valid prefix")]
    BeginWithoutPrefix,

    /// Too much prefixes
    #[error("Too much prefixes")]
    TooMuchPrefixes,

    /// Multiple special appendices like /MM, /AM or /6, /8, ...
    #[error("Multiple special appendices")]
    MultipleSpecialAppendices,
}

/// Special appendices that may not be interpreted as prefixes
const APPENDIX_SPECIAL: [&str; 7] = ["AM", "MM", "SAT", "P", "M", "QRP", "LH"];

/// Type of split
#[derive(PartialEq, Eq)]
enum PartType {
    /// Prefix
    Prefix,
    /// Everything other than a prefix
    Other,
}

/// State of the call element classification statemachine
#[derive(PartialEq, Eq)]
enum State {
    /// No prefix found so far
    NoPrefix,
    /// Single prefix
    SinglePrefix,
    /// Double prefix
    DoublePrefix,
    /// Found complete prefix, only appendices may follow
    PrefixComplete(u8),
}

/// Appendix that indicates that the calls entity may be ignored
#[derive(PartialEq, Eq, Clone)]
enum SpecialEntityAppendix {
    /// Maritime Mobile (/MM)
    Mm,
    /// Aeronautical Mobile (/AM)
    Am,
    /// Satellite, Internet or Repeater (/SAT)
    Sat,
}

/// Check if the callsign is whitelisted if the whitelist option is enabled for the entity of the callsign at the given point in time.
///
/// # Arguments
///
/// - `clublog`: Reference to ClubLog data
/// - `call`: Callsign to check
/// - `timestamp`: Timestamp to use for the check
///
/// # Returns
///
/// Returns true if the callsign is valid or false if whitelisting for that entity is enabled and the callsign is not on the whitelist.
pub fn check_whitelist(
    clublog: &dyn ClubLogQuery,
    call: &Callsign,
    timestamp: &DateTime<Utc>,
) -> bool {
    // Get entity for adif identifier
    // Note that not all valid adif identifiers refer to an entity (e.g. aeronautical mobile calls)
    if let Some(entity) = clublog.get_entity(call.adif, timestamp) {
        // Check if whitelisting is enabled
        if entity.whitelist == Some(true) {
            // Check if an exception for the call at the given point in time is present
            if let Some(prefix) = clublog.get_callsign_exception(&call.call, timestamp) {
                // There may be a callsign exception for a whitelisted entity but the exception refers a different adif identifier
                return prefix.adif == call.adif;
            }

            // Check if the given point in time is before the start of whitelisting for that entity
            if let Some(whitelist_start) = entity.whitelist_start {
                if *timestamp < whitelist_start {
                    return true;
                }
            }

            // Check if the given point in time is after the end of whitelisting for that entity
            if let Some(whitelist_end) = entity.whitelist_end {
                if *timestamp > whitelist_end {
                    return true;
                }
            }

            return false;
        }
    }

    true
}

/// Analyze callsign to get further information like the name of the entity or the AIDF DXCC identifier.
///
/// # Arguments:
///
/// - `clublog`: Reference to ClubLog data
/// - `call`: Callsign to analyze
/// - `timestamp`: Timestamp to use for the check
///
/// # Returns
///
/// Returns further information about the callsign or an error.
pub fn analyze_callsign(
    clublog: &dyn ClubLogQuery,
    call: &str,
    timestamp: &DateTime<Utc>,
) -> Result<Callsign, CallsignError> {
    // Strategy
    // Step 1: Check for an invalid operation
    // Step 2: Check for a callsign exception
    // Step 3: Classify each part of the callsign (split by '/') if it is a valid prefix or not
    // Step 4: Check for basic validity of the callsign by using the classification results and categorize the call into generic callsign structures
    // Step 5: Handle the call based on the determined category

    lazy_static! {
        static ref RE_COMPLETE_CALL: Regex = Regex::new(r"^[A-Z0-9]+[A-Z0-9/]*[A-Z0-9]+$").unwrap();
    }

    // Check that only allowed characters are present and the callsign does not begin or end with a /
    if !RE_COMPLETE_CALL.is_match(call) {
        return Err(CallsignError::BasicFormat);
    }

    // ### Step 1 ###
    // Check if the callsign was used in an invalid operation
    if clublog.is_invalid_operation(call, timestamp) {
        return Err(CallsignError::InvalidOperation);
    }

    // ### Step 2 ###
    // Check if clublog lists a callsign exception
    if let Some(call_exc) = clublog.get_callsign_exception(call, timestamp) {
        return Ok(Callsign::from_exception(call, call_exc));
    }

    // Split raw callsign into its parts
    let parts: Vec<&str> = call.split('/').collect();

    // ### Step 3 ###
    // Iterate through all parts of the callsign and check wether the part of the callsigns is a valid prefix or something else
    let mut parttypes: Vec<PartType> = Vec::with_capacity(parts.len());
    for (pos, part) in parts.iter().enumerate() {
        let pt = if get_prefix(clublog, part, timestamp, &parts[pos + 1..]).is_some() {
            // MM and AM may be valid prefixes or special appendices depending on the position within the complete callsign.
            // For example MM as a prefix evaluates to Scotland, MM as an appendix indicates a maritime mobile activation.
            // Special appendices are only valid as those if they are right at the beginning of the callsign.
            // Therefore ignore the first element of the call and check for special appendices beginning from the second element onwards.
            if pos >= 1 && APPENDIX_SPECIAL.contains(part) {
                PartType::Other
            } else {
                PartType::Prefix
            }
        } else {
            PartType::Other
        };
        parttypes.push(pt);
    }

    // ### Step 4 ###
    // Check for basic validity with a small statemachine.
    // For example check that the call begins with a prefix, has not too much prefixes, ...
    let mut state = State::NoPrefix;
    for parttype in parttypes.iter() {
        match (&state, parttype) {
            (State::NoPrefix, PartType::Prefix) => state = State::SinglePrefix,
            (State::NoPrefix, PartType::Other) => Err(CallsignError::BeginWithoutPrefix)?,
            (State::SinglePrefix, PartType::Prefix) => state = State::DoublePrefix,
            (State::SinglePrefix, PartType::Other) => state = State::PrefixComplete(1),
            (State::DoublePrefix, PartType::Prefix) => state = State::PrefixComplete(3),
            (State::DoublePrefix, PartType::Other) => state = State::PrefixComplete(2),
            (State::PrefixComplete(_), PartType::Prefix) => Err(CallsignError::TooMuchPrefixes)?,
            (State::PrefixComplete(_), PartType::Other) => (),
        }
    }

    // ### Step 5 ###
    match state {
        // The callsign consists of a single prefix and zero or more appendices
        State::SinglePrefix | State::PrefixComplete(1) => {
            // Complete homecall
            // Example: W1AW
            let homecall = &parts[0];

            // Prefix of the homecall
            // Example: W for the homecall W1AW
            // Unwrap is safe here, otherwise there is an internal error
            let mut homecall_prefix = get_prefix(clublog, homecall, timestamp, &parts[1..])
                .unwrap()
                .0;

            // Special appendix like /AM or /MM is present
            // Example: W1ABC/AM
            if let Some(appendix) = is_no_entity_by_appendix(&parts[1..])? {
                return Ok(match appendix {
                    SpecialEntityAppendix::Am => Callsign::new_aeronautical_mobile(call),
                    SpecialEntityAppendix::Mm => Callsign::new_maritime_mobile(call),
                    SpecialEntityAppendix::Sat => Callsign::new_satellite(call),
                });
            }

            // Check if a single digit appendix is present
            // If so, check if the single digit appendix changes the prefix to a different one
            // Example: "SV0ABC/9" where SV is Greece, but SV9 is Crete
            if let Some(pref) = is_different_prefix_by_single_digit_appendix(
                clublog,
                homecall,
                timestamp,
                &parts[1..],
            )? {
                homecall_prefix = pref;
            }

            // No special rule matched, just return information
            let mut callsign = Callsign::from_prefix(call, homecall_prefix);
            check_apply_cqzone_exception(clublog, &mut callsign, timestamp);
            Ok(callsign)
        }
        // The callsign consists of two prefixes and zero or more appendices
        State::DoublePrefix | State::PrefixComplete(2) => {
            // Get prefix information for both prefixes.
            let pref_first = get_prefix(clublog, parts[0], timestamp, &parts[1..]).unwrap();
            let pref_second = get_prefix(clublog, parts[1], timestamp, &parts[2..]).unwrap();

            // Check if the first prefix may be a valid special prefix like 3D2/R
            // Example: "3D2ABC/R" contains two valid prefixes at first sight, 3D2 and R but the first and second prefix together form the special prefix 3D2/R
            let pref = if pref_first.0.call.contains('/') {
                pref_first.0
            } else {
                // Decide which one to use by how many characters were removed from the potential prefix before it matched a prefix from the list.
                // The prefix which required less character removals wins.
                // This is probably not 100% correct, but seems good enough.
                if pref_first.1 <= pref_second.1 {
                    pref_first.0
                } else {
                    pref_second.0
                }
            };

            let mut callsign = Callsign::from_prefix(call, pref);
            check_apply_cqzone_exception(clublog, &mut callsign, timestamp);
            Ok(callsign)
        }
        // The callsign consists out of three prefixes and zero or more appendices
        // This is a very special case and only takes account of calls with a special prefix like 3D2/R and therefore callsigns like 3D2/W1ABC/R.
        // Calls like 3D2ABC/R are already covered, since there are only two potential valid prefixes.
        // The call 3D2/W1ABC/R contains three potential valid prefixes 3D2, W and R but 3D2/R is the actual prefix (according to my understanding of the special prefix annotation)
        State::PrefixComplete(3) => {
            let pref = get_prefix(clublog, parts[0], timestamp, &parts[1..]).unwrap();
            if pref.0.call.contains('/') {
                let mut callsign = Callsign::from_prefix(call, pref.0);
                check_apply_cqzone_exception(clublog, &mut callsign, timestamp);
                Ok(callsign)
            } else {
                Err(CallsignError::TooMuchPrefixes)
            }
        }
        _ => panic!("Internal error"),
    }
}

/// Check if a CQ zone exception exists based on the gathered callsign information.
/// If there is one, replace the CQ zone directly in the given callsign struct.
///
/// # Arguments
///
/// - `clublog`: Reference to ClubLog data
/// - `call`: Gathered callsign information
/// - `timestamp`: Timestamp to use for the check
///
/// # Returns
/// (None)
fn check_apply_cqzone_exception(
    clublog: &dyn ClubLogQuery,
    call: &mut Callsign,
    timestamp: &DateTime<Utc>,
) {
    if let Some(cqz) = clublog.get_zone_exception(&call.call, timestamp) {
        call.cqzone = Some(cqz);
    }
}

/// Check if the list of appendices contains an appendix with a single digit that may indicate a different prefix.
/// If there is such single digit appendix, replace the digit within the callsign and query the prefix information for the potential new prefix.
///
/// Example: "SV0ABC/9" where SV is Greece, but SV9 is Crete
///
/// # Arguments
///
/// - `clublog`: Reference to ClubLog data
/// - `homecall`: Part of the complete callsign that is assumend to be the homecall
/// - `timestamp`: Timestamp to use for the check
/// - `appendices`: List of appendices to the homecall
///
/// # Returns
///
/// A potential new prefix, `None` if nothing changed or an error.
fn is_different_prefix_by_single_digit_appendix<'a>(
    clublog: &'a dyn ClubLogQuery,
    homecall: &str,
    timestamp: &DateTime<Utc>,
    appendices: &[&str],
) -> Result<Option<&'a Prefix>, CallsignError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([A-Z0-9]+)(\d)([A-Z0-9]+)$").unwrap();
    }

    // Search for single digits in the list of appendices
    let single_digits: Vec<&&str> = appendices
        .iter()
        .filter(|e| {
            if e.len() == 1 {
                e.chars().next().unwrap().is_numeric()
            } else {
                false
            }
        })
        .collect();

    // Act based on how much single digit appendices were found
    let new_digit = match single_digits.len() {
        // Nothing to do if there is no single digit
        0 => return Ok(None),
        // If there is only a single digit, take it
        1 => single_digits[0],
        // For multiple single digits throw an error -> not sure which one to choose? Ignoring all would also be unexpected behaviour
        _ => return Err(CallsignError::MultipleSpecialAppendices),
    };

    // Assemble potential new intermediate call that will be used to check for a potential different prefix
    let new_homecall = RE.replace(homecall, format!("${{1}}{}${{3}}", new_digit));

    Ok(get_prefix(clublog, &new_homecall, timestamp, appendices).map(|i| i.0))
}

/// Check if a special appendix is part of the appendices list.
/// If such a speical appendix is present, it indicates that the actual prefix of the overall shall be ignored.
///
/// Example: /MM indicates maritime mobile and therefore does not reference an entity
///
/// # Arguments
///
/// - `appendices`: List of callsign appendices, like `QRP`, `5`, ...
///
/// # Returns
///
/// A potential special entity appendix or an error.
fn is_no_entity_by_appendix(
    appendices: &[&str],
) -> Result<Option<SpecialEntityAppendix>, CallsignError> {
    // Search for special appendices
    let a: Vec<SpecialEntityAppendix> = appendices
        .iter()
        .filter_map(|e| match *e {
            "MM" => Some(SpecialEntityAppendix::Mm),
            "AM" => Some(SpecialEntityAppendix::Am),
            "SAT" => Some(SpecialEntityAppendix::Sat),
            _ => None,
        })
        .collect();

    // Act based on how much special appendices were found
    match a.len() {
        // Zero found, nothing to do
        0 => Ok(None),
        // Single one found, return it
        1 => Ok(Some(a[0].clone())),
        // Multiple found, throw an error -> which one to choose?
        _ => Err(CallsignError::MultipleSpecialAppendices),
    }
}

/// Search for a matching prefix by brutforcing all possibilities.
/// The potential prefix will be shortened char by char from the back until a prefix matches.
/// Furthermore, to take in account of prefixes like SV/A, append all single char appendices to the end of the potential prefix before checking for a match.
///
/// # Arguments
///
/// - `clublog`: Reference to ClubLog data
/// - `potential_prefix`: Potential prefix to check against the data
/// - `timestamp`: Timestamp to use for the check
/// - `appendices`: List of callsign appendices, like `QRP`, `5`, ...
///
/// # Returns
///
/// If there is a match, next to the prefix information the number of removed chars is returned.
fn get_prefix<'a>(
    clublog: &'a dyn ClubLogQuery,
    potential_prefix: &str,
    timestamp: &DateTime<Utc>,
    appendices: &[&str],
) -> Option<(&'a Prefix, usize)> {
    let len_potential_prefix = potential_prefix.len();
    assert!(len_potential_prefix >= 1);

    // Search for single char appendices
    // For example SV/A is a valid prefix but indicates a different entity as the prefix SV
    let single_char_appendices: Vec<&&str> = appendices
        .iter()
        .filter(|e| {
            if e.len() == 1 {
                e.chars().next().unwrap().is_alphabetic()
            } else {
                false
            }
        })
        .collect();

    // Bruteforce all possibilities
    // Shortening the call from the back is required to due to calls like UA9ABC where both prefixes U and UA9 a potential matches,
    // but the more explicit one is the correct one.
    let mut prefix: Option<(&Prefix, usize)> = None;
    for cnt in (1..len_potential_prefix + 1).rev() {
        // Shortened call
        let slice = &potential_prefix[0..cnt];

        // Append all single chars to the call as <call>/<appendix> and check if the prefix is valid
        // This check is required for prefixes like SV/A where the callsign SV1ABC/A shall match too
        if let Some(pref) = single_char_appendices
            .iter()
            .find_map(|a| clublog.get_prefix(&format!("{}/{}", slice, a), timestamp))
        {
            prefix = Some((pref, len_potential_prefix - cnt));
            break;
        }

        // Check if prefix is valid
        if let Some(pref) = clublog.get_prefix(slice, timestamp) {
            prefix = Some((pref, len_potential_prefix - cnt));
            break;
        }
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{clublog::ClubLog, clublogmap::ClubLogMap};
    use lazy_static::lazy_static;
    use std::fs;

    fn read_clublog_xml() -> &'static dyn ClubLogQuery {
        lazy_static! {
            static ref CLUBLOG: ClubLogMap = ClubLogMap::from(
                ClubLog::parse(&fs::read_to_string("data/clublog/cty.xml").unwrap()).unwrap()
            );
        }

        &*CLUBLOG
    }

    #[test]
    fn clublog_prefix_entity_invalid() {
        let calls = vec!["X5ABC", "X5ABC/P", "X5/W1AW", "X5/W1AW/P"];

        let clublog = read_clublog_xml();
        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call,
                &DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            );
            assert_eq!(res, Err(CallsignError::BeginWithoutPrefix));
        }
    }

    #[test]
    fn clublog_special_appendix() {
        let calls = vec![
            ("KB5SIW/STS50", "2020-01-01T00:00:00Z"), // test for call exception record 2730
            ("ZY0RK", "1994-08-20T00:00:00Z"),        // test for callsign exception record 28169
        ];

        let clublog = read_clublog_xml();
        for call in calls.iter() {
            println!("Test for: {}", call.0);
            let res = analyze_callsign(
                clublog,
                call.0,
                &DateTime::parse_from_rfc3339(call.1).unwrap().into(),
            )
            .unwrap();
            assert!(res.is_special_entity());
        }
    }

    #[test]
    fn clublog_whitelist() {
        let params = vec![
            ("KH4AB", "1980-04-07T00:00:00Z", true), // Timestamp after start of whitelist and call is part of exception list
            ("KH4AB", "1981-01-01T00:00:00Z", false), // Timestamp after start of whitelist and call not part of exception list
        ];

        let clublog = read_clublog_xml();

        for param in params.iter() {
            println!("Test for: {}", param.0);
            let timestamp = &DateTime::parse_from_rfc3339(param.1).unwrap().into();
            let call = analyze_callsign(clublog, param.0, timestamp).unwrap();
            let res = check_whitelist(clublog, &call, timestamp);
            assert_eq!(param.2, res);
        }
    }

    #[test]
    fn special_appendix_am() {
        let calls = vec!["W1AW/AM", "W1AM/P/AM", "W1AW/AM/P", "W1AW/P/AM/7"];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call,
                &DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert!(res.is_special_entity());
        }
    }

    #[test]
    fn special_appendix_mm() {
        let calls = vec!["W1AW/MM", "W1AM/P/MM", "W1AW/MM/P", "W1AW/P/MM/7"];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call,
                &DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert!(res.is_special_entity());
        }
    }

    #[test]
    fn special_appendix_sat() {
        let calls = vec!["W1AW/SAT", "W1AM/P/SAT", "W1AW/SAT/P", "W1AW/P/SAT/7"];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call,
                &DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert!(res.is_special_entity());
        }
    }

    #[test]
    fn special_entity_prefix() {
        let calls = vec![
            ("SV1ABC/A", "2020-01-01T00:00:00Z", 180),    // Prefix SV/A
            ("SV2/W1AW/A", "2020-01-01T00:00:00Z", 180),  // Prefix SV/A
            ("3D2ABC/R", "2020-01-01T00:00:00Z", 460), // Prefix 3D2/R, where 3D2 and R are potential valid prefixes too
            ("3D2/W1ABC/R", "2020-01-01T00:00:00Z", 460), // Prefix 3D2/R, where 3D2 and R are potential valid prefixes too
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call.0,
                &DateTime::parse_from_rfc3339(call.1).unwrap().into(),
            )
            .unwrap();
            assert_eq!(res.adif, call.2);
        }
    }

    #[test]
    fn cqzone_exception() {
        let calls = vec![
            ("W1CBY/VE8", "1993-07-01T00:00:00Z", 1), // Record 548
            ("VE2BQB", "1992-01-01T00:00:00Z", 2),    // Record 35
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call.0,
                &DateTime::parse_from_rfc3339(call.1).unwrap().into(),
            )
            .unwrap();
            assert_eq!(res.cqzone.unwrap(), call.2);
        }
    }

    #[test]
    fn call_exceptions() {
        let calls = vec![
            ("AM70URE/8", "2019-05-01T00:00:00Z", 29),
            ("EA8VK/URE", "2021-01-01T00:00:00Z", 29),
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call.0,
                &DateTime::parse_from_rfc3339(call.1).unwrap().into(),
            )
            .unwrap();
            assert_eq!(res.adif, call.2);
        }
    }

    #[test]
    fn genuine_calls() {
        let calls = vec![
            ("W1ABC", 291),     // Basic call
            ("9A1ABC", 497),    // Call beginning with a number
            ("A71AB", 376),     // Call with two digits, one belonging to the prefix
            ("LM2T70Y", 266),   // Call with two separated numbers
            ("UA9ABC", 15),     // Check that the call is not matched for the prefix U
            ("U1ABC", 54),      // Counterexample for the test call above
            ("SV0ABC/9", 40),   // SV is Greece, but SV9 is Crete
            ("UA0JL/6", 54),    // UA0 is Asiatic Russia, but UA6 is European Russia
            ("MM/W1AW", 279),   // MM is Scotland and not Maritime Mobile
            ("F/W1AW", 227),    // F is France
            ("CE0Y/W1ABC", 47), // CE0Y is Easter Island, but CE would be Chile
            ("W1ABC/CE0Y", 47), // CE0Y is Easter Island, but CE would be Chile
            ("RW0A", 15),       // Call is also a prefix
            ("LS4AA/F", 227),   // LS is Argentina but F is France
            ("VE3LYC/KL7", 6),  // KL is Alaska
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            println!("Test for: {}", call.0);
            let res = analyze_callsign(
                clublog,
                call.0,
                &DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert_eq!(res.adif, call.1);
        }
    }
}
