//! Bounded independent presentation envelope. No gameplay state or native layouts.
use std::collections::{HashMap, VecDeque};
pub const MAX_TEXT: usize = 131072;
pub const MAX_PACKED: usize = 16384;
/// Version 2: u8 tag, little-endian u32 output size, then groups of eight
/// flags (low bit first). 0=literal; 1=LE u16 (length-3 high 4, distance-1 low 12).
pub fn pack(input: &[u8]) -> Option<Vec<u8>> {
    if input.is_empty() || input.len() > MAX_TEXT {
        return None;
    }
    let mut out = vec![2];
    out.extend((input.len() as u32).to_le_bytes());
    let mut positions: HashMap<[u8; 3], VecDeque<usize>> = HashMap::new();
    let mut at = 0;
    while at < input.len() {
        let flags = out.len();
        out.push(0);
        for bit in 0..8 {
            if at == input.len() {
                break;
            }
            let mut best = (0, 0);
            if at + 3 <= input.len() {
                if let Some(rows) = positions.get(&input[at..at + 3]) {
                    for &previous in rows.iter().rev().take(64) {
                        let distance = at - previous;
                        if distance > 4096 {
                            break;
                        }
                        let mut length = 0;
                        while length < 18
                            && at + length < input.len()
                            && input[previous + length] == input[at + length]
                        {
                            length += 1
                        }
                        if length > best.0 && length >= 3 {
                            best = (length, distance)
                        }
                    }
                }
            }
            let length = if best.0 >= 3 {
                out[flags] |= 1 << bit;
                out.extend((((best.0 - 3) as u16) << 12 | (best.1 - 1) as u16).to_le_bytes());
                best.0
            } else {
                out.push(input[at]);
                1
            };
            for p in at..at + length {
                if p + 3 <= input.len() {
                    let row = positions
                        .entry(input[p..p + 3].try_into().unwrap())
                        .or_default();
                    while row.front().is_some_and(|old| p - *old > 4096) || row.len() >= 64 {
                        row.pop_front();
                    }
                    row.push_back(p);
                }
            }
            at += length;
        }
        if out.len() > MAX_PACKED {
            return None;
        }
    }
    Some(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn unpack(data: &[u8]) -> Option<Vec<u8>> {
        if data.len() < 5 || data[0] != 2 {
            return None;
        }
        let n = u32::from_le_bytes(data[1..5].try_into().ok()?) as usize;
        if n == 0 || n > MAX_TEXT {
            return None;
        }
        let mut at = 5;
        let mut out = vec![];
        while out.len() < n {
            let flags = *data.get(at)?;
            at += 1;
            for bit in 0..8 {
                if out.len() == n {
                    if flags >> bit != 0 {
                        return None;
                    }
                    break;
                }
                if flags & (1 << bit) == 0 {
                    out.push(*data.get(at)?);
                    at += 1
                } else {
                    let word = u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?);
                    at += 2;
                    let dist = (word & 4095) as usize + 1;
                    let len = (word >> 12) as usize + 3;
                    if dist > out.len() || out.len() + len > n {
                        return None;
                    }
                    for _ in 0..len {
                        out.push(out[out.len() - dist]);
                    }
                }
            }
        }
        (at == data.len()).then_some(out)
    }
    #[test]
    fn repeated_owner_descriptions_and_overlapping_runs_roundtrip() {
        for data in [
            b"abcabcabcabc".to_vec(),
            vec![b'x'; MAX_TEXT],
            "description ".repeat(4000).into_bytes(),
            (0..1000).map(|i| (i % 251) as u8).collect(),
        ] {
            let packed = pack(&data).unwrap();
            assert_eq!(unpack(&packed).unwrap(), data);
            assert!(packed.len() <= MAX_PACKED)
        }
    }
    #[test]
    fn bounded_and_deterministic() {
        assert!(pack(&[]).is_none());
        assert!(pack(&vec![0; MAX_TEXT + 1]).is_none());
        assert_eq!(
            pack(b"abcabcabcabc").unwrap(),
            vec![2, 12, 0, 0, 0, 8, 97, 98, 99, 2, 96]
        );
    }
}
