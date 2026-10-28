//! 暦・タイムゾーン計算。依存crateなしの純Rust。
//! グレゴリオ暦の日数計算は Howard Hinnant のアルゴリズム。

/// 暦日 → 紀元1970-01-01を0とする日数
pub fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let yy: i64 = (y as i64) - if m < 3 { 1 } else { 0 };
    let era: i64 = if yy >= 0 { yy } else { yy - 399 } / 400;
    let yoe = yy - era * 400;
    let mp = if m > 2 { m as i64 - 3 } else { m as i64 + 9 };
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// 日数 → 暦日 (y, m, d)
pub fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((if m < 3 { y + 1 } else { y }) as i32, m, d)
}

/// 指定日の曜日（0=日 … 6=土）。1970-01-01 は木曜(4)。
pub fn weekday_days(z: i64) -> u32 {
    ((z + 4).rem_euclid(7)) as u32
}

pub const WD_KANJI: [&str; 7] = ["日", "月", "火", "水", "木", "金", "土"];

pub fn is_leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

pub fn last_day_of_month(m: u32, y: i32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
    }
}

/// 月の最終日曜日（EU夏時間の切替日に使用）
pub fn last_sunday(y: i32, m: u32) -> u32 {
    let ld = last_day_of_month(m, y);
    let w = weekday_days(days_from_civil(y, m, ld));
    ld - w
}

/// ある瞬間（絶対エポック秒）におけるヨーロッパ/ウィーン（=ザルツブルク）のUTCオフセット秒。
/// EUの現行夏時間規則: 3月最終日曜 01:00 UTC にCEST(+2)、10月最終日曜 01:00 UTC にCET(+1)。
pub fn vienna_offset_seconds(epoch_secs: i64) -> i64 {
    let days = epoch_secs.div_euclid(86400);
    let sod = epoch_secs.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    if is_cest(y, m, d, sod) {
        7200
    } else {
        3600
    }
}

pub fn is_cest(y: i32, m: u32, d: u32, utc_sod: i64) -> bool {
    match m {
        4..=9 => true,
        3 => {
            let lsm = last_sunday(y, 3);
            d > lsm || (d == lsm && utc_sod >= 3600)
        }
        10 => {
            let lso = last_sunday(y, 10);
            d < lso || (d == lso && utc_sod < 3600)
        }
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    pub y: i32,
    pub m: u32,
    pub d: u32,
    pub h: i64,
    pub mi: i64,
    pub s: i64,
    /// 0=日 … 6=土
    pub wd: u32,
}

/// エポック秒＋固定オフセット → 各地の暦時刻
pub fn to_local(local_epoch_secs: i64) -> LocalTime {
    let days = local_epoch_secs.div_euclid(86400);
    let sod = local_epoch_secs.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    LocalTime {
        y,
        m,
        d,
        h: sod / 3600,
        mi: (sod % 3600) / 60,
        s: sod % 60,
        wd: weekday_days(days),
    }
}

pub fn p2(n: i64) -> String {
    format!("{:02}", n)
}

pub fn fmt_hms(sod: i64) -> String {
    format!("{}:{}:{}", p2(sod / 3600), p2((sod % 3600) / 60), p2(sod % 60))
}

pub fn fmt_hm(sod: i64) -> String {
    format!("{}:{}", p2(sod / 3600), p2((sod % 3600) / 60))
}

pub fn fmt_ymd(t: &LocalTime) -> String {
    format!("{}年{}月{}日({})", t.y, t.m, t.d, WD_KANJI[t.wd as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_zero() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(weekday_days(0), 4); // 木曜
    }

    #[test]
    fn civil_roundtrip() {
        for (y, m, d) in [(1970, 1, 1), (1999, 12, 31), (2000, 2, 29), (2026, 3, 29), (2100, 3, 1)] {
            let z = days_from_civil(y, m, d);
            assert_eq!(civil_from_days(z), (y, m, d));
        }
        assert_eq!(days_from_civil(2000, 3, 1), 11017);
    }

    #[test]
    fn dst_rules_2026() {
        // 2026年: 夏時間は3/29開始、10/25終了
        assert_eq!(last_sunday(2026, 3), 29);
        assert_eq!(last_sunday(2026, 10), 25);
        assert!(!is_cest(2026, 3, 29, 3599));
        assert!(is_cest(2026, 3, 29, 3600));
        assert!(is_cest(2026, 10, 25, 3599));
        assert!(!is_cest(2026, 10, 25, 3600));
        assert!(is_cest(2026, 7, 1, 0));
        // 2026-10-20 はまだ夏時間（切替は10/25）
        assert_eq!(vienna_offset_seconds(days_from_civil(2026, 10, 20) * 86400 + 43200), 7200);
        assert_eq!(vienna_offset_seconds(days_from_civil(2026, 10, 26) * 86400 + 43200), 3600);
        // 夏: UTC+2
        assert_eq!(vienna_offset_seconds(days_from_civil(2026, 7, 15) * 86400 + 43200), 7200);
    }

    #[test]
    fn to_local_shapes() {
        let t = to_local(0 + 9 * 3600); // JST 1970-01-01 09:00:00
        assert_eq!((t.y, t.m, t.d, t.h), (1970, 1, 1, 9));
        assert_eq!(fmt_hms(t.h * 3600 + t.mi * 60 + t.s), "09:00:00");
    }
}
