// ============================================================================
// Filename: licensegen/src/util/datetime.rs
// Author: Leon McClatchey
// Company: Linktech Engineering LLC
// Created: 2026-03-02
// Modified: 2026-03-03
// Description: 
// ============================================================================


use chrono::{NaiveDate, NaiveDateTime, Datelike, Timelike};
use mysql_common::value::Value;

pub fn from_naive_date(d: NaiveDate) -> Value {
    Value::Date(
        d.year() as u16,
        d.month() as u8,
        d.day() as u8,
        0, 0, 0, 0,
    )
}
pub fn from_naive_datetime(dt: NaiveDateTime) -> Value {
    Value::Date(
        dt.year() as u16,
        dt.month() as u8,
        dt.day() as u8,
        dt.hour() as u8,
        dt.minute() as u8,
        dt.second() as u8,
        0,
    )
}

pub fn opt_i32(v: Option<i32>) -> Option<Value> {
    v.map(|x| x.into())
}

pub fn to_naive_date(v: Value) -> NaiveDate {
    match v {
        Value::Date(year, month, day, ..) => {
            NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .expect("invalid date")
        }
        _ => panic!("Expected MySQL DATE value"),
    }
}

pub fn to_naive_datetime(v: Value) -> NaiveDateTime {
    match v {
        Value::Date(year, month, day, hour, min, sec, micros) => {
            NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .unwrap()
                .and_hms_micro_opt(hour as u32, min as u32, sec as u32, micros)
                .unwrap()
        }
        _ => panic!("Expected MySQL DATETIME value"),
    }
}
