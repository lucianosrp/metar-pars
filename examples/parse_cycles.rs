use std::time::Instant;

use anyhow::{Ok, Result};
use metar_pars::METAR;


fn get_cycles() -> Result<String> {
    let url = "https://tgftp.nws.noaa.gov/data/observations/metar/cycles/09Z.TXT";
    let res = reqwest::blocking::get(url)?.text()?;
    Ok(res)
}

fn main() -> Result<(), anyhow::Error> {
    let res = get_cycles()?;
    let s = Instant::now();
    let lines: Vec<_> = res
        .lines()
        .filter(|x| !x.is_empty() && x.len() > 16 && x.contains("KTEB"))
        .map(|x| METAR::parse(x))
        .collect();

    dbg!(lines);
    println!("{:?}", s.elapsed());
    Ok(())
}
