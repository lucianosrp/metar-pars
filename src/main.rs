use metar_pars::METAR;
use std::time::Instant;

fn main() {
    let s = Instant::now();
    let sample = "METAR LICJ 141600Z 120120G50KT 090V150 1400 R04/P1500N R22/P1500U +SN BKN022 OVC050 M04/M07 Q1020 NOSIG 8849//91=";
    let metar = METAR::from_str(sample);
    dbg!(&metar);
    println!("{:?}", s.elapsed());
}
