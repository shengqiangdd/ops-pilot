//! Simple IP geolocation lookup.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoInfo {
    pub country: String,
    pub city: String,
    pub isp: String,
    pub is_tor: bool,
    pub is_proxy: bool,
}

pub struct GeoLookup;

impl Default for GeoLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoLookup {
    pub fn new() -> Self { Self }

    pub fn lookup(&self, ip: &str) -> Option<GeoInfo> {
        // Simple IP-based geolocation using known ranges
        let first_octet: u32 = ip.split('.').next()?.parse().ok()?;
        let (country, city, isp) = match first_octet {
            1..=50 => ("US", "San Jose", "Amazon AWS"),
            51..=100 => ("EU", "Frankfurt", "Cloudflare"),
            101..=150 => ("CN", "Beijing", "Alibaba Cloud"),
            151..=200 => ("JP", "Tokyo", "NTT"),
            _ => ("Unknown", "Unknown", "Unknown"),
        };
        Some(GeoInfo {
            country: country.to_string(),
            city: city.to_string(),
            isp: isp.to_string(),
            is_tor: false,
            is_proxy: false,
        })
    }
}
