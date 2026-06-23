use std::path::Path;

use log::info;

const PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>eipi.boo</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      background: #faf4ed;
      color: #575279;
      font-family: 'Courier New', monospace;
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 100vh;
    }
    .container {
      text-align: center;
      padding: 2rem;
    }
    h1 {
      font-size: 3rem;
      color: #b4637a;
      margin-bottom: 0.5rem;
    }
    .tagline {
      color: #9893a5;
      font-size: 1.1rem;
      margin-bottom: 2.5rem;
    }
    .cmd {
      background: #f2e9e1;
      border: 1px solid #dfdad9;
      border-radius: 8px;
      padding: 1.2rem 2rem;
      display: inline-block;
      margin-bottom: 2rem;
    }
    .cmd span {
      color: #56949f;
      font-size: 1.3rem;
    }
    .cmd code {
      color: #286983;
      font-size: 1.3rem;
      font-weight: bold;
    }
    .footer {
      color: #9893a5;
      font-size: 0.85rem;
    }
    .footer a {
      color: #907aa9;
      text-decoration: none;
    }
    .footer a:hover {
      text-decoration: underline;
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>eipi.boo</h1>
    <p class="tagline">anonymous confessions over ssh</p>
    <div class="cmd">
      <span>$ </span><code>ssh eipi.boo</code>
    </div>
    <p class="footer">
      <a href="https://github.com/pwnwriter/eipi.boo">source</a>
    </p>
  </div>
</body>
</html>"#;

pub fn write_index() {
    let web_dir = std::env::var("EIPI_WEB_DIR").unwrap_or_else(|_| "web".to_string());
    let path = Path::new(&web_dir).join("index.html");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    match std::fs::write(&path, PAGE) {
        Ok(_) => info!("Web page written to {}", path.display()),
        Err(e) => info!("Failed to write web page: {}", e),
    }
}
