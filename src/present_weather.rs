use nom::branch::alt;
use nom::combinator::opt;
use nom::IResult;
use nom::{bytes::complete::tag, combinator::value};
use std::str::FromStr;

use crate::tag_enum;
use anyhow::Error;

tag_enum! {
    Precipitation,
    "DZ" => Drizzle,
    "RA" => Rain,
    "SN" => Snow,
    "SG" => SnowGrains,
    "PL" => IcePellets,
    "IC" => IceCrystals,
    "GR" => Hail,
    "GS" => SmallHailSnowPellets,
    "UP" => UnknownPrecipitation

}

tag_enum! {
    Obscuration,
    "FG" => Fog,
    "BR" => Mist,
    "SA" => Sand,
    "DU" => Dust,
    "HZ" => Haze,
    "FU" => Smoke,
    "VA" => VolcanicAsh
}

tag_enum! {
    OtherPhenomena,
    "PO" => DustSandWhirls,
    "SQ" => Squall,
    "FC" => FunnelCloud,
    "DS" => Duststorm,
    "SS" => Sandstorm
}

tag_enum! {
    Characteristic,
    "TS" => Thunderstorm,
    "SH" => Shower,
    "FZ" => Freezing,
    "BL" => Blowing,
    "DR" => LowDrifting,
    "MI" => Shallow,
    "BC" => Patches,
    "PR" => Partial
}

tag_enum! {
    Intensity,
    "-" => Light,
    "+" => Heavy,
    "VC" => Vicinity
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct PresentWeather {
    intensity: Option<Intensity>,
    characteristic: Option<Characteristic>,
    precipitation: Option<Precipitation>,
    obscuration: Option<Obscuration>,
    other_phenomena: Option<OtherPhenomena>,
}

impl PresentWeather {
    fn parse(s: &str) -> IResult<&str, PresentWeather> {
        let (remaining, intensity) = opt(Intensity::parse)(s)?;
        let (remaining, characteristic) = opt(Characteristic::parse)(remaining)?;
        let (remaining, obscuration) = opt(Obscuration::parse)(remaining)?;
        let (remaining, precipitation) = opt(Precipitation::parse)(remaining)?;
        let (remaining, other_phenomena) = opt(OtherPhenomena::parse)(remaining)?;

        let present_weather = PresentWeather {
            intensity,
            characteristic,
            obscuration,
            precipitation,
            other_phenomena,
        };

        Ok((remaining, present_weather))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_conditions() {
        let input = "TS"; // Thunderstorm
        let expected = PresentWeather {
            intensity: None,
            characteristic: Some(Characteristic::Thunderstorm),
            obscuration: None,
            precipitation: None,
            other_phenomena: None,
        };
        let result = PresentWeather::parse(input).unwrap().1;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_rain_with_intensity() {
        let input = "+RA"; // Heavy Rain
        let expected = PresentWeather {
            intensity: Some(Intensity::Heavy),
            characteristic: None,
            obscuration: None,
            precipitation: Some(Precipitation::Rain),
            other_phenomena: None,
        };
        let result = PresentWeather::parse(input).unwrap().1;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_conditions() {
        let input = "+SH RA FG"; // Heavy Showers with Rain and Fog
        let expected = PresentWeather {
            intensity: Some(Intensity::Heavy),
            characteristic: Some(Characteristic::Shower),
            obscuration: Some(Obscuration::Fog),
            precipitation: Some(Precipitation::Rain),
            other_phenomena: None,
        };
        let result = PresentWeather::parse(input).unwrap().1;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_obscuration_and_other_phenomena() {
        let input = "DU VA PO"; // Dust, Volcanic Ash, Dust/Sand Whirls
        let expected = PresentWeather {
            intensity: None,
            characteristic: None,
            obscuration: Some(Obscuration::Dust),
            precipitation: None,
            other_phenomena: Some(OtherPhenomena::DustSandWhirls),
        };
        let result = PresentWeather::parse(input).unwrap().1;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_partial_obscuration() {
        let input = "PR FG"; // Partial Fog
        let expected = PresentWeather {
            intensity: None,
            characteristic: Some(Characteristic::Partial),
            obscuration: Some(Obscuration::Fog),
            precipitation: None,
            other_phenomena: None,
        };
        let result = PresentWeather::parse(input).unwrap().1;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_conditions() {
        let input = "SH UP VC"; // Shower, Unknown Precipitation, Vicinity
        let expected = PresentWeather {
            intensity: None,
            characteristic: Some(Characteristic::Shower),
            obscuration: None,
            precipitation: Some(Precipitation::UnknownPrecipitation),
            other_phenomena: None,
        };
        let result = PresentWeather::parse(input).unwrap().1;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_input() {
        let input = ""; // No input
        let expected = PresentWeather {
            intensity: None,
            characteristic: None,
            obscuration: None,
            precipitation: None,
            other_phenomena: None,
        };
        let result = PresentWeather::parse(input).unwrap().1;
        assert_eq!(result, expected);
    }
}
