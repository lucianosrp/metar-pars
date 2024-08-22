use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i32 as nomi32, i8 as nomi8},
    combinator::{map_opt, opt},
    multi::separated_list0,
    sequence::tuple,
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum RunwayPosition {
    Left,
    Center,
    Right,
}

impl FromStr for RunwayPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "L" => Ok(RunwayPosition::Left),
            "R" => Ok(RunwayPosition::Right),
            "C" => Ok(RunwayPosition::Center),
            _ => Err(format!("Cannot parse {}", s)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum VisibilityScale {
    Plus,
    Minus,
}

impl FromStr for VisibilityScale {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "P" => Ok(VisibilityScale::Plus),
            "M" => Ok(VisibilityScale::Minus),
            _ => Err(format!("Cannot parse {}", s)),
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum VisibilityStatus {
    Down,
    Up,
    No,
}
impl FromStr for VisibilityStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "D" => Ok(VisibilityStatus::Down),
            "U" => Ok(VisibilityStatus::Up),
            "N" => Ok(VisibilityStatus::No),
            _ => Err(format!("Cannot parse {}", s)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RunwayVisualRange {
    pub number: i8,
    pub position: Option<RunwayPosition>,
    pub visibility_meters: i32,
    pub visibility_scale: Option<VisibilityScale>,
    pub visibility_status: Option<VisibilityStatus>,
}

pub fn parse_rvr(s: &str) -> IResult<&str, RunwayVisualRange> {
    let meter_parser = nomi32;
    let position_parser = map_opt(
        opt(alt((tag("L"), tag("R"), tag("C")))),
        |s: Option<&str>| s.and_then(|x| x.parse::<RunwayPosition>().ok()),
    );

    let vis_scale_parser = map_opt(opt(alt((tag("M"), tag("P")))), |s: Option<&str>| {
        s.and_then(|x| x.parse::<VisibilityScale>().ok())
    });

    let vis_status_parser = map_opt(
        opt(alt((tag("D"), tag("U"), tag("N")))),
        |s: Option<&str>| s.and_then(|x| x.parse::<VisibilityStatus>().ok()),
    );

    let (other, (_, number, position, _, vis_scale, vis_meter, vis_status)) =
        tuple((
            tag("R"),
            nomi8,
            opt(position_parser),
            tag("/"),
            opt(vis_scale_parser),
            meter_parser,
            opt(vis_status_parser),
        ))(s.trim_start())?;
    Ok((
        other,
        RunwayVisualRange {
            number,
            position,
            visibility_meters: vis_meter,
            visibility_scale: vis_scale,
            visibility_status: vis_status,
        },
    ))
}
pub fn parse_rvrs(s: &str) -> IResult<&str, Vec<RunwayVisualRange>> {
    separated_list0(tag(" "), parse_rvr)(s)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_rvr_scale() -> anyhow::Result<()> {
        let res = parse_rvr("R25/M0075U")?.1;
        assert_eq!(
            res,
            RunwayVisualRange {
                number: 25,
                position: None,
                visibility_meters: 75,
                visibility_scale: Some(VisibilityScale::Minus),
                visibility_status: Some(VisibilityStatus::Up)
            }
        );

        let res = parse_rvr("R04/P1500N")?.1;
        assert_eq!(
            res,
            RunwayVisualRange {
                number: 4,
                position: None,
                visibility_meters: 1500,
                visibility_scale: Some(VisibilityScale::Plus),
                visibility_status: Some(VisibilityStatus::No)
            }
        );
        Ok(())
    }
    #[test]
    fn test_parse_rvr_position_scale_status() -> anyhow::Result<()> {
        let res = parse_rvr("R25L/P1075N")?.1;
        assert_eq!(
            res,
            RunwayVisualRange {
                number: 25,
                position: Some(RunwayPosition::Left),
                visibility_meters: 1075,
                visibility_scale: Some(VisibilityScale::Plus),
                visibility_status: Some(VisibilityStatus::No)
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_rvrs() -> anyhow::Result<()> {
        let res = parse_rvrs("R25L/M1075N R25C/P200U R25R/1000N")?.1;
        assert_eq!(
            res,
            vec![
                RunwayVisualRange {
                    number: 25,
                    position: Some(RunwayPosition::Left),
                    visibility_meters: 1075,
                    visibility_scale: Some(VisibilityScale::Minus),
                    visibility_status: Some(VisibilityStatus::No)
                },
                RunwayVisualRange {
                    number: 25,
                    position: Some(RunwayPosition::Center),
                    visibility_meters: 200,
                    visibility_scale: Some(VisibilityScale::Plus),
                    visibility_status: Some(VisibilityStatus::Up)
                },
                RunwayVisualRange {
                    number: 25,
                    position: Some(RunwayPosition::Right),
                    visibility_meters: 1000,
                    visibility_scale: None,
                    visibility_status: Some(VisibilityStatus::No)
                }
            ]
        );
        Ok(())
    }
}
