use std::ops::{Add, Range};

pub(crate) fn offset_range<T>(range: Range<T>, offset: T) -> Range<T>
where
    T: Add<Output = T> + Copy,
{
    range.start + offset..range.end + offset
}
