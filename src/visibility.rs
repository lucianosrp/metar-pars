use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::{
        complete::{digit1, multispace1},
        is_digit,
    },
    combinator::{map_res, opt, value},
    error::{context, ErrorKind},
    sequence::{pair, tuple},
    IResult,
};

use anyhow::Error;

use crate::tag_enum_parser;

#[derive(Debug, PartialEq, Clone)]
pub enum VisibilityDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl FromStr for VisibilityDirection {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "N" => Ok(VisibilityDirection::North),
            "NE" => Ok(VisibilityDirection::NorthEast),
            "E" => Ok(VisibilityDirection::East),
            "SE" => Ok(VisibilityDirection::SouthEast),
            "S" => Ok(VisibilityDirection::South),
            "SW" => Ok(VisibilityDirection::SouthWest),
            "W" => Ok(VisibilityDirection::West),
            "NW" => Ok(VisibilityDirection::NorthWest),
            _ => Err(anyhow::Error::msg("Erro while converting from str")),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Visibility {
    Meters(u16),
    StatuateMiles(f64),
    Cavok,
    Nsc,
    Skc,
    CustomDirection(Box<Visibility>, Box<Visibility>, VisibilityDirection),
}
impl Visibility {
    fn from_tuple(
        t: (Visibility, Option<(Visibility, &str)>),
    ) -> Result<Visibility, anyhow::Error> {
        match t.1 {
            Some(v) => Ok(Visibility::CustomDirection(
                Box::new(t.0),
                Box::new(v.0),
                VisibilityDirection::from_str(v.1)?,
            )),
            None => Ok(t.0),
        }
    }
}

impl FromStr for Visibility {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CAVOK" => return Ok(Visibility::Cavok),
            "NSC" => return Ok(Visibility::Nsc),
            "SKC" => return Ok(Visibility::Skc),
            _ => (),
        }

        s.parse::<u16>()
            .map(Visibility::Meters)
            .map_err(|_| anyhow::anyhow!("Cannot parse into Visibility"))
    }
}

fn parse_partial(
    s: &str,
) -> IResult<&str, (Option<(&str, &str)>, &str, Option<&str>, Option<&str>, &str)> {
    tuple((
        opt(pair(digit1, multispace1)),
        take_while(|c: char| is_digit(c as u8)),
        opt(tag("/")),
        opt(digit1),
        tag("SM"),
    ))(s)
}

fn partial_statuate_miles_parser(s: &str) -> IResult<&str, Visibility> {
    let (remainer, (d0, d1, t, d2, _)) = parse_partial(s)?;

    if let Some(sep) = t {
        if sep == "/" {
            if let Some(denominator) = d2 {
                let numerator: f64 = d1.parse().unwrap_or_default();
                let denominator: f64 = denominator.parse().unwrap_or_default();
                let mut partial_res = numerator / denominator;
                if let Some((whole, _)) = d0 {
                    partial_res += whole.parse::<f64>().unwrap_or_default()
                }
                return Ok((remainer, Visibility::StatuateMiles(partial_res)));
            }
        };
    } else {
        return Ok((
            remainer,
            Visibility::StatuateMiles(d1.parse().unwrap_or_default()),
        ));
    }
    panic!("Not a valid partial")
}

pub fn parse_visibility(s: &str) -> IResult<&str, Visibility> {
    let s = s.trim_start();
    alt((
        tag_enum_parser!(
            "CAVOK" => Visibility::Cavok,
            "NSC" => Visibility::Nsc,
            "SKC" => Visibility::Skc
        ),
        partial_statuate_miles_parser,
        context(
            "Visibility Meters",
            map_res(take_while(|x: char| is_digit(x as u8)), |s: &str| {
                s.parse()
                    .map_err(|_| nom::Err::Error(nom::error::Error::new(s, ErrorKind::Digit)))
            }),
        ),
    ))(s)
}

pub fn parse_visibility_full(s: &str) -> IResult<&str, Visibility> {
    let carinal_tags = alt((
        tag("NW"),
        tag("NE"),
        tag("SE"),
        tag("SW"),
        tag("N"),
        tag("E"),
        tag("S"),
        tag("W"),
    ));
    let s = s.trim();
    map_res(
        tuple((
            parse_visibility,
            opt(tuple((parse_visibility, carinal_tags))),
        )),
        |res| {
            Visibility::from_tuple(res).map_err(|_| {
                nom::Err::Error(nom::error::Error::new(
                    "Error while parsing Custom Direction",
                    ErrorKind::Digit,
                ))
            })
        },
    )(s)
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_visibility() -> anyhow::Result<()> {
        assert_eq!(parse_visibility("CAVOK")?.1, Visibility::Cavok);
        assert_eq!(parse_visibility("NSC")?.1, Visibility::Nsc);
        assert_eq!(parse_visibility("SKC")?.1, Visibility::Skc);
        assert_eq!(parse_visibility("9999")?.1, Visibility::Meters(9999));
        assert_eq!(parse_visibility("5000")?.1, Visibility::Meters(5000));
        assert_eq!(parse_visibility(" 1000")?.1, Visibility::Meters(1000));
        assert_eq!(
            parse_visibility("1/4SM")?.1,
            Visibility::StatuateMiles(0.25)
        );
        assert_eq!(parse_visibility("10SM")?.1, Visibility::StatuateMiles(10.0));
        assert_eq!(
            parse_visibility("1 1/2SM")?.1,
            Visibility::StatuateMiles(1.5)
        );
        Ok(())
    }

    #[test]
    fn test_parse_visibility_custom_direction() -> anyhow::Result<()> {
        assert_eq!(
            parse_visibility_full("2000 1200NW")?.1,
            Visibility::CustomDirection(
                Box::new(Visibility::Meters(2000)),
                Box::new(Visibility::Meters(1200)),
                VisibilityDirection::NorthWest
            )
        );
        assert_eq!(
            parse_visibility_full("3000 2000S")?.1,
            Visibility::CustomDirection(
                Box::new(Visibility::Meters(3000)),
                Box::new(Visibility::Meters(2000)),
                VisibilityDirection::South
            )
        );
        assert_eq!(
            parse_visibility_full("1 1/2SM 10SMS")?.1,
            Visibility::CustomDirection(
                Box::new(Visibility::StatuateMiles(1.5)),
                Box::new(Visibility::StatuateMiles(10.0)),
                VisibilityDirection::South
            )
        );
        Ok(())
    }
}
