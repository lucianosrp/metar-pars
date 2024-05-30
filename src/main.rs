#![allow(dead_code)]

use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Instant;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace1;
use nom::combinator::map_res;
use nom::multi::count;
use nom::sequence::tuple;
use nom::{bytes::complete::take, IResult};

fn take4(s: &str) -> IResult<&str, &str> {
    take(4usize)(s)
}

fn time(s: &str) -> IResult<&str, (Time, &str)> {
    let take2 = take(2usize);
    let time_component = map_res(count(take2, 3), |v| Time::from_vec(v));
    let mut time_parser = tuple((time_component, tag("Z")));
    time_parser(s)
}

fn wind(s: &str) -> IResult<&str, Wind> {
    let t2 = take(2usize);
    let t3 = take(3usize);
    let wind_speed = alt((&t2, &t3));
    let mut parser = map_res(tuple((&t3, wind_speed, &t3)), |(d, s, u)| {
        Wind::new(d, s, u)
    });
    parser(s)
}

fn parse_with_bounds(min: u8, max: u8, s: &str) -> Result<u8, ParseIntError> {
    let d = s.parse::<u8>()?;
    if d >= min && d <= max {
        Ok(d)
    } else {
        panic!("out of bounds")
    }
}

#[derive(Debug)]
enum WindUnit {
    MPS,
    KTS,
}

#[derive(Debug)]
struct Wind {
    direction: u8,
    speed: u8,
    unit: WindUnit,
}
impl Wind {
    fn new(d: &str, s: &str, u: &str) -> Result<Wind, Box<dyn std::error::Error>> {
        Ok(Wind {
            direction: d.parse()?,
            speed: s.parse()?,
            unit: u.parse()?,
        })
    }
}

impl FromStr for WindUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MPS" => Ok(WindUnit::MPS),
            "KTS" => Ok(WindUnit::KTS),
            _ => Err("Not a WindUnit".to_string()),
        }
    }
}

#[derive(Debug)]
struct Time {
    day: u8,
    hour: u8,
    minute: u8,
}

impl Time {
    fn from_vec(v: Vec<&str>) -> Result<Time, ParseIntError> {
        assert!(v.len() == 3);
        let day = parse_with_bounds(1, 31, v[0])?;
        let hour = parse_with_bounds(0, 23, v[1])?;
        let minute = parse_with_bounds(0, 59, v[2])?;
        Ok(Time { day, hour, minute })
    }
}

#[derive(Debug)]
struct METAR {
    station: String,
    time: Time,
    wind: Wind,
}

impl METAR {
    fn parse(s: &str) -> METAR {
        let res = tuple((take4, multispace1, time, multispace1, wind))(s).unwrap();
        METAR {
            station: res.1 .0.to_string(),
            time: res.1 .2 .0,
            wind: res.1 .4,
        }
    }
}

fn main() {
    let s = Instant::now();
    let sample = "LICJ 141600Z 12012MPS 090V150 1400 R04/P1500N R22/P1500U +SN BKN022 OVC050 M04/M07 Q1020 NOSIG 8849//91=";
    let metar = METAR::parse(&sample);
    println!("{:?}", metar);
    println!("{:?}", s.elapsed());
}
