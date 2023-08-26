//! Parser for the club log xml based country and callsign information.
//! Provides a few basic methods to query information out of the data.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer};
use std::vec::Vec;

/// ADIF DXCC identifier
pub type Adif = u16;

/// CQ zone
pub type CqZone = u8;

/// Errors
#[derive(Debug)]
pub struct Error;

impl ClubLog {
    /// Parse xml formatted content of the club log country file.
    pub fn parse(content: &str) -> Result<Self, Error> {
        quick_xml::de::from_str(content).map_err(|_| Error)
    }

    /// Get entity information by adif identifier.
    pub fn get_entity(&self, adif: Adif, timestamp: DateTime<FixedOffset>) -> Option<&Entity> {
        self.entities
            .list
            .iter()
            .find(|e| e.adif == adif && is_in_time_window(timestamp, e.start, e.end))
    }

    /// Get prefix information by callsign prefix.
    pub fn get_prefix(&self, prefix: &str, timestamp: DateTime<FixedOffset>) -> Option<&Prefix> {
        self.prefixes
            .list
            .iter()
            .find(|p| p.call == prefix && is_in_time_window(timestamp, p.start, p.end))
    }

    /// Get callsign exception information by callsign.
    pub fn get_callsign_exception(
        &self,
        callsign: &str,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<&CallsignException> {
        self.exceptions
            .list
            .iter()
            .find(|e| e.call == callsign && is_in_time_window(timestamp, e.start, e.end))
    }

    /// Get cq zone by callsign if an exception for the callsign exists.
    pub fn get_zone_exception(
        &self,
        callsign: &str,
        timestamp: DateTime<FixedOffset>,
    ) -> Option<CqZone> {
        let exc = self
            .zone_exceptions
            .list
            .iter()
            .find(|o| o.call == callsign && is_in_time_window(timestamp, o.start, o.end))?;

        Some(exc.zone)
    }

    /// Check if the callsign was used in an invalid operation.
    pub fn is_invalid_operation(&self, callsign: &str, timestamp: DateTime<FixedOffset>) -> bool {
        self.invalid_operations
            .list
            .iter()
            .find(|o| o.call == callsign && is_in_time_window(timestamp, o.start, o.end))
            .is_some()
    }
}

/// Check wether a timestamp is within an optional start and end timestamp.
fn is_in_time_window(
    timestamp: DateTime<FixedOffset>,
    start: Option<DateTime<FixedOffset>>,
    end: Option<DateTime<FixedOffset>>,
) -> bool {
    match (start, end) {
        (Some(tstart), Some(tend)) => timestamp >= tstart && timestamp <= tend,
        (Some(tstart), None) => timestamp >= tstart,
        (None, Some(tend)) => timestamp <= tend,
        (None, None) => true,
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
    pub xmlns: String,
    /// List of entities
    pub entities: Entities,
    /// List of exceptions
    pub exceptions: CallsignExceptions,
    /// List of prefixes
    pub prefixes: Prefixes,
    /// List of invalid operations
    pub invalid_operations: InvalidOperations,
    /// List of CQ zone exceptions
    pub zone_exceptions: ZoneExceptions,
}

/// List of entities / DXCCs
#[derive(Debug, Deserialize, PartialEq)]
pub struct Entities {
    #[serde(rename = "entity")]
    pub list: Vec<Entity>,
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
/// The list of approved callsigns is part of the [callsign exception](CallsignException) list.
/// May also check the field [whitelist_start](Entity::whitelist_start) after which contacts shall be checked against the whitelist.
/// The timstamp is not necessarily present if a entity is whitelisted.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Entity {
    /// ADIF identifier
    pub adif: Adif,
    /// Name
    pub name: String,
    /// Main callsign prefix
    pub prefix: String,
    /// Entity deleted after [end](Entity::end)
    pub deleted: bool,
    /// CQ zone
    pub cqz: CqZone,
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
    pub whitelist_start: Option<String>,
}

/// List of callsign exceptions
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "Exceptions")]
pub struct CallsignExceptions {
    #[serde(rename = "exception")]
    pub list: Vec<CallsignException>,
}

/// Callsign exception.
///
/// Represents an exceptions to a callsign [prefix](Prefix).
/// When searching for a matching entry the [callsign](CallsignException::call) must match exactly including prefix and suffix.
///
/// An entry may indicate a different value for the fields [adif](CallsignException::adif), [cqz](CallsignException::cqz), [cont](CallsignException::cont), [cont](CallsignException::cont), [lat](CallsignException::lat) or [lat](CallsignException::long) compared to the values of the matching [prefix](Prefix) entry.
/// While searching through the list of exceptions make sure to also validate against the optional [start](CallsignException::start) and [end](CallsignException::end) timestamps.
///
/// Valid callsigns for a [whitelisted entity](Entity::whitelist) are also part of the callsign exception list.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "Exception")]
pub struct CallsignException {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: u16,
    /// Callsign
    pub call: String,
    /// Name of entity
    pub entity: String,
    /// ADIF identifier
    pub adif: Adif,
    /// CQ zone
    pub cqz: CqZone,
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
pub struct Prefixes {
    #[serde(rename = "prefix")]
    pub list: Vec<Prefix>,
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
pub struct Prefix {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: u16,
    /// Callsign
    pub call: String,
    /// Name of entity
    pub entity: String,
    /// ADIF identifier
    pub adif: Option<Adif>, // FIXME: acc. to xsd no option required
    /// CQ zone
    pub cqz: Option<CqZone>, // FIXME: acc. to xsd no option required
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
pub struct InvalidOperations {
    #[serde(rename = "invalid")]
    pub list: Vec<InvalidOperation>,
}

/// Invalid operation.
///
/// An entry represents an invalid operation.
/// When searching for a matching entry the [callsign](InvalidOperation::call) must match exactly including prefix and suffix.
/// Furthermore, check the validity against the optional [start](InvalidOperation::start) and [end](InvalidOperation::end) timestamps.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "Invalid")]
pub struct InvalidOperation {
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
pub struct ZoneExceptions {
    #[serde(rename = "zone_exception")]
    pub list: Vec<ZoneException>,
}

/// CQ zone exception.
///
/// An entry represents a callsign, where the CQ zone of the entity is different.
/// When searching for a matching entry the [callsign](ZoneException::call) must match exactly including prefix and suffix.
/// Furthermore, check the validity against the optional [start](ZoneException::start) and [end](ZoneException::end) timestamps.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ZoneException {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: u16,
    /// Callsign
    pub call: String,
    /// CQ zone
    pub zone: CqZone,
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
    fn check_parser() {
        let clublog = read_clublog_xml();

        assert!(!clublog.xmlns.is_empty());

        assert!(clublog.entities.list.len() > 0);
        assert!(clublog.exceptions.list.len() > 0);
        assert!(clublog.prefixes.list.len() > 0);
        assert!(clublog.invalid_operations.list.len() > 0);
        assert!(clublog.zone_exceptions.list.len() > 0);
    }

    #[test]
    fn lookup_prefix_ok() {
        let clublog = read_clublog_xml();
        let info = clublog
            .get_prefix(
                "DA",
                DateTime::parse_from_rfc3339("2020-01-01T00:00:00+00:00").unwrap(),
            )
            .unwrap();
        assert_eq!(info.adif, Some(230));
    }

    #[test]
    fn lookup_prefix_ok_time() {
        let clublog = read_clublog_xml();
        let y1 = clublog
            .get_prefix(
                "Y2",
                DateTime::parse_from_rfc3339("1980-01-01T00:00:00+00:00").unwrap(),
            )
            .unwrap();
        let y2 = clublog
            .get_prefix(
                "Y2",
                DateTime::parse_from_rfc3339("1995-01-01T00:00:00+00:00").unwrap(),
            )
            .unwrap();

        assert_eq!(y1.adif, Some(229));
        assert_eq!(y2.adif, Some(230));
    }

    #[test]
    fn lookup_prefix_err() {
        let clublog = read_clublog_xml();
        let info = clublog.get_prefix(
            "FOO",
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(info.is_none());
    }

    #[test]
    fn callsign_exception_ok() {
        let clublog = read_clublog_xml();
        let call_exc = clublog.get_callsign_exception(
            "KC6RJW",
            DateTime::parse_from_rfc3339("2003-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(call_exc.is_some());
    }

    #[test]
    fn callsign_exception_err() {
        let clublog = read_clublog_xml();
        let call_exc = clublog.get_callsign_exception(
            "A1B",
            DateTime::parse_from_rfc3339("2001-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(call_exc.is_none());
    }

    #[test]
    fn invalid_operation_ok() {
        let clublog = read_clublog_xml();
        let invalid = clublog.is_invalid_operation(
            "T88A",
            DateTime::parse_from_rfc3339("1995-07-01T00:00:00+00:00").unwrap(),
        );
        assert!(invalid);
    }

    #[test]
    fn invalid_operation_err() {
        let clublog = read_clublog_xml();
        let invalid = clublog.is_invalid_operation(
            "DL1FOO",
            DateTime::parse_from_rfc3339("2001-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(!invalid);
    }

    #[test]
    fn zone_exception_ok() {
        let clublog = read_clublog_xml();
        let exception = clublog.get_zone_exception(
            "KD6WW/VY0",
            DateTime::parse_from_rfc3339("2003-07-30T12:00:00+00:00").unwrap(),
        );
        assert_eq!(exception, Some(1));
    }

    #[test]
    fn zone_exception_err() {
        let clublog = read_clublog_xml();
        let exception = clublog.get_zone_exception(
            "DL1FOO",
            DateTime::parse_from_rfc3339("2001-01-01T00:00:00+00:00").unwrap(),
        );
        assert!(exception.is_none());
    }
}
