use std::ops::Range;

pub fn ranges_overlap(a: Range<u16>, b: Range<u16>) -> bool {
    u16::min(a.start, b.start) <= u16::min(a.end, b.end)
}
