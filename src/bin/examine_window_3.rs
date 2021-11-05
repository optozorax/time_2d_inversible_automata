#![allow(clippy::unusual_byte_groupings)]
// заготовка для вычисления правил с размером окна 3
use permutation_string::*;
use time_2d_inversible_automata::*;

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

const ALL_ONES: u64 = repeat(0b1, 1);

const fn replace(x: u64, pat1: u64, pat2: u64) -> u64 {
    const BITS_FIRST: u64 = repeat(0b001, 3);
    let x1 = !(x ^ pat1);
    let x2 = x1 & (x1 >> 1) & (x1 >> 2) & BITS_FIRST;
    let x3 = x2 | (x2 << 1) | (x2 << 2);
    x3 & pat2
}

struct Rule([u64; 8]);

fn replace_all(x: u64, &Rule([r0, r1, r2, r3, r4, r5, r6, r7]): &Rule) -> u64 {
    replace(x, PAT0, r0)
        | replace(x, PAT1, r1)
        | replace(x, PAT2, r2)
        | replace(x, PAT3, r3)
        | replace(x, PAT4, r4)
        | replace(x, PAT5, r5)
        | replace(x, PAT6, r6)
        | replace(x, PAT7, r7)
}

fn rotate_left(x: u64, size: u32, mut count: u32) -> u64 {
    count %= size;
    ((x << count) | ((x >> (size - count)) & (PAT7 >> (size - count)))) & !(ALL_ONES << size)
}

fn rotate_right(x: u64, size: u32, mut count: u32) -> u64 {
    count %= size;
    ((x >> count) | ((x & (PAT7 >> (size - count))) << (size - count))) & !(ALL_ONES << size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let rule1 = Rule([PAT0, PAT1, PAT2, PAT3, PAT4, PAT5, PAT7, PAT6]);
        assert_eq!(replace_all(0b001_110_111, &rule1), 0b001_111_110);

        let rule2 = Rule([PAT0, PAT2, PAT1, PAT4, PAT3, PAT6, PAT5, PAT7]);
        assert_eq!(
            replace_all(0b001_010_011_100_101_110_111, &rule2),
            0b010_001_100_011_110_101_111
        );

        let x = repeat(0b01, 2);
        assert_eq!(x, rotate_right(rotate_left(x, 63, 1), 63, 1));
        assert_eq!(x, rotate_left(rotate_right(x, 63, 1), 63, 1));

        let mut random = Random::new(25025069121927896);
        let start = random.get();
        for i in 0..100 {
            assert_eq!(rotate_left(start, 63, i), {
                let mut x = start;
                for _ in 0..i {
                    x = rotate_left(x, 63, 1);
                }
                x
            });
            assert_eq!(rotate_right(start, 63, i), {
                let mut x = start;
                for _ in 0..i {
                    x = rotate_right(x, 63, 1);
                }
                x
            });
        }

        assert_eq!(rotate_left(0b0101, 4, 0), 0b0101);
        assert_eq!(rotate_left(0b0101, 4, 1), 0b1010);
        assert_eq!(rotate_left(0b0101, 4, 2), 0b0101);
        assert_eq!(rotate_left(0b0101, 4, 3), 0b1010);
        assert_eq!(rotate_left(0b0101, 4, 4), 0b0101);

        assert_eq!(rotate_left(0b1110, 4, 0), 0b1110);
        assert_eq!(rotate_left(0b1110, 4, 1), 0b1101);
        assert_eq!(rotate_left(0b1110, 4, 2), 0b1011);
        assert_eq!(rotate_left(0b1110, 4, 3), 0b0111);
        assert_eq!(rotate_left(0b1110, 4, 4), 0b1110);
    }
}

fn print(x: u64) {
    println!(
        ".{}",
        format!("{:063b}", x & !(1 << 63))
            .replace("0", " ")
            .replace("1", "█")
    );
}

fn step0(x: &mut u64, rule: &Rule) {
    *x = replace_all(*x, rule);
}

fn step1(x: &mut u64, rule: &Rule) {
    *x = rotate_right(*x, 63, 1);
    *x = replace_all(*x, rule);
    *x = rotate_left(*x, 63, 1);
}

fn step2(x: &mut u64, rule: &Rule) {
    *x = rotate_right(*x, 63, 1);
    *x = rotate_right(*x, 63, 1);
    *x = replace_all(*x, rule);
    *x = rotate_left(*x, 63, 1);
    *x = rotate_left(*x, 63, 1);
}

fn steps(x: &mut u64, rule: &Rule) {
    step0(x, rule);
    step1(x, rule);
    step2(x, rule);
    // *x = replace_all(*x, rule);
    // *x = rotate_right_63(*x);
    // *x = replace_all(*x, rule);
    // *x = rotate_right_63(*x);
    // *x = replace_all(*x, rule);
    // *x = rotate_left_63(*x);
    // *x = rotate_left_63(*x);
}

fn to_bools(x: u64, dupl: u64) -> Vec<bool> {
    // print(x);
    let mut result = vec![false; 63];
    for i in 0..63 {
        result[63 - i - 1] = ((x >> i) & 1) != 0;
    }
    let mut result2 = Vec::new();
    for _ in 0..dupl {
        result2.extend(result.iter());
    }
    result2
}

fn show_field(mut x: u64, rule: &Rule, steps: u64, dupl: u64, ni: u64) -> String {
    let mut result = vec![to_bools(x, dupl)];
    for _ in 0..steps / 3 {
        step0(&mut x, rule);
        result.push(to_bools(x, dupl));
        step1(&mut x, rule);
        result.push(to_bools(x, dupl));
        step2(&mut x, rule);
        result.push(to_bools(x, dupl));
    }
    let name = format!("img/{}_{}_{}_{}.png", x, steps, dupl, ni);
    draw_image(&name, result);
    name
}

const RULE0: Rule = Rule([PAT0, PAT1, PAT2, PAT3, PAT4, PAT5, PAT6, PAT7]);

fn num_to_rule(ni: u64) -> Rule {
    let p = PermutationInt::new(ni, 8);
    let p = PermutationIndex::try_from(p).unwrap();
    let p = PermutationArray::try_from(p).unwrap();
    let p: [usize; 8] = p.0.try_into().unwrap();
    Rule(p.map(|x| RULE0.0[x]))
}

fn num_to_rule_name(ni: u64) -> String {
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

fn print_col(ni: u64, img: &str, info: &str) {
    println!("<div class=\"automata-col\">",);
    println!("<span class=\"automata-name\"><b>{}</b></span><br>", ni);
    println!("<img class=\"pixelated\" src=\"{}\">", img);
    println!("<br><span class=\"automata-name\">{}</span>", info);
    println!("</div>");
}

fn print_all_rules() {
    let mut random = Random::new(25025069121927896);
    let start = random.get();
    // let start = 0b101_001_000_101_000_100_000_011_000_010_000_001;

    println!("{}", BEFORE);
    println!("{}", CONTAINER_START);
    for ni in 0..1000 {
        let rule = num_to_rule(ni);
        let name = show_field(start, &rule, 100, 1, ni);
        print_col(ni, &name, &num_to_rule_name(ni));
    }
    println!("{}", CONTAINER_END);
    println!("{}", AFTER);
}

fn find_gliders_633() {
    let ni = 633;
    let rule = num_to_rule(ni);
    println!("{}", BEFORE);
    println!("{}", CONTAINER_START);
    let iter = 0..200; // to find
                       // let iter = [0b111011010011101, 0b1110110100111010, 0b11101101001110100]; // to visualize
    for i in iter {
        print_col(
            ni,
            &show_field(i << 12, &rule, 200, 1, ni),
            &format!("data: {:b}", i),
        );
    }
    println!("{}", CONTAINER_END);
    println!("{}", AFTER);
}

// находит период и смещение глайдера, осциллятора или статичной картинки
fn period(mut x: u64, rule: &Rule) -> Option<(u64, i32)> {
    let y = x;
    for period in 1..60 {
        steps(&mut x, rule);
        for offset in 0..10 {
            if rotate_left(x, 63, offset * 3) == y {
                return Some((period, -(offset as i32)));
            }
            if rotate_right(x, 63, offset * 3) == y {
                return Some((period, (offset as i32)));
            }
        }
    }
    None
}

// определяет размер поля, считая нули слева
fn size(mut x: u64) -> u32 {
    let mut count = 0;
    while x != 0 {
        x >>= 3;
        count += 1;
    }
    count * 3
}

// находит минимальное число, описывающее текущее состояние, используются только вращения
fn minimize(x: u64) -> u64 {
    let mut my_min = x;
    for i in 0..=63 / 3 {
        my_min = my_min.min(rotate_left(x, 63, i * 3));
    }
    my_min
}

// находит минимальное число описывающее глайдер
fn minimize2(mut x: u64, rule: &Rule) -> u64 {
    let mut my_min = x;
    if let Some((period, _)) = period(x, rule) {
        for _ in 0..=period {
            steps(&mut x, rule);
            x = minimize(x);
            my_min = my_min.min(x);
        }
    }
    my_min
}

fn collide_gliders_633() {
    let ni = 633;
    let rule = num_to_rule(ni);

    let statics = [0b1, 0b11, 0b11101];
    let gliders = [
        0b101,
        0b1010,
        // Глайдера 1010 можно повторять сколько угодно
        // 0b10101010,
        // 0b101010101010,
        0b101101,
        0b1011111101,
        0b10010111111,
        0b1011111101101,
        // большой период у нижнего глайдера, убрал во имя экономии ОЗУ
        // 0b11010111101101,
    ];

    let statics_period = [2, 1, 2];
    let gliders_period = [4, 2, 13, 13, 28, 13 /*43*/];

    println!("{}", BEFORE);
    for (mut a, ap) in statics
        .iter()
        .chain(gliders.iter())
        .cloned()
        .zip(statics_period.iter().chain(gliders_period.iter()))
    {
        let ay = a;
        for (mut b, bp) in gliders.iter().cloned().zip(gliders_period.iter()) {
            if b == a {
                continue;
            }
            let by = b;
            a = ay;
            println!("{}", CONTAINER_START);
            for ai in 0..*ap {
                let size_a = size(a);
                b = by;
                for bi in 0..*bp {
                    let size_b = size(b);
                    let all_size = size_b + size_a + 3;
                    let mut offset = 33 - all_size / 2;
                    offset -= offset % 3;
                    print_col(
                        ni,
                        &show_field(
                            rotate_left(rotate_left(a, 63, size_b + 3) | b, 63, offset),
                            &rule,
                            100,
                            1,
                            633,
                        ),
                        &format!("a{} b{} ai{} bi{}", ay, by, ai, bi),
                    );
                    steps(&mut b, &rule);
                    b = minimize(b);
                }
                steps(&mut a, &rule);
                a = minimize(a);
            }
            println!("{}", CONTAINER_END);
        }
    }
    println!("{}", AFTER);
}

fn main() {
    print_all_rules();
    // find_gliders_633();
    // collide_gliders_633();
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
