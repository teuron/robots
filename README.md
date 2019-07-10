# A robots.txt parser and applicability checker for Rust

[![Build Status](https://travis-ci.org/teuron/robots.svg)](https://travis-ci.org/teuron/robots)
[![Crates.io](https://img.shields.io/crates/v/robots-parser.svg)](https://crates.io/crates/robots-parser)


## Usage

Add it to your ``Cargo.toml``:

```toml
[dependencies]
robots-parser = "0.10"
```

## Examples

### Parse and check from URL
```rust
use robots::RobotsParser;
use url::Url;

fn main() {
    let parsed = RobotsParser::parse_url(Url::new("https://www.google.com/robots.txt"))?;
    assert!(parsed.can_fetch("*", "https://www.google.com/search/about"));
}
```

### Parse and check from File

```rust
use robots::RobotsParser;

fn main() {
    let parsed = RobotsParser::parse_path("~/test-robots.txt"))?;
    assert!(parsed.can_fetch("*", "http://test.com/can_fetch"));
}
```

### Parse and check from &str

```rust
use robots::RobotsParser;

fn main() {
    let parsed = RobotsParser::parse_path("Disallow: /test"))?;
    assert!(!parsed.can_fetch("*", "http://test.com/test"));
}
```

## License

This work is released under Apache and MIT license. A copy of the licenses are provided in the [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) files.