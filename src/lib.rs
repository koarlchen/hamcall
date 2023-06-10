use lazy_static::lazy_static;
use regex::Regex;
use std::vec::Vec;
use thiserror::Error;

pub mod clublog;

/// Callsign
#[derive(Debug, PartialEq, Eq)]
pub struct Callsign {
    // Prefix of home callsign
    pub prefix: String,
    // Number of home callsign
    pub number: String,
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
        let str_prefix = if let Some(val) = self.add_prefix.clone() {
            format!("{}/", val)
        } else {
            String::new()
        };

        let str_suffix = if !self.add_suffix.is_empty() {
            format!("/{}", self.add_suffix.join("/"))
        } else {
            String::new()
        };

        format!("{}{}{}", str_prefix, self.homecall(), str_suffix)
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

    #[error("Internal error")]
    InternalError,
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
            Regex::new(r"^(?:(([A-Z]+)(\d+)([A-Z]+))|((\d[A-Z]+)(\d+)([A-Z]+)))$").unwrap();
    }

    // Check that only allowed characters are present and the callsign does not begin or end with a /
    if !RE_CALL.is_match(call) {
        return Err(CallsignError::InvalidFormat);
    }

    // Split raw callsign into its parts
    let splits: Vec<&str> = call.split('/').collect();

    // Extract all valid home calls from splits
    let homecalls: Vec<(String, String, String, String)> = splits
        .iter()
        .filter_map(|&part| {
            if let Some(caps) = RE_HOME_CALL.captures(part) {
                let offset = if caps.get(1).is_some() { 1 } else { 5 };
                Some((
                    String::from(&caps[offset]),
                    String::from(&caps[offset + 1]),
                    String::from(&caps[offset + 2]),
                    String::from(&caps[offset + 3]),
                ))
            } else {
                None
            }
        })
        .collect();

    // Check for number of found results
    let homecall = match homecalls.len() {
        0 => return Err(CallsignError::NoHomeCall),
        1 => homecalls.get(0).unwrap(),
        _ => return Err(CallsignError::MultipleHomeCalls),
    };

    // Calculate offset of home call within splits
    let call_offset = splits.iter().position(|&c| c == homecall.0).unwrap(); // TODO: Could the position extracted previously? Maybe by using enumerate on iter

    // Check for multiple additional prefixes
    if call_offset >= 2 {
        return Err(CallsignError::MultipleAdditionalPrefixes);
    }

    // Get additional prefix
    let add_prefix = if call_offset == 1 {
        Some(String::from(splits[0]))
    } else {
        None
    };

    // Build together callsign
    let res = Callsign {
        prefix: homecall.1.clone(),
        number: homecall.2.clone(),
        suffix: homecall.3.clone(),
        add_prefix,
        add_suffix: splits
            .into_iter()
            .skip(call_offset + 1)
            .map(String::from)
            .collect(),
    };

    // Basic cross check
    if call != res.call() {
        return Err(CallsignError::InternalError);
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_homecall() {
        let calls = vec![
            (
                "DA1BC",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: None,
                    add_suffix: Vec::new(),
                },
            ),
            (
                "W1AW",
                Callsign {
                    prefix: "W".into(),
                    number: "1".into(),
                    suffix: "AW".into(),
                    add_prefix: None,
                    add_suffix: Vec::new(),
                },
            ),
            (
                "1A2B",
                Callsign {
                    prefix: "1A".into(),
                    number: "2".into(),
                    suffix: "B".into(),
                    add_prefix: None,
                    add_suffix: Vec::new(),
                },
            ),
            (
                "DL100FW",
                Callsign {
                    prefix: "DL".into(),
                    number: "100".into(),
                    suffix: "FW".into(),
                    add_prefix: None,
                    add_suffix: Vec::new(),
                },
            ),
            (
                "DL9999DOK",
                Callsign {
                    prefix: "DL".into(),
                    number: "9999".into(),
                    suffix: "DOK".into(),
                    add_prefix: None,
                    add_suffix: Vec::new(),
                },
            ),
            (
                "3DA0AQ",
                Callsign {
                    prefix: "3DA".into(),
                    number: "0".into(),
                    suffix: "AQ".into(),
                    add_prefix: None,
                    add_suffix: Vec::new(),
                },
            ),
            (
                "3E100PC",
                Callsign {
                    prefix: "3E".into(),
                    number: "100".into(),
                    suffix: "PC".into(),
                    add_prefix: None,
                    add_suffix: Vec::new(),
                },
            ),
            (
                "F04OD",
                Callsign {
                    prefix: "F".into(),
                    number: "04".into(),
                    suffix: "OD".into(),
                    add_prefix: None,
                    add_suffix: Vec::new(),
                },
            ),
        ];

        for call in calls.into_iter() {
            assert_eq!(analyze_callsign(call.0), Ok(call.1));
        }
    }

    #[test]
    fn valid_prefix() {
        let calls = vec![
            (
                "OE/DA1BC",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: Some("OE".into()),
                    add_suffix: Vec::new(),
                },
            ),
            (
                "OE1/DA1BC",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: Some("OE1".into()),
                    add_suffix: Vec::new(),
                },
            ),
            (
                "1A/DA1BC",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: Some("1A".into()),
                    add_suffix: Vec::new(),
                },
            ),
            (
                "W1/DA1BC",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: Some("W1".into()),
                    add_suffix: Vec::new(),
                },
            ),
            (
                "3DA0/ZS6BCR",
                Callsign {
                    prefix: "ZS".into(),
                    number: "6".into(),
                    suffix: "BCR".into(),
                    add_prefix: Some("3DA0".into()),
                    add_suffix: Vec::new(),
                },
            ),
            (
                "4X3000/4X1BD",
                Callsign {
                    prefix: "4X".into(),
                    number: "1".into(),
                    suffix: "BD".into(),
                    add_prefix: Some("4X3000".into()),
                    add_suffix: Vec::new(),
                },
            ),
        ];

        for call in calls.into_iter() {
            assert_eq!(analyze_callsign(call.0), Ok(call.1));
        }
    }

    #[test]
    fn valid_suffix() {
        let calls = vec![
            (
                "DA1BC/P",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: None,
                    add_suffix: vec!["P".into()],
                },
            ),
            (
                "DA1BC/5",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: None,
                    add_suffix: vec!["5".into()],
                },
            ),
            (
                "DA1BC/EA5",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: None,
                    add_suffix: vec!["EA5".into()],
                },
            ),
            (
                "DA1BC/LH",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: None,
                    add_suffix: vec!["LH".into()],
                },
            ),
            (
                "DA1BC/1A",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: None,
                    add_suffix: vec!["1A".into()],
                },
            ),
            (
                "DA1BC/P/LH",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: None,
                    add_suffix: vec!["P".into(), "LH".into()],
                },
            ),
            (
                "DA1BC/P/LH/ABC",
                Callsign {
                    prefix: "DA".into(),
                    number: "1".into(),
                    suffix: "BC".into(),
                    add_prefix: None,
                    add_suffix: vec!["P".into(), "LH".into(), "ABC".into()],
                },
            ),
        ];

        for call in calls.into_iter() {
            assert_eq!(analyze_callsign(call.0), Ok(call.1));
        }
    }

    #[test]
    fn invalid_something() {
        let calls = [
            ("/DA1BC", CallsignError::InvalidFormat),
            ("DA1BC/", CallsignError::InvalidFormat),
            ("/DA1BC/", CallsignError::InvalidFormat),
            ("DAIBC", CallsignError::NoHomeCall),
            ("OE/DAIBC", CallsignError::NoHomeCall),
            ("1ABC", CallsignError::NoHomeCall),
            ("1ABC/P", CallsignError::NoHomeCall),
            ("DA1BC2", CallsignError::NoHomeCall),
        ];

        for call in calls.into_iter() {
            assert_eq!(analyze_callsign(call.0), Err(call.1));
        }
    }

    #[test]
    fn invalid_homecall() {
        let calls = [
            ("W1AW/DA1BC", CallsignError::MultipleHomeCalls),
            ("W1AW/P/DA1BC", CallsignError::MultipleHomeCalls),
            ("W1AW/P/DA1BC/LH", CallsignError::MultipleHomeCalls),
            ("W1AW/P/DA1BC/LH/P", CallsignError::MultipleHomeCalls),
            ("1A/W1AW/P/DA1BC/LH/P", CallsignError::MultipleHomeCalls),
        ];

        for call in calls.into_iter() {
            assert_eq!(analyze_callsign(call.0), Err(call.1));
        }
    }

    #[test]
    fn invalid_prefix() {
        let calls = [
            ("EA/OE/DA1BC", CallsignError::MultipleAdditionalPrefixes),
            ("EA/OE/DA1BC/P", CallsignError::MultipleAdditionalPrefixes),
        ];

        for call in calls.into_iter() {
            assert_eq!(analyze_callsign(call.0), Err(call.1));
        }
    }
}
