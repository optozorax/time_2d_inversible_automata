#![allow(clippy::unusual_byte_groupings)]

use std::collections::HashMap;
use std::collections::BTreeSet;
use time_2d_inversible_automata::*;
use permutation_string::*;

const PAT0: u64 = repeat_bit(0b000, 3);
const PAT1: u64 = repeat_bit(0b001, 3);
const PAT2: u64 = repeat_bit(0b010, 3);
const PAT3: u64 = repeat_bit(0b011, 3);
const PAT4: u64 = repeat_bit(0b100, 3);
const PAT5: u64 = repeat_bit(0b101, 3);
const PAT6: u64 = repeat_bit(0b110, 3);
const PAT7: u64 = repeat_bit(0b111, 3);

const ALL_ONES: u64 = repeat_bit(0b1, 1);

const fn replace(x: u64, pat1: u64, pat2: u64) -> u64 {
    const BITS_FIRST: u64 = repeat_bit(0b001, 3);
    let x1 = !(x ^ pat1);
    let x2 = x1 & (x1 >> 1) & (x1 >> 2) & BITS_FIRST;
    let x3 = x2 | (x2 << 1) | (x2 << 2);
    x3 & pat2
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Rule([u64; 8]);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Field {
    val: u64,
    size: u32,
}


impl Rule {
    pub const RULE0: Rule = Rule([PAT0, PAT1, PAT2, PAT3, PAT4, PAT5, PAT6, PAT7]);

    pub fn new(x: [u64; 8]) -> Self {
        Self(x)
    }

    pub fn num_to_rule(ni: u64) -> Rule {
        let p = PermutationInt::new(ni, 8);
        let p = PermutationIndex::try_from(p).unwrap();
        let p = PermutationArray::try_from(p).unwrap();
        let p: [usize; 8] = p.0.try_into().unwrap();
        Rule(p.map(|x| Self::RULE0.0[x]))
    }

    pub fn num_to_rule_name(ni: u64) -> String {
        let p = PermutationInt::new(ni, 8);
        let p = PermutationIndex::try_from(p).unwrap();
        let p = PermutationArray::try_from(p).unwrap();
        let p: [usize; 8] = p.0.try_into().unwrap();
        let mut res = String::new();
        for i in p {
            res.push(char::from(b'0' + i as u8));
        }
        res
    }

    pub fn replace_all(&self, mut x: &mut Field) {
        let &Rule([r0, r1, r2, r3, r4, r5, r6, r7]) = self;
        x.val = replace(x.val, PAT0, r0)
            | replace(x.val, PAT1, r1)
            | replace(x.val, PAT2, r2)
            | replace(x.val, PAT3, r3)
            | replace(x.val, PAT4, r4)
            | replace(x.val, PAT5, r5)
            | replace(x.val, PAT6, r6)
            | replace(x.val, PAT7, r7);
    }

    pub fn step0(&self, x: &mut Field) {
        self.replace_all(x);
    }

    pub fn step1(&self, x: &mut Field) {
        *x = x.rotate_right(1);
        self.replace_all(x);
        *x = x.rotate_left(1);
    }

    pub fn step2(&self, x: &mut Field) {
        *x = x.rotate_right(1);
        *x = x.rotate_right(1);
        self.replace_all(x);
        *x = x.rotate_left(1);
        *x = x.rotate_left(1);
    }

    pub fn steps(&self, x: &mut Field) {
        self.replace_all(x);
        *x = x.rotate_right(1);
        self.replace_all(x);
        *x = x.rotate_right(1);
        self.replace_all(x);
        *x = x.rotate_left(1);
        *x = x.rotate_left(1);
    }

    pub fn show_field(&self, mut x: Field, steps: u64, dupl: u64, ni: u64) -> String {
        let x_start = x.val;
        let mut result = vec![x.to_bools(dupl)];
        for _ in 0..steps / 3 {
            self.step0(&mut x);
            result.push(x.to_bools(dupl));
            self.step1(&mut x);
            result.push(x.to_bools(dupl));
            self.step2(&mut x);
            result.push(x.to_bools(dupl));
        }
        let name = format!("img/{}_{}_{}_{}_{}.png", x_start, x.size, steps, dupl, ni);
        draw_image(&name, result);
        name
    }
}

impl Field {
    pub fn new(val: u64, size: u32) -> Self {
        assert!(size % 3 == 0);
        assert!(val == (val & !(ALL_ONES << size)));
        Self {
            val,
            size,
        }
    }

    pub fn rotate_left(mut self, mut count: u32) -> Self {
        count %= self.size;
        self.val = ((self.val << count)
            | ((self.val >> (self.size - count)) & (PAT7 >> (self.size - count))))
            & !(ALL_ONES << self.size);
        self
    }

    pub fn rotate_right(mut self, mut count: u32) -> Self {
        count %= self.size;
        self.val = ((self.val >> count)
            | ((self.val & (PAT7 >> (self.size - count))) << (self.size - count)))
            & !(ALL_ONES << self.size);
        self
    }

    pub fn to_bools(&self, dupl: u64) -> Vec<bool> {
        let size = self.size as usize;
        let mut result = vec![false; size];
        for i in 0..size {
            result[size - i - 1] = ((self.val >> i) & 1) != 0;
        }
        let mut result2 = Vec::new();
        for _ in 0..dupl {
            result2.extend(result.iter());
        }
        result2
    }

    pub fn occupied_size(&self) -> u32 {
        occupied_size(self.val)
    }

    pub fn minimize(mut self) -> Self {
        let mut my_min = self.val;
        for i in 0..=self.size / 3 {
            my_min = my_min.min(self.rotate_left(i * 3).val);
        }
        self.val = my_min;
        self
    }

    pub fn centralize(self) -> Self {
        let mut offset = (self.size - self.occupied_size()) / 2;
        offset -= offset % 3;
        self.rotate_left(offset)
    }

    pub fn index(&self, pos: u32) -> bool {
        ((self.val >> pos) & 1) != 0
    }
}

pub fn occupied_size(mut x: u64) -> u32 {
    let mut count = 0;
    while x != 0 {
        x >>= 3;
        count += 3;
    }
    count
}

pub fn print_col(ni: u64, img: &str, info: &str) {
    println!("<div class=\"automata-col\">",);
    println!("<span class=\"automata-name\"><b>{}</b></span><br>", ni);
    println!("<img class=\"pixelated\" src=\"{}\">", img);
    println!("<br><span class=\"automata-name\">{}</span>", info);
    println!("</div>");
}

// находит период и смещение глайдера, осциллятора или статичной картинки
fn period(mut x: Field, rule: &Rule) -> Option<(u64, i32)> {
    let y = x;
    for period in 1..60 {
        rule.steps(&mut x);
        for offset in 0..3 {
            if x.rotate_left(offset * 3) == y {
                return Some((period, -(offset as i32)));
            }
            if x.rotate_right(offset * 3) == y {
                return Some((period, (offset as i32)));
            }
        }
    }
    None
}

// находит минимальное число описывающее глайдер
fn minimize2(mut x: Field, rule: &Rule, period: u64) -> Field {
    let mut my_min = x.val;
    for _ in 0..period {
        rule.steps(&mut x);
        x = x.minimize();
        my_min = my_min.min(x.val);
    }
    Field::new(my_min, x.size)
}

fn max_consecutive_zeros(mut x: u64) -> u32 {
    let mut max = 0;
    let mut count = 0;
    while x != 0 {
        if x & 1 == 1 {
            max = max.max(count);
            count = 0;
        } else {
            count += 1;
        }
        x >>= 1;
    }
    max
}

fn size_round3(x: u32) -> u32 {
    x / 3 + (((x % 3) != 0) as u32)
}

fn check_reach_everything(mut x: Field, rule: &Rule, period: u64) -> bool {
    let pos_start = (|| {
        for i in 0..x.size {
            if x.index(i) {
                return i;
            }
        }
        unreachable!()
    })();

    let mut reached = Field::new(1 << pos_start, x.size);

    let mut prev_x = x;

    for step in 0..=period * 3 {
        let mut reached2 = Field::new(0, x.size);

        let pos_start = (|| {
            for i in 0..x.size {
                if !x.index(i) {
                    return i;
                }
            }
            unreachable!()
        })();

        let mut state = 0;
        let mut current = 0;
        let mut used = false;
        for i in (0..x.size).map(|i| (pos_start + i) % x.size) {
            if x.index(i) {
                if state == 0 {
                    state = 1;
                    used = reached.index(i);
                    current = 1 << i;
                } else {
                    used |= reached.index(i);
                    current |= 1 << i;
                }
            } else if state == 1 {
                state = 0;
            }
            if used {
                reached2.val |= current;
            }
        }

        reached = reached2;

        prev_x = x;

        if step % 3 == 0 {
            rule.step0(&mut x);
        } else if step % 3 == 1 {
            rule.step1(&mut x);
        } else {
            rule.step2(&mut x);
        }
    }

    reached.val == prev_x.val
}

fn add_used(mut x: Field, period: u64, offset: i32, rule: &Rule, used: &mut HashMap<u64, (u64, i32)>) {
    for _ in 0..period {
        assert!(!used.contains_key(&x.val));
        used.insert(x.val, (period, offset));
        rule.steps(&mut x)
    }
/* 
    сначала находим максимальную штуку из 000, находим её старт и длину, соответственно мы можем найти старт и длину всего остального
    записываем это, и записываем следующее состояние, и записываем смещение относительного прошлого состояния
 */
}

// fn check_used(mut x: Field, period: u64, rule: &Rule, used: &mut HashMap<u64, (u64, i32)>) {
//     // let size = size_round3(occupied_size(x));
//     // for i in 1..=size {
// TOOD
/* 
    
    перебираем все размеры текущего паттерна
    смотрим, если паттерн совпадает с тем что уже использовалось
    если да, то перебираем все паттерны и смотрим, чтобы они были равны всем следующим с учётом смещения, использовать xor и rotate для паттерн матчинга
    если мы уже прошли до конца периода и такое случилось, то всё, текующий глайдер обладает другим глайдером, он не самостоятельный
 */
//     // }
// }

fn main() {
    println!("{}", BEFORE);
    for ni in 633..634 {
        // let ni = 633;
        let rule = Rule::num_to_rule(ni);

        let mut gliders = BTreeSet::new();
        // let mut used: HashMap<u64, (u64, i32)> = HashMap::new(); // val, (period, offset)
        for x in 1..100_000 {
            let size = size_round3(occupied_size(x));
            if let Some((period1, offset1)) = period(Field::new(x, (size + 2) * 3), &rule) {
                if let Some((period2, offset2)) = period(Field::new(x, (size + 3) * 3), &rule) {
                    if period1 == period2 && offset1 == offset2 {
                        let add_size = size_round3(max_consecutive_zeros(x));
                        let min = minimize2(Field::new(x, (size + add_size + 1) * 3), &rule, period1);

                        if check_reach_everything(min, &rule, period1) {
                            // add_used(min, period1, offset1, &rule, &mut used);
                            gliders.insert((min.val, period1, offset1));    
                        }
                    }
                }
            }
        }

        println!("{}", CONTAINER_START);
        for (x, period, offset) in gliders {
            if offset != 0 {
                let field = Field::new(x, 63);
                print_col(
                    ni,
                    &rule.show_field(field.centralize(), period * 3, 1, ni),
                    &format!("n{} p{} o{}", x, period, offset),
                );
            }
        }
        println!("{}", CONTAINER_END);
    }
    println!("{}", AFTER);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        color_backtrace::install();

        let rule1 = Rule::new([PAT0, PAT1, PAT2, PAT3, PAT4, PAT5, PAT7, PAT6]);

        let mut x = Field::new(0b001_110_111, 63);
        rule1.replace_all(&mut x);
        assert_eq!(x, Field::new(0b001_111_110, 63));

        let rule2 = Rule::new([PAT0, PAT2, PAT1, PAT4, PAT3, PAT6, PAT5, PAT7]);
        let mut x = Field::new(0b001_010_011_100_101_110_111, 63);
        rule2.replace_all(&mut x);
        assert_eq!(x, Field::new(0b010_001_100_011_110_101_111, 63));

        let x = Field::new(repeat_bit(0b01, 2), 63);
        assert_eq!(x, x.rotate_right(1).rotate_left(1));
        assert_eq!(x, x.rotate_left(1).rotate_right(1));

        let mut random = Random::new(25025069121927896);
        let start = Field::new(random.get() & !(ALL_ONES << 45), 45);
        for i in 0..100 {
            assert_eq!(start.rotate_left(i), {
                let mut x = start;
                for _ in 0..i {
                    x = x.rotate_left(1);
                }
                x
            });
            assert_eq!(start.rotate_right(i), {
                let mut x = start;
                for _ in 0..i {
                    x = x.rotate_right(1);
                }
                x
            });
        }

        // assert_eq!(Field::new(0b0101, 4).rotate_left(0), Field::new(0b0101, 4));
        // assert_eq!(Field::new(0b0101, 4).rotate_left(1), Field::new(0b1010, 4));
        // assert_eq!(Field::new(0b0101, 4).rotate_left(2), Field::new(0b0101, 4));
        // assert_eq!(Field::new(0b0101, 4).rotate_left(3), Field::new(0b1010, 4));
        // assert_eq!(Field::new(0b0101, 4).rotate_left(4), Field::new(0b0101, 4));

        // assert_eq!(Field::new(0b1110, 4).rotate_left(0), Field::new(0b1110, 4));
        // assert_eq!(Field::new(0b1110, 4).rotate_left(1), Field::new(0b1101, 4));
        // assert_eq!(Field::new(0b1110, 4).rotate_left(2), Field::new(0b1011, 4));
        // assert_eq!(Field::new(0b1110, 4).rotate_left(3), Field::new(0b0111, 4));
        // assert_eq!(Field::new(0b1110, 4).rotate_left(4), Field::new(0b1110, 4));

        let rule = Rule::num_to_rule(633);
        assert!(check_reach_everything(Field::new(5, 30), &rule, 4));
        assert!(check_reach_everything(Field::new(10, 30), &rule, 2));
        assert!(check_reach_everything(Field::new(45, 30), &rule, 13));
        assert!(!check_reach_everything(Field::new(165, 30), &rule, 4));
        assert!(!check_reach_everything(Field::new(175, 30), &rule, 4));
        assert!(!check_reach_everything(Field::new(325, 30), &rule, 4));
        assert!(!check_reach_everything(Field::new(490, 30), &rule, 2));
        assert!(!check_reach_everything(Field::new(650, 30), &rule, 2));
        assert!(check_reach_everything(Field::new(765, 30), &rule, 13));
        assert!(!check_reach_everything(Field::new(965, 30), &rule, 4));
        assert!(check_reach_everything(Field::new(1215, 30), &rule, 28));
        assert!(!check_reach_everything(Field::new(1285, 30), &rule, 4));
        assert!(!check_reach_everything(Field::new(2565, 30), &rule, 4));
        assert!(!check_reach_everything(Field::new(5610, 30), &rule, 2));
        assert!(check_reach_everything(Field::new(6125, 30), &rule, 13));
        assert!(check_reach_everything(Field::new(11755, 30), &rule, 43));
        assert!(check_reach_everything(Field::new(13805, 30), &rule, 43));
        assert!(!check_reach_everything(Field::new(23275, 30), &rule, 13));
        assert!(!check_reach_everything(Field::new(390650, 30), &rule, 13));
    }
}

const BEFORE: &str = "
<style>
.container { 
  display: flex; 
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: space-around;
  width: 90%;
  border: 1px solid gray;
  padding: 10px;
  margin: 5px;
}
.automata-col {
  border: 1px solid rgba(39,41,43,0.1); 
  background-color: rgba(39,41,43,0.03);
  padding: 10px;
  margin: 5px;
  min-width: 110px;
  /*flex: 1 0 0%;*/
}
.pixelated {
  -ms-interpolation-mode: nearest-neighbor;
  image-rendering: crisp-edges;
  image-rendering: pixelated;
}

.skip-img {
  display: none;
}

.skip-img, .both-img, .any-img {
  width: 150px;
  margin: 2px;
  border-radius: 0px;
}
.automata-name {
  font-size: 12pt;
  font-family: monospace;
}
.svg {
}
</style>

<center>
";

const CONTAINER_START: &str = "<div class=\"container\">";
const CONTAINER_END: &str = "</div>";

const AFTER: &str = "
</center>
";
