use regex::Regex;
use std::sync::OnceLock;

static DOMAIN_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn is_safe_hostname(hostname: &str) -> bool {
    // 1. Basic length validation
    if hostname.is_empty() || hostname.len() > 255 {
        return false;
    }

    // 2. Reject characters dangerous for URLs to prevent userinfo injection, port specification, and path/query manipulation
    if hostname.chars().any(|c| {
        matches!(
            c,
            '@' | ':' | '/' | '\\' | '?' | '#' | ' ' | '\r' | '\n' | '\t'
        )
    }) {
        return false;
    }

    // 3. Reject valid IP addresses (IPv4/IPv6)
    if hostname.parse::<std::net::IpAddr>().is_ok() {
        return false;
    }

    // 4. Reject purely numeric hostnames (decimal representation of IPv4)
    if hostname.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // 5. Require at least one dot and ensure labels are not empty
    let parts: Vec<&str> = hostname.split('.').collect();
    if parts.len() < 2 || parts.iter().any(|s| s.is_empty()) {
        return false;
    }

    // 6. Reject known non-public/reserved TLDs and ensure TLD is not purely numeric
    // Public TLDs must contain at least one letter (or be punycode xn--).
    // This helps prevent bypasses like '127.1' or hex/octal labels.
    if let Some(tld) = parts.last() {
        if !tld.chars().any(|c| c.is_ascii_alphabetic()) {
            return false;
        }

        let tld_lower = tld.to_lowercase();
        let reserved_tlds = [
            "local",
            "internal",
            "lan",
            "home",
            "host",
            "corp",
            "test",
            "invalid",
            "localhost",
            "onion",
            "example",
            "arpa",
        ];
        if reserved_tlds.contains(&tld_lower.as_str()) {
            return false;
        }
    }

    // 7. Use Regex to enforce standard domain name format
    let re = DOMAIN_REGEX.get_or_init(|| {
        Regex::new(
            r"^(?i)[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)+$",
        )
        .expect("Invalid domain regex")
    });

    re.is_match(hostname)
}

#[cfg(test)]
mod tests {
    use super::is_safe_hostname;

    #[test]
    fn test_is_safe_hostname() {
        // Valid domains
        assert!(is_safe_hostname("google.com"));
        assert!(is_safe_hostname("www.google.com"));
        assert!(is_safe_hostname("my-domain.co.uk"));
        assert!(is_safe_hostname("a.b.c.d.e.f.g.com"));

        // Invalid: Empty or too long
        assert!(!is_safe_hostname(""));
        assert!(!is_safe_hostname(&"a".repeat(256)));

        // Invalid: Suspicious characters
        assert!(!is_safe_hostname("user@google.com"));
        assert!(!is_safe_hostname("google.com:8080"));
        assert!(!is_safe_hostname("google.com/path"));
        assert!(!is_safe_hostname("google.com?query"));
        assert!(!is_safe_hostname("google.com#fragment"));
        assert!(!is_safe_hostname("google.com "));
        assert!(!is_safe_hostname("google\n.com"));

        // Invalid: IP addresses
        assert!(!is_safe_hostname("127.0.0.1"));
        assert!(!is_safe_hostname("8.8.8.8"));
        assert!(!is_safe_hostname("::1"));
        assert!(!is_safe_hostname("[::1]"));

        // Invalid: Numeric hostnames (decimal/hex/octal representations of IPs)
        assert!(!is_safe_hostname("2130706433")); // 127.0.0.1 in decimal
        assert!(!is_safe_hostname("0x7f000001")); // 127.0.0.1 in hex (no dots)
        assert!(!is_safe_hostname("127.1")); // Shortened IPv4
        assert!(!is_safe_hostname("0x7f.0.0.1")); // Hex-encoded labels
        assert!(!is_safe_hostname("0177.0.0.1")); // Octal labels

        // Invalid: Reserved/Internal TLDs
        assert!(!is_safe_hostname("localhost"));
        assert!(!is_safe_hostname("something.local"));
        assert!(!is_safe_hostname("my.internal"));
        assert!(!is_safe_hostname("test.test"));
        assert!(!is_safe_hostname("example.onion"));

        // Invalid: Formatting issues
        assert!(!is_safe_hostname(".google.com"));
        assert!(!is_safe_hostname("google.com."));
        assert!(!is_safe_hostname("google..com"));
    }
}
