mod api;
mod display;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mw", version = "0.1.0", about = "MarketSight CLI — live market & portfolio data from marketsight.app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current global market overview (indices, FX, commodities)
    Market {
        #[arg(long, help = "Print raw JSON")]
        raw: bool,
    },
    /// Show live quote for a symbol (e.g. AAPL, BTC-USD, ^GSPC)
    Quote {
        symbol: String,
        #[arg(long, help = "Print raw JSON")]
        raw: bool,
    },
    /// Show your saved MarketSight watchlist
    Watchlist {
        #[arg(long, help = "Print raw JSON")]
        raw: bool,
    },
    /// Show portfolio positions with live P&L
    Portfolio {
        #[arg(long, help = "Print raw JSON")]
        raw: bool,
    },
    /// Show latest financial news
    News {
        #[arg(long, help = "Print raw JSON")]
        raw: bool,
    },
    /// Show upcoming high-impact market events (FOMC, CPI, NFP, ECB…)
    Calendar {
        #[arg(long, help = "Print raw JSON")]
        raw: bool,
    },
    /// Show portfolio performance for today, this week, this month and since purchase
    Performance {
        #[arg(long, help = "Print raw JSON")]
        raw: bool,
    },
    /// Show performance for a single stock in your portfolio
    StockPerformance {
        symbol: String,
        #[arg(long, help = "Print raw JSON")]
        raw: bool,
    },
    /// Manage mw configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Save your MarketSight API key
    SetKey { key: String },
    /// Show current config
    Show,
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Market { raw } => {
            let data = api::get_market()?;
            if raw { display::show_raw(&data) } else { display::show_market(&data) }
        }
        Commands::Quote { symbol, raw } => {
            let data = api::get_quote(&symbol.to_uppercase())?;
            if raw { display::show_raw(&data) } else { display::show_quote(&data) }
        }
        Commands::Watchlist { raw } => {
            let data = api::get_watchlist()?;
            if raw { display::show_raw(&data) } else { display::show_watchlist(&data) }
        }
        Commands::Portfolio { raw } => {
            let data = api::get_portfolio()?;
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
