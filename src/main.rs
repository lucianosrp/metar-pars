#![allow(dead_code)]

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
    let w = Wind::new(direction, speed, ghust_speed, unit).unwrap();
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
    MPS,
    KT,
}

#[derive(Debug, PartialEq)]
enum WindDirection {
    DIRECT(u16),
    VARIABLE,
}

#[derive(Debug, PartialEq)]
struct Wind {
    direction: WindDirection,
    speed: u16,
    ghust_speed: Option<u16>,
    unit: WindUnit,
}
impl Wind {
    fn new(d: &str, s: &str, g: Option<&str>, u: &str) -> Result<Wind, Box<dyn std::error::Error>> {
        let g = g.and_then(|s| s.parse::<u16>().ok());
        Ok(Wind {
            direction: d.parse()?,
            speed: s.parse()?,
            ghust_speed: g,
            unit: u.parse()?,
        })
    }
}

impl FromStr for WindUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MPS" => Ok(WindUnit::MPS),
            "KT" => Ok(WindUnit::KT),
            _ => Err("Not a WindUnit".to_string()),
        }
    }
}
impl FromStr for WindDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u16>() {
            Ok(num) => Ok(WindDirection::DIRECT(num)),
            Err(_) => {
                if s == "VRB" {
                    Ok(WindDirection::VARIABLE)
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
    let sample = "METAR LICJ 141600Z 120120KT 090V150 1400 R04/P1500N R22/P1500U +SN BKN022 OVC050 M04/M07 Q1020 NOSIG 8849//91=";
    let metar = METAR::parse(&sample);

    println!("{:?}", metar);
    println!("{:?}", s.elapsed());
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_wind() {
        assert_eq!(
            wind("22010KT").unwrap().1,
            Wind::new("220", "10", None, "KT").unwrap()
        );

        assert_eq!(
            wind("220100MPS").unwrap().1,
            Wind::new("220", "100", None, "MPS").unwrap()
        );
        assert_eq!(
            wind("22010G40KT").unwrap().1,
            Wind::new("220", "10", Some(&"40"), "KT").unwrap()
        );

        assert_eq!(
            wind("VRB10G40KT").unwrap().1,
            Wind::new("VRB", "10", Some(&"40"), "KT").unwrap()
        )
    }

    #[test]
    fn test_parse() {
        let metars = vec![
    "METAR KLAX 221453Z 27007KT 10SM FEW025 BKN040 25/17 A3007 RMK AO2 SLP181 T02500170",
    "METAR KLAX 221453Z 27007KT 10SM FEW025 BKN040 25/17 A3007 RMK AO2 SLP181 T02500170",
    "METAR RJTT 221350Z 34015G25KT 9999 FEW030 SCT040 28/19 Q1011 NOSIG",
    "METAR EGLL 221255Z 20010KT 9999 SCT022 BKN030 17/13 Q1015",
    "METAR YSSY 221402Z 14008KT 9999 FEW030 25/18 Q1017 RMK RF00.0/000.0",
    "METAR KJFK 221358Z 23012KT 10SM FEW020 SCT035 23/15 A3012 RMK AO2 SLP175 T02330150",
    "METAR LFPG 221251Z 19007KT 9999 FEW020 SCT030 18/14 Q1016",
    "METAR ZSPD 221400Z 35018G30KT 9999 FEW025 SCT040 27/16 Q1013",
    "METAR CYYZ 221355Z 24014G22KT 9999 FEW020 SCT030 22/14 A2998 RMK SC8AC1 SLP164 T02220144",
    "METAR UUEE 221300Z VRB03KT 0400 R06/0800N R24/0800N FG VV001 06/06 Q1008",
    "METAR SBGR 221359Z 10008KT 9999 FEW030TCU SCT040 27/18 Q1012",
    "METAR RJTT 221350Z 34015G25KT 9999 FEW030 SCT040 28/19 Q1011 NOSIG",
    "METAR EGLL 221255Z 20010KT 9999 SCT022 BKN030 17/13 Q1015",
    "METAR YSSY 221402Z 14008KT 9999 FEW030 25/18 Q1017 RMK RF00.0/000.0",
    "METAR KJFK 221357Z 23012KT 10SM FEW020 SCT035 23/15 A3012 RMK AO2 SLP175 T02330150",
    "METAR LFPG 221251Z 19007KT 9999 FEW020 SCT030 18/14 Q1016",
    "METAR ZSPD 221400Z 35018G30KT 9999 FEW025 SCT040 27/16 Q1013",
    "METAR CYYZ 221355Z 24014G22KT 9999 FEW020 SCT030 22/14 A2998 RMK SC8AC1 SLP164 T02220144",
    "METAR UUEE 221300Z VRB03KT 0400 R06/0800N R24/0800N FG VV001 06/06 Q1008",
    "METAR SBGR 221359Z 10008KT 9999 FEW030TCU SCT040 27/18 Q1012",
    "METAR KSFO 221456Z 28006KT 10SM FEW030 21/15 A3006 RMK AO2 SLP180 T02110150",
    "METAR OMDB 221400Z 08006KT CAVOK 37/26 Q1008 NOSIG",
    "METAR EDDF 221254Z 18008KT 9999 SCT030 17/13 Q1015",
    "METAR VHHH 221401Z 07008KT 9999 FEW030 29/24 Q1014",
    "METAR RJAA 221352Z 33014KT 9999 FEW030 SCT045 27/18 Q1012",
    "METAR EHAM 221250Z 21009KT 9999 SCT025 BKN035 16/12 Q1016",
    "METAR ZBAA 221358Z 36016G28KT 9999 FEW020 SCT035 26/15 Q1014",
    "METAR CYUL 221356Z 23013G20KT 9999 FEW025 SCT035 21/13 A2999 RMK SC3AC1 SLP165 T02110133",
    "METAR UUWW 221300Z 29004KT 0200 R06/0600N R24/0600N FG VV001 04/04 Q1006",
    "METAR WMKK 221403Z 06007KT 2000 TSRA FEW025CB BKN040 29/26 Q1013",
    "METAR KORD 221455Z 26010KT 10SM FEW025 SCT040 23/14 A3004 RMK AO2 SLP178 T02330144",
    "METAR OMAA 221401Z 06005KT CAVOK 35/24 Q1009 NOSIG",
    "METAR EDDM 221253Z 19007KT 9999 SCT035 16/12 Q1016",
    "METAR VTBS 221402Z 08007KT 9999 FEW030 30/25 Q1013",
    "METAR RKSI 221351Z 32015KT 9999 FEW035 SCT050 26/17 Q1013",
    "METAR EGNX 221249Z 22008KT 9999 SCT020 BKN030 15/11 Q1017",
    "METAR ZGGG 221359Z 35015G25KT 9999 FEW020 SCT030 25/14 Q1015",
    "METAR CYOW 221357Z 22012G18KT 9999 FEW025 SCT035 20/12 A2999 RMK SC4AC1 SLP166 T02000122",
    "METAR UUDD 221300Z 28005KT 0150 R06/0500N R24/0500N FG VV001 03/03 Q1005",
    "METAR WSSS 221404Z 05006KT 8000 SHRA FEW025CB BKN040 28/26 Q1014",
    "METAR KPWK 221454Z 27008KT 10SM FEW025 BKN040 22/13 A3005 RMK AO2 SLP179 T02220133",
    "METAR LIMC 221402Z 12007KT CAVOK 34/23 Q1009 NOSIG",
];
        for metar in metars.iter() {
            println!("{}", &metar);
            let _ = METAR::parse(metar);
        }
    }
}
