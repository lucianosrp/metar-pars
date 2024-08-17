use std::str::FromStr;

use nom::bytes::complete::{tag, take_while};
use nom::character::{is_alphabetic, is_digit};
use nom::sequence::tuple;
use nom::{bytes::complete::take, IResult};

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

pub fn parse_wind(s: &str) -> IResult<&str, Wind> {
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

#[derive(Debug, PartialEq)]
pub enum WindUnit {
    Mps,
    Mph,
    Kt,
}

#[derive(Debug, PartialEq)]
pub enum WindDirection {
    Direct(u16),
    Variable,
}

#[derive(Debug, PartialEq)]
pub struct Wind {
    pub direction: WindDirection,
    pub speed: u16,
    pub gust_speed: Option<u16>,
    pub unit: WindUnit,
    pub variable_direction: Option<(u16, u16)>,
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
            parse_wind("22010KT").unwrap().1,
            Wind::new(WindDirection::Direct(220), 10, None, WindUnit::Kt, None).unwrap()
        );

        assert_eq!(
            parse_wind("220100MPS").unwrap().1,
            Wind::new(WindDirection::Direct(220), 100, None, WindUnit::Mps, None).unwrap()
        );
        assert_eq!(
            parse_wind("22010G40KT").unwrap().1,
            Wind::new(WindDirection::Direct(220), 10, Some(40), WindUnit::Kt, None).unwrap()
        );

        assert_eq!(
            parse_wind("22010G40KT 200V240").unwrap().1,
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
            parse_wind("VRB11G40KT").unwrap().1,
            Wind::new(WindDirection::Variable, 11, Some(40), WindUnit::Kt, None).unwrap()
        )
    }
}
