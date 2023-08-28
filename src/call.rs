use crate::clublog::{
    Adif, ClubLog, CqZone, Prefix, CALLSIGN_EXCEPTION_AERONAUTICAL_MOBILE,
    CALLSIGN_EXCEPTION_SATELLITE, PREFIX_INVALID, PREFIX_MARITIME_MOBILE,
};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

/// Callsign
#[derive(Debug, PartialEq)]
pub struct Callsign {
    pub call: String,
    pub dxcc: Option<String>,
    pub adif: Option<Adif>,
    pub cqzone: Option<CqZone>,
    pub continent: Option<String>,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub maritime_mobile: bool,
    pub aeronautical_mobile: bool,
    pub satelite: bool,
}

impl Callsign {
    /// Instantiate a new maritime mobile callsign
    pub fn new_maritime_mobile(call: &str) -> Callsign {
        Callsign {
            call: String::from(call),
            dxcc: None,
            adif: None,
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
            maritime_mobile: true,
            aeronautical_mobile: false,
            satelite: false,
        }
    }

    /// Instantiate a new aeronautical mobile callsign
    pub fn new_aeronautical_mobile(call: &str, adif: Adif) -> Callsign {
        Callsign {
            call: String::from(call),
            dxcc: None,
            adif: Some(adif),
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
            maritime_mobile: false,
            aeronautical_mobile: true,
            satelite: false,
        }
    }

    /// Instantiate a new satellite callsign
    pub fn new_satellite(call: &str, adif: Adif) -> Callsign {
        Callsign {
            call: String::from(call),
            dxcc: None,
            adif: Some(adif),
            cqzone: None,
            continent: None,
            longitude: None,
            latitude: None,
            maritime_mobile: false,
            aeronautical_mobile: false,
            satelite: true,
        }
    }
}

/// Possible reasons for an invalid callsign
#[derive(Error, Debug, PartialEq, Eq)]
pub enum CallsignError {
    #[error("Callsign is of invalid format or include invalid characters")]
    InvalidFormat,

    #[error("Invalid operation")]
    InvalidOperation,

    #[error("No home call")]
    NoHomeCall,

    #[error("Invalid home call")]
    InvalidHomeCall,

    #[error("Multiple prefixes")]
    MultiplePrefixes,

    #[error("Invalid prefix")]
    InvalidPrefix,

    #[error("Invalid suffix position")]
    SuffixPosition,

    #[error("Internal error")]
    InternalError(String),
}

/// Special suffixes which will not be searched for in the list of prefixes
const SUFFIX_IGNORE: [&str; 4] = ["P", "M", "QRP", "LH"];

/// Special suffixes which will not be searched for in the list of prefixes.
/// Furthermore, these prefiex indicate that there is no actual DXCC which could be assigned to them.
const SUFFIX_SPECIAL: [&str; 3] = ["AM", "MM", "SAT"];

/// Type of split
#[derive(Debug, PartialEq, Eq)]
enum PartType {
    Prefix,
    Suffix,
    HomeCall,
}

/// Part of a callsign
struct Element {
    /// Part of the callsign
    pub part: String,
    /// Type of the part of the callsign
    pub parttype: PartType,
}

pub fn analyze_callsign(
    clublog: &ClubLog,
    call: &str,
    timestamp: DateTime<Utc>,
) -> Result<Callsign, CallsignError> {
    lazy_static! {
        static ref RE_COMPLETE_CALL: Regex = Regex::new(r"^[A-Z0-9]+[A-Z0-9/]*[A-Z0-9]+$").unwrap();
        static ref RE_HOMECALL: Regex = Regex::new(r"([0-9]?[A-Z]+)([0-9]+)([A-Z]+)").unwrap();
    }

    // 1. Check that only allowed characters are present and the callsign does not begin or end with a /
    if !RE_COMPLETE_CALL.is_match(call) {
        return Err(CallsignError::InvalidFormat);
    }

    // 2. Check if the callsign was used in an invalid operation by checking clublog data
    if clublog.is_invalid_operation(call, timestamp) {
        return Err(CallsignError::InvalidOperation);
    }

    // 3. Check if clublog lists a callsign exception
    if let Some(call_exc) = clublog.get_callsign_exception(call, timestamp) {
        let mut res = Callsign {
            call: String::from(call),
            dxcc: Some(call_exc.entity.clone()),
            adif: Some(call_exc.adif),
            cqzone: Some(call_exc.cqz),
            continent: Some(call_exc.cont.clone()),
            longitude: Some(call_exc.long),
            latitude: Some(call_exc.lat),
            maritime_mobile: false,
            aeronautical_mobile: false,
            satelite: false,
        };

        match call_exc.entity.as_str() {
            CALLSIGN_EXCEPTION_AERONAUTICAL_MOBILE => res.aeronautical_mobile = true,
            CALLSIGN_EXCEPTION_SATELLITE => res.satelite = true,
            _ => (),
        }

        return Ok(res);
    }

    // Split raw callsign into its parts
    let splits: Vec<&str> = call.split('/').collect();

    // 4. Classification of the callsign parts but validate them later
    let mut elements: Vec<Element> = Vec::new();
    for split in splits.iter() {
        let split_type = match (
            SUFFIX_IGNORE.contains(split) || SUFFIX_SPECIAL.contains(split),
            get_prefix(clublog, split, timestamp)?.is_some(),
            RE_HOMECALL.is_match(split),
        ) {
            (true, _, _) => PartType::Suffix,
            (false, true, _) => PartType::Prefix,
            (false, false, false) => PartType::Suffix,
            (false, false, true) => PartType::HomeCall,
        };

        elements.push(Element {
            part: String::from(*split),
            parttype: split_type,
        })
    }

    // 5. Check that only a single home call is left
    let homecalls: Vec<&Element> = elements
        .iter()
        .filter(|e| e.parttype == PartType::HomeCall)
        .collect();
    if homecalls.len() != 1 {
        return Err(CallsignError::NoHomeCall);
    }

    // 6. Check that only a single prefix is left
    let prefixes: Vec<&Element> = elements
        .iter()
        .filter(|e| e.parttype == PartType::Prefix)
        .collect();
    if prefixes.len() > 1 {
        return Err(CallsignError::MultiplePrefixes);
    }

    // 7. Check that all possible suffixes are behind the home call
    for element in elements.iter() {
        match element.parttype {
            PartType::HomeCall => break,
            PartType::Suffix => return Err(CallsignError::SuffixPosition),
            _ => (),
        }
    }

    // 8. Check if the single home call has a valid prefix
    // Search from the end of the call char by char, add a char each round.
    //
    // Rational for counting strategy:
    // By example, the prefixes U1 and UA9 refer to different ADIF identifiers.
    // If start searching from the first character of the call onwards,
    // the call UA9ACB would get the ADIF identifier for the prefix U and not for the prefix UA9.
    let homecall = homecalls[0];
    let mut homecall_prefix: Option<&Prefix> = None;
    for cnt in (1..homecall.part.len() + 1).rev() {
        let to_check = &homecall.part[0..cnt];
        if let Some(pref) = clublog.get_prefix(to_check, timestamp) {
            homecall_prefix = Some(pref);
            break;
        }
    }
    if homecall_prefix.is_none() {
        return Err(CallsignError::InvalidHomeCall);
    }

    // Until here the callsign is checked against basic rules and the data from clublog.
    // If not already returned early the call is found to be valid.
    // From here on combine all information to return result.

    // If next to the home call an additional prefix is present select which one to choose
    let prefix = if prefixes.is_empty() {
        homecall_prefix.unwrap()
    } else {
        get_prefix(clublog, &prefixes[0].part, timestamp)?.unwrap()
    };

    // Handle prefixes that reference the entities 'INVALID' and 'MARITIME MOBILE'
    if prefix.adif.is_none() {
        match prefix.entity.as_str() {
            PREFIX_INVALID => return Err(CallsignError::InvalidPrefix),
            PREFIX_MARITIME_MOBILE => return Ok(Callsign::new_maritime_mobile(call)),
            _ => (),
        }
    }

    // Check if a special prefix was found that does not feature a entity record (namely adif identifiers 997 and 998)
    match prefix.entity.as_str() {
        CALLSIGN_EXCEPTION_AERONAUTICAL_MOBILE => {
            return Ok(Callsign::new_aeronautical_mobile(
                call,
                prefix.adif.ok_or(CallsignError::InternalError(
                    "Special prefix referencing /AM missing the adif field".into(),
                ))?,
            ))
        }
        CALLSIGN_EXCEPTION_SATELLITE => {
            return Ok(Callsign::new_satellite(
                call,
                prefix.adif.ok_or(CallsignError::InternalError(
                    "Special prefix referencing /SAT missing the adif field".into(),
                ))?,
            ))
        }
        _ => (),
    }

    // Handle special suffixes
    // Boath AM and SAT feature a record within the prefix list where also an adif identifier is given.
    // MM is not part of the prefix list and has therefore not adif identifier.
    // TODO: The following code assumes that only one of the special suffixes are present
    if let Some(suffix_special) = elements
        .iter()
        .filter(|e| e.parttype == PartType::Suffix)
        .find(|e| SUFFIX_SPECIAL.contains(&e.part.as_str()))
    {
        match suffix_special.part.as_str() {
            "AM" => {
                let pref =
                    clublog
                        .get_prefix("/AM", timestamp)
                        .ok_or(CallsignError::InternalError(
                            "Failed to get prefix for /AM".into(),
                        ))?;
                let adif = pref.adif.ok_or(CallsignError::InternalError(
                    "Expected adif field to not be None".into(),
                ))?;
                return Ok(Callsign::new_aeronautical_mobile(call, adif));
            }
            "SAT" => {
                let pref =
                    clublog
                        .get_prefix("/SAT", timestamp)
                        .ok_or(CallsignError::InternalError(
                            "Failed to get prefix for /SAT".into(),
                        ))?;
                let adif = pref.adif.ok_or(CallsignError::InternalError(
                    "Expected adif field to not be None".into(),
                ))?;
                return Ok(Callsign::new_satellite(call, adif));
            }
            "MM" => return Ok(Callsign::new_maritime_mobile(call)),
            _ => {
                return Err(CallsignError::InternalError(
                    "Unexpected special suffix. Failed to match to known value.".into(),
                ))
            }
        }
    }

    // Get referenced entity from prefix
    let entity = clublog
        .get_entity(
            prefix.adif.ok_or(CallsignError::InternalError(
                "Expected adif field to not be None".into(),
            ))?,
            timestamp,
        )
        .ok_or(CallsignError::InternalError(format!(
            "Failed to get entity for prefix with adif {}",
            prefix.adif.unwrap()
        )))?;

    Ok(Callsign {
        call: String::from(call),
        dxcc: Some(entity.name.clone()),
        adif: Some(entity.adif),
        cqzone: Some(entity.cqz),
        continent: Some(entity.cont.clone()),
        longitude: Some(entity.long),
        latitude: Some(entity.lat),
        maritime_mobile: false,
        aeronautical_mobile: false,
        satelite: false,
    })
}

/// Method to get prefix information from clublog xml.
///
/// If the given prefix is not part in the prefix list check if the last character is a digit.
/// If so, remove the last character and check again if the now shortened prefix is in the prefix list.
///
/// Rationale:
/// By example, the prefix CT7 is not in the prefix list, since the correct prefix would be CT.
/// Here the last digit of the prefix indicates, that a special part of the entity referenced by the prefix is activated, e.g. an island group.
/// Therefore consider such prefixes as potentially valid and check the shortened version for validity.
fn get_prefix<'a>(
    clublog: &'a ClubLog,
    prefix: &str,
    timestamp: DateTime<Utc>,
) -> Result<Option<&'a Prefix>, CallsignError> {
    lazy_static! {
        static ref RE_PREFIX: Regex = Regex::new(r"^.+?(\d)$").unwrap();
    }

    // 1. Check if the prefix is part of the clublog list
    if let Some(pref) = clublog.get_prefix(prefix, timestamp) {
        return Ok(Some(pref));
    }

    // 2. Remove last digit if present and check again if the shortened prefix is in the clublog list
    if prefix
        .chars()
        .last()
        .ok_or(CallsignError::InternalError(
            "Cannot check prefix with length of zero".into(),
        ))?
        .is_numeric()
    {
        // Remove last character
        let mut chars = prefix.chars();
        chars.next_back();
        let chars = chars.as_str();

        // Check if shortened prefix is in list
        if let Some(pref) = clublog.get_prefix(chars, timestamp) {
            return Ok(Some(pref));
        }
    }

    Ok(None)
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
    fn get_prefix_special() {
        let prefixes: Vec<(&str, Option<Adif>)> = vec![
            ("CT", Some(272)),
            ("CT7", Some(272)),
            ("CT7A", None),
            ("9A", Some(497)),
            ("9A8", Some(497)),
            ("9A8A", None),
            ("A7", Some(376)),
            ("A71", Some(376)),
            ("A71A", None),
            ("7", None),
        ];

        let clublog = read_clublog_xml();

        for prefix in prefixes.iter() {
            if let Some(res) = get_prefix(
                &clublog,
                prefix.0,
                DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap()
            {
                assert_eq!(res.adif.unwrap(), prefix.1.unwrap());
            } else {
                assert!(prefix.1.is_none());
            }
        }
    }

    #[test]
    fn clublog_prefix_entity_invalid() {
        // test for prefix record 5028

        let clublog = read_clublog_xml();
        let res = analyze_callsign(
            clublog,
            "X5ABC",
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        assert_eq!(res, Err(CallsignError::InvalidPrefix));
    }

    #[test]
    fn clublog_prefix_entity_mm() {
        // test for prefix record 7069

        let clublog = read_clublog_xml();
        let res = analyze_callsign(
            clublog,
            "EL0ABC",
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .into(),
        )
        .unwrap();
        assert!(res.maritime_mobile);
        assert!(!res.aeronautical_mobile);
        assert!(!res.satelite);
    }

    #[test]
    fn clublog_prefix_am() {
        // test for prefix record 10273

        let clublog = read_clublog_xml();
        let res = analyze_callsign(
            clublog,
            "RS80KEDR",
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .into(),
        )
        .unwrap();
        assert!(res.satelite);
        assert!(!res.aeronautical_mobile);
        assert!(!res.maritime_mobile);
    }

    #[test]
    fn clublog_call_exc_am() {
        // test for call exception record 2730

        let clublog = read_clublog_xml();
        let res = analyze_callsign(
            clublog,
            "KB5SIW/STS50",
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .into(),
        )
        .unwrap();
        assert!(res.aeronautical_mobile);
        assert!(!res.satelite);
        assert!(!res.maritime_mobile);
    }

    #[test]
    fn clublog_call_exc_sat() {
        // test for callsign exception record 28169

        let clublog = read_clublog_xml();
        let res = analyze_callsign(
            clublog,
            "ZY0RK",
            DateTime::parse_from_rfc3339("1994-08-20T00:00:00Z")
                .unwrap()
                .into(),
        )
        .unwrap();
        assert!(res.satelite);
        assert!(!res.aeronautical_mobile);
        assert!(!res.maritime_mobile);
    }

    #[test]
    fn special_suffix_am() {
        let calls = vec![
            "W1AW/AM",
            "W1AM/P/AM",
            "W1AW/AM/P",
            "W1AW/P/AM/7",
            "F/W1AW/AM",
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call,
                DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert!(res.aeronautical_mobile);
            assert!(!res.satelite);
            assert!(!res.maritime_mobile);
        }
    }

    #[test]
    fn special_suffix_mm() {
        let calls = vec![
            "W1AW/MM",
            "W1AM/P/MM",
            "W1AW/MM/P",
            "W1AW/P/MM/7",
            "F/W1AW/MM",
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call,
                DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert!(res.maritime_mobile);
            assert!(!res.aeronautical_mobile);
            assert!(!res.satelite);
        }
    }

    #[test]
    fn special_suffix_sat() {
        let calls = vec![
            "W1AW/SAT",
            "W1AM/P/SAT",
            "W1AW/SAT/P",
            "W1AW/P/SAT/7",
            "F/W1AW/SAT",
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call,
                DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert!(res.satelite);
            assert!(!res.aeronautical_mobile);
            assert!(!res.maritime_mobile);
        }
    }

    #[test]
    fn genuine_calls() {
        let calls = vec![
            ("W1ABC", 291),   // Basic call
            ("9A1ABC", 497),  // Call beginning with a number
            ("A71AB", 376),   // Call with two digits, one belonging to the prefix
            ("LM2T70Y", 266), // Call with two separated digts
            ("UA9ABC", 15),   // Check that the call is not matched for the prefix U
            ("U1ABC", 54),    // Counterexample for the test call above
        ];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call.0,
                DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert_eq!(res.adif.unwrap(), call.1);
        }
    }

    #[test]
    fn prefixes() {
        let calls = vec![("F/W1AW", 227), ("CE0Y/W1ABC", 47), ("W1ABC/CE0Y", 47)];

        let clublog = read_clublog_xml();

        for call in calls.iter() {
            let res = analyze_callsign(
                clublog,
                call.0,
                DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
            assert_eq!(res.adif.unwrap(), call.1);
        }
    }
}
