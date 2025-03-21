use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use metar::Metar;
use metar_pars::Metar as MetarPars;

fn criterion_benchmark(c: &mut Criterion) {
    // let s = "EGHI 282120Z 19015KT 140V220 6000 RA SCT006 BKN009 16/14 Q1006";
    let s = "METAR EGLL 151450Z 19015G28KT 150V220 3000 +TSRA SCT012 BKN025CB OVC080 16/12 Q1002 NOSIG TEMPO 1500 +TSGR BKN015CB
    RMK CB BASE 020/080 HAIL RASH WIND SHEAR REPORTED RWY09R";
    c.bench_function("metar", |b| b.iter(|| Metar::parse(black_box(s))));
    c.bench_function("metar-pars", |b| b.iter(|| MetarPars::parse(black_box(s))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
