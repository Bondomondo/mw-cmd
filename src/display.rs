use colored::*;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};
use serde_json::Value;

// ── formatting helpers ────────────────────────────────────────────────────────

fn thousands(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn fmt_f(v: f64, decimals: usize) -> String {
    let sign = if v < 0.0 { "-" } else { "" };
    let abs = v.abs();
    let int_part = abs as u64;
    let int_str = thousands(int_part);
    if decimals == 0 {
        return format!("{sign}{int_str}");
    }
    let factor = 10u64.pow(decimals as u32) as f64;
    let frac_part = ((abs - int_part as f64) * factor).round() as u64;
    format!("{sign}{int_str}.{frac_part:0>decimals$}")
}

fn compact(v: f64) -> String {
    let abs = v.abs();
    let sign = if v < 0.0 { "-" } else { "" };
    if abs >= 1_000_000.0 {
        format!("{sign}{:.2}M", abs / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{sign}{:.1}K", abs / 1_000.0)
    } else {
        format!("{sign}{:.2}", abs)
    }
}

fn f64_val(v: &Value) -> Option<f64> {
    v.as_f64()
}

fn str_val<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("—")
}

fn signed(v: f64, formatted: &str) -> String {
    if v >= 0.0 { format!("+{formatted}") } else { formatted.to_string() }
}

fn change_cell(change: Option<f64>, pct: Option<f64>, compact_mode: bool) -> Cell {
    let val = pct.or(change).unwrap_or(0.0);
    let color = if val >= 0.0 { Color::Green } else { Color::Red };
    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = change {
        let s = if compact_mode { compact(c) } else { fmt_f(c, 2) };
        parts.push(signed(c, &s));
    }
    if let Some(p) = pct {
        parts.push(format!("{:+.2}%", p));
    }
    let text = if parts.is_empty() { "—".to_string() } else { parts.join(" ") };
    Cell::new(text).fg(color)
}

fn header_cell(text: &str) -> Cell {
    Cell::new(text)
        .add_attribute(Attribute::Bold)
        .fg(Color::White)
}

fn dim(text: &str) -> String {
    text.bright_black().to_string()
}

fn new_table() -> Table {
    let mut t = Table::new();
    t.load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);
    t
}

// ── commands ──────────────────────────────────────────────────────────────────

pub fn show_market(data: &Value) {
    let quotes = match data["quotes"].as_array() {
        Some(q) => q,
        None => { eprintln!("No market data."); return; }
    };

    // group by region
    let mut regions: Vec<String> = Vec::new();
    let mut by_region: std::collections::BTreeMap<String, Vec<&Value>> = Default::default();
    for q in quotes {
        let region = q["region"].as_str().unwrap_or("Other").to_string();
        if !by_region.contains_key(&region) {
            regions.push(region.clone());
        }
        by_region.entry(region).or_default().push(q);
    }

    for region in &regions {
        let items = &by_region[region];
        println!("\n{}", region.bold().cyan());
        let mut t = new_table();
        t.set_header(vec![
            header_cell("Symbol"),
            header_cell("Name"),
            header_cell("Price"),
            header_cell("Change"),
            header_cell("Chg %"),
        ]);
        for q in items {
            let price = f64_val(&q["price"]);
            let change = f64_val(&q["change"]);
            let chg_pct = f64_val(&q["change_pct"]);
            t.add_row(vec![
                Cell::new(str_val(q, "symbol")).add_attribute(Attribute::Bold),
                Cell::new(str_val(q, "name")).fg(Color::DarkGrey),
                Cell::new(price.map_or("—".to_string(), |p| fmt_f(p, 2)))
                    .set_alignment(CellAlignment::Right),
                change_cell(change, None, false).set_alignment(CellAlignment::Right),
                change_cell(None, chg_pct, false).set_alignment(CellAlignment::Right),
            ]);
        }
        println!("{t}");
    }
}

pub fn show_quote(data: &Value) {
    let symbol = str_val(data, "symbol");
    let name = data["name"].as_str().unwrap_or("");
    let price = f64_val(&data["price"]);
    let change = f64_val(&data["change"]);
    let chg_pct = f64_val(&data["change_pct"]);
    let prev = f64_val(&data["prev_close"]);
    let high = f64_val(&data["dayHigh"]).or_else(|| f64_val(&data["high"]));
    let low  = f64_val(&data["dayLow"]).or_else(|| f64_val(&data["low"]));
    let vol  = f64_val(&data["volume"]);
    let mcap = f64_val(&data["market_cap"]).or_else(|| f64_val(&data["marketCap"]));

    println!("\n{} {}", symbol.bold().cyan(), dim(name));
    println!("{}", price.map_or("—".to_string(), |p| fmt_f(p, 2)).bold().white());

    let chg_str = {
        let c = change.map_or("—".to_string(), |c| format!("{:+.2}", c));
        let p = chg_pct.map_or("—".to_string(), |p| format!("{:+.2}%", p));
        let positive = chg_pct.or(change).unwrap_or(0.0) >= 0.0;
        let text = format!("{c}  {p}");
        if positive { text.green().to_string() } else { text.red().to_string() }
    };
    println!("{chg_str}");

    let mut t = new_table();
    t.set_header(vec![header_cell("Field"), header_cell("Value")]);
    if !name.is_empty() && name != symbol {
        t.add_row(vec![Cell::new("Name").fg(Color::DarkGrey), Cell::new(name)]);
    }
    if let Some(p) = prev {
        t.add_row(vec![Cell::new("Prev Close").fg(Color::DarkGrey), Cell::new(fmt_f(p, 2))]);
    }
    if let Some(h) = high {
        t.add_row(vec![Cell::new("Day High").fg(Color::DarkGrey), Cell::new(fmt_f(h, 2))]);
    }
    if let Some(l) = low {
        t.add_row(vec![Cell::new("Day Low").fg(Color::DarkGrey), Cell::new(fmt_f(l, 2))]);
    }
    if let Some(v) = vol {
        t.add_row(vec![Cell::new("Volume").fg(Color::DarkGrey), Cell::new(compact(v))]);
    }
    if let Some(m) = mcap {
        t.add_row(vec![Cell::new("Market Cap").fg(Color::DarkGrey), Cell::new(compact(m))]);
    }
    println!("{t}");
}

pub fn show_watchlist(data: &Value) {
    let quotes = match data["quotes"].as_array() {
        Some(q) => q,
        None => { eprintln!("Watchlist is empty."); return; }
    };
    let mut t = new_table();
    t.set_header(vec![
        header_cell("Symbol"),
        header_cell("Price"),
        header_cell("Change"),
        header_cell("Chg %"),
        header_cell("Prev Close"),
    ]);
    for q in quotes {
        t.add_row(vec![
            Cell::new(str_val(q, "symbol")).add_attribute(Attribute::Bold),
            Cell::new(f64_val(&q["price"]).map_or("—".to_string(), |p| fmt_f(p, 2)))
                .set_alignment(CellAlignment::Right),
            change_cell(f64_val(&q["change"]), None, false).set_alignment(CellAlignment::Right),
            change_cell(None, f64_val(&q["change_pct"]), false).set_alignment(CellAlignment::Right),
            Cell::new(f64_val(&q["prev_close"]).map_or("—".to_string(), |p| fmt_f(p, 2)))
                .set_alignment(CellAlignment::Right),
        ]);
    }
    println!("{t}");
}

pub fn show_portfolio(data: &Value) {
    if let Some(summary) = data["summary"].as_object() {
        let total_value = summary.get("total_value").and_then(|v| v.as_f64());
        let total_pnl   = summary.get("total_pnl").and_then(|v| v.as_f64());
        let total_pct   = summary.get("total_pnl_pct").and_then(|v| v.as_f64());

        println!("\n{}", "Portfolio Summary".bold().cyan());
        println!(
            "  {}  {}",
            dim("Total Value:"),
            total_value.map_or("—".to_string(), |v| compact(v)).bold().white()
        );
        let pnl_str = format!(
            "{}  {}",
            total_pnl.map_or("—".to_string(), |p| format!("{:+}", compact(p))),
            total_pct.map_or("—".to_string(), |p| format!("{:+.2}%", p)),
        );
        let positive = total_pct.or(total_pnl).unwrap_or(0.0) >= 0.0;
        println!("  {}  {}", dim("Total P&L:"),
            if positive { pnl_str.green().to_string() } else { pnl_str.red().to_string() });
        println!();
    }

    let positions = match data["positions"].as_array() {
        Some(p) => p,
        None => { eprintln!("No positions found."); return; }
    };

    let mut t = new_table();
    t.set_header(vec![
        header_cell("Symbol"),
        header_cell("Qty"),
        header_cell("Avg Cost"),
        header_cell("Price"),
        header_cell("Chg %"),
        header_cell("Value"),
        header_cell("P&L"),
        header_cell("P&L %"),
    ]);

    let mut total_value = 0f64;
    let mut total_cost  = 0f64;
    let mut total_pnl   = 0f64;

    for pos in positions {
        let qty   = f64_val(&pos["quantity"]).unwrap_or(0.0);
        let value = f64_val(&pos["value"]).unwrap_or(0.0);
        let cost  = f64_val(&pos["cost"]).unwrap_or(0.0);
        let pnl   = f64_val(&pos["pnl"]);
        let pnl_pct = f64_val(&pos["pnl_pct"]);
        total_value += value;
        total_cost  += cost;
        total_pnl   += pnl.unwrap_or(0.0);

        t.add_row(vec![
            Cell::new(str_val(pos, "symbol")).add_attribute(Attribute::Bold),
            Cell::new(fmt_f(qty, 0)).set_alignment(CellAlignment::Right),
            Cell::new(f64_val(&pos["avg_cost"]).map_or("—".to_string(), |v| fmt_f(v, 2)))
                .set_alignment(CellAlignment::Right),
            Cell::new(f64_val(&pos["price"]).map_or("—".to_string(), |v| fmt_f(v, 2)))
                .set_alignment(CellAlignment::Right),
            change_cell(None, f64_val(&pos["change_pct"]), false).set_alignment(CellAlignment::Right),
            Cell::new(compact(value)).set_alignment(CellAlignment::Right),
            change_cell(pnl, None, true).set_alignment(CellAlignment::Right),
            change_cell(None, pnl_pct, false).set_alignment(CellAlignment::Right),
        ]);
    }

    let total_pct = if total_cost > 0.0 { total_pnl / total_cost * 100.0 } else { 0.0 };
    t.add_row(vec![
        Cell::new("TOTAL").add_attribute(Attribute::Bold),
        Cell::new(""), Cell::new(""), Cell::new(""), Cell::new(""),
        Cell::new(compact(total_value)).add_attribute(Attribute::Bold).set_alignment(CellAlignment::Right),
        change_cell(Some(total_pnl), None, true).set_alignment(CellAlignment::Right),
        change_cell(None, Some(total_pct), false).set_alignment(CellAlignment::Right),
    ]);

    println!("{t}");
}

pub fn show_news(data: &Value) {
    let articles = match data["articles"].as_array() {
        Some(a) if !a.is_empty() => a,
        _ => {
            if let Some(err) = data["error"].as_str() {
                eprintln!("{} {}", "News unavailable:".yellow(), err);
            } else {
                eprintln!("{}", "No news articles available.".yellow());
            }
            return;
        }
    };
    for article in articles.iter().take(20) {
        let title     = article["title"].as_str().or_else(|| article["headline"].as_str()).unwrap_or("Untitled");
        let source    = article["source"].as_str().or_else(|| article["publisher"].as_str()).unwrap_or("");
        let sentiment = article["sentiment"].as_str().unwrap_or("");
        let url       = article["url"].as_str().or_else(|| article["link"].as_str()).unwrap_or("");
        let published = article["publishedAt"].as_str()
            .or_else(|| article["published_at"].as_str())
            .or_else(|| article["date"].as_str())
            .unwrap_or("");

        let sentiment_str = match sentiment.to_lowercase().as_str() {
            "positive" => sentiment.green().to_string(),
            "negative" => sentiment.red().to_string(),
            _          => sentiment.bright_black().to_string(),
        };

        println!("{}", title.bold().white());
        let meta = format!("{} {} {}", dim(source), dim(&published[..published.len().min(10)]), sentiment_str);
        println!("  {meta}");
        if !url.is_empty() { println!("  {}", url.bright_blue()); }
        println!();
    }
}

pub fn show_calendar(data: &Value) {
    let events = match data["events"].as_array() {
        Some(e) if !e.is_empty() => e,
        _ => { eprintln!("{}", "No upcoming events found.".yellow()); return; }
    };
    let mut t = new_table();
    t.set_header(vec![
        header_cell("Date"),
        header_cell("Time"),
        header_cell("Event"),
        header_cell("Impact"),
        header_cell("Forecast"),
        header_cell("Previous"),
    ]);
    for ev in events {
        let date     = ev["date"].as_str().unwrap_or("—");
        let time     = ev["time"].as_str().unwrap_or("");
        let name     = ev["event"].as_str().or_else(|| ev["name"].as_str()).unwrap_or("—");
        let impact   = ev["impact"].as_str().or_else(|| ev["importance"].as_str()).unwrap_or("");
        let forecast = ev["forecast"].as_str().or_else(|| ev["expected"].as_str()).unwrap_or("—");
        let previous = ev["previous"].as_str().or_else(|| ev["prior"].as_str()).unwrap_or("—");

        let impact_cell = match impact.to_lowercase().as_str() {
            "high"   => Cell::new(impact.to_uppercase()).fg(Color::Red),
            "medium" => Cell::new(impact.to_uppercase()).fg(Color::Yellow),
            "low"    => Cell::new(impact.to_uppercase()).fg(Color::Green),
            _        => Cell::new("—").fg(Color::DarkGrey),
        };

        t.add_row(vec![
            Cell::new(date),
            Cell::new(time),
            Cell::new(name).add_attribute(Attribute::Bold),
            impact_cell,
            Cell::new(forecast),
            Cell::new(previous),
        ]);
    }
    println!("{t}");
}

pub fn show_performance(data: &Value) {
    let value    = f64_val(&data["value"]);
    let cost     = f64_val(&data["cost"]);
    let priced   = data["positions_priced"].as_u64();
    let total    = data["positions_total"].as_u64();
    let windows  = &data["windows"];
    let since    = &data["since_purchase"];

    println!("\n{}", "Portfolio Performance".bold().cyan());
    let mut t = new_table();
    t.set_header(vec![
        header_cell("Period"),
        header_cell("Change"),
        header_cell("Change %"),
    ]);

    for (label, w) in &[
        ("Today",          &windows["day"]),
        ("This Week",      &windows["week"]),
        ("This Month",     &windows["month"]),
        ("Since Purchase", since),
    ] {
        let compact_mode = *label == "Since Purchase";
        t.add_row(vec![
            Cell::new(label).fg(Color::DarkGrey),
            change_cell(f64_val(&w["change"]), None, compact_mode).set_alignment(CellAlignment::Right),
            change_cell(None, f64_val(&w["change_pct"]), false).set_alignment(CellAlignment::Right),
        ]);
    }
    println!("{t}");

    let mut meta_parts = vec![];
    if let Some(v) = value { meta_parts.push(format!("{} {}", dim("Value:"), compact(v))); }
    if let Some(c) = cost  { meta_parts.push(format!("{} {}", dim("Cost:"), compact(c))); }
    if let (Some(p), Some(tot)) = (priced, total) {
        meta_parts.push(format!("{} {p}/{tot}", dim("Positions:")));
    }
    if !meta_parts.is_empty() {
        println!("  {}", meta_parts.join("  "));
    }
}

pub fn show_stock_performance(data: &Value) {
    let symbol  = str_val(data, "symbol");
    let price   = f64_val(&data["price"]);
    let windows = &data["windows"];
    let since   = &data["since_purchase"];

    println!("\n{} {}", symbol.bold().cyan(), "Performance".bold());
    let mut t = new_table();
    t.set_header(vec![
        header_cell("Period"),
        header_cell("Change"),
        header_cell("Change %"),
    ]);

    for (label, w) in &[
        ("Today",      &windows["day"]),
        ("This Week",  &windows["week"]),
        ("This Month", &windows["month"]),
    ] {
        t.add_row(vec![
            Cell::new(label).fg(Color::DarkGrey),
            change_cell(f64_val(&w["change"]), None, false).set_alignment(CellAlignment::Right),
            change_cell(None, f64_val(&w["change_pct"]), false).set_alignment(CellAlignment::Right),
        ]);
    }

    if !since.is_null() {
        t.add_row(vec![
            Cell::new("Since Purchase").fg(Color::DarkGrey),
            change_cell(f64_val(&since["change"]), None, true).set_alignment(CellAlignment::Right),
            change_cell(None, f64_val(&since["change_pct"]), false).set_alignment(CellAlignment::Right),
        ]);
    }
    println!("{t}");

    let qty  = f64_val(&since["quantity"]);
    let avg  = f64_val(&since["avg_cost"]);
    let val  = f64_val(&since["value"]);
    let cst  = f64_val(&since["cost"]);
    let mut meta = vec![];
    if let Some(p) = price { meta.push(format!("{} {}", dim("Price:"), fmt_f(p, 2))); }
    if let Some(q) = qty   { meta.push(format!("{} {}", dim("Qty:"), fmt_f(q, 0))); }
    if let Some(a) = avg   { meta.push(format!("{} {}", dim("Avg Cost:"), fmt_f(a, 2))); }
    if let Some(v) = val   { meta.push(format!("{} {}", dim("Value:"), compact(v))); }
    if let Some(c) = cst   { meta.push(format!("{} {}", dim("Cost:"), compact(c))); }
    if !meta.is_empty() { println!("  {}", meta.join("  ")); }
}

pub fn show_raw(data: &Value) {
    println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
}
