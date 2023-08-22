//! Parser for the club log xml based country information.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer};
use std::convert::From;
use std::vec::Vec;

/// Errors
#[derive(Debug)]
pub struct Error;

impl ClubLog {
    /// Parse xml formatted content of the club log country file.
    pub fn parse(content: &str) -> Result<Self, Error> {
        quick_xml::de::from_str(content).map_err(|_| Error)
    }

    /// Query callsign information for prefix.
    pub fn lookup_prefix(&self, prefix: &str) -> Option<CallInfo> {
        // Search for matching prefix
        let pref = self.prefixes.prefix.iter().find(|p| p.call == prefix)?;

        // Check wether an adif identifier is given
        if pref.adif.is_none() {
            return Some(pref.into());
        }

        // Search entity for prefix
        let entity = self
            .entities
            .entity
            .iter()
            .find(|e| e.adif == pref.adif.unwrap())?; // FIXME: Thats possible an internal error since all adif identifiers mentioned within a prefix should be present within the entities list
                                                      // TODO: Find returns only the first match. Is this a possible error here?

        // Return result
        Some(entity.into())
    }

    /// Check if a exception exists for that specific callsign.
    pub fn get_callsign_exception(
        &self,
        callsign: &str,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<CallInfo> {
        if let Some(exception) = self
            .exceptions
            .exception
            .iter()
            .find(|e| e.call == callsign)
        // TODO: Find returns only the first match. Is this a possible error here?
        {
            if match (exception.start, exception.end) {
                (Some(tstart), Some(tend)) => timestamp >= tstart && timestamp <= tend,
                (Some(tstart), None) => timestamp >= tstart,
                (None, Some(tend)) => timestamp <= tend,
                (None, None) => false,
            } {
                Some(exception.into())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if the callsign was used in an invalid operation.
    pub fn is_invalid_operation(&self, callsign: &str, timestamp: DateTime<FixedOffset>) -> bool {
        if let Some(operation) = self
            .invalid_operations
            .invalid
            .iter()
            .find(|o| o.call == callsign)
        // TODO: Find returns only the first match. Is this a possible error here?
        {
            match (operation.start, operation.end) {
                (Some(tstart), Some(tend)) => timestamp >= tstart && timestamp <= tend,
                (Some(tstart), None) => timestamp >= tstart,
                (None, Some(tend)) => timestamp <= tend,
                (None, None) => false,
            }
        } else {
            false
        }
    }

    // Check if the callsign at the given time falls within the period of activity in another CQ zone.
    pub fn get_zone_exception(
        &self,
        callsign: &str,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<u8> {
        if let Some(exception) = self
            .zone_exceptions
            .zone_exception
            .iter()
            .find(|o| o.call == callsign)
        // TODO: Find returns only the first match. Is this a possible error here?
        {
            if match (exception.start, exception.end) {
                (Some(tstart), Some(tend)) => timestamp >= tstart && timestamp <= tend,
                (Some(tstart), None) => timestamp >= tstart,
                (None, Some(tend)) => timestamp <= tend,
                (None, None) => false,
            } {
                Some(exception.zone)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Callsign information
#[derive(Debug)]
pub struct CallInfo {
    /// Name
    pub country: String,
    /// Main callsign prefix
    pub prefix: String,
    /// ADIF identifier
    pub adif: Option<u16>,
    /// CQ zone
    pub cqz: Option<u8>,
    /// Continent
    pub cont: Option<String>,
    /// Longitude
    pub long: Option<f32>,
    /// Latitude
    pub lat: Option<f32>,
}

impl From<&Entity> for CallInfo {
    fn from(entity: &Entity) -> Self {
        Self {
            country: entity.name.clone(),
            prefix: entity.prefix.clone(),
            adif: Some(entity.adif),
            cqz: Some(entity.cqz),
            cont: Some(entity.cont.clone()),
            long: Some(entity.long),
            lat: Some(entity.lat),
        }
    }
}

impl From<&Prefix> for CallInfo {
    fn from(prefix: &Prefix) -> Self {
        Self {
            country: prefix.entity.clone(),
            prefix: prefix.call.clone(), // TODO: prefix != callsign, maybe generic CallInfo not required. Use specific type instead
            adif: prefix.adif,
            cqz: prefix.cqz,
            cont: prefix.cont.clone(),
            long: prefix.long,
            lat: prefix.lat,
        }
    }
}

impl From<&Exception> for CallInfo {
    fn from(exception: &Exception) -> Self {
        Self {
            country: exception.entity.clone(),
            prefix: exception.call.clone(), // TODO: prefix != callsign, maybe generic CallInfo not required. Use specific type instead
            adif: Some(exception.adif),
            cqz: Some(exception.cqz),
            cont: Some(exception.cont.clone()),
            long: Some(exception.long),
            lat: Some(exception.lat),
        }
    }
}

/// Custom XML deserializer for a timestamp
fn parse_datetime<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)
}

/// Custom XML deserializer for an optional timestamp
fn parse_datetime_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Ok(Some(
        DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?,
    ))
}

/// Representation of the club logs callsign lookup data
#[derive(Debug, Deserialize)]
#[serde(rename = "clublog")]
pub struct ClubLog {
    /// Timestamp of data
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime")]
    #[serde(rename = "@date")]
    pub date: DateTime<FixedOffset>,
    /// XML namespace
    #[serde(rename = "@xmlns")]
    _xmlns: String,
    /// List of entities
    entities: Entities,
    /// List of exceptions
    exceptions: Exceptions,
    /// List of prefixes
    prefixes: Prefixes,
    /// List of invalid operations
    invalid_operations: InvalidOperations,
    /// List of CQ zone exceptions
    zone_exceptions: ZoneExceptions,
}

/// List of entities / DXCCs
#[derive(Debug, Deserialize, PartialEq)]
struct Entities {
    pub entity: Vec<Entity>,
}

/// Single entity / DXCC.
///
/// An entry represents a single entity or rather DXCC.
/// The field [prefix](Entity::prefix) contains the main prefix of that entity.
/// All other prefixes including the main one are part of the [prefixes](Prefix) list.
/// Make sure to also validate against the [start](Prefix::start) and [end](Prefix::end) timestamps.
///
/// The field [deleted](Entity::deleted) may indicate a time limited validity of that entity.
/// Check the field [end](Entity::end) on the last timestamp a contact with that entity was valid.
///
/// If the field [whitelist](Entity::whitelist) is set to `true`, the entity is probably on the most wanted DXCC list.
/// Therefore only approved callsigns shall be logged for that entity.
/// The list of approved callsigns is part of the [callsign exception](Exception) list.
/// May also check the field [whitelist_start](Entity::whitelist_start) after which contacts shall be checked against the whitelist.
/// The timstamp is not necessarily present if a entity is whitelisted.
#[derive(Debug, Deserialize, PartialEq)]
struct Entity {
    /// ADIF identifier
    pub adif: u16,
    /// Name
    pub name: String,
    /// Main callsign prefix
    pub prefix: String,
    /// Entity deleted after [end](Entity::end)
    pub deleted: bool,
    /// CQ zone
    pub cqz: u8,
    /// Continent
    pub cont: String,
    /// Longitude
    pub long: f32,
    /// Latitude
    pub lat: f32,
    /// Start timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<FixedOffset>>,
    /// End timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<FixedOffset>>,
    /// True if only a whitelist of callsigns are valid for this entity
    pub whitelist: Option<bool>,
    /// Timestamp afer which the whitelist shall be used
    /// TODO: assumption that the actual whitelist is part of the exception list
    pub whitelist_start: Option<String>,
}

/// List of callsign exceptions
#[derive(Debug, Deserialize, PartialEq)]
struct Exceptions {
    pub exception: Vec<Exception>,
}

/// Callsign exception.
///
/// Represents an exceptions to a callsign [prefix](Prefix).
/// An entry may indicate a different value for the fields [adif](Exception::adif), [cqz](Exception::cqz), [cont](Exception::cont), [cont](Exception::cont), [lat](Exception::lat) or [lat](Exception::long) compared to the values of the matching [prefix](Prefix) entry.
/// While searching through the list of exceptions make sure to also validate against the optional [start](Exception::start) and [end](Exception::end) timestamps.
///
/// Valid callsigns for a [whitelisted entity](Entity::whitelist) are also part of the callsign exception list.
#[derive(Debug, Deserialize, PartialEq)]
struct Exception {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: u16,
    /// Callsign
    pub call: String,
    /// Name of entity
    pub entity: String,
    /// ADIF identifier
    pub adif: u16,
    /// CQ zone
    pub cqz: u8,
    /// Continent
    pub cont: String,
    /// Longitude
    pub long: f32,
    /// Latitude
    pub lat: f32,
    /// Start timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<FixedOffset>>,
    /// End timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<FixedOffset>>,
}

/// List of callsign prefixes
#[derive(Debug, Deserialize, PartialEq)]
struct Prefixes {
    pub prefix: Vec<Prefix>,
}

/// Callsign prefix.
///
/// Each prefix is representated by a single entry.
/// For example the prefixes `DA` and `DB` do both refer to the same DXCC `FEDERAL REPUBLIC OF GERMANY`.
/// Even all other fields of the two entries feature the same data.
/// While searching for a matching prefix make sure to also validate against the optional [start](Prefix::start) and [end](Prefix::end) timestamps.
///
/// If the fields [adif](Prefix::adif), [cqz](Prefix::cqz), [cont](Prefix::cont), [long](Prefix::long) and [lat](Prefix::lat) are `None`
/// the [entity](Prefix::entity) field may be `INVALID` or `MARITIME MOBILE`.
#[derive(Debug, Deserialize, PartialEq)]
struct Prefix {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: u16,
    /// Callsign
    pub call: String,
    /// Name of entity
    pub entity: String,
    /// ADIF identifier
    pub adif: Option<u16>, // FIXME: acc. to xsd no option required
    /// CQ zone
    pub cqz: Option<u8>, // FIXME: acc. to xsd no option required
    /// Continent
    pub cont: Option<String>, // FIXME: acc. to xsd no option required
    /// Longitude
    pub long: Option<f32>, // FIXME: acc. to xsd no option required
    /// Latitude
    pub lat: Option<f32>, // FIXME: acc. to xsd no option required
    /// Start timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<FixedOffset>>,
    /// End timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<FixedOffset>>,
}

/// List of invalid operations
#[derive(Debug, Deserialize, PartialEq)]
struct InvalidOperations {
    pub invalid: Vec<Invalid>,
}

/// Single invalid operation
#[derive(Debug, Deserialize, PartialEq)]
struct Invalid {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: u16,
    /// Callsign
    pub call: String,
    /// Start timestamp of operation
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<FixedOffset>>,
    /// End timestamp of operation
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<FixedOffset>>,
}

/// List of CQ zone exceptions
#[derive(Debug, Deserialize, PartialEq)]
struct ZoneExceptions {
    pub zone_exception: Vec<ZoneException>,
}

/// Single CQ zone exception
#[derive(Debug, Deserialize, PartialEq)]
struct ZoneException {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: u16,
    /// Callsign
    pub call: String,
    /// CQ zone
    pub zone: u8,
    /// Start timestamp of exception
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<FixedOffset>>,
    /// End timestamp of exception
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<FixedOffset>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn read_clublog_xml() -> ClubLog {
        let raw = fs::read_to_string("data/clublog/cty.xml").unwrap();
        ClubLog::parse(&raw).unwrap()
    }

    #[test]
    fn lookup_prefix_ok() {
        let info = read_clublog_xml().lookup_prefix("DA").unwrap();
        assert_eq!(info.adif, Some(230));
    }

    #[test]
    fn lookup_prefix_err() {
        let info = read_clublog_xml().lookup_prefix("FOO");
        assert!(info.is_none());
    }

    #[test]
    fn callsign_exception_ok() {
        let call_exc = read_clublog_xml().get_callsign_exception(
            "KC6RJW",
            DateTime::parse_from_rfc3339("2003-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(call_exc.is_some());
    }

    #[test]
    fn callsign_exception_err() {
        let call_exc = read_clublog_xml().get_callsign_exception(
            "A1B",
            DateTime::parse_from_rfc3339("2001-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(call_exc.is_none());
    }

    #[test]
    fn invalid_operation_ok() {
        let invalid = read_clublog_xml().is_invalid_operation(
            "T88A",
            DateTime::parse_from_rfc3339("1995-07-01T00:00:00+00:00").unwrap(),
        );
        assert!(invalid);
    }

    #[test]
    fn invalid_operation_err() {
        let invalid = read_clublog_xml().is_invalid_operation(
            "DL1FOO",
            DateTime::parse_from_rfc3339("2001-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(!invalid);
    }

    #[test]
    fn zone_exception_ok() {
        let exception = read_clublog_xml().get_zone_exception(
            "KD6WW/VY0",
            DateTime::parse_from_rfc3339("2003-07-30T12:00:00+00:00").unwrap(),
        );
        assert_eq!(exception, Some(1));
    }

    #[test]
    fn zone_exception_err() {
        let exception = read_clublog_xml().get_zone_exception(
            "DL1FOO",
            DateTime::parse_from_rfc3339("2001-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(exception.is_none());
    }
}
