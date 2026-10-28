//! ザルツブルク（オーストリア）の日の出・日の入り。
//! NOAA 太陽計算の近似式（スプレッドシート版: Edward Williams / NOAA）。

use crate::civil::{days_from_civil};

pub struct SunEvents {
    /// その暦日内のUTC時刻（分）。None は極夜・白夜で計算不能のとき
    pub sunrise_utc_min: Option<f64>,
    pub sunset_utc_min: Option<f64>,
}

pub fn sun_for_date(y: i32, m: u32, d: u32, lat_deg: f64, lon_deg: f64) -> SunEvents {
    const PI: f64 = std::f64::consts::PI;
    let n = days_from_civil(y, m, d) - days_from_civil(y, 1, 1) + 1;
    let g = 2.0 * PI / 365.0 * ((n - 1) as f64);

    let eqtime = 229.18
        * (0.000075
            + 0.001868 * g.cos()
            - 0.032077 * g.sin()
            - 0.014615 * (2.0 * g).cos()
            - 0.040849 * (2.0 * g).sin());

    let decl = 0.006918 - 0.399912 * g.cos() + 0.070257 * g.sin() - 0.006758 * (2.0 * g).cos()
        + 0.000907 * (2.0 * g).sin() - 0.002697 * (3.0 * g).cos() + 0.00148 * (3.0 * g).sin();

    let lat_r = lat_deg.to_radians();
    let cos_zenith = 90.833_f64.to_radians().cos();
    let arg = cos_zenith / (lat_r.cos() * decl.cos()) - lat_r.tan() * decl.tan();

    if !(-1.0..=1.0).contains(&arg) {
        // 太陽が沈まない/昇らない
        return SunEvents {
            sunrise_utc_min: None,
            sunset_utc_min: None,
        };
    }
    let ha_deg = arg.acos().to_degrees();
    let noon = 720.0 - 4.0 * lon_deg - eqtime;

    SunEvents {
        sunrise_utc_min: Some(noon - 4.0 * ha_deg),
        sunset_utc_min: Some(noon + 4.0 * ha_deg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAT: f64 = 47.8095; // ザルツブルク
    const LON: f64 = 13.0550;

    fn local_disp(y: i32, m: u32, d: u32, utc_min: f64, off_s: i64) -> i64 {
        let days = days_from_civil(y, m, d);
        let um = utc_min as i64;
        (days * 86400 + um * 60 + off_s) / 60
    }

    #[test]
    fn salzburg_spring_equinox() {
        // 2026-03-20（春分）: 日の出≈06:19, 日の入り≈18:26 CET (UTC+1)
        let s = sun_for_date(2026, 3, 20, LAT, LON);
        let off = 3600;
        let sr = local_disp(2026, 3, 20, s.sunrise_utc_min.unwrap(), off);
        let ss = local_disp(2026, 3, 20, s.sunset_utc_min.unwrap(), off);
        let sr_hm = sr % 1440;
        let ss_hm = ss % 1440;
        assert!((6 * 60 + 12..=6 * 60 + 26).contains(&sr_hm), "sunrise {}:{} min={}", sr_hm / 60, sr_hm % 60, sr_hm);
        assert!((18 * 60 + 15..=18 * 60 + 35).contains(&ss_hm), "sunset {}:{} min={}", ss_hm / 60, ss_hm % 60, ss_hm);
    }

    #[test]
    fn salzburg_mid_october() {
        // 2026-10-20: 日の出≈07:45, 日の入り≈18:03 CEST (UTC+2)
        let s = sun_for_date(2026, 10, 20, LAT, LON);
        let off = 7200;
        let sr = local_disp(2026, 10, 20, s.sunrise_utc_min.unwrap(), off);
        let ss = local_disp(2026, 10, 20, s.sunset_utc_min.unwrap(), off);
        let sr_hm = sr % 1440;
        let ss_hm = ss % 1440;
        assert!((7 * 60 + 25..=7 * 60 + 55).contains(&sr_hm), "sunrise min={}", sr_hm);
        assert!((17 * 60 + 50..=18 * 60 + 18).contains(&ss_hm), "sunset min={}", ss_hm);
    }

    #[test]
    fn tokyo_midsummer() {
        // 東京 2026-06-21: 日の出≈4:25, 日の入り≈19:00
        let s = sun_for_date(2026, 6, 21, 35.6586, 139.7454);
        let sr = (s.sunrise_utc_min.unwrap() as i64 + 9 * 60) % 1440;
        let ss = (s.sunset_utc_min.unwrap() as i64 + 9 * 60) % 1440;
        assert!((4 * 60 + 15..=4 * 60 + 40).contains(&sr), "sunrise min={}", sr);
        assert!((18 * 60 + 45..=19 * 60 + 15).contains(&ss), "sunset min={}", ss);
    }
}
