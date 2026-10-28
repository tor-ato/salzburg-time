//! HTMLレンダリング（サーバーサイドRust）。
//! 時刻・時差・変換表・日の出日の入り・行事がすべてリクエスト時にSSRされる。
//! クライアントJSは「秒針のtick」と「日替わりリロード」の最小限のみ。

use crate::civil::*;
use crate::events::events_for_year;
use crate::solar::sun_for_date;

pub const SALZ_LAT: f64 = 47.8095; // ザルツブルク（旧市街）
pub const SALZ_LON: f64 = 13.0550;

const HEAD: &str = r#"<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>ザルツブルク時間 | Salzburg × Japan</title>
<link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Crect width='24' height='24' rx='5' fill='%230b1220'/%3E%3Ccircle cx='12' cy='12' r='8.5' fill='none' stroke='%23ffd166' stroke-width='1.6'/%3E%3Cpath d='M12 7.2v5.2l3.4 2' stroke='%23ffd166' stroke-width='1.6' fill='none' stroke-linecap='round'/%3E%3C/svg%3E">
<style>"#;

const STYLE: &str = r#"
:root{--bg:#0b1220;--card:#111a2e;--ink:#eaeef7;--mut:#93a1bd;--gold:#ffd166;--line:#22314f}
*{box-sizing:border-box}
body{margin:0;background:radial-gradient(1100px 700px at 85% -10%,#1d2f58 0%,var(--bg) 55%) fixed,#0b1220;color:var(--ink);font-family:-apple-system,BlinkMacSystemFont,"Segoe UI","Hiragino Kaku Gothic ProN","Noto Sans JP",Meiryo,sans-serif;line-height:1.7}
a{color:var(--gold)}
.wrap{max-width:840px;margin:0 auto;padding:26px 18px 64px}
.card{background:linear-gradient(180deg,rgba(255,255,255,.035),rgba(255,255,255,0)),var(--card);border:1px solid var(--line);border-radius:16px;padding:20px 22px;margin-bottom:14px}
label,.label{font-size:13px;color:var(--mut);letter-spacing:.04em}
.badge{display:inline-block;background:var(--gold);color:#21242c;border-radius:999px;padding:1px 10px;font-size:12px;font-weight:700;margin-left:6px}
.badge.blue{background:#7fb4ff}
.mono{font-variant-numeric:tabular-nums;letter-spacing:.03em}
.big{font:800 clamp(46px,10vw,72px)/1.15 ui-monospace,SFMono-Regular,Menlo,Consolas,"JetBrains Mono",monospace;color:var(--gold);text-shadow:0 0 24px rgba(255,209,102,.25)}
.big2{font:700 clamp(30px,6vw,44px)/1.2 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.small{font-size:13px}.mut{color:var(--mut)}
.hero{text-align:center;padding:30px 22px}
.greet{display:flex;gap:12px;align-items:baseline;justify-content:center;margin-bottom:14px;flex-wrap:wrap}
.ga{font-size:clamp(20px,4vw,30px);font-weight:800;color:#fff}
.gj{color:var(--mut);font-size:13px}
.diff{background:rgba(255,255,255,.03);border:1px dashed var(--line);border-radius:12px;padding:10px 16px;font-size:14px}
.chars b,.diff b{color:var(--gold)}
.grid2{display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:14px}
.grid2 .card{margin-bottom:0}
h3{margin:0 0 10px;font-size:15px}
form{display:flex;gap:10px;align-items:center;flex-wrap:wrap}
input[type=number]{background:#0c1526;color:var(--ink);border:1px solid var(--line);border-radius:10px;padding:9px 12px;font-size:16px;width:110px}
button{background:var(--gold);border:0;border-radius:10px;padding:9px 18px;font-weight:800;cursor:pointer;color:#21242c;font-size:14px}
.ql{color:var(--mut);text-decoration:none;border:1px solid var(--line);border-radius:999px;padding:4px 12px;font-size:12px}
.ql:hover{color:var(--gold);border-color:var(--gold)}
.result{margin-top:14px;border-top:1px solid var(--line);padding-top:14px}
.result table{width:100%;border-collapse:collapse}
.result th{text-align:left;color:var(--mut);font-weight:600;font-size:13px;padding:6px 8px;white-space:nowrap}
.result td{padding:6px 8px;font-variant-numeric:tabular-nums}
.result b{color:var(--gold);font-size:1.15em}
.chips{display:flex;flex-direction:column;gap:6px;margin-top:12px}
.chip{background:#0d1729;border:1px solid var(--line);border-radius:10px;padding:5px 10px;font-size:13px;display:flex;justify-content:space-between;font-variant-numeric:tabular-nums;white-space:nowrap}
.chip.cur{border-color:var(--gold);color:var(--gold)}
.chip .ar{color:#55648c}
.chip .dd{color:var(--mut);font-size:11px;margin-left:4px}
.sun{display:flex;gap:16px;flex-wrap:wrap;font-size:14px}
.sun b{color:var(--gold);font-variant-numeric:tabular-nums}
.till{margin-top:10px;font-weight:700;color:var(--gold)}
table.ev{width:100%;border-collapse:collapse;font-size:14px}
table.ev td{padding:7px 8px;border-bottom:1px solid var(--line);vertical-align:top}
table.ev td.d{width:92px;color:var(--mut);white-space:nowrap;font-variant-numeric:tabular-nums}
table.ev td.s{color:var(--mut);font-size:12.5px}
table.ev tr.hl td{background:rgba(255,209,102,.1)}
table.ev tr.past{opacity:.4}
.tag{font-size:11px;color:var(--gold);border:1px solid rgba(255,209,102,.5);border-radius:6px;padding:0 6px;margin-left:6px}
.banner{background:rgba(255,209,102,.08);border:1px solid rgba(255,209,102,.35);border-radius:10px;padding:8px 14px;margin-bottom:10px;font-size:14px}
ul.tl{list-style:none;margin:0;padding:0}
ul.tl li{padding:9px 0;border-bottom:1px solid var(--line);font-size:14px}
ul.tl li:last-child{border-bottom:0}
.kana{color:var(--mut);font-size:12px;margin-right:8px}
footer{margin-top:22px;font-size:12px;color:var(--mut);border-top:1px solid var(--line);padding-top:16px}
footer b{color:#c6d2e8}
"#;

const SCRIPT: &str = r#"(function () {
  var t = document.getElementById('tdata').textContent.split('|').map(Number);
  var bNow = t[0], bSalz = t[1], bJst = t[2], flag = t[3], target = t[4];
  var baseDay = Math.floor(bSalz / 86400);
  function p(n) { return (n < 10 ? '0' : '') + n; }
  function hms(sec) {
    sec = ((sec % 86400) + 86400) % 86400;
    return p(Math.floor(sec / 3600)) + ':' + p(Math.floor((sec % 3600) / 60)) + ':' + p(Math.floor(sec % 60));
  }
  function rem(s) {
    if (s >= 86400) return Math.floor(s / 86400) + '日' + Math.floor((s % 86400) / 3600) + '時間';
    if (s >= 3600) return Math.floor(s / 3600) + '時間' + Math.floor((s % 3600) / 60) + '分';
    return Math.max(1, Math.ceil(s / 60)) + '分';
  }
  function tick() {
    var now = Date.now();
    var drift = Math.floor((now - bNow) / 1000);
    var salz = bSalz + drift, jst = bJst + drift;
    var se = document.getElementById('salz');
    var je = document.getElementById('jst');
    if (se) se.textContent = hms(salz);
    if (je) je.textContent = hms(jst);
    if (Math.floor(salz / 86400) !== baseDay) { location.reload(); return; }
    var nowSec = Math.floor(now / 1000);
    var d = target - nowSec;
    if (d <= 0) { location.reload(); return; }
    var el = document.getElementById('tevent');
    if (el) el.textContent = (flag ? '日の出まで' : '日の入りまで') + ' あと ' + rem(d);
  }
  tick();
  setInterval(tick, 1000);
})();
"#;

const GREET_HTML: &str = r#"<div class="card"><h3>🇦🇹 オーストリア流の挨拶</h3><ul class="tl">
<li><b>Grüß Gott</b> <span class="kana">グリュース・ゴット</span>「神様のご加護があらんように」。オーストリアいちばん普通の挨拶（ドイツの "Guten Tag" に相当）</li>
<li><b>Servus</b> <span class="kana">セルヴース</span>親しい人への「こんにちは／じゃあまた」。挨拶と別れの両方に使える</li>
<li><b>Baba</b> <span class="kana">バーバ</span>「じゃあね〜」の気軽な帰り際の挨拶</li>
<li><b>Hawidere</b> <span class="kana">ハウヴィデーレ</span>"Wie geht's?"（元気？）が転訛した祝福を込めたご挨拶</li>
<li><b>Oida</b> <span class="kana">オイダ</span>ウィーンの若者言葉「ねー／マジで？」。呼びかけにも使う</li>
<li><b>Leiwand</b> <span class="kana">ライヴァント</span>「最高！／いいね！」という意味の褒め言葉</li>
</ul></div>"#;

const FOOT: &str = r#"<footer><b>Cloudflare Workers</b> で動いています。時刻・夏時間の切替・日の出日の入り・年中行事の計算は、すべてこのサイト内で実行しています（外部APIなし）。
ザルツブルクはオーストリア全土と同じ <b>Europe/Vienna（CET/CEST）</b>。EUの夏時間は 3月最終日曜 02:00→03:00、10月最終日曜 03:00→02:00 に切り替わります。</footer>"#;

fn human_hours(h: i64) -> String {
    if h >= 24 {
        format!("{}日{}時間", h / 24, h % 24)
    } else {
        format!("{}時間", h)
    }
}

fn human_remaining(secs: i64) -> String {
    if secs >= 86400 {
        format!("{}日{}時間", secs / 86400, (secs % 86400) / 3600)
    } else if secs >= 3600 {
        format!("{}時間{}分", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}分", (secs + 59) / 60)
    }
}

/// 24時間の変換チップ。at_to_jp=true なら「ザルツブルク→日本」
fn chip_block(at_to_jp: bool, dh: i64, cur_h: i64) -> String {
    let mut out = String::from("<div class=\"chips\">");
    for h in 0i64..24 {
        let (o, tag) = if at_to_jp {
            let o = h + dh;
            if o >= 24 {
                (o - 24, "翌日")
            } else {
                (o, "")
            }
        } else {
            let o = h - dh;
            if o < 0 {
                (o + 24, "前日")
            } else {
                (o, "")
            }
        };
        let cur = if h == cur_h { " cur" } else { "" };
        let taghtml = if tag.is_empty() {
            String::new()
        } else {
            format!("<span class=\"dd\">{tag}</span>")
        };
        out.push_str(&format!(
            "<div class=\"chip{cur}\"><span>{:02}:00</span><span class=\"ar\">→</span><span>{:02}:00{}{}</span></div>",
            h, o, "", taghtml
        ));
    }
    out.push_str("</div>");
    out
}

/// 日の出入りUTU分をザルツブルクの現地時刻表示に変換
fn fmt_local_min(y: i32, m: u32, d: u32, utc_min: f64) -> String {
    let z = days_from_civil(y, m, d);
    let um = utc_min as i64;
    let off = vienna_offset_seconds(z * 86400 + um * 60);
    let lm = um + off / 60;
    format!("{}:{}", p2(lm / 60), p2(lm % 60))
}

pub fn page(now_ms: u64, later_hours: Option<i64>) -> String {
    let now_secs = (now_ms as i64) / 1000;
    let off_s = vienna_offset_seconds(now_secs);
    let is_summer = off_s == 7200;
    let dh = 9 - off_s / 3600; // 7 (夏) / 8 (冬)
    let salz_abs = now_secs + off_s;
    let jst_abs = now_secs + 9 * 3600;
    let ls = to_local(salz_abs);
    let lj = to_local(jst_abs);

    // --- 挨拶（ザルツブルク時刻ベース）
    let (greet_at, greet_jp) = match ls.h {
        5..=9 => ("Guten Morgen!", "おはようございます"),
        10..=16 => ("Grüß Gott", "こんにちは（オーストリア定番の挨拶）"),
        17..=20 => ("Guten Abend", "こんばんは"),
        _ => ("Gute Nacht", "おやすみなさい"),
    };
    let season = if is_summer {
        "夏時間 CEST (UTC+2)"
    } else {
        "冬時間 CET (UTC+1)"
    };

    let hero = format!(
        "<div class=\"card hero\"><div class=\"greet\"><span class=\"ga\">{}</span><span class=\"gj\">{}</span></div><div class=\"label\">🇦🇹 ザルツブルク Salzburg<span class=\"badge\">{}</span></div><div class=\"big mono\" id=\"salz\">{}</div><div class=\"small mut\">{}</div></div>\n<div class=\"card\"><div class=\"label\">🇯🇵 日本時間<span class=\"badge blue\">JST (UTC+9)</span></div><div class=\"big2 mono\" id=\"jst\">{}</div><div class=\"small mut\">{}</div></div>\n<p class=\"diff\">🇯🇵 日本はザルツブルクより <b>{}時間</b> 進んでいます。 <span class=\"mut small\">（夏は7時間差、冬は8時間差）</span></p>",
        greet_at,
        greet_jp,
        season,
        fmt_hms(ls.h * 3600 + ls.mi * 60 + ls.s),
        fmt_ymd(&ls),
        fmt_hms(lj.h * 3600 + lj.mi * 60 + lj.s),
        fmt_ymd(&lj),
        dh
    );

    // --- あと◯時間 tool
    let mut qlinks = String::new();
    for h in 1i64..=12 {
        qlinks.push_str(&format!("<a class=\"ql\" href=\"/?h={h}\">+{h}時間</a> "));
    }
    let val_attr = match later_hours {
        Some(h) => format!("value=\"{h}\""),
        None => String::new(),
    };
    let form_html = format!(
        "<div class=\"card\"><h3>🧮 今から◯時間後は何時？</h3><form method=\"get\" action=\"/\"><input type=\"number\" name=\"h\" min=\"1\" max=\"12\" placeholder=\"例: 9\" {val}><button>計算</button>{qlinks}</form>{result}</div>",
        val = val_attr,
        qlinks = qlinks,
        result = match later_hours {
            Some(h) => {
                let t = now_secs + h * 3600;
                let off_t = vienna_offset_seconds(t);
                let tsl = to_local(t + off_t);
                let tjl = to_local(t + 9 * 3600);
                let dstn = if off_t != off_s {
                    "<div class=\"small\" style=\"color:var(--gold);margin-top:6px\">※ 夏時間・冬時間の切替をまたいでいます</div>"
                } else {
                    ""
                };
                format!(
                    "<div class=\"result\"><b>今から{}</b><table class=\"rt\"><tr><th>日本 🇯🇵</th><td>{} <b>{}</b></td></tr><tr><th>ザルツブルク 🇦🇹</th><td>{} <b>{}</b></td></tr></table>{}</div>",
                    format!("{}後", human_hours(h)),
                    fmt_ymd(&tjl),
                    fmt_hm(tjl.h * 3600 + tjl.mi * 60),
                    fmt_ymd(&tsl),
                    fmt_hm(tsl.h * 3600 + tsl.mi * 60),
                    dstn
                )
            }
            None => String::new(),
        }
    );

    // --- 変換表
    let chips_at = chip_block(true, dh, ls.h);
    let chips_jp = chip_block(false, dh, lj.h);
    let tables_html = format!(
        "<div class=\"grid2\"><div class=\"card\"><h3>🔄 ザルツブルク → 日本</h3><p class=\"small mut\">ザルツブルクの0〜23時が日本では何時か（+{}時間）</p>{}</div><div class=\"card\"><h3>🔄 日本 → ザルツブルク</h3><p class=\"small mut\">日本の0〜23時がザルツブルクでは何時か（−{}時間）</p>{}</div></div>",
        dh, chips_at, dh, chips_jp
    );

    // --- 日の出入り
    let z0 = days_from_civil(ls.y, ls.m, ls.d);
    let sun = sun_for_date(ls.y, ls.m, ls.d, SALZ_LAT, SALZ_LON);
    let mut cands: Vec<(bool, i64)> = Vec::new();
    for k in 0..2i64 {
        let (yy, mm, dd) = civil_from_days(z0 + k);
        let s = sun_for_date(yy, mm, dd, SALZ_LAT, SALZ_LON);
        if let Some(sr) = s.sunrise_utc_min {
            cands.push((true, (z0 + k) * 86400 + (sr as i64) * 60));
        }
        if let Some(ss) = s.sunset_utc_min {
            cands.push((false, (z0 + k) * 86400 + (ss as i64) * 60));
        }
    }
    cands.retain(|&(_, t)| t > now_secs);
    cands.sort_by_key(|&(_, t)| t);
    let (tevent, ev_flag, ev_target) = match cands.first() {
        Some(&(is_sr, t)) => (
            format!(
                "{}まで あと {}",
                if is_sr { "日の出" } else { "日の入り" },
                human_remaining(t - now_secs)
            ),
            is_sr,
            t,
        ),
        None => ("—".to_string(), false, 0),
    };
    let solar_html = match (sun.sunrise_utc_min, sun.sunset_utc_min) {
        (Some(sr), Some(ss)) => {
            let dur = (ss - sr) as i64;
            format!(
               "<div class=\"card\"><h3>☀️ ザルツブルクの日の出・日の入り（{}年{}月{}日）</h3><div class=\"sun\"><span>🌅 日の出 <b>{}</b></span><span>🌇 日の入り <b>{}</b></span><span>☀️昼の長さ <b>{}時間{}分</b></span></div><div class=\"till\" id=\"tevent\">{}</div><p class=\"small mut\">NOAA近似式（緯度47.81°N・経度13.05°E）+ 夏時間の切替にも対応</p></div>",
                ls.y, ls.m, ls.d,
                fmt_local_min(ls.y, ls.m, ls.d, sr),
                fmt_local_min(ls.y, ls.m, ls.d, ss),
                dur / 60, dur % 60,
                tevent
            )
        }
        _ => "<div class=\"card\"><h3>☀️ ザルツブルクの日の出・日の入り</h3><p class=\"mut\">この緯度では計算できません</p></div>".to_string(),
    };

    // --- 年中行事
    let yev = events_for_year(ls.y);
    let today_day = days_from_civil(ls.y, ls.m, ls.d);
    let mut rows = String::new();
    for ev in &yev {
        let wd = weekday_days(ev.day);
        let cls = if ev.day == today_day {
            " class=\"hl\""
        } else if ev.day < today_day {
            " class=\"past\""
        } else {
            ""
        };
        let tag = if ev.holiday {
            "<span class=\"tag\">祝日</span>"
        } else {
            ""
        };
        rows.push_str(&format!(
            "<tr{}><td class=\"d\">{:02}/{:02} ({})</td><td class=\"n\">{} {}</td><td class=\"s\">{}</td></tr>",
            cls, ev.m, ev.d, WD_KANJI[wd as usize], ev.name, tag, ev.note
        ));
    }
    let banner = match yev.iter().find(|e| e.day == today_day) {
        Some(ev) => format!(
            "<div class=\"banner\">🎉 今日（{:02}/{:02}）は <b>{}</b> — {}</div>",
            ev.m, ev.d, ev.name, ev.note
        ),
        None => match yev.iter().find(|e| e.day > today_day) {
            Some(ev) => format!(
                "<div class=\"banner\">⏳ 次の行事は <b>{}</b>（{:02}/{:02}）まであと <b>{}日</b></div>",
                ev.name, ev.m, ev.d, ev.day - today_day
            ),
            None => "<div class=\"banner\">年内の行事はすべて終了しました</div>".to_string(),
        },
    };
    let events_html = format!(
        "<div class=\"card\"><h3>🎄 オーストリアの年中行事（{}年）</h3>{}<table class=\"ev\">{}</table><p class=\"small mut\">「祝日」= オーストリアの公的休日（Feiertag）</p></div>",
        ls.y, banner, rows
    );

    // --- 組み立て
    let mut html = String::with_capacity(20000);
    html.push_str(HEAD);
    html.push_str(STYLE);
    html.push_str("</style></head><body>\n<div class=\"wrap\">\n");
    html.push_str(&hero);
    html.push_str(&form_html);
    html.push_str(&tables_html);
    html.push_str(&solar_html);
    html.push_str(&events_html);
    html.push_str(GREET_HTML);
    html.push_str(FOOT);
    html.push_str("\n</div>\n");
    html.push_str(&format!(
        "<span id=\"tdata\" hidden>{}|{}|{}|{}|{}</span>\n",
        now_ms as i64, salz_abs, jst_abs, ev_flag as i64, ev_target
    ));
    html.push_str("<script>");
    html.push_str(SCRIPT);
    html.push_str("</script>\n</body></html>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_contains_cities() {
        let html = page(1_792_450_000_000, None);
        assert!(html.contains("ザルツブルク"), "タイトルあり");
        assert!(html.contains("日本時間"));
        assert!(html.contains("tdata"));
    }

    #[test]
    fn page_with_later_hours() {
        let html = page(1_792_450_000_000, Some(9));
        assert!(html.contains("9時間後"), "計算結果が入る");
        let plain = page(1_792_450_000_000, None);
        assert!(!plain.contains("9時間後"), "平常時は出ない");
    }
}
