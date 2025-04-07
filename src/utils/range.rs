use std::ops::Range;

pub fn ranges_overlap(a: Range<u16>, b: Range<u16>) -> bool {
    a.end >= b.start && b.end >= a.start
}
