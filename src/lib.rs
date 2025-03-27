//! A high-performance, zero-allocation URL query string parser.
//!
//! This crate provides a stack-based URL query parser that extracts query parameters
//! without any heap allocations. It's designed for performance-critical environments
//! where heap allocations are undesirable or restricted.
//!
//! # Features
//!
//! - Zero heap allocations
//! - Stack-based storage with fixed-size buffers
//! - UTF-8 safe
//! - Percent-decoding support
//! - Const-generic based configuration
//!
//! # Performance
//!
//! This crate is optimized for performance by:
//! - Avoiding all heap allocations
//! - Using fixed-size arrays for storage
//! - Minimizing string operations
//! - Direct byte manipulation for parsing
//!
//! # Example
//!
//! ```rust
//! use urlquerystring::StackQueryParams;
//!
//! let url = "https://example.com/path?name=John&age=25&city=New%20York";
//! let mut params = StackQueryParams::default();
//! params.parse_from_url(url);
//!
//! assert_eq!(params.get("name"), Some("John"));
//! assert_eq!(params.get("city"), Some("New York")); // Automatically percent-decoded
//! ```
//!
//! # Limitations
//!
//! Due to the stack-based design, there are some limitations:
//! - Maximum number of parameters (default: 16)
//! - Maximum key length (default: 32 bytes)
//! - Maximum value length (default: 128 bytes)
//!
//! These limits can be customized using const generics if needed.

/// Default maximum number of query parameters that can be stored.
pub const MAX_QUERY_PARAMS: usize = 16;

/// Default maximum length for parameter keys in bytes.
pub const MAX_KEY_LEN: usize = 32;

/// Default maximum length for parameter values in bytes.
pub const MAX_VALUE_LEN: usize = 128;

/// A key-value pair with fixed-size storage.
///
/// This struct stores a single query parameter using fixed-size buffers
/// to avoid heap allocations. The size of the buffers is determined by
/// the const generic parameters `KEY_SIZE` and `VALUE_SIZE`.
///
/// # Examples
///
/// ```rust
/// use urlquerystring::{StackParam, MAX_KEY_LEN, MAX_VALUE_LEN};
///
/// let param = StackParam::<MAX_KEY_LEN, MAX_VALUE_LEN>::new();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StackParam<const KEY_SIZE: usize, const VALUE_SIZE: usize> {
    key: StackString<KEY_SIZE>,
    value: StackString<VALUE_SIZE>,
}

impl<const KEY_SIZE: usize, const VALUE_SIZE: usize> StackParam<KEY_SIZE, VALUE_SIZE> {
    /// Creates a new empty parameter with zero-initialized buffers.
    ///
    /// This operation is zero-cost and performs no heap allocations.
    pub fn new() -> Self {
        StackParam {
            key: StackString::new(),
            value: StackString::new(),
        }
    }

    /// Returns the key as a string slice.
    ///
    /// This is a zero-cost operation that returns a view into the internal buffer.
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Returns the value as a string slice.
    ///
    /// This is a zero-cost operation that returns a view into the internal buffer.
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// A stack-based URL query parameters container.
///
/// This struct provides a high-performance container for URL query parameters
/// that uses only stack-based memory allocation. The size limits are determined
/// by the const generic parameters:
///
/// - `PARAM_COUNT`: Maximum number of parameters
/// - `KEY_SIZE`: Maximum size of each key in bytes
/// - `VALUE_SIZE`: Maximum size of each value in bytes
///
/// # Examples
///
/// ```rust
/// use urlquerystring::{StackQueryParams, MAX_QUERY_PARAMS, MAX_KEY_LEN, MAX_VALUE_LEN};
///
/// // Use default sizes
/// let mut params = StackQueryParams::default();
///
/// // Or specify custom sizes
/// let mut custom_params = StackQueryParams::<32, 64, 256>::new();
/// ```
#[derive(Debug)]
pub struct StackQueryParams<
    const PARAM_COUNT: usize,
    const KEY_SIZE: usize,
    const VALUE_SIZE: usize,
> {
    params: [StackParam<KEY_SIZE, VALUE_SIZE>; PARAM_COUNT],
    count: usize,
}

impl Default for StackQueryParams<MAX_QUERY_PARAMS, MAX_KEY_LEN, MAX_VALUE_LEN> {
    /// Creates a new query parameters container with the default size limits.
    ///
    /// This is equivalent to calling `StackQueryParams::new()` with the default
    /// const parameters.
    fn default() -> Self {
        Self::new()
    }
}

impl<const PARAM_COUNT: usize, const KEY_SIZE: usize, const VALUE_SIZE: usize>
    StackQueryParams<PARAM_COUNT, KEY_SIZE, VALUE_SIZE>
{
    /// Creates a new empty container.
    ///
    /// This operation is zero-cost and performs no heap allocations.
    /// The size of the container is determined by the const generic parameters.
    pub fn new() -> Self {
        StackQueryParams {
            params: [StackParam::<KEY_SIZE, VALUE_SIZE>::new(); PARAM_COUNT],
            count: 0,
        }
    }

    /// Parses a URL query string with automatic percent-decoding.
    ///
    /// This method efficiently parses the query string portion of a URL,
    /// automatically handling percent-encoded characters and plus signs.
    /// It performs no heap allocations and uses only stack-based memory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let url = "https://example.com/path?name=John+Doe&city=New%20York";
    /// let mut params = StackQueryParams::default();
    /// params.parse_from_url(url);
    ///
    /// assert_eq!(params.get("name"), Some("John Doe"));
    /// assert_eq!(params.get("city"), Some("New York"));
    /// ```
    pub fn parse_from_url(&mut self, url: &str) {
        // Find where the query string starts (after '?')
        let query = match url.find('?') {
            Some(pos) => &url[pos + 1..],
            None => return, // No query parameters
        };

        // Track our position in the query string
        let mut start = 0;
        let bytes = query.as_bytes();

        while start < bytes.len() && self.count < PARAM_COUNT {
            // Find the end of this parameter (& or end of string)
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'&' {
                end += 1;
            }

            // Process this parameter
            let pair = &query[start..end];

            // Find the equals sign
            let mut param = &mut self.params[self.count];
            if let Some(eq_pos) = pair.find('=') {
                let key_str = &pair[0..eq_pos];
                let value_str = &pair[eq_pos + 1..];

                if !key_str.is_empty() {
                    // Decode key and value
                    let key_decoded = percent_decode::<KEY_SIZE>(key_str);
                    let value_decoded = percent_decode::<VALUE_SIZE>(value_str);

                    // Store in our parameter
                    param.key = key_decoded;
                    param.value = value_decoded;
                    self.count += 1;
                }
            } else if !pair.is_empty() {
                // Key with no value
                let key_decoded = percent_decode::<KEY_SIZE>(pair);
                param.key = key_decoded;
                self.count += 1;
            }

            // Move to the next parameter
            start = end + 1;
        }
    }

    /// Returns the value associated with the given key.
    ///
    /// This is a zero-cost operation that returns a view into the internal buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let mut params = StackQueryParams::default();
    /// params.parse_from_url("https://example.com/path?name=John");
    ///
    /// assert_eq!(params.get("name"), Some("John"));
    /// assert_eq!(params.get("missing"), None);
    /// ```
    pub fn get(&self, key: &str) -> Option<&str> {
        for i in 0..self.count {
            if self.params[i].key() == key {
                return Some(self.params[i].value());
            }
        }
        None
    }

    /// Returns the number of parameters currently stored.
    ///
    /// This is a zero-cost operation that returns the current count.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if no parameters are stored.
    ///
    /// This is a zero-cost operation.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns an iterator over all key-value pairs.
    ///
    /// The iterator yields tuples of `(&str, &str)` representing the key and value
    /// of each parameter. This is a zero-cost operation that returns views into
    /// the internal buffers.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        (0..self.count).map(move |i| (self.params[i].key(), self.params[i].value()))
    }
}

/// A stack-based string type with fixed-size storage.
///
/// This struct provides a string-like interface using a fixed-size buffer
/// to avoid heap allocations. The size of the buffer is determined by the
/// const generic parameter `SIZE`.
///
/// # Safety
///
/// This type ensures that only valid UTF-8 is stored and provides safe
/// access to the underlying bytes.
#[derive(Debug, Clone, Copy)]
pub struct StackString<const SIZE: usize> {
    buf: [u8; SIZE],
    len: usize,
}

impl<const SIZE: usize> StackString<SIZE> {
    /// Creates a new empty stack string.
    ///
    /// This operation is zero-cost and performs no heap allocations.
    pub fn new() -> Self {
        StackString {
            buf: [0; SIZE],
            len: 0,
        }
    }

    /// Pushes a character to the string if there's room.
    ///
    /// This method safely handles UTF-8 encoding and ensures the buffer
    /// doesn't overflow. It performs no heap allocations.
    pub fn push(&mut self, c: char) {
        let mut buf = [0u8; 4]; // UTF-8 chars can be up to 4 bytes
        let char_bytes = c.encode_utf8(&mut buf).as_bytes();

        // Check if we have enough space left
        if self.len + char_bytes.len() <= SIZE {
            self.buf[self.len..self.len + char_bytes.len()].copy_from_slice(char_bytes);
            self.len += char_bytes.len();
        }
    }

    /// Returns the current length in bytes.
    ///
    /// This is a zero-cost operation.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the string is empty.
    ///
    /// This is a zero-cost operation.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the string as a string slice.
    ///
    /// This is a zero-cost operation that returns a view into the internal buffer.
    pub fn as_str(&self) -> &str {
        // This is safe because we only insert valid UTF-8 characters
        std::str::from_utf8(&self.buf[0..self.len]).unwrap_or("")
    }
}

impl<const SIZE: usize> AsRef<str> for StackString<SIZE> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Decodes percent-encoded URL components.
///
/// This function efficiently decodes percent-encoded characters in URLs
/// (e.g., converting %20 to a space character) using only stack-based memory.
/// The size of the output buffer is determined by the const generic parameter
/// `OUTPUT_SIZE`.
///
/// # Examples
///
/// ```rust
/// use urlquerystring::percent_decode;
///
/// let decoded = percent_decode::<32>("hello%20world");
/// assert_eq!(decoded.as_str(), "hello world");
/// ```
pub fn percent_decode<const OUTPUT_SIZE: usize>(input: &str) -> StackString<OUTPUT_SIZE> {
    let mut result = StackString::<OUTPUT_SIZE>::new();
    let mut i = 0;
    let bytes = input.as_bytes();

    while i < bytes.len() && result.len() < OUTPUT_SIZE {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            // Try to decode the hex value
            if let (Some(hi), Some(lo)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                result.push((hi << 4 | lo) as char);
                i += 3;
            } else {
                result.push('%');
                i += 1;
            }
        } else if bytes[i] == b'+' {
            result.push(' ');
            i += 1;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let url = "https://example.com/path?name=John&age=25&city=New%20York";

        let mut params = StackQueryParams::default();
        params.parse_from_url(url);

        assert_eq!(params.len(), 3);
        assert_eq!(params.get("name"), Some("John"));
        assert_eq!(params.get("age"), Some("25"));
        assert_eq!(params.get("city"), Some("New York")); // Now decoded
    }

    #[test]
    fn test_no_query_params() {
        let url = "https://example.com/path";

        let mut params = StackQueryParams::default();
        params.parse_from_url(url);

        assert_eq!(params.len(), 0);
        assert!(params.is_empty());
    }

    #[test]
    fn test_empty_value() {
        let url = "https://example.com/path?param=";

        let mut params = StackQueryParams::default();
        params.parse_from_url(url);

        assert_eq!(params.len(), 1);
        assert_eq!(params.get("param"), Some(""));
    }

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode::<32>("hello+world").as_str(), "hello world");
        assert_eq!(
            percent_decode::<32>("hello%20world").as_str(),
            "hello world"
        );
        assert_eq!(percent_decode::<32>("50%25").as_str(), "50%");
        assert_eq!(percent_decode::<32>("a%2Fb%2Fc").as_str(), "a/b/c");
        assert_eq!(percent_decode::<32>("a+b+c").as_str(), "a b c");
    }

    #[test]
    fn test_max_params_limit() {
        // Create a URL with more than MAX_QUERY_PARAMS parameters
        let mut url = String::from("https://example.com/path?");
        for i in 0..MAX_QUERY_PARAMS + 5 {
            if i > 0 {
                url.push('&');
            }
            url.push_str(&format!("param{}=value{}", i, i));
        }

        let mut params = StackQueryParams::default();
        params.parse_from_url(&url);

        // Should only have parsed MAX_QUERY_PARAMS
        assert_eq!(params.len(), MAX_QUERY_PARAMS);
    }

    #[test]
    fn test_key_value_length_limits() {
        // Create a key and value that exceed the length limits
        let long_key = "a".repeat(MAX_KEY_LEN + 10);
        let long_value = "b".repeat(MAX_VALUE_LEN + 10);
        let url = format!("https://example.com/path?{}={}", long_key, long_value);

        let mut params = StackQueryParams::default();
        params.parse_from_url(&url);

        assert_eq!(params.len(), 1);

        // The parsed key should be truncated
        let expected_key = "a".repeat(MAX_KEY_LEN);
        let expected_value = "b".repeat(MAX_VALUE_LEN);

        // Get the first key directly since we can't lookup by the full long key
        let (actual_key, actual_value) = params.iter().next().unwrap();

        assert_eq!(actual_key, expected_key);
        assert_eq!(actual_value, expected_value);
    }
}
