use std::str::FromStr;

use crate::tag_enum;
use anyhow::Error;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i32 as nomi32, i8 as nomi8},
    combinator::{opt, value},
    multi::separated_list0,
    sequence::tuple,
    IResult,
};

use crate::utils::Parseable;
tag_enum! {
    RunwayPosition,
    "L" => Left,
    "C" => Center,
    "R" => Right
}

tag_enum! {
    VisibilityScale,
    "P" => Plus,
    "M" => Minus
}

tag_enum! {
    VisibilityStatus,
    "D" => Down,
    "U" => Up,
    "N" => No
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

    let position_parser = RunwayPosition::parse;
    let vis_scale_parser = VisibilityScale::parse;
    let vis_status_parser = VisibilityStatus::parse;

    let (other, (_, number, position, _, vis_scale, vis_meter, vis_status)) = tuple((
        tag("R"),
        nomi8,
        opt(position_parser),
        tag("/"),
        opt(vis_scale_parser),
        meter_parser,
        opt(vis_status_parser),
    ))(s)?;
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
        let res = parse_rvrs("R25L/M1075N R25C/P200U")?.1;
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
                }
            ]
        );
        Ok(())
    }
}
