//! Version negotiation logic.

use super::types::ApiVersion;

/// Negotiate the best API version based on client preferences.
pub fn negotiate_version(
    accept_header: Option<&str>,
    supported: &[ApiVersion],
) -> ApiVersion {
    if let Some(accept) = accept_header {
        // Parse Accept-Version header (e.g., "v2, v1;q=0.5")
        let preferences = parse_version_preferences(accept);

        for (version, _weight) in preferences {
            if supported.contains(&version) {
                return version;
            }
        }
    }

    // Return latest supported version
    supported.iter().max().copied().unwrap_or(ApiVersion::LATEST)
}

fn parse_version_preferences(header: &str) -> Vec<(ApiVersion, f32)> {
    let mut preferences: Vec<(ApiVersion, f32)> = header
        .split(',')
        .filter_map(|part| {
            let parts: Vec<&str> = part.trim().split(';').collect();
            let version_str = parts.first()?.trim();
            let version = ApiVersion::parse(version_str)?;

            let weight = parts
                .get(1)
                .and_then(|q| {
                    q.trim()
                        .strip_prefix("q=")
                        .and_then(|w| w.parse::<f32>().ok())
                })
                .unwrap_or(1.0);

            Some((version, weight))
        })
        .collect();

    // Sort by weight descending
    preferences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    preferences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiate_version_simple() {
        let result = negotiate_version(
            Some("v2"),
            &[ApiVersion::V1, ApiVersion::V2],
        );
        assert_eq!(result, ApiVersion::V2);
    }

    #[test]
    fn test_negotiate_version_weighted() {
        let result = negotiate_version(
            Some("v1;q=0.5, v2;q=0.9"),
            &[ApiVersion::V1, ApiVersion::V2],
        );
        assert_eq!(result, ApiVersion::V2);
    }

    #[test]
    fn test_negotiate_version_unsupported() {
        let result = negotiate_version(
            Some("v3"),
            &[ApiVersion::V1, ApiVersion::V2],
        );
        assert_eq!(result, ApiVersion::V2); // Latest supported
    }

    #[test]
    fn test_negotiate_version_no_header() {
        let result = negotiate_version(
            None,
            &[ApiVersion::V1, ApiVersion::V2],
        );
        assert_eq!(result, ApiVersion::V2); // Latest supported
    }

    #[test]
    fn test_parse_version_preferences() {
        let preferences = parse_version_preferences("v1;q=0.5, v2;q=0.9, v1;q=0.7");
        assert_eq!(preferences.len(), 3);
        assert_eq!(preferences[0], (ApiVersion::V2, 0.9));
        assert_eq!(preferences[1], (ApiVersion::V1, 0.7));
        assert_eq!(preferences[2], (ApiVersion::V1, 0.5));
    }

    #[test]
    fn test_parse_version_preferences_no_weights() {
        let preferences = parse_version_preferences("v2, v1");
        assert_eq!(preferences.len(), 2);
        assert_eq!(preferences[0], (ApiVersion::V2, 1.0));
        assert_eq!(preferences[1], (ApiVersion::V1, 1.0));
    }
}