// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! HashMap based implementation of the [ClubLogQuery] trait.

use crate::clublog::{
    Adif, CallsignException, ClubLog, CqZone, Entity, InvalidOperation, Prefix, ZoneException,
};
use crate::clublogquery::{is_in_time_window, ClubLogQuery};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::convert::From;
use std::vec::Vec;

/// HashMap based implementation of the [ClubLogQuery] trait
pub struct ClubLogMap {
    entities: HashMap<Adif, Entity>,
    prefixes: HashMap<String, Vec<Prefix>>,
    callsign_exceptions: HashMap<String, Vec<CallsignException>>,
    invalid_operations: HashMap<String, Vec<InvalidOperation>>,
    zone_exceptions: HashMap<String, Vec<ZoneException>>,
}

impl From<ClubLog> for ClubLogMap {
    fn from(clublog: ClubLog) -> Self {
        let mut entities: HashMap<Adif, Entity> = HashMap::new();
        for entity in clublog.entities.list.into_iter() {
            entities.insert(entity.adif, entity);
        }

        let mut callsign_exceptions: HashMap<String, Vec<CallsignException>> = HashMap::new();
        for exception in clublog.exceptions.list.into_iter() {
            if let Some(value) = callsign_exceptions.get_mut(&exception.call) {
                value.push(exception);
            } else {
                callsign_exceptions.insert(exception.call.clone(), vec![exception]);
            }
        }

        let mut prefixes: HashMap<String, Vec<Prefix>> = HashMap::new();
        for prefix in clublog.prefixes.list.into_iter() {
            if let Some(value) = prefixes.get_mut(&prefix.call) {
                value.push(prefix);
            } else {
                prefixes.insert(prefix.call.clone(), vec![prefix]);
            }
        }

        let mut invalid_operations: HashMap<String, Vec<InvalidOperation>> = HashMap::new();
        for invalid_operation in clublog.invalid_operations.list.into_iter() {
            if let Some(value) = invalid_operations.get_mut(&invalid_operation.call) {
                value.push(invalid_operation);
            } else {
                invalid_operations.insert(invalid_operation.call.clone(), vec![invalid_operation]);
            }
        }

        let mut zone_exceptions: HashMap<String, Vec<ZoneException>> = HashMap::new();
        for zone_exception in clublog.zone_exceptions.list.into_iter() {
            if let Some(value) = zone_exceptions.get_mut(&zone_exception.call) {
                value.push(zone_exception);
            } else {
                zone_exceptions.insert(zone_exception.call.clone(), vec![zone_exception]);
            }
        }

        ClubLogMap {
            entities,
            callsign_exceptions,
            prefixes,
            invalid_operations,
            zone_exceptions,
        }
    }
}

impl ClubLogQuery for ClubLogMap {
    fn get_entity(&self, adif: Adif, timestamp: &DateTime<Utc>) -> Option<&Entity> {
        if let Some(entity) = self.entities.get(&adif) {
            if is_in_time_window(timestamp, entity.start, entity.end) {
                return Some(entity);
            }
        }
        None
    }

    fn get_prefix(&self, prefix: &str, timestamp: &DateTime<Utc>) -> Option<&Prefix> {
        self.prefixes
            .get(prefix)?
            .iter()
            .find(|p| is_in_time_window(timestamp, p.start, p.end))
    }

    fn get_callsign_exception(
        &self,
        callsign: &str,
        timestamp: &DateTime<Utc>,
    ) -> Option<&CallsignException> {
        self.callsign_exceptions
            .get(callsign)?
            .iter()
            .find(|ce| is_in_time_window(timestamp, ce.start, ce.end))
    }

    fn get_zone_exception(&self, callsign: &str, timestamp: &DateTime<Utc>) -> Option<CqZone> {
        self.zone_exceptions
            .get(callsign)?
            .iter()
            .find(|ze| is_in_time_window(timestamp, ze.start, ze.end))
            .map(|ze| ze.zone)
    }

    fn is_invalid_operation(&self, callsign: &str, timestamp: &DateTime<Utc>) -> bool {
        self.invalid_operations.get(callsign).map_or(false, |io| {
            io.iter()
                .any(|io| is_in_time_window(timestamp, io.start, io.end))
        })
    }
}
