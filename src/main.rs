use std::io::Write;

use tabled::{settings::Style, Table, Tabled};

struct FieldSpec {
    #[allow(dead_code)]
    name: &'static str,
    offset: usize,
    length: usize,
}

impl FieldSpec {
    const fn new(name: &'static str, length: usize, offset: usize) -> Self {
        Self {
            name,
            offset,
            length,
        }
    }
    const fn mask(&self) -> u128 {
        ((1u128 << self.length) - 1) << self.offset
    }
    const fn get(&self, value: u128) -> u128 {
        (value & self.mask()) >> self.offset
    }
    #[allow(dead_code)]
    fn set(&self, value: u128, field: u128) -> u128 {
        (value & !self.mask()) | ((field << self.offset) & self.mask())
    }
}

const UPGRADE_SPEC: FieldSpec = FieldSpec::new("upgrade", 1, 127);
const OBJECT_INFO_SPEC: FieldSpec = FieldSpec::new("object_info", 76, 51);
const OBJECT_CONTENT_SPEC: FieldSpec = FieldSpec::new("object_content", 51, 0);

const SHARD_SPEC: FieldSpec = FieldSpec::new("shard", 6, 121);
const POOL_SPEC: FieldSpec = FieldSpec::new("pool", 12, 109);
const REVERSE_HASH_SPEC: FieldSpec = FieldSpec::new("reverse_hash", 16, 93);
const LOCAL_OBJECT_ID_SPEC: FieldSpec = FieldSpec::new("local_object_id", 42, 51);
const LOCAL_CLONE_ID_SPEC: FieldSpec = FieldSpec::new("local_clone_id", 23, 28);
const IS_METADATA_SPEC: FieldSpec = FieldSpec::new("is_metadata", 1, 27);
const BLOCK_OFFSET_SPEC: FieldSpec = FieldSpec::new("block_offset", 27, 0);

const PREFIX_MASK: u128 = UPGRADE_SPEC.mask() | OBJECT_INFO_SPEC.mask();

fn parse_laddr(s: &str) -> u128 {
    let start = if s.starts_with("L") {
        1
    } else if s.starts_with("0x") {
        2
    } else {
        0
    };
    u128::from_str_radix(&s[start..], 16).unwrap()
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
            value,
        }
    }
}

fn build_laddr_rows(laddr: u128) -> Vec<LaddrRow> {
    if laddr & PREFIX_MASK == 0 {
        return vec![
            LaddrRow::new("laddr", format!("{:x}", laddr)),
            LaddrRow::new(
                "object_content",
                format!("{:x}", OBJECT_CONTENT_SPEC.get(laddr)),
            ),
        ];
    }
    vec![
        LaddrRow::new("laddr", format!("{:x}", laddr)),
        LaddrRow::new("prefix", format!("{:x}", laddr & PREFIX_MASK)),
        LaddrRow::new("upgrade", format!("{}", UPGRADE_SPEC.get(laddr) == 1)),
        LaddrRow::new("shard", format!("{}", SHARD_SPEC.get(laddr))),
        LaddrRow::new("pool", format!("{}", POOL_SPEC.get(laddr))),
        LaddrRow::new(
            "reverse_hash",
            format!("{:x}", REVERSE_HASH_SPEC.get(laddr)),
        ),
        LaddrRow::new(
            "local_object_id",
            format!("{}", LOCAL_OBJECT_ID_SPEC.get(laddr)),
        ),
        LaddrRow::new(
            "local_clone_id",
            format!("{}", LOCAL_CLONE_ID_SPEC.get(laddr)),
        ),
        LaddrRow::new(
            "is_metadata",
            format!("{}", IS_METADATA_SPEC.get(laddr) == 1),
        ),
        LaddrRow::new(
            "offset_16",
            format!(
                "{:x}(*4096={:x})",
                BLOCK_OFFSET_SPEC.get(laddr),
                BLOCK_OFFSET_SPEC.get(laddr) * 4096
            ),
        ),
        LaddrRow::new(
            "offset_10",
            format!(
                "{}(*4096={})",
                BLOCK_OFFSET_SPEC.get(laddr),
                BLOCK_OFFSET_SPEC.get(laddr) * 4096
            ),
        ),
    ]
}

fn main() {
    print!("Enter laddr_t: ");
    std::io::stdout().flush().unwrap();

    let laddr = {
        let mut laddr = String::new();
        std::io::stdin().read_line(&mut laddr).unwrap();
        if laddr.trim().is_empty() {
            return;
        }
        parse_laddr(laddr.trim())
    };

    let v = build_laddr_rows(laddr);
    let mut t = Table::new(v);
    t.with(Style::sharp());
    println!("{}", t.to_string());
}
