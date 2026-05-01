# 💥 safeBuster

> A fast, async web fuzzer written in Rust — inspired by ffuf, built for flexibility and speed.

---

## Overview

safeBuster is a web fuzzing tool that takes raw HTTP request files (copy-pasted straight from Burp Suite or your browser devtools) and replaces every `FUZZ` placeholder with words from a wordlist. It supports both GET and POST requests, custom headers, URL path fuzzing, subdomain fuzzing, and parameter fuzzing — all powered by Tokio's async runtime for high throughput.

---

## Features

- **`FUZZ` keyword replacement** — Place `FUZZ` anywhere in a raw HTTP request file: URL path, host header, query params, POST body, or custom headers
- **Raw HTTP request file input** — Feed it a raw request file exactly as captured from Burp Suite or browser devtools; safeBuster parses the method, host, HTTP version, headers, and body automatically
- **GET & POST parameter fuzzing** — Extracts and fuzzes both GET query parameters and POST body parameters
- **Custom header support** — Full HTTP header parsing and injection from the request file
- **Async execution** — Built on Tokio with `futures-util` and `tokio-stream` for non-blocking concurrent requests
- **Thread pool** — Additional `threadpool` for CPU-bound tasks alongside async I/O
- **Cross-thread messaging** — `crossbeam-channel` for efficient producer/consumer communication between threads
- **CLI via clap** — Clean, structured command-line interface with `--help` support and required-flag validation

---

## Tech Stack

| Dependency | Purpose |
|------------|---------|
| `clap` 4.5 | CLI argument parsing with derive macros |
| `tokio` (full) | Async runtime for concurrent HTTP requests |
| `reqwest` (blocking + json) | HTTP client for sending fuzzed requests |
| `futures-util` | Async stream and future utilities |
| `tokio-stream` | Streaming support with IO utilities |
| `threadpool` | Worker thread pool for parallel task execution |
| `crossbeam-channel` | Fast multi-producer/consumer channel between threads |

---

## Installation

```bash
git clone https://github.com/Cythonic1/safeBuster.git
cd safeBuster
cargo build --release
./target/release/safebuster --help
```

---

## Usage

### Fuzz from a raw HTTP request file

The primary workflow is to copy a raw HTTP request (e.g. from Burp Suite) into a file, place `FUZZ` wherever you want to inject wordlist entries, then point safeBuster at it.

**Example request file (`req.txt`):**
```
GET /FUZZ HTTP/2
Host: example.com
User-Agent: Mozilla/5.0
Accept: */*
```

```bash
./target/release/safebuster -r req.txt -w /path/to/wordlist.txt
```

### Fuzz subdomains

```
GET /index.php HTTP/2
Host: FUZZ.example.com
User-Agent: Mozilla/5.0
```

```bash
./target/release/safebuster -r req.txt -w /path/to/subdomains.txt
```

### Fuzz POST parameters

```
POST /login HTTP/2
Host: example.com
Content-Type: application/x-www-form-urlencoded

username=admin&password=FUZZ
```

```bash
./target/release/safebuster -r req.txt -w /path/to/passwords.txt
```

> `FUZZ` can appear **multiple times** anywhere in the request file — in the path, host, headers, query string, or body.

---

## Example Request Files

Two sample request files are included in the repo to demonstrate the format:

**`headers_test`** — GET request with `FUZZ` in both the URL path and Host header:
```
GET /FUZZ.some.php HTTP/2
Host: FUZZ.example.com
User-Agent: Mozilla/5.0 ...
```

**`headers_test_2`** — POST request with `FUZZ` in the Host header and a binary-encoded body:
```
POST /log?format=json HTTP/2
Host: FUZZ.google.com
Content-Type: application/binary
...
```

---

## Project Structure

```
safeBuster/
├── src/
│   ├── main.rs          # Entry point, CLI dispatch
│   ├── fuzzer.rs        # Core fuzzing engine (wordlist iteration, FUZZ replacement)
│   ├── parser.rs        # Raw HTTP request file parser (method, host, headers, body)
│   ├── http.rs          # HTTP request sender via reqwest
│   └── shared.rs        # Shared utilities used across modules
├── headers_test         # Sample GET request file with FUZZ placeholders
├── headers_test_2       # Sample POST request file with FUZZ placeholders
├── Cargo.toml
└── Cargo.lock
```

---

## Author

**Cythonic1** — [GitHub](https://github.com/Cythonic1)

> For educational and authorized penetration testing purposes only. Do not use against systems you do not have explicit permission to test.
