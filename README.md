# mw

Command-line interface for [MarketSight](https://www.marketsight.app) — live global market data and portfolio tracking in your terminal.

## Features

- Global market overview — indices, FX, commodities, bonds grouped by region
- Live quotes for any symbol (stocks, ETFs, crypto, indices)
- Your MarketSight watchlist with live prices
- Portfolio positions with unrealised P&L
- Portfolio performance for today, this week, this month and since purchase
- Per-stock performance breakdown
- Upcoming macro events — FOMC, ECB, CPI, NFP
- Latest financial news with sentiment labels
- `--raw` flag on every command to get the underlying JSON

## Installation

Requires [Rust](https://rustup.rs).

```bash
git clone https://github.com/Bondomondo/mw-cmd
cd mw-cmd
cargo build --release
sudo cp target/release/mw /usr/local/bin/mw
```

## Configuration

Get your API key from **marketsight.app → Settings → API**, then:

```bash
mw config set-key YOUR_API_KEY
```

The key is stored in `~/.config/mw/api_key`. You can also set it via the `MARKETWATCH_API_KEY` environment variable.

## Usage

```
mw <COMMAND>

Commands:
  market             Global market overview (indices, FX, commodities)
  quote <SYMBOL>     Live quote for a symbol (e.g. AAPL, BTC-USD, ^GSPC)
  watchlist          Your saved MarketSight watchlist
  portfolio          Portfolio positions with live P&L
  performance        Portfolio performance: today / this week / this month / since purchase
  stock-performance  Per-stock performance for a single position
  news               Latest financial news
  calendar           Upcoming high-impact events (FOMC, CPI, NFP, ECB…)
  brief              AI-generated market brief (overview, equities, commodities, currencies)
  portfolio-brief    AI portfolio analysis with risk ratings, suggested changes, and per-position recommendations
  config             Manage API key
```

### Examples

```bash
mw market
mw quote NVDA
mw watchlist
mw portfolio
mw performance
mw stock-performance SSAB-B.ST
mw calendar
mw news
mw brief
mw brief --force          # bypass server cache to get a fresh brief
mw portfolio-brief
mw portfolio-brief --force

# Raw JSON output
mw portfolio --raw
mw quote AAPL --raw
```

## License

MIT
