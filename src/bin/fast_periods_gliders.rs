/*! # Быстрое вычисление обратимых 1D автоматов.
 * 
 * Здесь в первую очередь находится код для быстрого вычислнеия 1D обратимых автоматов на Margolus Neighborhood. Делается это за счёт битовых операций, итоговый ассемблер получается космически маленьким и быстрым.
 * 
 * Здесь вычисляется несколько вещей касательно обратимых 1D автоматов:
 * * Период повторения рандомных и повторяющихся паттернов
 * * Наличие глайдеров в пустом поле и в повторяющихся паттернах
 * 
 * Результаты такие, что период для случайных это константа (192 или 64), а глайдеров особо нету, в автомате 1 так точно, а в остальных они ну тупо состоят из одной клетки и могут сразу пускать глайдеров в две стороны.
 * 
 * Я писал об этом начиная отсюда: https://t.me/automatter/367
 */

use std::collections::BTreeMap;
use std::collections::HashMap;

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

const PAT0: u64 = repeat(0b00, 2);
const PAT1: u64 = repeat(0b01, 2);
const PAT2: u64 = repeat(0b10, 2);
const PAT3: u64 = repeat(0b11, 2);

fn replace(x: u64, pat1: u64, pat2: u64) -> u64 {
    const BITS_ODD: u64 = repeat(0b01, 2);
    let x1 = !(x ^ pat1);
    let x2 = x1 & (x1 >> 1) & BITS_ODD;
    let x3 = x2 | (x2 << 1);
    x3 & pat2
}

fn replace_all(x: u64, (r0, r1, r2, r3): (u64, u64, u64, u64)) -> u64 {
    replace(x, PAT0, r0) | replace(x, PAT1, r1) | replace(x, PAT2, r2) | replace(x, PAT3, r3)
}

fn print(x: u64) {
    println!(
        ".{}",
        format!("{:064b}", x).replace("0", " ").replace("1", "█")
    );
}

fn step0(x: &mut u64, rule: &(u64, u64, u64, u64)) {
    *x = replace_all(*x, *rule);
}

fn step1(x: &mut u64, rule: &(u64, u64, u64, u64)) {
    *x = x.rotate_left(1);
    *x = replace_all(*x, *rule);
    *x = x.rotate_right(1);
}

fn steps(x: &mut u64, rule: &(u64, u64, u64, u64)) {
    step0(x, rule);
    step1(x, rule);
}

fn show_field(mut x: u64, rule: &(u64, u64, u64, u64), steps: u64) {
    print(x);
    for _ in 0..steps / 2 {
        step0(&mut x, rule);
        print(x);
        step1(&mut x, rule);
        print(x);
    }
}

fn period(mut x: u64, rule: &(u64, u64, u64, u64)) -> u64 {
    let y = x;
    step0(&mut x, rule);
    step1(&mut x, rule);

    let mut count = 2;
    while y != x {
        steps(&mut x, rule);
        count += 2;
    }
    count
}

fn is_glider(mut x: u64, rule: &(u64, u64, u64, u64)) -> Option<(u32, u32)> {
    let y = x;
    for count in 1..30 {
        // количество меньше 32, ибо дальше уже практически всё повторяется.
        steps(&mut x, rule);
        for offset in 1..10 {
            // максимальный offset взят с потолка, по идее большие не нужны, хотя бы маленьких найти
            if (x.rotate_left(offset * 2) == y || x.rotate_right(offset * 2) == y) && x != 0 {
                return Some((offset * 2, count * 2));
            }
        }
    }
    None
}

fn for_each_repeated_pattern(mut f: impl FnMut(u64)) {
    f(0);
    f(0xFFFFFFFFFFFFFFFF);
    for repeat_size in 1..=16 {
        for repeated in (1..(1 << (repeat_size - 1))).map(|x| repeat(x, repeat_size)) {
            f(repeated);
        }
    }
}

fn find_periods(rule: &(u64, u64, u64, u64)) {
    let mut periods = HashMap::new();
    let mut period_examples = BTreeMap::new();
    for x in 0..1_000_000 {
        let p = period(x, rule);
        *periods.entry(p).or_insert(0) += 1;
        *period_examples.entry(p).or_insert(x) = x;
    }
    println!(
        "count of different periods: {:?}; examples of that periods: {:?}",
        periods, period_examples
    );
    for (p, x) in period_examples {
        println!("\nperiod {} pattern:", p);
        show_field(x, rule, p);
    }
}

fn find_periods_repeated(rule: &(u64, u64, u64, u64)) {
    let mut periods = HashMap::new();
    let mut period_examples = BTreeMap::new();
    for_each_repeated_pattern(|repeated| {
        let p = period(repeated, rule);
        *periods.entry(p).or_insert(0) += 1;
        *period_examples.entry(p).or_insert(repeated) = repeated;
    });
    println!(
        "count of different periods of repeated patterns: {:?}; examples of that periods: {:?}",
        periods, period_examples
    );
    for (p, x) in period_examples {
        println!("\nperiod {} pattern:", p);
        show_field(x, rule, p);
    }
}

fn find_gliders(rule: &(u64, u64, u64, u64)) {
    let mut gliders_example = BTreeMap::new();
    for x in 1..10_000_000 {
        if let Some(data) = is_glider(x, rule) {
            *gliders_example.entry(data).or_insert(x) = x;
        }
    }
    println!("example of gliders: {:?}", gliders_example);
    for ((offset, count), x) in gliders_example {
        println!("\nglider with offset {} and period {} pattern:", offset, count);
        show_field(x, rule, count.into());
    }
}

fn find_gliders_in_ether(rule: &(u64, u64, u64, u64)) {
    let mut gliders_example = BTreeMap::new();
    for_each_repeated_pattern(|repeated| {
        for x in (1..512).map(|x| (x + repeated).rotate_left(32)) {
            // .rotate_left(32) чтобы глайдер был по центру
            if let Some(data) = is_glider(x, rule) {
                *gliders_example.entry(data).or_insert(x) = x;
            }
        }
    });
    println!("example of gliders in ether: {:?}", gliders_example);
    for ((offset, count), x) in gliders_example {
        println!("\nglider in ether with offset {} and period {} pattern:", offset, count);
        show_field(x, rule, count.into());
    }
}

fn examine_rule(rule_no: u64, rule: (u64, u64, u64, u64)) {
    println!("--------------------------------------------------------");
    println!("working for rule {}", rule_no);
    find_periods(&rule);
    find_periods_repeated(&rule);

    find_gliders(&rule);
    find_gliders_in_ether(&rule);
}

fn main() {
    examine_rule(1, (PAT0, PAT1, PAT3, PAT2));
    examine_rule(2, (PAT0, PAT2, PAT1, PAT3));
    examine_rule(3, (PAT0, PAT2, PAT3, PAT1));
    examine_rule(18, (PAT3, PAT0, PAT1, PAT2));
    examine_rule(21, (PAT3, PAT1, PAT2, PAT0));
}
