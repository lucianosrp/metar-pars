use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map_res, opt};
use nom::multi::count;
use nom::sequence::tuple;
use nom::{bytes::complete::take, IResult};
use visibility::{parse_visibility, Visibility};
use wind::{parse_wind, Wind};
pub mod visibility;
pub mod wind;

fn parse_with_bounds(min: u8, max: u8, s: &str) -> anyhow::Result<u8> {
    match s.parse::<u8>() {
        Ok(d) if d >= min && d <= max => Ok(d),
        Ok(_) => Err(anyhow::anyhow!("Value out of bounds")),
        Err(e) => Err(e.into()),
    }
}
fn take4(s: &str) -> IResult<&str, &str> {
    take(4usize)(s)
}

fn time(s: &str) -> IResult<&str, (Time, &str)> {
    let s = s.trim_start();
    let take2 = take(2usize);
    let time_component = map_res(count(take2, 3), |v| Time::from_vec(v));
    let mut time_parser = tuple((time_component, tag("Z")));
    time_parser(s)
}

fn report_type(s: &str) -> IResult<&str, ReportType> {
    let parser = opt(alt((tag("AUTO"), tag("NIL"))));
    map_res(parser, |x: Option<&str>| x.unwrap_or("").parse())(s.trim_start())
}

#[derive(Debug, PartialEq)]
enum ReportType {
    Manual,
    Auto,
    Nil,
}

impl FromStr for ReportType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AUTO" => Ok(Self::Auto),
            "NIL" => Ok(Self::Nil),
            _ => Ok(Self::Manual),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Time {
    day: u8,
    hour: u8,
    minute: u8,
}

impl Time {
    fn from_vec(v: Vec<&str>) -> anyhow::Result<Time> {
        assert!(v.len() == 3);
        let day = parse_with_bounds(1, 31, v[0])?;
        let hour = parse_with_bounds(0, 23, v[1])?;
        let minute = parse_with_bounds(0, 59, v[2])?;
        Ok(Time { day, hour, minute })
    }
}

#[derive(Debug, PartialEq)]
pub struct METAR {
    report_type: ReportType,
    station: String,
    time: Time,
    wind: Wind,
    visibility: Visibility,
}

impl METAR {
    pub fn parse(s: &str) -> Result<METAR, nom::Err<nom::error::Error<&str>>> {
        let (_, (station, (time, _), report_type, wind, visibility)) =
            tuple((take4, time, report_type, parse_wind, parse_visibility))(
                s.trim_start_matches("METAR").trim(),
            )?;

        Ok(METAR {
            report_type,
            station: station.to_owned(),
            time,
            wind,
            visibility,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_report_type() -> anyhow::Result<()> {
        assert_eq!(report_type("AUTO")?.1, ReportType::Auto);
        assert_eq!(report_type("NIL")?.1, ReportType::Nil);
        assert_eq!(report_type("Something else")?.1, ReportType::Manual);
        assert_eq!(report_type("")?.1, ReportType::Manual);
        Ok(())
    }
}
