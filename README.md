<div align="center">

<img width="100%" alt="header" src="https://capsule-render.vercel.app/api?type=waving&height=210&text=Alphea%20Connect%20Bot&fontAlign=50&fontAlignY=36&fontSize=56&desc=Multi-Account%20Mining%20Automation%20Client%20for%20Alphea%20Connect&descAlign=50&descAlignY=58"/>

<img alt="typing" src="https://readme-typing-svg.demolab.com?font=Inter&size=18&duration=3000&pause=650&center=true&vCenter=true&width=900&lines=Auto+Sign-in+%7C+Session+Cached;Auto+Mining+Shift+%7C+12+Hour+Daily+Target;Auto+Quest+Claiming+%7C+Each+Cycle;Proxy+Support+%7C+Multi-Account"/>

<p>
  <img alt="rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white"/>
  <img alt="platform" src="https://img.shields.io/badge/Platform-Alphea%20Connect-111111"/>
  <img alt="multi-account" src="https://img.shields.io/badge/Multi--Account-Supported-111111"/>
  <img alt="build" src="https://github.com/Yuurisan-N1/Alphea-Application/actions/workflows/build.yml/badge.svg"/>
  <img alt="author" src="https://img.shields.io/badge/by-Yuurisandesu-111111"/>
</p>

<p>
  <b>Alphea Connect Bot</b> is a mining automation client for the Alphea Connect platform.<br/>
  It handles email sign-in with a one-time verification code (asked only on the first run per account), caches the session token, and on every cycle runs the full 12-hour mining shift with per-minute heartbeats, auto-renews the session when it expires, claims finished quests, and then sleeps until the next cycle, all across multiple accounts with proxy support.<br/>
  Built and distributed by <b>Yuurisandesu</b>.
</p>

</div>

---

## Table of Contents

- [Requirements](#requirements)
- [Install Rust](#install-rust)
- [Installation](#installation)
- [Configuration](#configuration)
- [First-Time Setup](#first-time-setup)
- [Running the Bot](#running-the-bot)
- [Run Modes](#run-modes)
- [Download Prebuilt Binary](#download-prebuilt-binary)
- [Features](#features)
- [File Structure](#file-structure)
- [Disclaimer](#disclaimer)

---

## Requirements

- Rust `1.70+` (includes `cargo`, only needed if building from source)
- Git
- `screen` (optional, only for detached runs on Linux and macOS)

---

## Install Rust

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:

```bash
rustc --version
```

### Windows

Download and run the installer from https://rustup.rs, then reopen the terminal and verify:

```cmd
rustc --version
```

### Termux (Android)

```bash
pkg update && pkg install proot-distro
proot-distro install ubuntu
proot-distro login ubuntu
```

Then inside Ubuntu:

```bash
apt update && apt install -y curl git build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

## Installation

**Clone the repository:**

```bash
git clone https://github.com/Yuurisan-N1/Alphea-Application.git
cd Alphea-Application
```

**Build the binary:**

```bash
cargo build --release
# or
./run.sh build
```

---

## Configuration

### 1. Accounts (data.txt)

Fill `data.txt` with one Gmail address per line. Blank lines and lines starting with `#` are ignored:

```
first@gmail.com
second@gmail.com
```

### 2. Proxy (proxy.txt)

Fill `proxy.txt` with proxies, one per line (optional, leave empty to connect directly):

```
http://user:pass@host:port
socks5://user:pass@host:port
```

Supported schemes are `http`, `https`, `socks5`, `socks5h`, and `socks`. A missing port defaults to 1080.

### 3. Bot Settings (config.json)

`sleep_seconds` controls how many seconds the bot waits between cycles. The default is 3600 (one hour).

---

## First-Time Setup

Each account must complete a one-time verification before the bot can run. The bot will ask for the code automatically on the first run. After verification, the session is cached and no code is needed again.

When prompted, open the Gmail inbox and type the 6-digit code, then press Enter:

```
Verification code for fi***st@gmail.com:
```

After that the bot signs in and starts the mining shift on its own.

You should see the banner, then the sign-in, then one line that updates in place:

```
The mining shift for the account holds 61 of 43200 valid seconds 1
```

The number climbs every minute until the daily target is reached.

---

## Running the Bot

Make `run.sh` executable first (Linux / macOS / Termux):

```bash
chmod +x run.sh
```

Then choose a mode:

```bash
./run.sh run        # build if needed and run in foreground
./run.sh bg         # build if needed and run detached in a screen session
./run.sh screen     # attach to the running screen session
./run.sh stop       # stop the detached screen session
./run.sh probe      # read earnings and reward summary without claiming anything
./run.sh selftest   # offline checks, no network
./run.sh build      # build only, do not run
```

For screen mode, attach anytime:

```bash
screen -r alphea
```

Press `Ctrl+C` once to stop the bot cleanly when running in foreground.

### Using make (Linux / macOS / Termux)

```bash
make release    # optimized release build
make start      # build release and run immediately
make run        # build debug and run
make check      # check for errors without building
make fmt        # format the code
make clean      # remove all build artifacts
make size       # show release binary size
```

### Manual run

```bash
# Linux / macOS / Termux
./target/release/alphea-bot

# Windows
.\target\release\alphea-bot.exe
```

---

## Run Modes

| Mode | Command | Description |
|---|---|---|
| run | `./run.sh run` | Full cycle: mining shift, quest claiming, sleep |
| bg | `./run.sh bg` | Same as run but detached in a screen session |
| screen | `./run.sh screen` | Attach to the running screen session |
| probe | `./run.sh probe` | Read earnings and reward summary without claiming |
| selftest | `./run.sh selftest` | Offline checks, no network |
| stop | `./run.sh stop` | Stop the detached screen session |
| build | `./run.sh build` | Build only |

---

## Download Prebuilt Binary

Prebuilt binaries are automatically compiled on every push to main via GitHub Actions across 3 platforms.

Download the latest binaries from the Actions page:
https://github.com/Yuurisan-N1/Alphea-Application/actions/workflows/build.yml

Open the latest successful run and scroll to the Artifacts section.

| Artifact | Platform |
|---|---|
| `alphea-bot-linux-x86_64` | Linux x86_64 |
| `alphea-bot-windows-x86_64` | Windows x86_64 |
| `alphea-bot-macos-aarch64` | macOS Apple Silicon |

**Linux / macOS / Termux after downloading:**

```bash
chmod +x alphea-bot-linux-x86_64
./alphea-bot-linux-x86_64
```

**Windows:**

```
.\alphea-bot-windows-x86_64.exe
```

Place the binary in the same folder as `data.txt`, `proxy.txt`, and `config.json` before running.

---

## Features

### Sign-in and Session Cache
Each account signs in once with a 6-digit code sent to its Gmail inbox. The session token is cached in `session.json` after the first successful sign-in. On later runs the token is reused and the code is skipped. If the token expires the bot renews it automatically, including mid-shift.

### Auto Mining Shift
The bot starts a mining session for each account and sends a heartbeat every 60 seconds. The shift runs until the server reports 43200 valid seconds (12 hours). If the heartbeat is refused the bot renews the session token and continues without stopping the shift.

### Auto Quest Claiming
After the mining shift ends, the bot fetches the quest list and claims every quest that is in a claimable state. Each claim is logged individually.

### Multi Account
All accounts in `data.txt` are processed in parallel within every cycle. Each account runs its own mining shift and uses its own session cache.

### Proxy Support
Proxies are loaded from `proxy.txt`. HTTP and SOCKS5 are both supported. Proxy credentials are masked in log output. Running without proxies is fully supported.

### Auto Countdown
After all accounts complete a cycle, the bot waits for the configured sleep duration and then starts the next cycle automatically.

---

## File Structure

```text
Alphea-Application/
├── .github/
│   └── workflows/
│       └── build.yml                    # CI: 3-platform matrix build
├── assets/
│   ├── icon.ico                         # Windows binary icon
│   └── icon.png                         # App icon
├── src/
│   ├── activity/
│   │   ├── earnings.rs                  # Earnings and points fetch
│   │   ├── foreground.rs                # Mining session start and end
│   │   └── mod.rs
│   ├── challenge/
│   │   ├── device.rs                    # Device credential register and login
│   │   ├── mod.rs
│   │   ├── request.rs                   # Verification code request
│   │   └── verify.rs                    # Code submission and token issuance
│   ├── cli/
│   │   ├── dispatch.rs                  # CLI mode selector
│   │   ├── mod.rs
│   │   ├── probe.rs                     # Probe mode: read-only earnings check
│   │   └── selftest.rs                  # Offline self-test checks
│   ├── net/
│   │   ├── client.rs                    # HTTP client builder
│   │   ├── device.rs                    # Per-account device fingerprint
│   │   ├── mod.rs
│   │   ├── proxy.rs                     # Proxy parser and masker
│   │   └── proxy_load.rs                # Proxy file loader
│   ├── quest/
│   │   ├── claim.rs                     # Quest claim request
│   │   ├── list.rs                      # Quest list fetch
│   │   └── mod.rs
│   ├── reward/
│   │   ├── mod.rs
│   │   └── summary.rs                   # Reward point summary fetch
│   ├── run/
│   │   ├── cycle.rs                     # Main automation loop and mining shift
│   │   └── mod.rs
│   ├── support/
│   │   ├── cfg/                         # Config loader and schema
│   │   ├── consts/                      # App and network constants
│   │   ├── log/                         # Log init, colored format, progress helpers
│   │   ├── banner.rs                    # ASCII banner and terminal title
│   │   ├── env.rs                       # Account file loader
│   │   ├── error.rs                     # Error sanitizer and refusal mapper
│   │   ├── mod.rs
│   │   ├── signal.rs                    # Ctrl+C interrupt watcher
│   │   └── store.rs                     # Session token cache read and write
│   └── main.rs                          # Entry point, runtime setup, mode dispatch
├── .gitignore
├── build.rs                             # Build script: embeds icon into Windows binary
├── Cargo.lock
├── Cargo.toml                           # Project manifest and dependencies
├── config.json                          # Sleep duration setting
├── data.txt                             # Account list, one Gmail address per line
├── HOWTO.txt                            # Quick start guide
├── LICENSE                              # GNU AGPL v3.0
├── Makefile                             # make targets: build, release, run, start, clean, size
├── proxy.txt                            # Proxy list (optional)
├── README.md                            # This file
└── run.sh                               # Run helper: run, bg, screen, probe, selftest, stop, build
```

---

## Disclaimer

This tool is built for educational and technical exploration purposes. Use it wisely and at your own responsibility.

---

<div align="center">
<img width="100%" alt="footer" src="https://capsule-render.vercel.app/api?type=waving&height=120&section=footer"/>
</div>