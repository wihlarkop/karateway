use karateway_core::models::{RuleType, WhitelistRule};
use pingora_http::RequestHeader;
use tracing::{debug, warn};

/// Validates a request against whitelist rules
pub struct WhitelistValidator;

impl WhitelistValidator {
    /// Check if a request is allowed by the whitelist rules
    /// Returns (allowed, rule_name) - if allowed is false, rule_name contains the blocking rule name
    pub fn validate_request(
        rules: &[WhitelistRule],
        req_header: &RequestHeader,
        client_ip: Option<&str>,
    ) -> (bool, Option<String>) {
        if rules.is_empty() {
            // No whitelist rules = allow all
            debug!("No whitelist rules configured, allowing request");
            return (true, None);
        }

        debug!("Validating request against {} whitelist rules", rules.len());

        // Check each rule in priority order (rules should already be sorted by priority)
        for rule in rules {
            debug!("Checking whitelist rule: {} (type: {})", rule.rule_name, rule.rule_type);

            let allowed = match rule.rule_type {
                RuleType::Ip => Self::validate_ip_rule(rule, client_ip),
                RuleType::ApiKey => Self::validate_api_key_rule(rule, req_header),
                RuleType::Jwt => Self::validate_jwt_rule(rule, req_header),
                RuleType::Custom => {
                    warn!("Custom whitelist rules not yet implemented");
                    false
                }
            };

            if allowed {
                debug!("Request allowed by whitelist rule: {}", rule.rule_name);
                return (true, Some(rule.rule_name.clone()));
            }
        }

        debug!("Request denied: no matching whitelist rule found");
        (false, None)
    }

    /// Validate IP-based whitelist rule
    fn validate_ip_rule(rule: &WhitelistRule, client_ip: Option<&str>) -> bool {
        let client_ip = match client_ip {
            Some(ip) => ip,
            None => {
                debug!("No client IP provided for IP whitelist rule");
                return false;
            }
        };

        // Parse allowed IPs from config
        let allowed_ips = match rule.config.get("allowed_ips") {
            Some(ips) => match ips.as_array() {
                Some(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<&str>>(),
                None => {
                    warn!("Invalid allowed_ips format in rule {}", rule.rule_name);
                    return false;
                }
            },
            None => {
                warn!("No allowed_ips configured in rule {}", rule.rule_name);
                return false;
            }
        };

        debug!("Checking client IP {} against {} allowed IPs", client_ip, allowed_ips.len());

        // Check if client IP matches any allowed IP or CIDR range
        for allowed_ip in allowed_ips {
            if Self::ip_matches(client_ip, allowed_ip) {
                debug!("Client IP {} matches allowed IP/CIDR {}", client_ip, allowed_ip);
                return true;
            }
        }

        debug!("Client IP {} did not match any allowed IPs", client_ip);
        false
    }

    /// Check if a client IP matches an allowed IP or CIDR range
    fn ip_matches(client_ip: &str, allowed_pattern: &str) -> bool {
        // If pattern contains '/', it's a CIDR range
        if allowed_pattern.contains('/') {
            // TODO: Implement CIDR matching
            // For now, exact match only
            warn!("CIDR matching not yet implemented, using exact match");
            client_ip == allowed_pattern.split('/').next().unwrap_or("")
        } else {
            // Exact IP match
            client_ip == allowed_pattern
        }
    }

    /// Validate API key-based whitelist rule
    fn validate_api_key_rule(rule: &WhitelistRule, req_header: &RequestHeader) -> bool {
        // Get API key from header
        let api_key = match req_header.headers.get("X-API-Key") {
            Some(header_value) => match header_value.to_str() {
                Ok(key) => key,
                Err(_) => {
                    debug!("Invalid X-API-Key header format");
                    return false;
                }
            },
            None => {
                debug!("No X-API-Key header found");
                return false;
            }
        };

        // Get allowed API keys from config
        let allowed_keys = match rule.config.get("allowed_keys") {
            Some(keys) => match keys.as_array() {
                Some(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<&str>>(),
                None => {
                    warn!("Invalid allowed_keys format in rule {}", rule.rule_name);
                    return false;
                }
            },
            None => {
                warn!("No allowed_keys configured in rule {}", rule.rule_name);
                return false;
            }
        };

        debug!("Checking API key against {} allowed keys", allowed_keys.len());

        // Check if API key matches any allowed key
        let matches = allowed_keys.contains(&api_key);
        if matches {
            debug!("API key matched");
        } else {
            debug!("API key did not match any allowed keys");
        }

        matches
    }

    /// Validate JWT-based whitelist rule
    fn validate_jwt_rule(rule: &WhitelistRule, req_header: &RequestHeader) -> bool {
        // Get JWT from Authorization header
        let auth_header = match req_header.headers.get("Authorization") {
            Some(header_value) => match header_value.to_str() {
                Ok(value) => value,
                Err(_) => {
                    debug!("Invalid Authorization header format");
                    return false;
                }
            },
            None => {
                debug!("No Authorization header found");
                return false;
            }
        };

        // Extract JWT token (remove "Bearer " prefix)
        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            debug!("Authorization header doesn't start with 'Bearer '");
            return false;
        };

        // TODO: Implement JWT validation
        // For now, just check if token is present and matches expected patterns
        let _jwt_secret = rule.config.get("jwt_secret").and_then(|v| v.as_str());
        let _allowed_issuers = rule.config.get("allowed_issuers");
        let _allowed_audiences = rule.config.get("allowed_audiences");

        warn!("JWT validation not fully implemented yet");

        // Basic check: token should have 3 parts separated by dots
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() == 3 {
            debug!("JWT token has valid format (3 parts)");
            // TODO: Actually validate the JWT signature and claims
            true
        } else {
            debug!("JWT token has invalid format");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_ip_matches_exact() {
        assert!(WhitelistValidator::ip_matches("192.168.1.1", "192.168.1.1"));
        assert!(!WhitelistValidator::ip_matches("192.168.1.1", "192.168.1.2"));
    }
}
