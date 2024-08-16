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

// [upgrade:1][pool:12][shard:8][reverse_hash:32][local_object_id:27][is_metadata:1][object_content:47]
const UPGRADE_SPEC: FieldSpec = FieldSpec::new("upgrade", 1, 127);
const POOL_SPEC: FieldSpec = FieldSpec::new("pool", 12, 115);
const SHARD_SPEC: FieldSpec = FieldSpec::new("shard", 8, 107);
const REVERSE_HASH_SPEC: FieldSpec = FieldSpec::new("reverse_hash", 32, 75);
const LOCAL_OBJECT_ID_SPEC: FieldSpec = FieldSpec::new("local_object_id", 27, 48);
const IS_METADATA_SPEC: FieldSpec = FieldSpec::new("is_metadata", 1, 47);
const OBJECT_CONTENT_SPEC: FieldSpec = FieldSpec::new("object_content", 47, 0);

const OBJECT_INFO_MASK: u128 =
    POOL_SPEC.mask() | SHARD_SPEC.mask() | REVERSE_HASH_SPEC.mask() | LOCAL_OBJECT_ID_SPEC.mask();
const PREFIX_MASK: u128 = UPGRADE_SPEC.mask() | OBJECT_INFO_MASK;

fn parse_laddr(s: &str) -> u128 {
    if s.starts_with("L") {
        u128::from_str_radix(&s[1..], 16).unwrap()
    } else {
        u128::from_str_radix(&s[..], 16).unwrap()
    }
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

fn build_laddr_rows(laddr: u128, offset_bits: usize) -> Vec<LaddrRow> {
    let local_clone_id_spec = FieldSpec::new(
        "local_clone_id",
        OBJECT_CONTENT_SPEC.length - offset_bits,
        offset_bits,
    );
    let offset_spec = FieldSpec::new("offset", offset_bits, 0);
    vec![
        LaddrRow::new("laddr", format!("{:x}", laddr)),
        LaddrRow::new("prefix", format!("{:x}", laddr & PREFIX_MASK)),
        LaddrRow::new("upgrade", format!("{}", UPGRADE_SPEC.get(laddr) == 1)),
        LaddrRow::new("pool", format!("{}", POOL_SPEC.get(laddr))),
        LaddrRow::new("shard", format!("{}", SHARD_SPEC.get(laddr))),
        LaddrRow::new(
            "reverse_hash",
            format!("{:x}", REVERSE_HASH_SPEC.get(laddr)),
        ),
        LaddrRow::new(
            "local_object_id",
            format!("{}", LOCAL_OBJECT_ID_SPEC.get(laddr)),
        ),
        LaddrRow::new(
            "is_metadata",
            format!("{}", IS_METADATA_SPEC.get(laddr) == 1),
        ),
        LaddrRow::new(
            "object_content",
            format!("{:x}", OBJECT_CONTENT_SPEC.get(laddr)),
        ),
        LaddrRow::new(
            "local_clone_id",
            format!("{:x}", local_clone_id_spec.get(laddr)),
        ),
        LaddrRow::new(
            "offset_16",
            format!(
                "{:x}(*4096={:x})",
                offset_spec.get(laddr),
                offset_spec.get(laddr) * 4096
            ),
        ),
        LaddrRow::new(
            "offset",
            format!(
                "{}(*4096={})",
                offset_spec.get(laddr),
                offset_spec.get(laddr) * 4096
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

    print!("Enter offset bits(default 15): ");
    std::io::stdout().flush().unwrap();

    let offset_bits = {
        let mut offset_bits = String::new();
        std::io::stdin().read_line(&mut offset_bits).unwrap();
        if !offset_bits.trim().is_empty() {
            offset_bits.trim().parse::<usize>().unwrap()
        } else {
            15
        }
    };

    let v = build_laddr_rows(laddr, offset_bits);
    let mut t = Table::new(v);
    t.with(Style::sharp());
    println!("{}", t.to_string());
}
