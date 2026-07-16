use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use log::info;

use crate::server::AppState;

fn landing_page(
    confessions: i64,
    humans: i64,
    replies: i64,
    reactions: i64,
    online: usize,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>eipi.boo</title>
  <meta property="og:title" content="eipi.boo">
  <meta property="og:description" content="{} confessions from {} strangers over ssh">
  <meta property="og:type" content="website">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{
      background: #faf4ed;
      color: #575279;
      font-family: 'JetBrains Mono', 'Courier New', monospace;
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 100vh;
    }}
    .container {{
      text-align: center;
      padding: 2rem;
    }}
    h1 {{
      font-size: 3rem;
      color: #b4637a;
      margin-bottom: 0.5rem;
    }}
    .tagline {{
      color: #9893a5;
      font-size: 1.1rem;
      margin-bottom: 2rem;
    }}
    .stats {{
      display: flex;
      justify-content: center;
      gap: 2rem;
      margin-bottom: 2rem;
      flex-wrap: wrap;
    }}
    .stat {{
      text-align: center;
    }}
    .stat-num {{
      font-size: 1.8rem;
      color: #b4637a;
      font-weight: bold;
      display: block;
    }}
    .stat-label {{
      font-size: 0.8rem;
      color: #9893a5;
    }}
    .cmd {{
      background: #f2e9e1;
      border: 1px solid #dfdad9;
      border-radius: 8px;
      padding: 1.2rem 2rem;
      display: inline-block;
      margin-bottom: 1.5rem;
    }}
    .cmd span {{
      color: #56949f;
      font-size: 1.3rem;
    }}
    .cmd code {{
      color: #286983;
      font-size: 1.3rem;
      font-weight: bold;
    }}
    .online {{
      color: #56949f;
      font-size: 0.85rem;
      margin-bottom: 1.5rem;
    }}
    .online .dot {{
      display: inline-block;
      width: 8px;
      height: 8px;
      background: #56949f;
      border-radius: 50%;
      margin-right: 4px;
      animation: pulse 2s infinite;
    }}
    @keyframes pulse {{
      0%, 100% {{ opacity: 1; }}
      50% {{ opacity: 0.4; }}
    }}
    .footer {{
      color: #9893a5;
      font-size: 0.85rem;
    }}
    .footer a {{
      color: #907aa9;
      text-decoration: none;
    }}
    .footer a:hover {{
      text-decoration: underline;
    }}
  </style>
</head>
<body>
  <div class="container">
    <h1>eipi.boo</h1>
    <p class="tagline">confess over ssh</p>
    <div class="stats">
      <div class="stat">
        <span class="stat-num">{}</span>
        <span class="stat-label">confessions</span>
      </div>
      <div class="stat">
        <span class="stat-num">{}</span>
        <span class="stat-label">humans</span>
      </div>
      <div class="stat">
        <span class="stat-num">{}</span>
        <span class="stat-label">replies</span>
      </div>
      <div class="stat">
        <span class="stat-num">{}</span>
        <span class="stat-label">reactions</span>
      </div>
    </div>
    <div class="cmd">
      <span>$ </span><code>ssh eipi.boo</code>
    </div>
    <p class="online"><span class="dot"></span>{} online now</p>
    <p class="footer">
      <a href="https://github.com/pwnwriter/eipi.boo">source</a>
    </p>
  </div>
</body>
</html>"#,
        confessions, humans, confessions, humans, replies, reactions, online,
    )
}

fn build_ascii_card(
    text: &str,
    age: &str,
    love: i64,
    replies: i64,
    reactions: i64,
    position: &str,
) -> String {
    let iw: usize = 46;
    let wrapped = crate::model::confession::wrap_text(text, iw - 4);

    let b = "#9893a5"; // border color
    let mut lines: Vec<String> = Vec::new();

    // top border
    lines.push(format!(
        "<span style=\"color:{b}\">╭{}╮</span>",
        "─".repeat(iw)
    ));

    // dots + age
    let age_display = format!(" {} ", age);
    let dots = "<span style=\"color:#b4637a\">●</span> <span style=\"color:#ea9d34\">●</span> <span style=\"color:#56949f\">●</span>";
    let dots_pad = iw.saturating_sub(7 + age_display.len());
    lines.push(format!(
        "<span style=\"color:{b}\">│</span>  {}{}<span style=\"color:{b}\">{}</span><span style=\"color:{b}\">│</span>",
        dots,
        " ".repeat(dots_pad),
        html_escape(&age_display),
    ));

    // separator
    lines.push(format!(
        "<span style=\"color:{b}\">│{}│</span>",
        "─".repeat(iw)
    ));

    // empty line
    lines.push(format!(
        "<span style=\"color:{b}\">│</span>{}<span style=\"color:{b}\">│</span>",
        " ".repeat(iw)
    ));

    // text lines
    for line in &wrapped {
        let dcols = line.chars().count();
        let right_pad = iw.saturating_sub(dcols + 2);
        lines.push(format!(
            "<span style=\"color:{b}\">│</span>  {}{}<span style=\"color:{b}\">│</span>",
            html_escape(line),
            " ".repeat(right_pad)
        ));
    }

    // empty line
    lines.push(format!(
        "<span style=\"color:{b}\">│</span>{}<span style=\"color:{b}\">│</span>",
        " ".repeat(iw)
    ));

    // separator
    lines.push(format!(
        "<span style=\"color:{b}\">│{}│</span>",
        "─".repeat(iw)
    ));

    // footer
    let mut footer_left = String::from("  ");
    let mut footer_left_len: usize = 2;
    if love > 0 {
        footer_left.push_str(&format!("<span style=\"color:#b4637a\">♥</span> {}", love));
        footer_left_len += 2 + love.to_string().len();
    }
    if replies > 0 {
        footer_left.push_str(&format!(
            "  <span style=\"color:#56949f\">↩</span> {}",
            replies
        ));
        footer_left_len += 4 + replies.to_string().len();
    }
    if reactions > 0 {
        footer_left.push_str(&format!(
            "  <span style=\"color:#ea9d34\">✦</span> {}",
            reactions
        ));
        footer_left_len += 4 + reactions.to_string().len();
    }
    let pos_part = format!("{}  ", position);
    let footer_pad = iw.saturating_sub(footer_left_len + pos_part.len());
    lines.push(format!(
        "<span style=\"color:{b}\">│</span>{}{}<span style=\"color:{b}\">{}</span><span style=\"color:{b}\">│</span>",
        footer_left,
        " ".repeat(footer_pad),
        pos_part
    ));

    // bottom border with stem
    let mid = iw / 2;
    lines.push(format!(
        "<span style=\"color:{b}\">╰{}┬{}╯</span>",
        "─".repeat(mid),
        "─".repeat(iw - mid - 1)
    ));

    // ascii man
    let center = mid + 1;
    let face = match text.len() {
        0..70 => "\\(^_^)/",
        70..150 => "\\(o_o)/",
        150..220 => "\\(>_<)/",
        _ => "\\(x_x)/",
    };
    lines.push(format!(
        "<span style=\"color:{b}\">{}│</span>",
        " ".repeat(center)
    ));
    lines.push(format!(
        "<span style=\"color:{b}\">{}{}</span>",
        " ".repeat(center.saturating_sub(3)),
        face
    ));
    lines.push(format!(
        "<span style=\"color:{b}\">{}│</span>",
        " ".repeat(center)
    ));
    lines.push(format!(
        "<span style=\"color:{b}\">{}/ \\</span>",
        " ".repeat(center.saturating_sub(1))
    ));

    lines.join("\n")
}

fn build_replies_html(replies: &[crate::model::reply::Reply]) -> String {
    if replies.is_empty() {
        return String::new();
    }

    let mut out = format!(
        "<div class=\"replies\"><div class=\"replies-head\">↩ {} {}</div>",
        replies.len(),
        if replies.len() == 1 {
            "reply"
        } else {
            "replies"
        },
    );

    for reply in replies {
        let age = crate::model::confession::time_ago(&reply.replied_at);
        out.push_str(&format!(
            "<div class=\"reply\"><div class=\"reply-head\"><span class=\"reply-name\">{}</span><span class=\"reply-time\">· {}</span></div><div class=\"reply-text\">{}</div></div>",
            html_escape(&reply.name),
            html_escape(&age),
            html_escape(&reply.text),
        ));
    }

    out.push_str("</div>");
    out
}

fn confession_page(
    id: i64,
    text: &str,
    age: &str,
    love: i64,
    reactions: i64,
    replies: &[crate::model::reply::Reply],
    total: i64,
) -> String {
    let reply_count = replies.len() as i64;
    let truncated: String = text.chars().take(160).collect();
    let og_desc = format!(
        "{} | {} reactions, {} replies",
        truncated, reactions, reply_count
    );
    let position = format!("{}/{}", id, total);
    let ascii_card = build_ascii_card(text, age, love, reply_count, reactions, &position);
    let replies_html = build_replies_html(replies);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>confession #{} | eipi.boo</title>
  <meta property="og:title" content="confession #{} | eipi.boo">
  <meta property="og:description" content="{}">
  <meta property="og:type" content="article">
  <meta property="og:url" content="https://eipi.boo/c/{}">
  <meta name="twitter:card" content="summary">
  <meta name="twitter:title" content="confession #{} | eipi.boo">
  <meta name="twitter:description" content="{}">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{
      background: #faf4ed;
      color: #575279;
      font-family: 'JetBrains Mono', 'Courier New', monospace;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: center;
      min-height: 100vh;
    }}
    .ascii {{
      white-space: pre;
      font-size: 14px;
      line-height: 1.4;
      color: #575279;
      font-family: 'JetBrains Mono', 'Courier New', monospace;
    }}
    .ascii .border {{ color: #9893a5; }}
    .ascii .dots-red {{ color: #b4637a; }}
    .ascii .dots-yellow {{ color: #ea9d34; }}
    .ascii .dots-green {{ color: #56949f; }}
    .ascii .age {{ color: #9893a5; }}
    .ascii .heart {{ color: #b4637a; }}
    .ascii .man {{ color: #9893a5; }}
    .cta {{
      text-align: center;
      margin-top: 2rem;
    }}
    .cta .cmd {{
      background: #f2e9e1;
      border: 1px solid #dfdad9;
      border-radius: 6px;
      padding: 0.6rem 1.2rem;
      display: inline-block;
    }}
    .cta .cmd code {{
      color: #286983;
      font-weight: bold;
    }}
    .actions {{
      margin-top: 1.5rem;
      display: flex;
      gap: 0.5rem;
      flex-wrap: wrap;
      justify-content: center;
    }}
    .actions button {{
      background: #f2e9e1;
      border: 1px solid #dfdad9;
      border-radius: 6px;
      padding: 0.5rem 1rem;
      color: #575279;
      font-family: 'JetBrains Mono', 'Courier New', monospace;
      font-size: 0.8rem;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 0.4rem;
      transition: background 0.15s;
    }}
    .actions button:hover {{
      background: #dfdad9;
    }}
    .actions button.copied {{
      background: #56949f;
      color: #faf4ed;
      border-color: #56949f;
    }}
    .actions .share-row {{
      display: flex;
      gap: 0.4rem;
    }}
    .actions .share-row button {{
      padding: 0.5rem 0.7rem;
    }}
    .replies {{
      width: 100%;
      max-width: 480px;
      margin: 2rem auto 0;
      padding: 0 1rem;
    }}
    .replies-head {{
      color: #56949f;
      font-size: 0.85rem;
      margin-bottom: 0.75rem;
    }}
    .reply {{
      background: #f2e9e1;
      border-left: 2px solid #907aa9;
      border-radius: 4px;
      padding: 0.6rem 0.9rem;
      margin-bottom: 0.6rem;
    }}
    .reply-head {{
      display: flex;
      gap: 0.5rem;
      align-items: baseline;
      margin-bottom: 0.3rem;
    }}
    .reply-name {{
      color: #907aa9;
      font-weight: bold;
      font-size: 0.8rem;
    }}
    .reply-time {{
      color: #9893a5;
      font-size: 0.75rem;
    }}
    .reply-text {{
      color: #575279;
      font-size: 0.9rem;
      line-height: 1.4;
      white-space: pre-wrap;
      word-break: break-word;
    }}
  </style>
</head>
<body>
  <pre class="ascii" id="card">{}</pre>
  {}
  <div class="cta">
    <p style="color: #9893a5; font-size: 0.85rem; margin-bottom: 0.5rem;">react and reply over ssh</p>
    <div class="cmd"><code>$ ssh eipi.boo</code></div>
  </div>
  <div class="actions">
    <button onclick="copyLink()"><span id="copy-icon">⎘</span> <span id="copy-text">copy link</span></button>
    <div class="share-row">
      <button onclick="shareX()" title="share on X">𝕏</button>
      <button onclick="shareFB()" title="share on Facebook">f</button>
      <button onclick="shareWA()" title="share on WhatsApp">w</button>
      <button id="native-share" onclick="nativeShare()" title="share" style="display:none">↗ share</button>
    </div>
    <button onclick="saveImage()">⤓ save image</button>
  </div>
  <canvas id="canvas" style="display:none;"></canvas>
  <script>
  const pageUrl = 'https://eipi.boo/c/{}';
  const shareText = 'anonymous confession on eipi.boo';

  function copyLink() {{
    navigator.clipboard.writeText(pageUrl).then(() => {{
      const btn = document.getElementById('copy-text');
      const icon = document.getElementById('copy-icon');
      btn.textContent = 'copied!';
      icon.textContent = '✓';
      btn.parentElement.classList.add('copied');
      setTimeout(() => {{
        btn.textContent = 'copy link';
        icon.textContent = '⎘';
        btn.parentElement.classList.remove('copied');
      }}, 2000);
    }});
  }}

  function shareX() {{
    window.open('https://x.com/intent/tweet?url=' + encodeURIComponent(pageUrl) + '&text=' + encodeURIComponent(shareText), '_blank');
  }}

  function shareFB() {{
    window.open('https://www.facebook.com/sharer/sharer.php?u=' + encodeURIComponent(pageUrl), '_blank');
  }}

  function shareWA() {{
    window.open('https://wa.me/?text=' + encodeURIComponent(shareText + ' ' + pageUrl), '_blank');
  }}

  function nativeShare() {{
    navigator.share({{ title: 'confession on eipi.boo', text: shareText, url: pageUrl }}).catch(() => {{}});
  }}

  if (navigator.share) {{
    document.getElementById('native-share').style.display = 'inline-flex';
  }}

  async function saveImage() {{
    await document.fonts.ready;
    const pre = document.getElementById('card');
    const canvas = document.getElementById('canvas');
    const ctx = canvas.getContext('2d');
    const lines = pre.textContent.split('\n');
    const fontSize = 14;
    const lineHeight = fontSize * 1.4;
    const padding = 40;
    ctx.font = fontSize + 'px JetBrains Mono, Courier New, monospace';
    let maxWidth = 0;
    for (const line of lines) {{
      const w = ctx.measureText(line).width;
      if (w > maxWidth) maxWidth = w;
    }}
    canvas.width = maxWidth + padding * 2;
    canvas.height = lines.length * lineHeight + padding * 2;
    ctx.fillStyle = '#faf4ed';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.font = fontSize + 'px JetBrains Mono, Courier New, monospace';
    ctx.fillStyle = '#575279';
    ctx.textBaseline = 'top';
    for (let i = 0; i < lines.length; i++) {{
      ctx.fillText(lines[i], padding, padding + i * lineHeight);
    }}
    ctx.fillStyle = '#9893a5';
    ctx.font = '12px JetBrains Mono, Courier New, monospace';
    ctx.fillText('eipi.boo', canvas.width - padding - ctx.measureText('eipi.boo').width, canvas.height - 24);
    const link = document.createElement('a');
    link.download = 'confession-{}.png';
    link.href = canvas.toDataURL('image/png');
    link.click();
  }}
  </script>
</body>
</html>"#,
        id,
        id,
        html_escape(&og_desc),
        id,
        id,
        html_escape(&og_desc),
        ascii_card,
        replies_html,
        id,
        id,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

async fn landing(State(state): State<Arc<AppState>>) -> Html<String> {
    let db = state.db.lock();
    let stats = crate::db::stats(&db);
    drop(db);
    let online = state.online.load(std::sync::atomic::Ordering::Relaxed);
    Html(landing_page(
        stats.confessions,
        stats.humans,
        stats.replies,
        stats.reactions,
        online,
    ))
}

async fn confession(Path(id): Path<i64>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock();
    let Some(c) = crate::db::get_by_id(&db, id) else {
        drop(db);
        return (
            axum::http::StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            Html(String::from(
                "<h1>not found</h1><p><a href=\"/\">back to eipi.boo</a></p>",
            )),
        );
    };

    let age = crate::model::confession::time_ago(&c.created_at);
    let reactions: i64 = c.reactions.iter().map(|r| r.count).sum();
    let love = crate::model::confession::love_reactions(&c);
    let replies = crate::db::get_replies(&db, id);
    let stats = crate::db::stats(&db);
    drop(db);

    let page = confession_page(
        id,
        &c.text,
        &age,
        love,
        reactions,
        &replies,
        stats.confessions,
    );
    (
        axum::http::StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(page),
    )
}

pub async fn serve(state: Arc<AppState>) {
    let http_addr =
        std::env::var("EIPI_HTTP_LISTEN").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let app = Router::new()
        .route("/", get(landing))
        .route("/c/{id}", get(confession))
        .with_state(state);

    info!("Starting HTTP server on {}", http_addr);

    let listener = match tokio::net::TcpListener::bind(&http_addr).await {
        Ok(l) => l,
        Err(e) => {
            log::warn!("Failed to bind HTTP server on {}: {}", http_addr, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        log::warn!("HTTP server error: {}", e);
    }
}
