//! This crate provides two modules to work with ham radio callsigns.
//! The first module [clublog](clublog) implements a parser for the ClubLog XML data.
//! Based an that data the module [call](call) provides an analyzer for a callsign to get further information like the entity or the continent.

pub mod call;
pub mod clublog;
