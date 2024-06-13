use metar_pars::METAR;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let s = Instant::now();
    let sample = "METAR LICJ 141600Z 120120G50KT 090V150 CAVOK R04/P1500N R22/P1500U +SN BKN022 OVC050 M04/M07 Q1020 NOSIG 8849//91=";
    let metar = METAR::parse(sample)?;
    dbg!(metar);
    println!("{:?}", s.elapsed());
    Ok(())
}
