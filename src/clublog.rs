// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Implementation of a parser based on deserialization for the ClubLog XML based entity and callsign information.
//! Next to that, the module provides a few basic methods to query information from the parsed data.
//!
//! The example `clublog.rs` shows the basic usage of this module.

use crate::clublogquery::ClubLogQuery;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use std::vec::Vec;

/// ADIF DXCC identifier
pub type Adif = u16;

/// CQ zone
pub type CqZone = u8;

/// Record identifier
pub type RecordId = u16;

/// Special value for the entity of a callsign exception that is invalid
pub const CALLSIGN_EXCEPTION_INVALID: &str = "INVALID";

/// Special value for the entity of a callsign exception that is used maritime mobile
pub const CALLSIGN_EXCEPTION_MARITIME_MOBILE: &str = "MARITIME MOBILE";

/// Special value for the entity of a callsign exception that is aeronautical mobile only
pub const CALLSIGN_EXCEPTION_AERONAUTICAL_MOBILE: &str = "AERONAUTICAL MOBILE";

/// Special value for the entity of a callsign exception that is satellite, internet or repeater only
pub const CALLSIGN_EXCEPTION_SATELLITE: &str = "SATELLITE, INTERNET OR REPEATER";

/// Special ADIF identifier representing an unknown entity
pub const ADIF_ID_NO_DXCC: Adif = 0;

/// Errors
#[derive(Debug)]
pub struct Error;

impl ClubLogQuery for ClubLog {
    fn get_entity(&self, adif: Adif, timestamp: &DateTime<Utc>) -> Option<&Entity> {
        self.entities
            .list
            .iter()
            .find(|e| e.adif == adif && is_in_time_window(timestamp, e.start, e.end))
    }
    fn get_prefix(&self, prefix: &str, timestamp: &DateTime<Utc>) -> Option<&Prefix> {
        self.prefixes
            .list
            .iter()
            .find(|p| p.call == prefix && is_in_time_window(timestamp, p.start, p.end))
    }
    fn get_callsign_exception(
        &self,
        callsign: &str,
        timestamp: &DateTime<Utc>,
    ) -> Option<&CallsignException> {
        self.exceptions
            .list
            .iter()
            .find(|e| e.call == callsign && is_in_time_window(timestamp, e.start, e.end))
    }
    fn get_zone_exception(&self, callsign: &str, timestamp: &DateTime<Utc>) -> Option<CqZone> {
        let exc = self
            .zone_exceptions
            .list
            .iter()
            .find(|o| o.call == callsign && is_in_time_window(timestamp, o.start, o.end))?;

        Some(exc.zone)
    }
    fn is_invalid_operation(&self, callsign: &str, timestamp: &DateTime<Utc>) -> bool {
        self.invalid_operations
            .list
            .iter()
            .any(|o| o.call == callsign && is_in_time_window(timestamp, o.start, o.end))
    }
}

impl ClubLog {
    /// Parse XML formatted content of the ClubLog data file.
    ///
    /// # Arguments
    ///
    /// - `content`: Content of the data file
    ///
    /// # Returns
    ///
    /// Parsed ClubLog data or an error
    pub fn parse(content: &str) -> Result<Self, Error> {
        quick_xml::de::from_str(content).map_err(|_| Error)
    }
}

/// Check whether a timestamp is within an optional start and end time range.
///
/// # Arguments
///
/// - `timestamp`: Timestamp to use for the check
/// - `start`: Start timestamp of the time window
/// - `end`: End timestamp of the time window
///
/// # Returns
///
/// True if time timestamp is within the time window, false otherwise
pub fn is_in_time_window(
    timestamp: &DateTime<Utc>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> bool {
    match (start, end) {
        (Some(tstart), Some(tend)) => timestamp >= &tstart && timestamp <= &tend,
        (Some(tstart), None) => timestamp >= &tstart,
        (None, Some(tend)) => timestamp <= &tend,
        (None, None) => true,
    }
}

/// Custom XML deserializer for a timestamp
///
/// # Arguments
///
/// - `deserializer`: Deserializer
///
/// # Returns
///
/// Timestamp or an error
fn parse_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&s)
        .map(|d| d.into())
        .map_err(serde::de::Error::custom)
}

/// Custom XML deserializer for an optional timestamp
///
/// # Arguments
///
/// - `deserializer`: Deserializer
///
/// # Returns
///
/// Optional timestamp or an error
fn parse_datetime_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Ok(Some(
        DateTime::parse_from_rfc3339(&s)
            .map(|d| d.into())
            .map_err(serde::de::Error::custom)?,
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
    pub date: DateTime<Utc>,
    /// List of entities
    pub entities: Entities,
    /// List of callsign exceptions
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
/// May also have a look at the timestamps [whitelist_start](Entity::whitelist_start) and [whitelist_end](Entity::whitelist_end) to check whether a whitelist check is required or not.
/// Note, that the whitlist timstamps are not necessarily present if a entity is whitelisted.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Entity {
    /// ADIF identifier
    pub adif: Adif,
    /// Name
    pub name: String,
    /// Main callsign prefix
    pub prefix: String,
    /// Entity deleted/invalid
    pub deleted: bool,
    /// CQ zone
    pub cqz: Option<CqZone>,
    /// Continent
    pub cont: Option<String>,
    /// Longitude
    pub long: Option<f32>,
    /// Latitude
    pub lat: Option<f32>,
    /// Start timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<Utc>>,
    /// End timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<Utc>>,
    /// True if only whitelisted of callsigns are valid for this entity
    pub whitelist: Option<bool>,
    /// Timestamp after which the whitelist shall be used
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub whitelist_start: Option<DateTime<Utc>>,
    /// Timestamp after which the whitelist shall not be used anymore
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub whitelist_end: Option<DateTime<Utc>>,
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
/// An entry represents an exception to a callsigns actual [prefix](Prefix).
/// When searching for a matching entry, the [callsign](CallsignException::call) must match exactly including prefix, suffix and appendix.
///
/// An entry indicates a different value for the field [adif](CallsignException::adif)
/// The fields [cqz](CallsignException::cqz), [cont](CallsignException::cont), [cont](CallsignException::cont), [lat](CallsignException::lat) or [lat](CallsignException::long) are optional and may contain different information compared to the values of the matching [prefix](Prefix) entry.
/// While searching through the list of exceptions make sure to also validate against the optional [start](CallsignException::start) and [end](CallsignException::end) timestamps.
///
/// A few callsign exceptions refer in their [entity](CallsignException::entity) field special entity names for [maritime mobile](CALLSIGN_EXCEPTION_MARITIME_MOBILE), [aeronautical mobile](CALLSIGN_EXCEPTION_AERONAUTICAL_MOBILE) and [satellite, internet or repeater](CALLSIGN_EXCEPTION_SATELLITE).
/// Within those entries the [adif](CallsignException::adif) field is set to zero (see also [ADIF_ID_NO_DXCC]) according to the ADIF specification.
/// Note that the special zero adif identifier is not part of the [entity list](Entities).
///
/// The [entity](CallsignException::entity) field may also contain the string [INVALID](CALLSIGN_EXCEPTION_INVALID),
/// In this special case the callsign is invalid.
/// The information, if a call is invalid is also part of the [invalid operations list](InvalidOperations).
/// There are historical reasons, why the same information is part of two lists.
///
/// Note: Valid callsigns for a [whitelisted entity](Entity::whitelist) are also part of the callsign exception list.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "Exception")]
pub struct CallsignException {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: RecordId,
    /// Callsign
    pub call: String,
    /// Name of entity
    pub entity: String,
    /// ADIF identifier
    pub adif: Adif,
    /// CQ zone
    pub cqz: Option<CqZone>,
    /// Continent
    pub cont: Option<String>,
    /// Longitude
    pub long: Option<f32>,
    /// Latitude
    pub lat: Option<f32>,
    /// Start timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<Utc>>,
    /// End timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<Utc>>,
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
/// For example the prefixes `DL` and `DO` do both refer to the same DXCC `FEDERAL REPUBLIC OF GERMANY`.
/// Even all other fields of that two entries feature the same data.
/// While searching for a matching prefix make sure to also validate against the optional [start](Prefix::start) and [end](Prefix::end) timestamps.
///
/// Note: While searching for a prefix, next to obvious prefixes like `DL`, there are also speical ones listed like `SV/A`.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Prefix {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: RecordId,
    /// Callsign
    pub call: String,
    /// Name of entity
    pub entity: String,
    /// ADIF identifier
    pub adif: Adif,
    /// CQ zone
    pub cqz: Option<CqZone>,
    /// Continent
    pub cont: Option<String>,
    /// Longitude
    pub long: Option<f32>,
    /// Latitude
    pub lat: Option<f32>,
    /// Start timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<Utc>>,
    /// End timestamp of validity
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<Utc>>,
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
/// When searching for a matching entry the [callsign](InvalidOperation::call) must match exactly including prefix, suffix and appendix.
/// Furthermore, check the validity against the optional [start](InvalidOperation::start) and [end](InvalidOperation::end) timestamps.
///
/// Note: this information is for historical reasons also part of the [callsign exceptions](CallsignException).
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "Invalid")]
pub struct InvalidOperation {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: RecordId,
    /// Callsign
    pub call: String,
    /// Start timestamp of operation
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<Utc>>,
    /// End timestamp of operation
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<Utc>>,
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
/// When searching for a matching entry the [callsign](ZoneException::call) must match exactly including prefix, suffix and appendix.
/// Furthermore, check the validity against the optional [start](ZoneException::start) and [end](ZoneException::end) timestamps.
#[derive(Debug, Deserialize, PartialEq)]
pub struct ZoneException {
    /// Identifier
    #[serde(rename = "@record")]
    pub record: RecordId,
    /// Callsign
    pub call: String,
    /// CQ zone
    pub zone: CqZone,
    /// Start timestamp of exception
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub start: Option<DateTime<Utc>>,
    /// End timestamp of exception
    #[serde(default)]
    #[serde(deserialize_with = "parse_datetime_opt")]
    pub end: Option<DateTime<Utc>>,
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
                &DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
        assert_eq!(info.adif, 230);
    }

    #[test]
    fn lookup_prefix_ok_time() {
        let clublog = read_clublog_xml();
        let y1 = clublog
            .get_prefix(
                "Y2",
                &DateTime::parse_from_rfc3339("1980-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();
        let y2 = clublog
            .get_prefix(
                "Y2",
                &DateTime::parse_from_rfc3339("1995-01-01T00:00:00Z")
                    .unwrap()
                    .into(),
            )
            .unwrap();

        assert_eq!(y1.adif, 229);
        assert_eq!(y2.adif, 230);
    }

    #[test]
    fn lookup_prefix_err() {
        let clublog = read_clublog_xml();
        let info = clublog.get_prefix(
            "FOO",
            &DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        assert!(info.is_none());
    }

    #[test]
    fn callsign_exception_ok() {
        let clublog = read_clublog_xml();
        let call_exc = clublog.get_callsign_exception(
            "KC6RJW",
            &DateTime::parse_from_rfc3339("2003-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        assert!(call_exc.is_some());
    }

    #[test]
    fn callsign_exception_err() {
        let clublog = read_clublog_xml();
        let call_exc = clublog.get_callsign_exception(
            "A1B",
            &DateTime::parse_from_rfc3339("2001-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        assert!(call_exc.is_none());
    }

    #[test]
    fn invalid_operation_ok() {
        let clublog = read_clublog_xml();
        let invalid = clublog.is_invalid_operation(
            "T88A",
            &DateTime::parse_from_rfc3339("1995-07-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        assert!(invalid);
    }

    #[test]
    fn invalid_operation_err() {
        let clublog = read_clublog_xml();
        let invalid = clublog.is_invalid_operation(
            "DL1FOO",
            &DateTime::parse_from_rfc3339("2001-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        assert!(!invalid);
    }

    #[test]
    fn zone_exception_ok() {
        let clublog = read_clublog_xml();
        let exception = clublog.get_zone_exception(
            "KD6WW/VY0",
            &DateTime::parse_from_rfc3339("2003-07-30T12:00:00Z")
                .unwrap()
                .into(),
        );
        assert_eq!(exception, Some(1));
    }

    #[test]
    fn zone_exception_err() {
        let clublog = read_clublog_xml();
        let exception = clublog.get_zone_exception(
            "DL1FOO",
            &DateTime::parse_from_rfc3339("2001-01-01T00:00:00Z")
                .unwrap()
                .into(),
        );
        assert!(exception.is_none());
    }
}
