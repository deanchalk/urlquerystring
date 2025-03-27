# urlquerystring

[![Crates.io](https://img.shields.io/crates/v/urlquerystring)](https://crates.io/crates/urlquerystring)
[![Documentation](https://docs.rs/urlquerystring/badge.svg)](https://docs.rs/urlquerystring)
[![License](https://img.shields.io/crates/l/urlquerystring)](LICENSE)

A high-performance, zero-allocation URL query string parser for Rust. This crate provides a stack-based implementation that extracts query parameters without any heap allocations, making it ideal for performance-critical environments.

## Features

- **Zero Heap Allocations**: All memory is allocated on the stack
- **High Performance**: Direct byte manipulation and minimal string operations
- **UTF-8 Safe**: Properly handles UTF-8 encoded query parameters
- **Percent-Decoding**: Built-in support for URL percent-encoding
- **Const-Generic Based**: Flexible size configuration through const generics
- **Zero-Cost Abstractions**: All operations are zero-cost where possible

## Example

```rust
use urlquerystring::StackQueryParams;

let url = "https://example.com/path?name=John&age=25&city=New%20York";
let mut params = StackQueryParams::default();
params.parse_from_url(url);

assert_eq!(params.get("name"), Some("John"));
assert_eq!(params.get("city"), Some("New York")); // Automatically percent-decoded
```

## Performance

This crate is designed for maximum performance:

- No heap allocations in the core functionality
- Fixed-size buffers for predictable memory usage
- Direct byte manipulation for parsing
- Zero-cost abstractions for common operations
- Efficient percent-decoding implementation

## Usage

### Basic Usage

```rust
use urlquerystring::StackQueryParams;

let url = "https://example.com/path?name=John&age=25";
let mut params = StackQueryParams::default();
params.parse_from_url(url);

// Access parameters
assert_eq!(params.get("name"), Some("John"));
assert_eq!(params.get("age"), Some("25"));
```

### Custom Size Limits

```rust
use urlquerystring::StackQueryParams;

// Create a container with custom size limits
let mut params = StackQueryParams::<32, 64, 256>::new();
params.parse_from_url("https://example.com/path?param=value");
```

### Iterating Over Parameters

```rust
use urlquerystring::StackQueryParams;

let mut params = StackQueryParams::default();
params.parse_from_url("https://example.com/path?a=1&b=2");

for (key, value) in params.iter() {
    println!("{} = {}", key, value);
}
```

## Limitations

Due to the stack-based design, there are some limitations:

- Maximum number of parameters (default: 16)
- Maximum key length (default: 32 bytes)
- Maximum value length (default: 128 bytes)

These limits can be customized using const generics if needed.

## When to Use This Crate

This crate is particularly useful in:

- Performance-critical applications
- Embedded systems
- Environments where heap allocations are restricted
- High-throughput web servers
- Systems with limited memory

## Alternatives

If you need more flexibility or don't require zero-allocation parsing:

- [url](https://crates.io/crates/url): Full URL parsing with heap allocations
- [serde_urlencoded](https://crates.io/crates/serde_urlencoded): Serialization-focused query parsing

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Acknowledgments

- Inspired by the need for high-performance URL parsing in embedded systems
- Built with zero-allocation principles in mind
- Designed for maximum performance and safety