//! Analyzer for callsigns based on the data from the ClubLog XML to get further information like the callsigns entity.
//!
//! The example `call.rs` shows the basic usage of this module.

use crate::clublog::{
    Adif, CallsignException, ClubLog, CqZone, Prefix, ADIF_ID_AERONAUTICAL_MOBILE,
    ADIF_ID_MARITIME_MOBILE, ADIF_ID_SATELLITE, PREFIX_INVALID, PREFIX_MARITIME_MOBILE,
};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

/// Callsign
#[derive(Debug, PartialEq)]
pub struct Callsign {
    /// Complete callsign
    pub call: String,
    /// Name of entity
    pub dxcc: Option<String>,
    /// ADIF identifier
    pub adif: Option<Adif>,
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
    /// Check if callsign is assigned to a special entity like /MM, /AM or /SAT.
    pub fn is_special_entity(&self) -> Option<SpecialEntitySuffix> {
        match self.adif {
            Some(ADIF_ID_MARITIME_MOBILE) => Some(SpecialEntitySuffix::Mm),
            Some(ADIF_ID_AERONAUTICAL_MOBILE) => Some(SpecialEntitySuffix::Am),
            Some(ADIF_ID_SATELLITE) => Some(SpecialEntitySuffix::Sat),
            _ => None,
        }
    }

    /// Instantiate a new maritime mobile callsign
    fn new_maritime_mobile(call: &str) -> Callsign {
        Callsign {
            call: String::from(call),
            dxcc: None,
            adif: Some(ADIF_ID_MARITIME_MOBILE),
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
        }
    }

    /// Instantiate a new aeronautical mobile callsign
    fn new_aeronautical_mobile(call: &str) -> Callsign {
        Callsign {
            call: String::from(call),
            dxcc: None,
            adif: Some(ADIF_ID_AERONAUTICAL_MOBILE),
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
        }
    }

    /// Instantiate a new satellite callsign
    fn new_satellite(call: &str) -> Callsign {
        Callsign {
            call: String::from(call),
            dxcc: None,
            adif: Some(ADIF_ID_SATELLITE),
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
        }
    }

    /// Instantiate a new callsign from a clublog prefix
    fn from_prefix(call: &str, prefix: &Prefix) -> Callsign {
        Callsign {
            call: String::from(call),
            dxcc: Some(prefix.entity.clone()),
            adif: prefix.adif,
            cqzone: prefix.cqz,
            continent: prefix.cont.clone(),
            longitude: prefix.long,
            latitude: prefix.lat,
        }
    }

    /// Instantiate a new callsign from a clublog callsign exception
    fn from_exception(call: &str, exc: &CallsignException) -> Callsign {
        Callsign {
            call: String::from(call),
            dxcc: Some(exc.entity.clone()),
            adif: Some(exc.adif),
            cqzone: Some(exc.cqz),
            continent: Some(exc.cont.clone()),
            longitude: Some(exc.long),
            latitude: Some(exc.lat),
        }
    }
}

/// Possible reasons for an invalid callsign
#[derive(Error, Debug, PartialEq)]
pub enum CallsignError {
    /// Callsign is of invalid format or includes invalid characters
    #[error("Callsign is of invalid format or includes invalid characters")]
    BasicFormat,

    /// Callsign is invalid
    #[error("Callsign is invalid")]
    Invalid(&'static str),
}

/// Special suffixes that may not be interpreted as prefixes
const SUFFIX_SPECIAL: [&str; 7] = ["AM", "MM", "SAT", "P", "M", "QRP", "LH"];

/// Type of split
#[derive(PartialEq, Eq)]
enum PartType {
    /// Prefix
    Prefix,
    /// Everything other than a prefix
    Other,
}

/// Part of a callsign
struct Element {
    /// Part of the callsign
    pub part: String,
    /// Type of the part of the callsign
    pub parttype: PartType,
}

/// State of the call element classification statemachine
#[derive(PartialEq, Eq)]
enum State {
    /// No prefix found so far
    NoPrefix,
    /// Single prefix found, another prefix or suffixes may follow
    SinglePrefix,
    /// Found complete prefix, only suffixes may follow
    PrefixComplete(u8),
}

/// Suffix that indicates that the calls entity may be ignored
#[derive(PartialEq, Eq)]
pub enum SpecialEntitySuffix {
    /// Maritime Mobile
    Mm,
    /// Aeronautical Mobile
    Am,
    /// Satellite, Internet or Repeater
    Sat,
}

/// Check if the callsign is whitelisted if the whitelist option is enabled for the entity of the callsign at the given point in time.
/// Returns true if the callsign is valid, false if whitelisting for that entity is enabled and the callsign is not on the whitelist.
pub fn check_whitelist(
    clublog: &ClubLog,
    call: &str,
    adif: Adif,
    timestamp: &DateTime<Utc>,
) -> bool {
    // Get entity for adif identifier
    // Note that not all valid adif identifiers refer to same entity (e.g. aeronautical mobile calls)
    if let Some(entity) = clublog.get_entity(adif, timestamp) {
        // Check if whitelisting is enabled
        if entity.whitelist == Some(true) {
            // Check if an exception for the call at the given point in time is present
            if let Some(prefix) = clublog.get_callsign_exception(call, timestamp) {
                // There may be a callsign exception for a whitelisted entity but the exception refers a different adif identifier
                return prefix.adif == adif;
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

/// Analyze callsign to get further information.
pub fn analyze_callsign(
    clublog: &ClubLog,
    call: &str,
    timestamp: &DateTime<Utc>,
) -> Result<Callsign, CallsignError> {
    lazy_static! {
        static ref RE_COMPLETE_CALL: Regex = Regex::new(r"^[A-Z0-9]+[A-Z0-9/]*[A-Z0-9]+$").unwrap();
    }

    // Check that only allowed characters are present and the callsign does not begin or end with a /
    if !RE_COMPLETE_CALL.is_match(call) {
        return Err(CallsignError::BasicFormat);
    }

    // Check if the callsign was used in an invalid operation
    if clublog.is_invalid_operation(call, timestamp) {
        return Err(CallsignError::Invalid("Invalid operation"));
    }

    // Check if clublog lists a callsign exception
    if let Some(call_exc) = clublog.get_callsign_exception(call, timestamp) {
        return Ok(Callsign::from_exception(call, call_exc));
    }

    // Split raw callsign into its parts
    let parts: Vec<&str> = call.split('/').collect();

    // Iterate through all parts of the callsign and check wether the part of the callsigns is a valid prefix or something else
    let mut elements: Vec<Element> = Vec::with_capacity(parts.len());
    for (pos, part) in parts.iter().enumerate() {
        // TODO: Is the assumption below correct for very special prefixes like SV/A? -> What if SV is not a prefix but SV/A is a valid prefix?
        let pt = if get_prefix(clublog, part, timestamp, &[]).is_some() {
            // MM and AM may be valid prefixes or special suffixes depending on the position within the complete callsign.
            // For example MM as a prefix evaluates to Scotland, MM as a suffix indicates a maritime mobile activation.
            // Special suffixes are only valid as those if they are right at the beginning of the callsign.
            // Therefore ignore the first element of the call and check for special suffixes beginning from the second element onwards.
            if pos >= 1 && is_special_suffix(part) {
                PartType::Other
            } else {
                PartType::Prefix
            }
        } else {
            PartType::Other
        };
        elements.push(Element {
            part: String::from(*part),
            parttype: pt,
        });
    }

    // Check for basic validity with a small statemachine.
    // For example check that the call begins with a prefix, has no more than two consecutive prefixes, ...
    let mut state = State::NoPrefix;
    for element in elements.iter() {
        match (&state, &element.parttype) {
            (State::NoPrefix, PartType::Prefix) => state = State::SinglePrefix,
            (State::NoPrefix, PartType::Other) => Err(CallsignError::Invalid(
                "Callsign does not begin with a valid prefix",
            ))?,
            (State::SinglePrefix, PartType::Prefix) => state = State::PrefixComplete(2),
            (State::SinglePrefix, PartType::Other) => state = State::PrefixComplete(1),
            (State::PrefixComplete(_), PartType::Prefix) => {
                Err(CallsignError::Invalid("Unexpected third prefix"))?
            }
            (State::PrefixComplete(_), PartType::Other) => (),
        }
    }

    assert!(
        state == State::PrefixComplete(1)
            || state == State::PrefixComplete(2)
            || state == State::SinglePrefix
    );

    // Possible state 1:
    // The callsign consists of only one part with no prefix nor suffix
    if state == State::SinglePrefix {
        let prefix = get_prefix(clublog, call, timestamp, &[]).unwrap().0;

        let res = if is_mm_entity(prefix) {
            Callsign::new_maritime_mobile(call)
        } else {
            let mut callsign = Callsign::from_prefix(call, prefix);
            apply_cqzone_exception(clublog, &mut callsign, timestamp);
            callsign
        };

        return Ok(res);
    }

    // Possible state 2:
    // The callsign consists of a single prefix and zero or more suffixes
    if state == State::PrefixComplete(1) {
        // Complete homecall
        // Example: W1AW
        let homecall: &String = &elements[0].part;

        // Prefix of the homecall
        // Example: W for the homecall W1AW
        let homecall_prefix = get_prefix(clublog, homecall, timestamp, &elements[1..])
            .unwrap()
            .0;

        // Special suffix like /AM or /MM is present
        // Example: W1AW/AM
        if let Some(suffix) = is_no_entity_by_suffix(&elements[1..])? {
            return Ok(match suffix {
                SpecialEntitySuffix::Am => Callsign::new_aeronautical_mobile(call),
                SpecialEntitySuffix::Mm => Callsign::new_maritime_mobile(call),
                SpecialEntitySuffix::Sat => Callsign::new_satellite(call),
            });
        }

        // Entity name referenced in prefix is /MM
        // Example: prefix record 7069
        if is_mm_entity(homecall_prefix) {
            return Ok(Callsign::new_maritime_mobile(call));
        }

        // Check if a single digit suffix is present
        // If so, check if the single digit suffix changes the prefix to a different one
        // Example: "SV0ABC/9" where SV is Greece, but SV9 is Crete
        if let Some(pref) = is_different_prefix_by_single_digit_suffix(
            clublog,
            homecall,
            timestamp,
            &elements[1..],
        )? {
            let mut callsign = Callsign::from_prefix(call, pref.0);
            apply_cqzone_exception(clublog, &mut callsign, timestamp);
            return Ok(callsign);
        }

        // No special rule matched, just return information
        let mut callsign = Callsign::from_prefix(call, homecall_prefix);
        apply_cqzone_exception(clublog, &mut callsign, timestamp);
        return Ok(callsign);
    }

    // Possible state 3:
    // The callsign consists of two prefixes and one or more suffixes
    if state == State::PrefixComplete(2) {
        // Get prefix information for both prefixes.
        // Decide which one to use by how many characters were removed from the potential prefix before it matched a prefix from the list.
        // The prefix which required less character removals wins.
        // This is probably not 100% correct, but seems good enough.
        let pref_first = get_prefix(clublog, &elements[0].part, timestamp, &elements[1..]).unwrap();
        let pref_second =
            get_prefix(clublog, &elements[1].part, timestamp, &elements[1..]).unwrap();

        let pref = if pref_first.1 <= pref_second.1 {
            pref_first.0
        } else {
            pref_second.0
        };

        let mut callsign = Callsign::from_prefix(call, pref);
        apply_cqzone_exception(clublog, &mut callsign, timestamp);
        return Ok(callsign);
    }

    panic!("Should not end here");
}

/// Update CQ zone of callsign if a zone exception is present
fn apply_cqzone_exception(clublog: &ClubLog, call: &mut Callsign, timestamp: &DateTime<Utc>) {
    if let Some(cqz) = clublog.get_zone_exception(&call.call, timestamp) {
        call.cqzone = Some(cqz);
    }
}

/// Check if the list of suffixes contains a suffix with a single digit that may indicate a different prefix.
/// If there is such single digit suffix replace the digit within the callsign and query information for the new prefix.
/// Example: "SV0ABC/9" where SV is Greece, but SV9 is Crete
fn is_different_prefix_by_single_digit_suffix<'a>(
    clublog: &'a ClubLog,
    homecall: &str,
    timestamp: &DateTime<Utc>,
    suffixes: &[Element],
) -> Result<Option<(&'a Prefix, usize)>, CallsignError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([A-Z0-9]+)(\d)([A-Z0-9]+)$").unwrap();
    }

    // Search for single digits in the list of suffixes
    let single_digits: Vec<&Element> = suffixes
        .iter()
        .filter(|e| is_single_digit_suffix(&e.part))
        .collect();
    match single_digits.len() {
        0 => return Ok(None),
        1 => (),
        _ => {
            // TODO: Should this be treated like an error? Just take the first? Ignore all of them?
            return Err(CallsignError::Invalid(
                "Multiple single digit suffixes found",
            ));
        }
    }
    let new_digit = &single_digits[0].part;

    // TODO: RE.replace probably not required here, just assemble new call from both capture groups and the new digit like below
    let new_homecall = RE.replace(homecall, format!("${{1}}{}${{3}}", new_digit));

    Ok(get_prefix(clublog, &new_homecall, timestamp, suffixes))
}

/// Check if the entity named in the prefix indicates a maritime mobile callsign
fn is_mm_entity(prefix: &Prefix) -> bool {
    prefix.entity == PREFIX_MARITIME_MOBILE
}

/// Check if the entity named in the prefix indicates an invalid operation
fn is_invalid_prefix(prefix: &Prefix) -> bool {
    prefix.entity == PREFIX_INVALID
}

/// Check if a special suffix is present that indicates that the actual prefix of the call is not relevant.
/// Such a suffix may for example be MM (maritime mobile).
fn is_no_entity_by_suffix(
    suffixes: &[Element],
) -> Result<Option<SpecialEntitySuffix>, CallsignError> {
    let s: Vec<&Element> = suffixes
        .iter()
        .filter(|e| suffix_indicates_no_entity(&e.part).is_some())
        .collect();

    match s.len() {
        0 => Ok(None),
        1 => Ok(suffix_indicates_no_entity(&s[0].part)),
        _ => Err(CallsignError::Invalid(
            "Multiple suffixes that indicate no entity",
        )),
    }
}

/// Check if a potential suffix equals a special suffix which requires special treatment of the call prefix.
/// For example AM (aeronautical mobile) indicates that the call prefix information may not need to be considered.
fn suffix_indicates_no_entity(potential_suffix: &str) -> Option<SpecialEntitySuffix> {
    match potential_suffix {
        "MM" => Some(SpecialEntitySuffix::Mm),
        "AM" => Some(SpecialEntitySuffix::Am),
        "SAT" => Some(SpecialEntitySuffix::Sat),
        _ => None,
    }
}

/// Check if the potential suffix is a special suffix.
/// See [SUFFIX_SPECIAL].
fn is_special_suffix(potential_suffix: &str) -> bool {
    SUFFIX_SPECIAL.contains(&potential_suffix)
}

/// Check if the potential suffix is a single digit suffix
fn is_single_digit_suffix(potential_suffix: &str) -> bool {
    if potential_suffix.len() == 1 {
        potential_suffix.chars().next().unwrap().is_numeric()
    } else {
        false
    }
}

/// Check if the potential suffix is a single char suffix
fn is_single_char_suffix(potential_suffix: &str) -> bool {
    if potential_suffix.len() == 1 {
        potential_suffix.chars().next().unwrap().is_alphabetic()
    } else {
        false
    }
}

/// Search for a matching prefix by brutforcing all possibilities.
/// The potential prefix will be shortened char by char from the back until a prefix matches.
/// Furthermore, to take in account of prefixes like SV/A, append all single char suffixes to the end of the potential prefix before checking for a match.
/// Next to the prefix information the number of removed chars is returned.
fn get_prefix<'a>(
    clublog: &'a ClubLog,
    potential_prefix: &str,
    timestamp: &DateTime<Utc>,
    suffixes: &[Element],
) -> Option<(&'a Prefix, usize)> {
    let len_potential_prefix = potential_prefix.len();
    assert!(len_potential_prefix >= 1);

    // Search for single char suffixes
    // For example SV/A is a valid prefix but indicates a different entity as the prefix SV
    let single_char_suffixes: Vec<&Element> = suffixes
        .iter()
        .filter(|e| is_single_char_suffix(&e.part))
        .collect::<Vec<&Element>>();

    // Bruteforce all possibilities
    // Shortening the call from the back is required to due to calls like UA9ABC where both prefixes U and UA9 a potential matches,
    // but the more explicit one is the correct one.
    let mut prefix: Option<(&Prefix, usize)> = None;
    for cnt in (1..len_potential_prefix + 1).rev() {
        // Shortened call
        let slice = &potential_prefix[0..cnt];

        // Append all single chars to the call as <call>/<suffix> and check if the prefix is valid
        // This check is required for prefixes like SV/A where the callsign SV1ABC/A shall match to
        for suffix in &single_char_suffixes {
            if let Some(pref) = clublog.get_prefix(&format!("{}/{}", slice, suffix.part), timestamp)
            {
                prefix = Some((pref, len_potential_prefix - cnt));
                break;
            }
        }
        if prefix.is_some() {
            break;
        }

        // Check if prefix is valid
        if let Some(pref) = clublog.get_prefix(slice, timestamp) {
            if !is_invalid_prefix(pref) {
                prefix = Some((pref, len_potential_prefix - cnt));
            }
            break;
        }
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::fs;

    fn read_clublog_xml() -> &'static ClubLog {
        lazy_static! {
            static ref CLUBLOG: ClubLog =
                ClubLog::parse(&fs::read_to_string("data/clublog/cty.xml").unwrap()).unwrap();
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

            assert!(res.is_err()); // TODO: Somehow check for exact error, prevent pass of test case caused by internal error
        }
    }

    #[test]
    fn clublog_special_suffix() {
        let calls = vec![
            ("EL0ABC", "2020-01-01T00:00:00Z", ADIF_ID_MARITIME_MOBILE), // test for prefix record 7069
            (
                "KB5SIW/STS50",
                "2020-01-01T00:00:00Z",
                ADIF_ID_AERONAUTICAL_MOBILE,
            ), // test for call exception record 2730
            ("ZY0RK", "1994-08-20T00:00:00Z", ADIF_ID_SATELLITE), // test for callsign exception record 28169
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

            assert_eq!(res.adif.unwrap(), call.2);
        }
    }

    #[test]
    fn clublog_whitelist() {
        let calls = vec![
            ("KH4AB", 174, "1975-01-01T00:00:00Z", true), // Timestamp before start of whitelist
            ("KH4AB", 174, "1981-01-01T00:00:00Z", false), // Timestamp after start of whitelist and call not part of exception list
            ("KH4AB", 174, "1980-04-07T00:00:00Z", true), // Timestamp after start of whitelist and call is part of exception list
            ("KH4AB", 174, "1983-01-02T00:00:00Z", false), // Timestamp after start of whitelist, call is part of exception list but with different adif identifier
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            println!("Test for: {}", call.0);
            let res = check_whitelist(
                clublog,
                call.0,
                call.1,
                &DateTime::parse_from_rfc3339(call.2).unwrap().into(),
            );
            assert_eq!(call.3, res);
        }
    }

    #[test]
    fn special_suffix_am() {
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
            assert_eq!(res.adif.unwrap(), ADIF_ID_AERONAUTICAL_MOBILE);
        }
    }

    #[test]
    fn special_suffix_mm() {
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
            assert_eq!(res.adif.unwrap(), ADIF_ID_MARITIME_MOBILE);
        }
    }

    #[test]
    fn special_suffix_sat() {
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
            assert_eq!(res.adif.unwrap(), ADIF_ID_SATELLITE);
        }
    }

    #[test]
    fn special_entity_prefix() {
        let calls = vec![
            ("SV1ABC/A", "2020-01-01T00:00:00Z", 180),
            ("SV2/W1AW/A", "2020-01-01T00:00:00Z", 180),
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call.0,
                &DateTime::parse_from_rfc3339(call.1).unwrap().into(),
            )
            .unwrap();
            assert_eq!(res.adif.unwrap(), call.2);
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
            assert_eq!(res.adif.unwrap(), call.2);
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
            assert_eq!(res.adif.unwrap(), call.1);
        }
    }
}
