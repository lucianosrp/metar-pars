use nom::{bytes::complete::take, IResult};

fn take4(s: &str) -> IResult<&str, &str> {
    take(4usize)(s)
}

fn main() {
    let sample = "LBBG 041600Z 12012MPS 090V150 1400 R04/P1500N R22/P1500U +SN BKN022 OVC050 M04/M07 Q1020 NOSIG 8849//91=";
    if let Ok((_, icao)) = take4(&sample) {
        println!("{:?}", icao)
    }
}
