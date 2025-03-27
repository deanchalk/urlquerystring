/// A high-performance, zero-allocation URL query string parser.
///
/// This crate provides a stack-based implementation for parsing URL query strings
/// without any heap allocations. It's designed for performance-critical environments
/// where memory allocation overhead needs to be minimized.
///
/// # Features
///
/// - Zero heap allocations
/// - High performance through direct byte manipulation
/// - UTF-8 safe parameter handling
/// - Built-in percent-decoding support
/// - Const-generic based size configuration
/// - Zero-cost abstractions where possible
///
/// # Examples
///
/// ```rust
/// use urlquerystring::StackQueryParams;
///
/// let url = "https://example.com/path?name=John&age=25&city=New%20York";
/// let params = StackQueryParams::new(url);
///
/// assert_eq!(params.get("name"), Some("John"));
/// assert_eq!(params.get("city"), Some("New York")); // Automatically percent-decoded
/// ```

/// Default maximum number of query parameters that can be stored.
pub const MAX_PARAM_COUNT: usize = 16;

/// Default maximum length for parameter keys in bytes.
pub const MAX_KEY_SIZE: usize = 32;

/// Default maximum length for parameter values in bytes.
pub const MAX_VALUE_SIZE: usize = 128;

/// A stack-based parameter type with fixed-size storage for key and value.
///
/// This struct provides a container for a single query parameter with fixed-size
/// buffers for both the key and value. The sizes are determined by the const
/// generic parameters `KEY_SIZE` and `VALUE_SIZE`.
///
/// # Examples
///
/// ```rust
/// use urlquerystring::StackParam;
///
/// let param = StackParam::<32, 128>::new();
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

/// A stack-based query parameters container with fixed-size storage.
///
/// This struct provides a container for URL query parameters with fixed-size
/// storage for both the number of parameters and their individual key/value sizes.
/// All memory is allocated on the stack, making it suitable for performance-critical
/// environments.
///
/// The size limits are determined by the const generic parameters:
/// - `PARAM_COUNT`: Maximum number of parameters that can be stored
/// - `KEY_SIZE`: Maximum length for parameter keys in bytes
/// - `VALUE_SIZE`: Maximum length for parameter values in bytes
///
/// # Examples
///
/// ```rust
/// use urlquerystring::StackQueryParams;
///
/// // Using default size limits
/// let params = StackQueryParams::new("https://example.com/path?name=John");
///
/// // Using custom size limits
/// let params = StackQueryParams::<32, 64, 256>::custom_new("https://example.com/path?param=value");
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


impl<const PARAM_COUNT: usize, const KEY_SIZE: usize, const VALUE_SIZE: usize>
    StackQueryParams<PARAM_COUNT, KEY_SIZE, VALUE_SIZE>
{
    /// Creates a new query parameters container with custom size limits.
    ///
    /// This constructor allows you to specify custom size limits for the number of
    /// parameters, key length, and value length. It immediately parses the provided
    /// URL string.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL string to parse query parameters from
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let params = StackQueryParams::<32, 64, 256>::custom_new(
    ///     "https://example.com/path?param=value"
    /// );
    /// ```
    pub fn custom_new(url: &str) -> Self {
        let mut stack_query_params = StackQueryParams {
            params: [StackParam::<KEY_SIZE, VALUE_SIZE>::new(); PARAM_COUNT],
            count: 0,
        };
        stack_query_params.parse_from_url(url);
        stack_query_params
    }
}

impl StackQueryParams<MAX_PARAM_COUNT, MAX_KEY_SIZE, MAX_VALUE_SIZE> {
    /// Creates a new query parameters container with default size limits.
    ///
    /// This constructor uses the default size limits defined by the constants:
    /// - `MAX_PARAM_COUNT`: 16 parameters
    /// - `MAX_KEY_SIZE`: 32 bytes
    /// - `MAX_VALUE_SIZE`: 128 bytes
    ///
    /// It immediately parses the provided URL string.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL string to parse query parameters from
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let params = StackQueryParams::new("https://example.com/path?name=John");
    /// ```
    pub fn new(url: &str) -> Self {
        let mut stack_query_params = StackQueryParams {
            params: [StackParam::<MAX_KEY_SIZE, MAX_VALUE_SIZE>::new(); MAX_PARAM_COUNT],
            count: 0,
        };
        stack_query_params.parse_from_url(url);
        stack_query_params
    }
}

impl<const PARAM_COUNT: usize, const KEY_SIZE: usize, const VALUE_SIZE: usize>
    StackQueryParams<PARAM_COUNT, KEY_SIZE, VALUE_SIZE>
{
    /// Parses a URL query string with automatic percent-decoding.
    ///
    /// This method efficiently parses the query string portion of a URL,
    /// automatically handling percent-encoded characters and plus signs.
    /// It performs no heap allocations and uses only stack-based memory.
    ///
    /// The method will:
    /// - Skip any URL portion before the '?' character
    /// - Parse key-value pairs separated by '&'
    /// - Handle empty values (e.g., "key=")
    /// - Automatically percent-decode both keys and values
    /// - Stop parsing if the parameter count limit is reached
    ///
    /// # Arguments
    ///
    /// * `url` - The URL string to parse query parameters from
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let mut params = StackQueryParams::new("https://example.com/path");
    /// params.parse_from_url("https://example.com/path?name=John&age=25");
    /// ```
    fn parse_from_url(&mut self, url: &str) {
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
            let param = &mut self.params[self.count];
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
    /// The search is case-sensitive.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Option<&str>` - The value if found, None otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let params = StackQueryParams::new("https://example.com/path?name=John");
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
    /// The count will never exceed the `PARAM_COUNT` limit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let params = StackQueryParams::new("https://example.com/path?name=John&age=25");
    /// assert_eq!(params.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if no parameters are stored.
    ///
    /// This is a zero-cost operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let params = StackQueryParams::new("https://example.com/path");
    /// assert!(params.is_empty());
    ///
    /// let params = StackQueryParams::new("https://example.com/path?name=John");
    /// assert!(!params.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns an iterator over all key-value pairs.
    ///
    /// The iterator yields tuples of `(&str, &str)` representing the key and value
    /// of each parameter. This is a zero-cost operation that returns views into
    /// the internal buffers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackQueryParams;
    ///
    /// let params = StackQueryParams::new("https://example.com/path?name=John&age=25");
    /// let pairs: Vec<_> = params.iter().collect();
    ///
    /// assert_eq!(pairs.len(), 2);
    /// assert!(pairs.contains(&("name", "John")));
    /// assert!(pairs.contains(&("age", "25")));
    /// ```
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
///
/// # Examples
///
/// ```rust
/// use urlquerystring::StackString;
///
/// let mut s = StackString::<32>::new();
/// s.push('H');
/// s.push('e');
/// s.push('l');
/// s.push('l');
/// s.push('o');
///
/// assert_eq!(s.as_str(), "Hello");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StackString<const SIZE: usize> {
    buf: [u8; SIZE],
    len: usize,
}

impl<const SIZE: usize> StackString<SIZE> {
    /// Creates a new empty stack string.
    ///
    /// This operation is zero-cost and performs no heap allocations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackString;
    ///
    /// let s = StackString::<32>::new();
    /// assert!(s.is_empty());
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `c` - The character to append
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackString;
    ///
    /// let mut s = StackString::<32>::new();
    /// s.push('H');
    /// s.push('e');
    /// s.push('l');
    /// s.push('l');
    /// s.push('o');
    ///
    /// assert_eq!(s.as_str(), "Hello");
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackString;
    ///
    /// let mut s = StackString::<32>::new();
    /// s.push('H');
    /// s.push('e');
    /// s.push('l');
    /// s.push('l');
    /// s.push('o');
    ///
    /// assert_eq!(s.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the string is empty.
    ///
    /// This is a zero-cost operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackString;
    ///
    /// let mut s = StackString::<32>::new();
    /// assert!(s.is_empty());
    ///
    /// s.push('H');
    /// assert!(!s.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the string as a string slice.
    ///
    /// This is a zero-cost operation that returns a view into the internal buffer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use urlquerystring::StackString;
    ///
    /// let mut s = StackString::<32>::new();
    /// s.push('H');
    /// s.push('e');
    /// s.push('l');
    /// s.push('l');
    /// s.push('o');
    ///
    /// assert_eq!(s.as_str(), "Hello");
    /// ```
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
fn percent_decode<const OUTPUT_SIZE: usize>(input: &str) -> StackString<OUTPUT_SIZE> {
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

        let params = StackQueryParams::new(url);

        assert_eq!(params.len(), 3);
        assert_eq!(params.get("name"), Some("John"));
        assert_eq!(params.get("age"), Some("25"));
        assert_eq!(params.get("city"), Some("New York")); // Now decoded
    }

    #[test]
    fn test_no_query_params() {
        let url = "https://example.com/path";

        let params = StackQueryParams::new(url);

        assert_eq!(params.len(), 0);
        assert!(params.is_empty());
    }

    #[test]
    fn test_empty_value() {
        let url = "https://example.com/path?param=";

        let params = StackQueryParams::new(url);

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
        for i in 0..MAX_PARAM_COUNT + 5 {
            if i > 0 {
                url.push('&');
            }
            url.push_str(&format!("param{}=value{}", i, i));
        }

        let params = StackQueryParams::new(&url);

        // Should only have parsed MAX_QUERY_PARAMS
        assert_eq!(params.len(), MAX_PARAM_COUNT);
    }

    #[test]
    fn test_key_value_length_limits() {
        // Create a key and value that exceed the length limits
        let long_key = "a".repeat(MAX_KEY_SIZE + 10);
        let long_value = "b".repeat(MAX_VALUE_SIZE + 10);
        let url = format!("https://example.com/path?{}={}", long_key, long_value);

        let params = StackQueryParams::new(&url);

        assert_eq!(params.len(), 1);

        // The parsed key should be truncated
        let expected_key = "a".repeat(MAX_KEY_SIZE);
        let expected_value = "b".repeat(MAX_VALUE_SIZE);

        // Get the first key directly since we can't lookup by the full long key
        let (actual_key, actual_value) = params.iter().next().unwrap();

        assert_eq!(actual_key, expected_key);
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn test_custom_new() {
        let url = "https://example.com/path?name=John&age=25&city=New%20York";

        let params = StackQueryParams::<8, 16, 64>::custom_new(url);

        assert_eq!(params.len(), 3);
        assert_eq!(params.get("name"), Some("John"));
        assert_eq!(params.get("age"), Some("25"));
        assert_eq!(params.get("city"), Some("New York"));
    }
}
