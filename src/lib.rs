// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This crate provides two modules to work with ham radio callsigns.
//! The first module [clublog] implements a parser for the ClubLog XML data.
//! Based an that data the module [call] provides an analyzer for a callsign to get further information like the entity or the continent.

pub mod call;
pub mod clublog;
