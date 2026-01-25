use std::time::Duration;
use std::sync::OnceLock;
use serde::{Serialize};
use serde::de::{self};
use regex::Regex;

/*
 *
 *  Following Rust's EBNF example
 *  
 *  Digit  ::= [0-9]
 *  Sign   ::= [+-]
 *  Exp    ::= 'e' Sign? Digit+
 *  Number ::= (
 *                Digit+
 *                  |
 *                Digit+ '.' Digit*
 *                  |
 *                Digit* '.' Digit+
 *             ) Exp?
 *  Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
 *  
 *  Meaning as a regex we have
 *
 *  Digit  ::= '[0-9]'
 *  Sign   ::= '[+-]'
 *  Exp    ::= 'e[+-]?[0-9]+'
 *  Number ::= (?:
 *                (?:[0-9]*[.][0-9]+)
 *                  |
 *                (?:[0-9]+[.][0-9]*)
 *                  |
 *                (?:[0-9]+)
 *             )(?:e[+-]?[0-9]+)?
 *  Float  ::= (?:[+-]?(?:infinity|inf|nan|(?:(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)))
 *
 *
 *  Since we only care about a subset of that
 *
 *    (?:(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)
 *
 *  From here we add units
 *
 *
 *    (?:(?<days>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:days|day|d))
 *    (?:(?<hours>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:hours|hrs|hr|h))
 *    (?:(?<minutes>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:minutes|mins|min|m))
 *    (?:(?<seconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:seconds|secs|sec|s))
 *    (?:(?<milliseconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:milliseconds|millis|ms))
 *    (?:(?<nanoseconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:nanoseconds|ns))
 *
 *    TODO: add ticks
 *    (?:(?<ticks>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:ticks|cycles|t))
 *
 *  Combine into a single string
 *
 *    ^(?:(?<days>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:days|day|d))\s*(?:(?<hours>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:hours|hrs|hr|h))\s*(?:(?<minutes>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:minutes|mins|min|m))\s*(?:(?<seconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:seconds|secs|sec|s))\s*(?:(?<milliseconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:milliseconds|millis|ms))\s*(?:(?<nanoseconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:nanoseconds|ns))\s*$
 *
 *  There ya go
 *
 */

static DATE_TIME_LOCK: OnceLock<Regex> = OnceLock::new();

fn get_date_time_regex() -> &'static Regex {
    DATE_TIME_LOCK.get_or_init(|| {
        Regex::new(r#"^(?:(?<days>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:days|day|d))?\s*(?:(?<hours>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:hours|hrs|hr|h))?\s*(?:(?<minutes>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:minutes|mins|min|m))?\s*(?:(?<seconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:seconds|secs|sec|s))?\s*(?:(?<milliseconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:milliseconds|millis|ms))?\s*(?:(?<nanoseconds>(?:(?:[0-9]*[.][0-9]+)|(?:[0-9]+[.][0-9]*)|(?:[0-9]+))(?:e[+-]?[0-9]+)?)(?:nanoseconds|ns))?\s*$"#).unwrap()
    })
}

/// NiceDuration is a wrapper around `std::time::Duration` to allow for Durations to be built from
/// human readable strings.
#[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Eq,Ord,Hash,Default)]
pub struct NiceDuration {
    data: Duration,
}
impl PartialEq<Duration> for NiceDuration {
    fn eq(&self, other: &Duration) -> bool {
        self.data.eq(other)
    }
}
impl From<Duration> for NiceDuration {
    fn from(data: Duration) -> Self { Self { data } }
}
impl NiceDuration {

    /// returns standard duration
    pub fn get_duration(&self) -> Duration {
        self.data.clone()
    }

    fn to_parts(&self) -> (u64,u64,u64,u64,u64,u64) {
        let mut secs = self.data.as_secs();
        let days = secs / 86_400u64;
        secs -= days * 86_400u64;
        let hours = secs / 3_600u64;
        secs -= hours * 3_600u64;
        let minutes = secs / 60u64;
        secs -= minutes * 60u64;
        let seconds = secs;

        let ms = (self.data.as_millis() - (self.data.as_secs() as u128 * 1000u128)) as u64;
        let ns = (self.data.as_nanos() - (self.data.as_millis() * 1_000_000u128)) as u64;

        (days,hours,minutes,seconds,ms,ns)
    }
    fn to_le(&self) -> [u8;16] {
        self.data.as_nanos().to_le_bytes()
    }
    fn from_le(arg: &[u8]) -> Self {
        let mut value = [0u8;16];
        for (idx, v) in arg.iter().enumerate().filter(|(idx,_)| *idx < 16) {
            value[idx] = *v;
        } 
        let mut value = u128::from_le_bytes(value);
        // integer division truncates
        let seconds = value / 1_000_000_000u128;
        let real_seconds = seconds as u64;
        value -= seconds * 1_000_000_000u128;
        let real_nanos = value as u32;
        Self { data: Duration::new(real_seconds, real_nanos) }
    }
}

impl Serialize for NiceDuration {
    fn serialize<S>(&self, s: S) -> Result<S::Ok,S::Error>
    where
        S: serde::ser::Serializer,
    {
        if s.is_human_readable() {
            let (days,hours,minutes,seconds,ms,ns) = self.to_parts();
            let mut buff = String::new();
            if days > 0 {
                std::fmt::write(&mut buff, format_args!("{}d", days)).unwrap();
            }
            if hours > 0 {
                std::fmt::write(&mut buff, format_args!("{}h", hours)).unwrap();
            }
            if minutes > 0 {
                std::fmt::write(&mut buff, format_args!("{}m", minutes)).unwrap();
            }
            if seconds > 0 {
                std::fmt::write(&mut buff, format_args!("{}s", seconds)).unwrap();
            }
            if ms > 0 {
                std::fmt::write(&mut buff, format_args!("{}ms", ms)).unwrap();
            }
            if ns > 0 {
                std::fmt::write(&mut buff, format_args!("{}ns", ns)).unwrap();
            }
            s.serialize_str(&buff)
        } else {
            let arg: [u8;16] = self.to_le();
            let slice = weird(&arg);
            s.serialize_bytes(slice)
        }
    }
}

impl<'de> de::Deserialize<'de> for NiceDuration {
    fn deserialize<D>(d: D) -> Result<Self,D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if d.is_human_readable() {
            d.deserialize_string(NiceDurationVisitor::default())
        } else {
            d.deserialize_bytes(NiceDurationVisitor::default())
        }
    }
}

fn deserialize_str(arg: &str) -> Option<Duration> {
    let caps = get_date_time_regex().captures(arg)?;
    let (days, fract_days) = read_value(&caps, "days", 0.0);
    let (hours, fract_hours) = read_value(&caps, "hours", fract_days * 24.0);
    let (mins, fract_mins) = read_value(&caps, "minutes", fract_hours * 60.0);
    let (secs, fract_secs) = read_value(&caps, "seconds", fract_mins * 60.0);
    let whole_seconds = Duration::from_secs(
        days * 86_400
            +
        hours * 3_600
            +
        mins * 60
            +
        secs
    );

    let (ms, fract_ms) = read_value(&caps, "milliseconds", fract_secs * 1000.0);
    let (ns, _) = read_value(&caps, "nanoseconds", fract_ms * 1_000_000.0);
    let fract = Duration::from_nanos(ms * 1_000_000 + ns);

    Some(whole_seconds + fract)
}

fn read_value(arg: &regex::Captures<'_>, name: &str, residue: f64) -> (u64,f64) {
    use std::str::FromStr;

    let (res_whole, res_fract) = {
        let whole = residue.abs().floor() as u64;
        let fract = residue.abs().fract();
        (whole,fract)
    };
    let (val_whole,val_fract) = match arg.name(name) {
        Option::None => (0u64,0.0f64),
        Option::Some(cap) => {
            let s = cap.as_str().trim();
            match u64::from_str(s) {
                Ok(x) => {
                    (x,0.0f64)
                }
                Err(_) => {
                    let x = f64::from_str(s).unwrap();
                    let whole = x.abs().floor() as u64;
                    let fract = x.abs().fract();
                    (whole,fract)
                }
            }
        }
    };

    let new_fract = res_fract + val_fract;
    let update_whole = new_fract.floor() as u64;
    let final_fract = new_fract.fract();

    (val_whole + res_whole + update_whole, final_fract)
}

#[derive(Default)]
struct NiceDurationVisitor {
    _idk: usize,
}
impl<'de> de::Visitor<'de> for NiceDurationVisitor {
    type Value = NiceDuration;
    fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("a string containing numbers following a marker of the span (days|hours|seconds|minutes|milliseconds|)")
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value,E> {
        Ok(NiceDuration::from_le(v))
    }
    fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<Self::Value,E> {
        Ok(NiceDuration::from_le(v))
    }
    fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value,E> {
        Ok(NiceDuration::from_le(v.as_slice()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value,E> {
        deserialize_str(&v)
            .map(NiceDuration::from)
            .ok_or_else(|| E::custom(format!("input string: '{}' did not match regexp needed to make sense of duration", v)))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value,E> {
        deserialize_str(v)
            .map(NiceDuration::from)
            .ok_or_else(|| E::custom(format!("input string: '{}' did not match regexp needed to make sense of duration", v)))
    }
    fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value,E> {
        deserialize_str(v)
            .map(NiceDuration::from)
            .ok_or_else(|| E::custom(format!("input string: '{}' did not match regexp needed to make sense of duration", v)))
    }
}

#[test]
fn parse_duration() {
    const DUT: &'static [ (&'static str, Duration) ] = &[
        ("5ms", Duration::from_millis(5)),
        ("10hrs", Duration::from_secs( 10 * 60 * 60 )),
        ("2s", Duration::from_secs(2)),
        ("10h5s15ms", Duration::from_millis( (10 * 60 * 60 * 1000) + (5 * 1000) + 15 )),
    ];
   
    for (s,expected) in DUT {
        let output = deserialize_str(s).unwrap();
        assert_eq!(output,*expected);
    }
}

fn weird<'a>(arr: &'a [u8;16]) -> &'a [u8] {
    let mut i = 15usize;
    for idx in (0..=15usize).rev() {
        if arr[idx] == 0 {
            i -= 1;
            continue;
        } else {
            break;
        }
    }
    &arr[0..=i]
}
#[test]
fn test_weird_slicing() {
    const DUT: &'static [Duration] = &[
        Duration::from_millis(5),
        Duration::from_secs( 10 * 60 * 60 ),
        Duration::from_secs(2),
        Duration::from_millis( (10 * 60 * 60 * 1000) + (5 * 1000) + 15 ),
    ];

    for dur in DUT.iter() {
        let base: Duration = dur.clone();
        let nice: NiceDuration = NiceDuration::from(dur.clone());
        assert_eq!(nice, base);

        let out = nice.to_le();
        let nice_2: NiceDuration = NiceDuration::from_le(&out);
        assert_eq!(nice, nice_2);
    }
}
