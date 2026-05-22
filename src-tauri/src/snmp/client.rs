// src-tauri/src/snmp/client.rs

use super::oids::{self, SupplyKind};
use serde::{Deserialize, Serialize};
use snmp::{SyncSession, Value};
use std::time::Duration;

// ─── Data Structures ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Supply {
    pub supply_type: String,
    pub name:        String,
    pub level:       i64,
    pub max_level:   i64,
    pub percent:     u8,
    pub is_low:      bool,
    pub is_critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterSnapshot {
    pub ip:         String,
    pub sys_descr:  Option<String>,
    pub sys_name:   Option<String>,
    pub status:     String,
    pub supplies:   Vec<Supply>,
    pub page_count: Option<i64>,
    pub reachable:  bool,
}

// ─── Config ───────────────────────────────────────────────────────────────────

pub struct SnmpConfig {
    pub community: String,
    pub timeout:   Duration,
    pub retries:   u32,
}

impl Default for SnmpConfig {
    fn default() -> Self {
        Self {
            community: "public".into(),
            timeout:   Duration::from_secs(3),
            retries:   2,
        }
    }
}

// ─── Client ───────────────────────────────────────────────────────────────────

pub struct SnmpClient {
    config: SnmpConfig,
}

impl SnmpClient {
    pub fn new(config: SnmpConfig) -> Self {
        Self { config }
    }

    /// Full poll: status + all supplies + page count
    pub fn poll(&self, ip: &str, low_threshold: u8, crit_threshold: u8) -> PrinterSnapshot {
        let addr      = format!("{}:161", ip);
        let community = self.config.community.as_bytes().to_vec();

        let mut session = match SyncSession::new(
            addr,
            &community,
            Some(self.config.timeout),
            self.config.retries as i32,
        ) {
            Ok(s)  => s,
            Err(_) => return PrinterSnapshot {
                ip:         ip.to_string(),
                sys_descr:  None,
                sys_name:   None,
                status:     "offline".into(),
                supplies:   vec![],
                page_count: None,
                reachable:  false,
            },
        };

        let sys_descr  = self.get_string(&mut session, oids::SYS_DESCR);
        let sys_name   = self.get_string(&mut session, oids::SYS_NAME);
        let status     = self.get_printer_status(&mut session);
        let supplies   = self.get_supplies(&mut session, low_threshold, crit_threshold);
        let page_count = self.get_integer(&mut session, oids::PAGE_COUNT);

        PrinterSnapshot {
            ip: ip.to_string(),
            sys_descr,
            sys_name,
            status,
            supplies,
            page_count,
            reachable: true,
        }
    }

    /// Quick probe — only sysDescr, short timeout
    pub fn probe(&self, ip: &str) -> Option<String> {
        let addr      = format!("{}:161", ip);
        let community = self.config.community.as_bytes().to_vec();

        SyncSession::new(addr, &community, Some(Duration::from_secs(1)), 1)
            .ok()
            .and_then(|mut s| self.get_string(&mut s, oids::SYS_DESCR))
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn get_string(&self, session: &mut SyncSession, oid: &[u32]) -> Option<String> {
        session.get(oid).ok().and_then(|mut resp| {
            resp.varbinds.next().and_then(|(_, v)| {
                if let Value::OctetString(bytes) = v {
                    Some(String::from_utf8_lossy(bytes).trim().to_string())
                } else {
                    None
                }
            })
        })
    }

    fn get_integer(&self, session: &mut SyncSession, oid: &[u32]) -> Option<i64> {
        session.get(oid).ok().and_then(|mut resp| {
            resp.varbinds.next().and_then(|(_, v)| match v {
                // Value variants hold values directly (not references) — no deref needed
                Value::Integer(n)    => Some(n as i64),
                Value::Counter32(n)  => Some(n as i64),
                Value::Counter64(n)  => Some(n as i64),
                Value::Unsigned32(n) => Some(n as i64),  // also covers Gauge32 (same ASN.1 tag)
                _                    => None,
            })
        })
    }

    fn get_printer_status(&self, session: &mut SyncSession) -> String {
        match self.get_integer(session, oids::PRINTER_STATUS) {
            Some(3) => "online".into(),    // idle
            Some(4) => "printing".into(),  // printing
            Some(5) => "online".into(),    // warmup
            Some(1) | Some(2) => "unknown".into(),
            Some(_) => "warning".into(),
            None    => "offline".into(),
        }
    }

    fn get_supplies(
        &self,
        session: &mut SyncSession,
        low_th: u8,
        crit_th: u8,
    ) -> Vec<Supply> {
        let mut supplies = Vec::new();

        for i in 1u32..=8 {
            let level = match self.get_integer(session, &oids::supply_level(i)) {
                Some(v) if v >= -2 => v,
                _ => break, // no more supply entries at this index
            };
            let max     = self.get_integer(session, &oids::supply_max(i)).unwrap_or(0);
            let name    = self.get_string(session, &oids::supply_description(i))
                              .unwrap_or_default();
            let color   = self.get_string(session, &oids::supply_color(i))
                              .unwrap_or_default();
            let stype_n = self.get_integer(session, &oids::supply_type(i))
                              .unwrap_or(0) as i32;

            if max <= 0 { continue; }

            let pct = if level >= 0 {
                ((level as f64 / max as f64) * 100.0)
                    .round()
                    .clamp(0.0, 100.0) as u8
            } else {
                0
            };

            let kind = SupplyKind::from_snmp(stype_n, &color, &name);
            let display_name = if name.is_empty() {
                kind.as_str().replace('_', " ")
            } else {
                name
            };

            supplies.push(Supply {
                supply_type: kind.as_str().into(),
                name:        display_name,
                level,
                max_level:   max,
                percent:     pct,
                is_low:      pct <= low_th,
                is_critical: pct <= crit_th,
            });
        }

        supplies
    }
}
