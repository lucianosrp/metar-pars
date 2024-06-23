use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::{
        complete::{digit1, multispace1},
        is_digit,
    },
    combinator::{map_res, opt},
    error::{context, ErrorKind},
    sequence::{pair, tuple},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Visibility {
    Meters(u16),
    StatuateMiles(f64),
    Cavok,
    Nsc,
    Skc,
}

impl FromStr for Visibility {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check for special cases first
        match s {
            "CAVOK" => return Ok(Visibility::Cavok),
            "NSC" => return Ok(Visibility::Nsc),
            "SKC" => return Ok(Visibility::Skc),
            _ => (),
        }

        s.parse::<u16>()
            .map(Visibility::Meters)
            .map_err(|_| anyhow::Error::msg("Cannot parse into Visibility"))
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
        map_res(tag("CAVOK"), |_| Visibility::from_str("CAVOK")),
        map_res(tag("NSC"), |_| Visibility::from_str("NSC")),
        map_res(tag("SKC"), |_| Visibility::from_str("SKC")),
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_visibility() {
        assert_eq!(parse_visibility("CAVOK").unwrap().1, Visibility::Cavok);
        assert_eq!(parse_visibility("NSC").unwrap().1, Visibility::Nsc);
        assert_eq!(parse_visibility("SKC").unwrap().1, Visibility::Skc);
        assert_eq!(
            parse_visibility("9999").unwrap().1,
            Visibility::Meters(9999)
        );
        assert_eq!(
            parse_visibility("5000").unwrap().1,
            Visibility::Meters(5000)
        );
        assert_eq!(
            parse_visibility(" 1000").unwrap().1,
            Visibility::Meters(1000)
        );
        assert_eq!(
            parse_visibility("1/4SM").unwrap().1,
            Visibility::StatuateMiles(0.25)
        );
        assert_eq!(
            parse_visibility("10SM").unwrap().1,
            Visibility::StatuateMiles(10.0)
        );
        assert_eq!(
            parse_visibility("1 1/2SM").unwrap().1,
            Visibility::StatuateMiles(1.5)
        );
    }
}
