// заготовка для вычисления правил с размером окна 3

const fn repeat(num: u64, size: u8) -> u64 {
    let mut ans = 0u64;
    let mut i = 0;
    let max = 64 / size;
    while i < max {
        ans |= num << (i * size);
        i += 1;
    }
    ans
}

const PAT0: u64 = repeat(0b000, 3);
const PAT1: u64 = repeat(0b001, 3);
const PAT2: u64 = repeat(0b010, 3);
const PAT3: u64 = repeat(0b011, 3);
const PAT4: u64 = repeat(0b100, 3);
const PAT5: u64 = repeat(0b101, 3);
const PAT6: u64 = repeat(0b110, 3);
const PAT7: u64 = repeat(0b111, 3);

const fn replace(x: u64, pat1: u64, pat2: u64) -> u64 {
    const BITS_FIRST: u64 = repeat(0b001, 3);
    let x1 = !(x ^ pat1);
    let x2 = x1 & (x1 >> 1) & (x1 >> 2) & BITS_FIRST;
    let x3 = x2 | (x2 << 1) | (x2 << 2);
    x3 & pat2
}

pub struct Rule([u64; 8]);

pub fn replace_all(
    x: u64, 
    Rule([r0, r1, r2, r3, r4, r5, r6, r7]): Rule,
) -> u64 {
    replace(x, PAT0, r0) | 
        replace(x, PAT1, r1) | 
        replace(x, PAT2, r2) | 
        replace(x, PAT3, r3) | 
        replace(x, PAT4, r4) | 
        replace(x, PAT5, r5) | 
        replace(x, PAT6, r6) | 
        replace(x, PAT7, r7)
}
