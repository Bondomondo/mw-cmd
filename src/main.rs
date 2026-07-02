mod api;
mod display;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mw",
    version = "0.2.0",
    about = "MarketSight CLI — live market & portfolio data from marketsight.app",
    long_about = "MarketSight CLI — live market & portfolio data from marketsight.app\n\nRequires a MarketSight API key. Set it once with:\n  mw config set-key <KEY>\nor via the MARKETWATCH_API_KEY environment variable."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

const RANGE_LONG_HELP: &str = "\
Time window for change / change_pct values:

  today   vs previous close (default)
  week    last 7 days
  3m      last 3 months
  6m      last 6 months
  1y      last 1 year

Common aliases are accepted (1w, 3mo, 6mo, 1yr, year).
An unrecognised value returns an error listing the valid options.";

#[derive(Subcommand)]
enum Commands {
    /// Show global market overview — indices, FX, commodities, bonds grouped by region
    Market {
        #[arg(long, value_name = "RANGE", help = "Time range: today (default), week, 3m, 6m, 1y", long_help = RANGE_LONG_HELP)]
        range: Option<String>,
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show live quote for a symbol
    Quote {
        #[arg(value_name = "SYMBOL", help = "Ticker symbol, e.g. AAPL, BTC-USD, ^GSPC, SSAB-B.ST")]
        symbol: String,
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show your saved MarketSight watchlist with live prices
    Watchlist {
        #[arg(long, value_name = "RANGE", help = "Time range: today (default), week, 3m, 6m, 1y", long_help = RANGE_LONG_HELP)]
        range: Option<String>,
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show portfolio positions with live prices and unrealised P&L
    Portfolio {
        #[arg(
            long,
            value_name = "RANGE",
            help = "Time range for the Chg % column: today (default), week, 3m, 6m, 1y",
            long_help = "Time window for the per-position change / change_pct column.\n\n  today   vs previous close (default)\n  week    last 7 days\n  3m      last 3 months\n  6m      last 6 months\n  1y      last 1 year\n\nP&L and P&L % always reflect total return since purchase regardless of range."
        )]
        range: Option<String>,
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show latest financial news with sentiment labels
    News {
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show upcoming high-impact macro events (FOMC, ECB, CPI, NFP…)
    Calendar {
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show portfolio performance for today, this week, this month and since purchase
    Performance {
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show performance breakdown for a single stock in your portfolio
    StockPerformance {
        #[arg(value_name = "SYMBOL", help = "Ticker symbol of a position in your portfolio, e.g. AAPL")]
        symbol: String,
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show AI-generated market brief (overview, equities, commodities, currencies)
    Brief {
        #[arg(long, help = "Bypass the server cache and generate a fresh brief")]
        force: bool,
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Show AI portfolio analysis — risk ratings, position recommendations, suggested changes
    PortfolioBrief {
        #[arg(long, help = "Bypass the server cache and generate a fresh brief")]
        force: bool,
        #[arg(long, help = "Print raw JSON instead of formatted output")]
        raw: bool,
    },
    /// Manage mw configuration (API key)
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Save your MarketSight API key (stored in ~/.config/mw/api_key)
    SetKey {
        #[arg(value_name = "KEY", help = "API key from marketsight.app → Settings → API")]
        key: String,
    },
    /// Show the current API key source and prefix
    Show,
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Market { raw, range } => {
            let data = api::get_market(range.as_deref())?;
            if raw { display::show_raw(&data) } else { display::show_market(&data) }
        }
        Commands::Quote { symbol, raw } => {
            let data = api::get_quote(&symbol.to_uppercase())?;
            if raw { display::show_raw(&data) } else { display::show_quote(&data) }
        }
        Commands::Watchlist { raw, range } => {
            let data = api::get_watchlist(range.as_deref())?;
            if raw { display::show_raw(&data) } else { display::show_watchlist(&data) }
        }
        Commands::Portfolio { raw, range } => {
            let data = api::get_portfolio(range.as_deref())?;
            if raw { display::show_raw(&data) } else { display::show_portfolio(&data) }
        }
        Commands::News { raw } => {
            let data = api::get_news()?;
            if raw { display::show_raw(&data) } else { display::show_news(&data) }
        }
        Commands::Calendar { raw } => {
            let data = api::get_calendar()?;
            if raw { display::show_raw(&data) } else { display::show_calendar(&data) }
        }
        Commands::Performance { raw } => {
            let data = api::get_performance()?;
            if raw { display::show_raw(&data) } else { display::show_performance(&data) }
        }
        Commands::StockPerformance { symbol, raw } => {
            let data = api::get_stock_performance(&symbol.to_uppercase())?;
            if raw { display::show_raw(&data) } else { display::show_stock_performance(&data) }
        }
        Commands::Brief { force, raw } => {
            let data = api::get_brief(force)?;
            if raw { display::show_raw(&data) } else { display::show_brief(&data) }
        }
        Commands::PortfolioBrief { force, raw } => {
            let data = api::get_portfolio_brief(force)?;
            if raw { display::show_raw(&data) } else { display::show_portfolio_brief(&data) }
        }
        Commands::Config { action } => match action {
            ConfigAction::SetKey { key } => api::save_api_key(&key)?,
            ConfigAction::Show => {
                let env_key = std::env::var("MARKETWATCH_API_KEY").ok();
                let path = api::config_key_path();
                if let Some(k) = env_key {
                    println!("API key: env var MARKETWATCH_API_KEY ({}…)", &k[..k.len().min(8)]);
                } else if path.exists() {
                    let k = std::fs::read_to_string(&path).unwrap_or_default();
                    let k = k.trim();
                    println!("API key: {} ({}…)", path.display(), &k[..k.len().min(8)]);
                } else {
                    eprintln!("No API key configured. Run: mw config set-key <key>");
                }
            }
        },
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
