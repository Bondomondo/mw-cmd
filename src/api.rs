use std::fs;
use std::path::PathBuf;

use reqwest::blocking::Client;
use serde_json::Value;

const BASE_URL: &str = "https://www.marketsight.app/ext/v1";

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config/mw/api_key")
}

pub fn load_api_key() -> Result<String, String> {
    if let Ok(k) = std::env::var("MARKETWATCH_API_KEY") {
        if !k.is_empty() {
            return Ok(k);
        }
    }
    let path = config_path();
    fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|_| format!(
            "No API key found. Set MARKETWATCH_API_KEY or run: mw config set-key <key>"
        ))
}

pub fn save_api_key(key: &str) -> Result<(), String> {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| e.to_string())?;
    fs::write(&path, key.trim())
        .map_err(|e| e.to_string())?;
    // chmod 600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    println!("API key saved to {}", path.display());
    Ok(())
}

pub fn config_key_path() -> PathBuf {
    config_path()
}

fn get_with_timeout(path: &str, query: Option<&[(&str, &str)]>, timeout_secs: u64) -> Result<Value, String> {
    let key = load_api_key()?;
    let url = format!("{BASE_URL}/{}", path.trim_start_matches('/'));
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| e.to_string())?;

    let mut req = client.get(&url).header("X-Api-Key", &key);
    if let Some(params) = query {
        req = req.query(params);
    }

    let resp = req.send().map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }
    resp.json::<Value>().map_err(|e| format!("JSON parse error: {e}"))
}

fn get(path: &str, query: Option<&[(&str, &str)]>) -> Result<Value, String> {
    get_with_timeout(path, query, 20)
}

pub fn get_market() -> Result<Value, String> {
    get("market", None)
}

pub fn get_news() -> Result<Value, String> {
    get("news", None)
}

pub fn get_calendar() -> Result<Value, String> {
    get("calendar", None)
}

pub fn get_watchlist() -> Result<Value, String> {
    get("watchlist", None)
}

pub fn get_quote(symbol: &str) -> Result<Value, String> {
    get("quote", Some(&[("symbol", symbol)]))
}

pub fn get_portfolio() -> Result<Value, String> {
    get("portfolio", None)
}

pub fn get_performance() -> Result<Value, String> {
    get("performance", None)
}

pub fn get_stock_performance(symbol: &str) -> Result<Value, String> {
    get(&format!("performance/{symbol}"), None)
}

pub fn get_brief(force: bool) -> Result<Value, String> {
    let params: Option<Vec<(&str, &str)>> = if force { Some(vec![("force", "true")]) } else { None };
    get_with_timeout("brief", params.as_deref(), 120)
}

pub fn get_portfolio_brief(force: bool) -> Result<Value, String> {
    let params: Option<Vec<(&str, &str)>> = if force { Some(vec![("force", "true")]) } else { None };
    get_with_timeout("portfolio/brief", params.as_deref(), 120)
}
