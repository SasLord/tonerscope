// src-tauri/src/scanner/network.rs

use crate::snmp::{SnmpClient, SnmpConfig};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub ip:           String,
    pub is_reachable: bool,
    pub is_snmp_open: bool,
    pub sys_descr:    Option<String>,
    pub brand:        Option<String>,
    pub model:        Option<String>,
}

pub struct NetworkScanner {
    community: String,
}

impl NetworkScanner {
    pub fn new(community: &str) -> Self {
        Self { community: community.to_string() }
    }

    /// Scan entire subnet concurrently; calls progress_cb(done, total) after each host
    pub async fn scan_subnet(
        &self,
        subnet: &str,
        progress_cb: impl Fn(u32, u32) + Send + Sync + 'static,
    ) -> Vec<ScanResult> {
        let network: IpNetwork = match subnet.parse() {
            Ok(n)  => n,
            Err(_) => return vec![],
        };

        // Collect usable host addresses (skip network address & broadcast)
        let hosts: Vec<IpAddr> = network
            .iter()
            .filter(|ip| is_usable_host(ip, &network))
            .collect();

        let total     = hosts.len() as u32;
        let community = Arc::new(self.community.clone());
        let cb        = Arc::new(progress_cb);
        let completed = Arc::new(std::sync::atomic::AtomicU32::new(0));

        let mut set = JoinSet::new();

        for ip in hosts {
            let comm = community.clone();
            let cb2  = cb.clone();
            let done = completed.clone();

            set.spawn_blocking(move || {
                let cfg = SnmpConfig {
                    community: (*comm).clone(),
                    timeout:   Duration::from_secs(1),
                    retries:   1,
                };
                let client  = SnmpClient::new(cfg);
                let ip_str  = ip.to_string();
                let descr   = client.probe(&ip_str);

                let current = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                cb2(current, total);

                descr.map(|d| {
                    let brand = detect_brand(&d);
                    let model = detect_model(&d, &brand);
                    ScanResult {
                        ip:           ip_str,
                        is_reachable: true,
                        is_snmp_open: true,
                        sys_descr:    Some(d),
                        brand:        Some(brand),
                        model:        Some(model),
                    }
                })
            });
        }

        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            if let Ok(Some(r)) = res {
                results.push(r);
            }
        }

        // Sort by IP address numerically
        results.sort_by(|a, b| {
            let a_parts: Vec<u8> = a.ip.split('.').filter_map(|p| p.parse().ok()).collect();
            let b_parts: Vec<u8> = b.ip.split('.').filter_map(|p| p.parse().ok()).collect();
            a_parts.cmp(&b_parts)
        });

        results
    }
}

/// Filter out network address (last octet 0) and broadcast (last octet 255)
/// Works for both IPv4 /24 and other subnets via ipnetwork bounds.
fn is_usable_host(ip: &IpAddr, network: &IpNetwork) -> bool {
    // network_address() and broadcast() are only meaningful for IPv4
    match (ip, network) {
        (IpAddr::V4(addr), IpNetwork::V4(net)) => {
            *addr != net.network() && *addr != net.broadcast()
        }
        // For IPv6 just include all addresses in range
        _ => true,
    }
}

fn detect_brand(descr: &str) -> String {
    let d = descr.to_lowercase();
    if d.contains("pantum")                   { return "pantum".into();  }
    if d.contains("kyocera")                  { return "kyocera".into(); }
    if d.contains("hp") || d.contains("hewlett") { return "hp".into();  }
    if d.contains("canon")                    { return "canon".into();   }
    "other".into()
}

fn detect_model(descr: &str, brand: &str) -> String {
    let brand_label = match brand {
        "pantum"  => "Pantum",
        "kyocera" => "Kyocera",
        "hp"      => "HP",
        "canon"   => "Canon",
        _         => return descr.split_whitespace().take(3).collect::<Vec<_>>().join(" "),
    };

    if let Some(pos) = descr.find(brand_label) {
        let after = &descr[pos + brand_label.len()..];
        let model = after.split([',', ';', '\n']).next().unwrap_or("").trim();
        if !model.is_empty() {
            return model.to_string();
        }
    }

    descr.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
}
