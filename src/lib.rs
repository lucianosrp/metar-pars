use std::num::ParseIntError;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::{is_alphabetic, is_digit};
use nom::combinator::{map_res, opt};
use nom::multi::count;
use nom::sequence::tuple;
use nom::{bytes::complete::take, IResult};
use visibility::{parse_visibility, Visibility};
mod visibility;

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

fn parse_v(s: &str) -> IResult<&str, &str> {
    tag("V")(s)
}

fn parse_variable_wind_direction(s: &str) -> Result<(&str, Option<(u16, u16)>), anyhow::Error> {
    let components = tuple((
        take_while(|x: char| is_digit(x as u8)),
        parse_v,
        take_while(|x: char| is_digit(x as u8)),
    ))(s.trim());

    match components {
        Ok((rest, (d1, _, d2))) => Ok((rest, Some((d1.parse::<u16>()?, d2.parse::<u16>()?)))),
        Err(_) => Ok((s, None)),
    }
}

fn wind(s: &str) -> IResult<&str, Wind> {
    let s = s.trim_start();
    let (rest, direction) = take(3usize)(s)?;
    let (rest, speed) = take_while(|x: char| is_digit(x as u8))(rest)?;
    let gust_speed: Option<&str> = None;

    let (rest, gust_speed) = if &rest[..1] == "G" {
        // Gusting
        let (rest, gust_speed) = take_while(|x: char| is_digit(x as u8))(&rest[1..])?;
        (rest, Some(gust_speed))
    } else {
        (rest, gust_speed)
    };

    let (rest, unit) = take_while(|x: char| is_alphabetic(x as u8))(rest)?;
    let (rest, variable_components) = parse_variable_wind_direction(rest).unwrap();

    let w = Wind::from_str(direction, speed, gust_speed, unit, variable_components).unwrap();
    Ok((rest, w))
}

fn parse_with_bounds(min: u8, max: u8, s: &str) -> Result<u8, ParseIntError> {
    match s.parse::<u8>() {
        Ok(d) if d >= min && d <= max => Ok(d),
        Ok(_) => panic!("Value out of bounds"),
        Err(e) => Err(e),
    }
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

#[derive(Debug, PartialEq)]
enum WindUnit {
    Mps,
    Mph,
    Kt,
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
enum WindDirection {
    Direct(u16),
    Variable,
}

#[derive(Debug, PartialEq)]
struct Wind {
    direction: WindDirection,
    speed: u16,
    gust_speed: Option<u16>,
    unit: WindUnit,
    variable_direction: Option<(u16, u16)>,
}
impl Wind {
    fn new(
        direction: WindDirection,
        speed: u16,
        gust_speed: Option<u16>,
        unit: WindUnit,
        variable_direction: Option<(u16, u16)>,
    ) -> anyhow::Result<Wind> {
        Ok(Wind {
            direction,
            speed,
            gust_speed,
            unit,
            variable_direction,
        })
    }

    fn from_str(
        direction: &str,
        speed: &str,
        gust_speed: Option<&str>,
        unit: &str,
        variable_direction: Option<(u16, u16)>,
    ) -> anyhow::Result<Wind> {
        let direction: WindDirection = direction.parse()?;
        let gust_speed = gust_speed.and_then(|s| s.parse::<u16>().ok());
        let speed: u16 = speed.parse()?;
        let unit: WindUnit = unit.parse()?;

        Wind::new(direction, speed, gust_speed, unit, variable_direction)
    }
}

impl FromStr for WindUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MPS" => Ok(WindUnit::Mps),
            "MPH" => Ok(WindUnit::Mph),
            "KT" => Ok(WindUnit::Kt),
            _ => Err(anyhow::Error::msg("Not a WindUnit")),
        }
    }
}
impl FromStr for WindDirection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u16>() {
            Ok(num) => Ok(WindDirection::Direct(num)),
            Err(_) => {
                if s == "VRB" {
                    Ok(WindDirection::Variable)
                } else {
                    Err(anyhow::Error::msg(format!(
                        "{:?} Not a valid WindDirection",
                        s
                    )))
                }
            }
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
    fn from_vec(v: Vec<&str>) -> Result<Time, ParseIntError> {
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
            tuple((take4, time, report_type, wind, parse_visibility))(
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
    fn test_report_type() {
        assert_eq!(report_type("AUTO").unwrap().1, ReportType::Auto);
        assert_eq!(report_type("NIL").unwrap().1, ReportType::Nil);
        assert_eq!(report_type("Something else").unwrap().1, ReportType::Manual);
        assert_eq!(report_type("").unwrap().1, ReportType::Manual);
    }
    #[test]
    fn test_variable_wind() {
        assert_eq!(
            parse_variable_wind_direction("20V40").unwrap(),
            (("", Some((20, 40))))
        );

        assert_eq!(
            parse_variable_wind_direction("200V40").unwrap(),
            (("", Some((200, 40))))
        );
        assert_eq!(
            parse_variable_wind_direction("200V240").unwrap(),
            (("", Some((200, 240))))
        );
        assert_eq!(
            parse_variable_wind_direction("20V240").unwrap(),
            (("", Some((20, 240))))
        )
    }
    #[test]
    fn test_wind() {
        assert_eq!(
            wind("22010KT").unwrap().1,
            Wind::new(WindDirection::Direct(220), 10, None, WindUnit::Kt, None).unwrap()
        );

        assert_eq!(
            wind("220100MPS").unwrap().1,
            Wind::new(WindDirection::Direct(220), 100, None, WindUnit::Mps, None).unwrap()
        );
        assert_eq!(
            wind("22010G40KT").unwrap().1,
            Wind::new(WindDirection::Direct(220), 10, Some(40), WindUnit::Kt, None).unwrap()
        );

        assert_eq!(
            wind("22010G40KT 200V240").unwrap().1,
            Wind::new(
                WindDirection::Direct(220),
                10,
                Some(40),
                WindUnit::Kt,
                Some((200, 240))
            )
            .unwrap()
        );

        assert_eq!(
            wind("VRB10G40KT").unwrap().1,
            Wind::new(WindDirection::Variable, 10, Some(40), WindUnit::Kt, None).unwrap()
        )
    }
}
