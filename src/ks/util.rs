// Copyright 2022 by Daniel Winkelman. All rights reserved.

use std::collections::BTreeMap;

pub fn popcnt64(x: u64) -> usize {
    let mut output = x;
    output = (output & 0x5555555555555555) + ((output & 0xaaaaaaaaaaaaaaaa) >> 1);
    output = (output & 0x3333333333333333) + ((output & 0xcccccccccccccccc) >> 2);
    output = (output & 0x0f0f0f0f0f0f0f0f) + ((output & 0xf0f0f0f0f0f0f0f0) >> 4);
    output = (output & 0x00ff00ff00ff00ff) + ((output & 0xff00ff00ff00ff00) >> 8);
    output = (output & 0x0000ffff0000ffff) + ((output & 0xffff0000ffff0000) >> 16);
    output = (output & 0x00000000ffffffff) + ((output & 0xffffffff00000000) >> 32);
    output as usize
}

pub fn onehot(x: u64) -> Option<usize> {
    (popcnt64(x) == 1).then(|| {
        let mut output = 0;
        let mut scaling = 32;
        while x >> output != 1 && scaling != 0 {
            if x >> (output + scaling) != 0 {
                output += scaling
            }
            scaling >>= 1;
        }
        output as usize
    })
}

#[allow(unused)]
pub fn get_population_distribution<T>(
    data: &mut dyn Iterator<Item = &T>,
    size_fn: fn(&T) -> usize,
) -> BTreeMap<usize, usize> {
    let mut size_map = BTreeMap::new();
    for size in data.map(size_fn) {
        *size_map.entry(size).or_insert(0) += 1;
    }
    size_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popcnt() {
        assert_eq!(popcnt64(0x0123456789abcdef), 32);
        assert_eq!(popcnt64(0xfedcba9876543210), 32);
    }

    #[test]
    fn test_onehot() {
        assert_eq!(onehot(0x3), None);
        for i in 0..64 {
            assert_eq!(onehot(1 << i), Some(i));
        }
    }
}
