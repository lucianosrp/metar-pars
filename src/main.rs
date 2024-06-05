use std::error::Error;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Instant;

use nom::bytes::complete::{tag, take_while};
use nom::character::complete::multispace1;
use nom::character::{is_alphabetic, is_digit};
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

fn parse_v(s: &str) -> IResult<&str, &str> {
    tag("V")(s)
}

fn parse_variable_wind_direction(s: &str) -> Result<(&str, Option<(u16, u16)>), Box<dyn Error>> {
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
    let (rest, direction) = take(3usize)(s)?;
    let (rest, speed) = take_while(|x: char| is_digit(x as u8))(rest)?;
    let ghust_speed: Option<&str> = None;

    let (rest, ghust_speed) = if &rest[..1] == "G" {
        // Ghusting
        let (rest, ghust_speed) = take_while(|x: char| is_digit(x as u8))(&rest[1..])?;
        (rest, Some(ghust_speed))
    } else {
        (rest, ghust_speed)
    };

    let (rest, unit) = take_while(|x: char| is_alphabetic(x as u8))(rest)?;
    let (rest, variable_components) = parse_variable_wind_direction(rest).unwrap();

    let w = Wind::from_str(direction, speed, ghust_speed, unit, variable_components).unwrap();
    Ok((rest, w))
}

fn parse_with_bounds(min: u8, max: u8, s: &str) -> Result<u8, ParseIntError> {
    let d = s.parse::<u8>()?;
    if d >= min && d <= max {
        Ok(d)
    } else {
        panic!("out of bounds")
    }
}

#[derive(Debug, PartialEq)]
enum WindUnit {
    Mps,
    Mph,
    Kt,
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
    ghust_speed: Option<u16>,
    unit: WindUnit,
    variable_direction: Option<(u16, u16)>,
}
impl Wind {
    fn new(
        direction: WindDirection,
        speed: u16,
        ghust_speed: Option<u16>,
        unit: WindUnit,
        variable_direction: Option<(u16, u16)>,
    ) -> Result<Wind, Box<dyn std::error::Error>> {
        Ok(Wind {
            direction,
            speed,
            ghust_speed,
            unit,
            variable_direction,
        })
    }

    fn from_str(
        direction: &str,
        speed: &str,
        ghust_speed: Option<&str>,
        unit: &str,
        variable_direction: Option<(u16, u16)>,
    ) -> Result<Wind, Box<dyn std::error::Error>> {
        let direction: WindDirection = direction.parse()?;
        let ghust_speed = ghust_speed.and_then(|s| s.parse::<u16>().ok());
        let speed: u16 = speed.parse()?;
        let unit: WindUnit = unit.parse()?;

        Wind::new(direction, speed, ghust_speed, unit, variable_direction)
    }
}

impl FromStr for WindUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MPS" => Ok(WindUnit::Mps),
            "MPH" => Ok(WindUnit::Mph),
            "KT" => Ok(WindUnit::Kt),
            _ => Err("Not a WindUnit".to_string()),
        }
    }
}
impl FromStr for WindDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u16>() {
            Ok(num) => Ok(WindDirection::Direct(num)),
            Err(_) => {
                if s == "VRB" {
                    Ok(WindDirection::Variable)
                } else {
                    Err("Not a valid WindDirection".to_string())
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
struct METAR {
    station: String,
    time: Time,
    wind: Wind,
}

impl METAR {
    fn parse(s: &str) -> METAR {
        let s = s.replace("METAR ", "");
        let res = tuple((take4, multispace1, time, multispace1, wind))(&s).unwrap();
        METAR {
            station: res.1 .0.to_string(),
            time: res.1 .2 .0,
            wind: res.1 .4,
        }
    }
}

fn main() {
    let s = Instant::now();
    let sample = "METAR LICJ 141600Z 120120G50KT 090V150 1400 R04/P1500N R22/P1500U +SN BKN022 OVC050 M04/M07 Q1020 NOSIG 8849//91=";
    let metar = METAR::parse(sample);
    println!("{:?}", metar);
    println!("{:?}", s.elapsed());
}

#[cfg(test)]
mod test {

    use super::*;

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
