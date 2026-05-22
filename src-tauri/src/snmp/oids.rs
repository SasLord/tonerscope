// src-tauri/src/snmp/oids.rs

// ─── Standard Printer MIB (RFC 3805) ─────────────────────────────────────────

/// sysDescr — описание устройства (определяем бренд/модель)
pub const SYS_DESCR: &[u32] = &[1, 3, 6, 1, 2, 1, 1, 1, 0];

/// sysName — имя устройства
pub const SYS_NAME: &[u32] = &[1, 3, 6, 1, 2, 1, 1, 5, 0];

/// hrPrinterStatus — статус принтера (1=other 2=unknown 3=idle 4=printing 5=warmup)
pub const PRINTER_STATUS: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 5, 1, 1, 1];

/// hrPrinterDetectedErrorState — биты ошибок принтера (Phase 2)
#[allow(dead_code)]
pub const PRINTER_ERROR: &[u32] = &[1, 3, 6, 1, 2, 1, 25, 3, 5, 1, 2, 1];

/// prtMarkerLifeCount — счётчик страниц (всего)
pub const PAGE_COUNT: &[u32] = &[1, 3, 6, 1, 2, 1, 43, 10, 2, 1, 4, 1, 1];

// ─── Supply / Consumable OIDs ─────────────────────────────────────────────────
// Индекс i = номер расходника (1-based). Функции ниже строят OID динамически.

/// prtMarkerSuppliesDescription — название расходника (String)
pub fn supply_description(i: u32) -> Vec<u32> {
    vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 6, 1, i]
}

/// prtMarkerSuppliesType — тип расходника (enum)
/// 3=toner 4=wasteToner 5=ink 7=ribbonInk 11=solidWax 16=drum
pub fn supply_type(i: u32) -> Vec<u32> {
    vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 4, 1, i]
}

/// prtMarkerSuppliesMaxCapacity — максимальный уровень
pub fn supply_max(i: u32) -> Vec<u32> {
    vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 8, 1, i]
}

/// prtMarkerSuppliesLevel — текущий уровень (-3 = unknown, -2 = no restriction)
pub fn supply_level(i: u32) -> Vec<u32> {
    vec![1, 3, 6, 1, 2, 1, 43, 11, 1, 1, 9, 1, i]
}

/// prtMarkerColorantValue — цвет расходника (String: "black", "cyan", ...)
pub fn supply_color(i: u32) -> Vec<u32> {
    vec![1, 3, 6, 1, 2, 1, 43, 12, 1, 1, 4, 1, i]
}

// ─── Vendor-specific OIDs (used in Phase 2 for brand-specific page counts) ───

#[allow(dead_code)]
pub mod pantum {
    /// Pantum total page count (может отличаться от стандарта)
    pub const PAGE_COUNT: &[u32] = &[1, 3, 6, 1, 4, 1, 40945, 1, 1, 2, 15, 0];
}

#[allow(dead_code)]
pub mod kyocera {
    /// Kyocera total impressions
    pub const PAGE_COUNT: &[u32] = &[1, 3, 6, 1, 4, 1, 1347, 42, 2, 1, 1, 4, 1, 1];
    /// Kyocera drum remaining (%)
    pub const DRUM_REMAINING: &[u32] = &[1, 3, 6, 1, 4, 1, 1347, 42, 3, 10, 5, 0];
}

#[allow(dead_code)]
pub mod hp {
    /// HP total engine page count
    pub const PAGE_COUNT: &[u32] = &[1, 3, 6, 1, 4, 1, 11, 2, 3, 9, 4, 2, 1, 4, 1, 5, 0];
}

#[allow(dead_code)]
pub mod canon {
    /// Canon counter total
    pub const PAGE_COUNT: &[u32] = &[1, 3, 6, 1, 4, 1, 1602, 1, 1, 1, 1, 10, 0];
}

// ─── Supply type mapping ──────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum SupplyKind {
    TonerBlack,
    TonerCyan,
    TonerMagenta,
    TonerYellow,
    Drum,
    Fuser,
    Waste,
    Other,
}

impl SupplyKind {
    /// Map from snmpSupplyType integer + color string
    pub fn from_snmp(supply_type: i32, color: &str, description: &str) -> Self {
        let desc_low = description.to_lowercase();
        let color_low = color.to_lowercase();

        if desc_low.contains("drum") || desc_low.contains("барабан") {
            return Self::Drum;
        }
        if desc_low.contains("fuser") || desc_low.contains("термо") {
            return Self::Fuser;
        }
        if desc_low.contains("waste") || desc_low.contains("отработ") {
            return Self::Waste;
        }

        // supply_type 3 = toner
        if supply_type == 3 || desc_low.contains("toner") || desc_low.contains("тонер") {
            return match color_low.as_str() {
                "cyan"    => Self::TonerCyan,
                "magenta" => Self::TonerMagenta,
                "yellow"  => Self::TonerYellow,
                _         => Self::TonerBlack,
            };
        }

        Self::Other
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TonerBlack   => "toner_black",
            Self::TonerCyan    => "toner_cyan",
            Self::TonerMagenta => "toner_magenta",
            Self::TonerYellow  => "toner_yellow",
            Self::Drum         => "drum",
            Self::Fuser        => "fuser",
            Self::Waste        => "waste",
            Self::Other        => "other",
        }
    }
}
