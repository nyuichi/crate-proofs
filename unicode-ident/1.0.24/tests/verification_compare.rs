#[path = "tables/mod.rs"]
#[allow(dead_code)]
mod tables;

fn in_ranges(codepoint: u32, ranges: &[(u32, u32)]) -> bool {
    let mut low = 0;
    let mut high = ranges.len();
    while low < high {
        let middle = low + (high - low) / 2;
        let (start, end) = ranges[middle];
        if codepoint < start {
            high = middle;
        } else if codepoint > end {
            low = middle + 1;
        } else {
            return true;
        }
    }
    false
}

#[test]
fn compressed_tables_match_unicode_17_ranges() {
    for codepoint in 0..=char::MAX as u32 {
        let Some(ch) = char::from_u32(codepoint) else {
            continue;
        };
        assert_eq!(
            unicode_ident::is_xid_start(ch),
            in_ranges(codepoint, tables::XID_START),
            "XID_Start mismatch at U+{codepoint:04X}",
        );
        assert_eq!(
            unicode_ident::is_xid_continue(ch),
            in_ranges(codepoint, tables::XID_CONTINUE),
            "XID_Continue mismatch at U+{codepoint:04X}",
        );
    }
}
