use std::io::Write;

use regex::Regex;
use tabled::{settings::Style, Table, Tabled};

const HIGH_POOL_MASK: u64 = 0xFF << 56;
const HIGH_SHARD_MASK: u64 = 0xFF << 48;
const HIGH_CRUSH_MASK: u64 = 0xFFFFFFFF << 16;
const HIGH_RANDOM_MASK: u64 = 0xFFFF;

mod no_shadow {
    pub const LOW_RANDOM_MASK: u64 = u64::MAX << 46;
}

mod has_shadow {
    pub const LOW_RANDOM_MASK: u64 = u64::MAX << 47;
    pub const LOW_SHADOW_MASK: u64 = 1 << 46;
}

const LOW_METADATA_MASK: u64 = 1 << 45;
const LOW_SNAP_MASK: u64 = 1 << 44;
const LOW_LOCAL_SNAP_ID_MASK: u64 = 0xFFFFFFFF << 12;
const LOW_OFFSET_MASK: u64 = (1 << 12) - 1;

#[derive(Debug, Clone, Copy)]
struct Laddr {
    low: u64,
    high: u64,
}

impl Laddr {
    fn pool(&self) -> u8 {
        ((self.high & HIGH_POOL_MASK) >> 56) as u8
    }
    fn shard(&self) -> u8 {
        ((self.high & HIGH_SHARD_MASK) >> 48) as u8
    }
    fn crush(&self) -> u32 {
        ((self.high & HIGH_CRUSH_MASK) >> 16) as u32
    }
    fn random(&self, has_shadow: bool) -> u64 {
        if has_shadow {
            ((self.high & HIGH_RANDOM_MASK) << 17)
                | ((self.low & has_shadow::LOW_RANDOM_MASK) >> 47)
        } else {
            ((self.high & HIGH_RANDOM_MASK) << 18) | ((self.low & no_shadow::LOW_RANDOM_MASK) >> 46)
        }
    }
    fn shadow(&self) -> bool {
        (self.low & has_shadow::LOW_SHADOW_MASK) != 0
    }
    fn with_shadow(&self, shadow: bool) -> Laddr {
        let low = if shadow {
            self.low | has_shadow::LOW_SHADOW_MASK
        } else {
            self.low & !has_shadow::LOW_SHADOW_MASK
        };
        Laddr { low, ..*self }
    }
    fn metadata(&self) -> bool {
        (self.low & LOW_METADATA_MASK) != 0
    }
    fn with_metadata(&self, metadata: bool) -> Laddr {
        let low = if metadata {
            self.low | LOW_METADATA_MASK
        } else {
            self.low & !LOW_METADATA_MASK
        };
        Laddr { low, ..*self }
    }
    fn snap(&self) -> bool {
        (self.low & LOW_SNAP_MASK) != 0
    }
    fn with_snap(&self, snap: bool) -> Laddr {
        let low = if snap {
            self.low | LOW_SNAP_MASK
        } else {
            self.low & !LOW_SNAP_MASK
        };
        Laddr { low, ..*self }
    }
    fn local_snap_id(&self) -> u32 {
        ((self.low & LOW_LOCAL_SNAP_ID_MASK) >> 12) as u32
    }
    fn offset(&self) -> u32 {
        ((self.low & LOW_OFFSET_MASK) as u32) << 12
    }
    fn get_object_prefix(&self, has_shadow: bool) -> Laddr {
        let low = if has_shadow {
            self.low & has_shadow::LOW_RANDOM_MASK
        } else {
            self.low & no_shadow::LOW_RANDOM_MASK
        };
        Self { low, ..*self }
    }
    fn get_onode_prefix(&self) -> Laddr {
        Laddr {
            low: self.low & !LOW_OFFSET_MASK,
            high: self.high,
        }
    }
}

impl From<&str> for Laddr {
    fn from(s: &str) -> Self {
        if s.starts_with("0x") {
            let u = u128::from_str_radix(&s[2..], 16).unwrap();
            let low = u as u64;
            let high = (u >> 64) as u64;
            Self { low, high }
        } else {
            // low = 1234, high = 5678
            let re = Regex::new(r"low = (\d+), high = (\d+)").unwrap();
            let caps = re.captures(s).unwrap();
            let low: u64 = caps[1].parse().unwrap();
            let high: u64 = caps[2].parse().unwrap();
            Self { low, high }
        }
    }
}

fn fmt_laddr(laddr: &Laddr) -> String {
    format!("{:#x}{:016x}", laddr.high, laddr.low)
}

#[derive(Tabled)]
struct LaddrRow {
    property: String,
    value: String,
}

impl LaddrRow {
    fn new(property: &str, value: String) -> Self {
        Self {
            property: property.to_string(),
            value: value,
        }
    }
}

fn build_laddr_rows(laddr: Laddr) -> Vec<LaddrRow> {
    vec![
        LaddrRow::new("literal", fmt_laddr(&laddr)),
        LaddrRow::new(
            "object_prefix_no_shadow",
            fmt_laddr(&laddr.get_object_prefix(false)),
        ),
        LaddrRow::new(
            "object_prefix_has_shadow",
            fmt_laddr(&laddr.get_object_prefix(true)),
        ),
        LaddrRow::new("onode_prefix", fmt_laddr(&laddr.get_onode_prefix())),
        LaddrRow::new("with_shadow", fmt_laddr(&laddr.with_shadow(true))),
        LaddrRow::new("without_shadow", fmt_laddr(&laddr.with_shadow(false))),
        LaddrRow::new("with_metadata", fmt_laddr(&laddr.with_metadata(true))),
        LaddrRow::new("without_metadata", fmt_laddr(&laddr.with_metadata(false))),
        LaddrRow::new("with_snap", fmt_laddr(&laddr.with_snap(true))),
        LaddrRow::new("without_snap", fmt_laddr(&laddr.with_snap(false))),
        LaddrRow::new("pool", laddr.pool().to_string()),
        LaddrRow::new("shard", laddr.shard().to_string()),
        LaddrRow::new("crush", format!("{:#x}", laddr.crush())),
        LaddrRow::new("random_no_shadow", format!("{:#x}", laddr.random(false))),
        LaddrRow::new("random_has_shadow", format!("{:#x}", laddr.random(true))),
        LaddrRow::new("shadow", laddr.shadow().to_string()),
        LaddrRow::new("metadata", laddr.metadata().to_string()),
        LaddrRow::new("snap", laddr.snap().to_string()),
        LaddrRow::new("local_snap_id", laddr.local_snap_id().to_string()),
        LaddrRow::new("offset", laddr.offset().to_string()),
    ]
}

fn main() {
    print!("Enter laddr_t: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let laddr = Laddr::from(input.trim());
    let v = build_laddr_rows(laddr);
    let mut t = Table::new(v);
    t.with(Style::sharp());
    println!("{}", t.to_string());
}
