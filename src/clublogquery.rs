// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Trait definition on how to access ClubLog data.

use crate::clublog::{Adif, CallsignException, CqZone, Entity, Prefix};
use chrono::{DateTime, Utc};

/// Definitions on how to access ClubLog data
pub trait ClubLogQuery {
    /// Get entity information by adif identifier.
    ///
    /// # Arguments
    ///
    /// - `adif`: ADIF identifier
    /// - `timestamp`: Timestamp to use for the check
    ///
    /// # Returns
    ///
    /// Entity information, if present
    fn get_entity(&self, adif: Adif, timestamp: &DateTime<Utc>) -> Option<&Entity>;

    /// Get prefix information by callsign prefix.
    ///
    /// # Arguments
    ///
    /// - `prefix`: Callsigns prefix, like `DL`
    /// - `timestamp`: Timestamp to use for the check
    ///
    /// # Returns
    ///
    /// Prefix information, if present
    fn get_prefix(&self, prefix: &str, timestamp: &DateTime<Utc>) -> Option<&Prefix>;

    /// Get callsign exception information by callsign.
    ///
    /// # Arguments
    ///
    /// - `callsign`: Complete callsign
    /// - `timestamp`: Timestamp to use for the check
    ///
    /// # Returns
    ///
    /// Callsign exception information, if present
    fn get_callsign_exception(
        &self,
        callsign: &str,
        timestamp: &DateTime<Utc>,
    ) -> Option<&CallsignException>;

    /// Get cq zone by callsign if an exception for the callsign exists.
    ///
    /// # Arguments
    ///
    /// - `callsign`: Complete callsign
    /// - `timestamp`: Timestamp to use for the check
    ///
    /// # Returns
    ///
    /// CQ zone exception, if present
    fn get_zone_exception(&self, callsign: &str, timestamp: &DateTime<Utc>) -> Option<CqZone>;

    /// Check if the callsign was used in an invalid operation.
    ///
    /// # Arguments
    ///
    /// - `callsign`: Complete callsign
    /// - `timestamp`: Timestamp to use for the check
    ///
    /// # Returns
    ///
    /// True if the operation is invalid, false otherwise
    fn is_invalid_operation(&self, callsign: &str, timestamp: &DateTime<Utc>) -> bool;
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
