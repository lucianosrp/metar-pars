use std::time::Instant;

use anyhow::{Ok, Result};
use metar_pars::Metar;

fn get_cycles() -> Result<String> {
    let url = "https://tgftp.nws.noaa.gov/data/observations/metar/cycles/14Z.TXT";
    let res = reqwest::blocking::get(url)?.text()?;
    Ok(res)
}

fn main() -> Result<()> {
    let res = get_cycles()?;
    let s = Instant::now();
    let lines: Vec<_> = res
        .lines()
        .filter(|x| !x.is_empty() && x.len() > 16 && x.contains("EGLL"))
        .map(|x| Metar::parse(x))
        .collect();

    dbg!(lines);
    println!("{:?}", s.elapsed());
    Ok(())
}
