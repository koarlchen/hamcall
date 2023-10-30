// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This crate provides a few modules to work with ham radio callsigns.
//! The first module [clublog] implements a parser for the ClubLog XML data and further implements the [ClubLogQuery](clublogquery::ClubLogQuery) trait.
//! For faster access, the module [clublogmap] implements the trait based on HashMaps.
//! Using the trait, the module [call] provides an analyzer for a callsign to get further information like the entity or the continent.

pub mod call;
pub mod clublog;
pub mod clublogmap;
pub mod clublogquery;
