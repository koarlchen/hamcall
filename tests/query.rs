use chrono::DateTime;
use hamcall::clublog::ClubLog;
use hamcall::clublogmap::ClubLogMap;
use hamcall::clublogquery::ClubLogQuery;
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
fn lookup_prefix_ok() {
    let clublog = read_clublog_xml();
    lookup_prefix_ok_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    lookup_prefix_ok_impl(&clublog);
}

fn lookup_prefix_ok_impl(clublog: &dyn ClubLogQuery) {
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
    lookup_prefix_ok_time_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    lookup_prefix_ok_time_impl(&clublog);
}

fn lookup_prefix_ok_time_impl(clublog: &dyn ClubLogQuery) {
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
    lookup_prefix_err_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    lookup_prefix_err_impl(&clublog);
}

fn lookup_prefix_err_impl(clublog: &dyn ClubLogQuery) {
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
    callsign_exception_ok_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    callsign_exception_ok_impl(&clublog);
}

fn callsign_exception_ok_impl(clublog: &dyn ClubLogQuery) {
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
    callsign_exception_err_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    callsign_exception_err_impl(&clublog);
}

fn callsign_exception_err_impl(clublog: &dyn ClubLogQuery) {
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
    invalid_operation_ok_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    invalid_operation_ok_impl(&clublog);
}

fn invalid_operation_ok_impl(clublog: &dyn ClubLogQuery) {
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
    invalid_operation_err_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    invalid_operation_err_impl(&clublog);
}

fn invalid_operation_err_impl(clublog: &dyn ClubLogQuery) {
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
    zone_exception_ok_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    zone_exception_ok_impl(&clublog);
}

fn zone_exception_ok_impl(clublog: &dyn ClubLogQuery) {
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
    zone_exception_err_impl(clublog);
    let clublog = ClubLogMap::from(clublog.clone());
    zone_exception_err_impl(&clublog);
}

fn zone_exception_err_impl(clublog: &dyn ClubLogQuery) {
    let exception = clublog.get_zone_exception(
        "DL1FOO",
        &DateTime::parse_from_rfc3339("2001-01-01T00:00:00Z")
            .unwrap()
            .into(),
    );
    assert!(exception.is_none());
}
