use worker::*;

mod civil;
mod events;
mod render;
mod solar;

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    if !matches!(req.method(), Method::Get | Method::Head) {
        return Response::error("Method Not Allowed", 405);
    }

    // クエリパラメータ: ?h=<1..12> 「今から◯時間後」の変換
    let mut later: Option<i64> = None;
    if let Ok(url) = req.url() {
        if let Some(q) = url.query() {
            for pair in q.split('&') {
                let Some(v) = pair.strip_prefix("h=") else { continue };
                if v.len() <= 5 {
                    if let Ok(n) = v.parse::<i64>() {
                        if (1..=12).contains(&n) {
                            later = Some(n);
                        }
                    }
                }
            }
        }
    }

    let now_ms = Date::now().as_millis();
    let html = render::page(now_ms, later);

    Response::builder()
        .with_header("Cache-Control", "no-store")?
        .from_html(html)
}
