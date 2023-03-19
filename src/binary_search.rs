use std::cmp::Ordering;

pub fn bisect_left<F>(mut left: u64, mut right: u64, mut f: F) -> u64
where
    F: FnMut(u64) -> Ordering,
{
    while left < right {
        let center = (left + right) / 2;
        match f(center) {
            Ordering::Less => left = center + 1,
            _ => right = center,
        }
    }
    left
}

pub fn bisect_right<F>(mut left: u64, mut right: u64, mut f: F) -> u64
where
    F: FnMut(u64) -> Ordering,
{
    while left < right {
        let center = (left + right) / 2;
        match f(center) {
            Ordering::Less | Ordering::Equal => left = center + 1,
            _ => right = center,
        }
    }
    left
}
