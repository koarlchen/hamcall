use std::vec::Vec;
use serde::Deserialize;

#[derive(Debug)]
pub struct Error;

pub fn parse(content: &str) -> Result<Clublog, Error> {
    quick_xml::de::from_str(content).map_err(|_| Error)
}

#[derive(Debug, Deserialize)]
#[serde(rename = "clublog")]
pub struct Clublog {
    #[serde(rename = "@date")]
    pub date: String, // actually datetime
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    pub entities: Entities,
    pub exceptions: Exceptions,
    pub prefixes: Prefixes,
    pub invalid_operations: InvalidOperations,
    pub zone_exceptions: ZoneExceptions,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Entities {
    pub entity: Vec<Entity>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Entity {
    pub adif: u16,
    pub name: String,
    pub prefix: String,
    pub deleted: bool,
    pub cqz: u8,
    pub cont: String,
    pub long: f32,
    pub lat: f32,
    pub start: Option<String>, // TODO: actually datetime
    pub end: Option<String>, // TODO: actually datetime
    pub whitelist: Option<bool>,
    pub whitelist_start: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Exceptions {
    pub exception: Vec<Exception>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Exception {
    #[serde(rename = "@record")]
    pub record: u16,
    pub call: String,
    pub entity: String,
    pub adif: u16,
    pub cqz: u8,
    pub cont: String,
    pub long: f32,
    pub lat: f32,
    pub start: Option<String>, // TODO: actually datetime
    pub end: Option<String>, // TODO: actually datetime
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Prefixes {
    pub prefix: Vec<Prefix>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Prefix {
    #[serde(rename = "@record")]
    pub record: u16,
    pub call: String,
    pub entity: String,
    pub adif: Option<u16>, // FIXME: acc. to xsd no option required
    pub cqz: Option<u8>, // FIXME: acc. to xsd no option required
    pub cont: Option<String>, // FIXME: acc. to xsd no option required
    pub long: Option<f32>, // FIXME: acc. to xsd no option required
    pub lat: Option<f32>, // FIXME: acc. to xsd no option required
    pub start: Option<String>, // TODO: actually datetime
    pub end: Option<String>, // TOOO: actually datetime
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct InvalidOperations {
    pub invalid: Vec<Invalid>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Invalid {
    #[serde(rename = "@record")]
    pub record: u16,
    pub call: String,
    pub start: Option<String>, // TODO: actually datetime
    pub end: Option<String>, // TODO: actually datetime
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ZoneExceptions {
    pub zone_exception: Vec<ZoneException>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ZoneException {
    #[serde(rename = "@record")]
    pub record: u16,
    pub call: String,
    pub zone: u8,
    pub start: Option<String>, // TODO: actually datetime
    pub end: Option<String>, // TODO: actually datetime
}