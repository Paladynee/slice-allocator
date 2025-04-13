/// Calculates which address after `base` is valid for a type of alignment `align` to exist.
#[inline]
#[must_use]
pub const fn next_aligned_addr(base: usize, align: usize) -> usize {
    let total = base.next_multiple_of(align);

    // let's make sure our result is really aligned:
    assert!(total % align == 0, "next_aligned_addr is not aligned");

    total
}
