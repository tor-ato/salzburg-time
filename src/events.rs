//! オーストリア（特にザルツブルク周辺）の年中行事。
//! 祝日・移動祝日は復活祭日付（グレゴリオ暦・無名アルゴリズム）から算出。

use crate::civil::{civil_from_days, days_from_civil, weekday_days};

#[derive(Debug, Clone)]
pub struct Ev {
    /// days_from_civil ベースの暦日番号
    pub day: i64,
    pub y: i32,
    pub m: u32,
    pub d: u32,
    pub name: &'static str,
    pub note: &'static str,
    /// 正式な公的祝日（Feiertag）か
    pub holiday: bool,
}

/// 復活祭（グレゴリオ暦）の日付 (month, day)
pub fn easter_gregorian(y: i32) -> (u32, u32) {
    let a = y % 19;
    let b = y / 100;
    let c = y % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let mo = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * mo + 114) / 31;
    let day = ((h + l - 7 * mo + 114) % 31) + 1;
    (month as u32, day as u32)
}

/// 月の第 n 日曜日（n=1..5）
pub fn nth_sunday(y: i32, m: u32, n: u32) -> u32 {
    let w = weekday_days(days_from_civil(y, m, 1));
    let first = 1 + ((7 + 7 - w) % 7); // 本来は (7 - w) % 7 だが、u32の引き算あふれを避ける
    first + (n - 1) * 7
}

fn add(v: &mut Vec<Ev>, day: i64, name: &'static str, note: &'static str, holiday: bool) {
    let (y, m, d) = civil_from_days(day);
    v.push(Ev {
        day,
        y,
        m,
        d,
        name,
        note,
        holiday,
    });
}

pub fn events_for_year(y: i32) -> Vec<Ev> {
    let (em, ed) = easter_gregorian(y);
    let e = days_from_civil(y, em, ed);
    let mut v: Vec<Ev> = Vec::new();

    // 固定日
    add(&mut v, days_from_civil(y, 1, 6), "Heilige Drei Könige", "公現祭（三王来朝）。正式な公的祝日", true);
    add(&mut v, days_from_civil(y, 4, 30), "Walpurgisnacht", "魔女の夜。春の訪れを祝うボンファイヤー", false);
    add(&mut v, days_from_civil(y, 5, 1), "Staatsfeiertag / Maibaum", "メーデー（祝日）。村の広場には春のマスト「Maibaum」が立つ", true);
    add(&mut v, days_from_civil(y, 7, 25), "Salzburger Festspiele（概ね〜8/31）", "ザルツブルク音楽祭。モーツァルトの聖地が舞台になる夏", false);
    add(&mut v, days_from_civil(y, 8, 15), "Mariä Himmelfahrt", "聖母被昇天。公的祝日", true);
    add(&mut v, days_from_civil(y, 10, 26), "Nationalfeiertag", "建国記念日。1955年の永世中立国宣言を記念", true);
    add(&mut v, days_from_civil(y, 11, 1), "Allerheiligen", "諸聖人の日。祝日。墓地は蝋燭の灯で埋まる", true);
    add(&mut v, days_from_civil(y, 11, 2), "Allerseelen", "死者の日。墓参りの日", false);
    add(&mut v, days_from_civil(y, 11, 11), "Fasching 開幕（11.11 11:11）", "カーニバル・シーズン公式スタート", false);
    add(&mut v, days_from_civil(y, 11, 20), "Salzburger Christkindlmarkt（概ね〜12/24）", "大聖堂広場のクリスマス市が始まる", false);
    add(&mut v, days_from_civil(y, 12, 5), "Krampusnacht", "クランプスの夜。悪魔の仮面をかぶった行列が街を練り歩く", false);
    add(&mut v, days_from_civil(y, 12, 6), "Nikolaustag", "ニコラウスが子どもに菓子を配る", false);
    add(&mut v, days_from_civil(y, 12, 8), "Mariä Empfängnis", "無原罪の御宿り。公的祝日", true);
    add(&mut v, days_from_civil(y, 12, 24), "Heiliger Abend", "クリスマスイブ。オーバンドルフ（ザルツブルク近郊）で「きよしこの夜」が世界初演された日", false);
    add(&mut v, days_from_civil(y, 12, 25), "Christtag", "クリスマス。公的祝日", true);
    add(&mut v, days_from_civil(y, 12, 26), "Stephanitag", "聖ステファノの日。公的祝日", true);
    add(&mut v, days_from_civil(y, 12, 31), "Silvester", "大晦日。世界遺産の旧市街が爆竹と鐘の音に包まれる", false);

    // 復活祭から算出（移動祝日）
    add(&mut v, e - 49, "Faschingの最終週末", "Krapfen（カウンダー菓子）があちこちに並ぶ", false);
    add(&mut v, e - 47, "Faschingsdienstag", "謝肉祭フィナーレ。仮面パレードと大騒ぎ", false);
    add(&mut v, e - 46, "Aschermittwoch", "灰の水曜日。ここから四旬節", false);
    add(&mut v, e - 2, "Karfreitag", "聖金曜日。公的祝日", true);
    add(&mut v, e, "Ostersonntag", "復活祭。子どもたちは庭や家じゅうに隠された色づけ卵（Osterei）を探す", false);
    add(&mut v, e + 1, "Ostermontag", "復活祭の月曜。公的祝日", true);
    add(&mut v, e + 39, "Christi Himmelfahrt / Vatertag", "キリスト昇天日。オーストリアでは父の日も同じ日", true);
    add(&mut v, e + 49, "Pfingstsonntag", "聖霊降臨祭", false);
    add(&mut v, e + 50, "Pfingstmontag", "聖霊降臨祭の月曜。公的祝日", true);
    add(&mut v, e + 60, "Fronleichnam", "聖体の祝日。華やかな行列。公的祝日", true);

    // 母の日＆アドベント
    add(&mut v, days_from_civil(y, 5, nth_sunday(y, 5, 2)), "Muttertag", "母の日（5月第2日曜）", false);

    // アドベント: 第4日曜 = 12/24 前（含む）の日曜
    let wd_1224 = weekday_days(days_from_civil(y, 12, 24)) as i64;
    let advent4 = days_from_civil(y, 12, 24) - wd_1224;
    add(&mut v, advent4 - 21, "1. Advent", "アドベント開始。クリスマス市が本番に", false);
    add(&mut v, advent4, "4. Advent", "アドベント最終日曜。いよいよクリスマス", false);

    v.sort_by_key(|ev| (ev.day, ev.name));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civil::{days_from_civil, fmt_hm};

    #[test]
    fn easter_known_dates() {
        assert_eq!(easter_gregorian(2024), (3, 31));
        assert_eq!(easter_gregorian(2025), (4, 20));
        assert_eq!(easter_gregorian(2026), (4, 5));
        assert_eq!(easter_gregorian(2027), (3, 28));
    }

    #[test]
    fn nth_sunday_checks() {
        // 2026年5月の母の日=第2日曜=10日
        assert_eq!(nth_sunday(2026, 5, 2), 10);
        // 2025年5月=11日
        assert_eq!(nth_sunday(2025, 5, 2), 11);
    }

    #[test]
    fn events_sorted_full_year() {
        let v = events_for_year(2026);
        assert!(v.len() >= 28);
        // ソート済み確認
        for w in v.windows(2) {
            assert!(w[0].day <= w[1].day);
        }
        assert_eq!(easter_gregorian(2026), (4, 5));
        // 国籍記念日が含まれている
        assert!(v.iter().any(|e| e.note.contains("永世中立") || e.note.contains("中立")));
    }
}
