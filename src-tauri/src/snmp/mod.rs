// src-tauri/src/snmp/mod.rs

pub mod client;
pub mod oids;

pub use client::{PrinterSnapshot, SnmpClient, SnmpConfig};
// Supply is used by commands/printer.rs via serde serialization of PrinterSnapshot
#[allow(unused_imports)]
pub use client::Supply;
