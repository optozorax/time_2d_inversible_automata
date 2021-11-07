#![allow(clippy::unusual_byte_groupings)]

use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use permutation_string::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use time_2d_inversible_automata::*;

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

    pub fn steps_count(&self, x: &mut Field, count: u64) {
        for _ in 0..count {
            self.steps(x);
        }
    }

    pub fn show_field(
        &self,
        mut x: Field,
        steps: u64,
        dupl: u64,
        ni: u64,
        img_name: &str,
    ) -> String {
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
        std::fs::create_dir_all(format!("data/img_{}", img_name)).unwrap();
        let name = format!(
            "img_{}/{}_{}_{}_{}_{}.png",
            img_name, x_start, x.size, steps, dupl, ni
        );
        draw_image(&format!("data/{}", name), result);
        name
    }
}

impl Field {
    pub fn new(val: u64, size: u32) -> Self {
        assert!(size % 3 == 0);
        assert!(size <= 64);
        assert!(val == (val & !(ALL_ONES << size)));
        Self { val, size }
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

    pub fn rotate(self, count: i32) -> Self {
        if count < 0 {
            self.rotate_left((-count) as u32)
        } else {
            self.rotate_right(count as u32)
        }
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

    pub fn occupied_size3(&self) -> u32 {
        occupied_size3(self.val)
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
        let mut offset = (self.size - self.occupied_size3()) / 2;
        offset -= offset % 3;
        self.rotate_left(offset)
    }

    pub fn index(&self, pos: u32) -> bool {
        ((self.val >> pos) & 1) != 0
    }
}

pub fn occupied_size3(mut x: u64) -> u32 {
    let mut count = 0;
    while x != 0 {
        x >>= 3;
        count += 3;
    }
    count
}

pub fn occupied_size(mut x: u64) -> u32 {
    let mut count = 0;
    while x != 0 {
        x >>= 1;
        count += 1;
    }
    count
}

pub fn print_col(file: &mut File, ni: u64, img: &str, info: &str) {
    writeln!(file, "<div class=\"automata-col\">",).unwrap();
    writeln!(
        file,
        "<span class=\"automata-name\"><b>{}</b></span><br>",
        ni
    )
    .unwrap();
    writeln!(file, "<img class=\"pixelated\" src=\"{}\">", img).unwrap();
    writeln!(file, "<br><span class=\"automata-name\">{}</span>", info).unwrap();
    writeln!(file, "</div>").unwrap();
}

// находит период и смещение глайдера, осциллятора или статичной картинки
fn period(mut x: Field, rule: &Rule) -> Option<(u64, i32)> {
    let y = x;
    for period in 1..60 {
        rule.steps(&mut x);
        for offset in 0..4 {
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

fn is_same_after(mut x: Field, rule: &Rule, steps: u64, offset: i32) -> bool {
    let y = x;
    rule.steps_count(&mut x, steps);
    x.rotate(offset * 3) == y
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

fn size_round3(x: u32) -> u32 {
    x / 3 + (((x % 3) != 0) as u32)
}

fn size_round2(x: u32) -> u32 {
    x / 2 + (((x % 2) != 0) as u32)
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

// В цикличном массиве находит в каком месте начинается данный паттерн и его длину. Для этого находит нули максимального размера чтобы считать их пустым полем
fn find_pattern_start(x: Field) -> i32 {
    let mut my_min = x.val;
    let mut min_offset: i32 = 0;

    for i in 1..=size_round2(x.size / 3) {
        let new = x.rotate_left(i * 3);
        if new.val < my_min {
            my_min = new.val;
            min_offset = -(i as i32);
        }

        let new = x.rotate_right(i * 3);
        if new.val < my_min {
            my_min = new.val;
            min_offset = i as i32;
        }
    }

    min_offset
}

struct Bits(u64);

impl std::fmt::Debug for Bits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0b{:b}", self.0)
    }
}

fn add_used(mut x: Field, period: u64, rule: &Rule, used: &mut HashMap<u64, (u64, u64, i32)>) {
    for _ in 0..=period {
        let start_offset = find_pattern_start(x);
        let prev = x;
        rule.steps(&mut x);
        used.insert(
            prev.minimize().val,
            (
                period,
                x.minimize().val,
                find_pattern_start(x) - start_offset,
            ),
        );
    }
}

// предполагается что x уже минимизировано
fn check_used(
    mut x: Field,
    period: u64,
    rule: &Rule,
    used: &HashMap<u64, (u64, u64, i32)>,
) -> bool {
    let size = occupied_size(x.val);
    let y = x;
    'outer: for i in 1..=size {
        x = y;
        let pat = !(1 << 63) >> (63 - i);
        let possible_glider = x.val & pat;
        if let Some((period_local, mut next, mut offset_local)) =
            used.get(&possible_glider).cloned()
        {
            if period % period_local == 0 {
                let mut offset = offset_local;
                for _ in 0..period {
                    rule.steps(&mut x);
                    let pat = !(1 << 63) >> (63 - occupied_size(next));
                    let now_sub = x.rotate(offset * 3).val & pat;
                    if now_sub != next {
                        continue 'outer;
                    }

                    let (_, b, c) = *used.get(&next).unwrap();
                    next = b;
                    offset_local = c;

                    offset += offset_local;
                }
                return true;
            }
        }
    }
    false
}

// Определяет является ли данный паттерн глайдером и находит его минимальную форму
fn is_this_glider(x: u64, rule: &Rule) -> Option<(Field, u64, i32)> {
    let size = size_round3(occupied_size3(x));
    if (size + 9) * 3 > 63 {
        return None;
    }
    let (period, offset) = period(Field::new(x, (size + 3) * 3), rule)?;
    for i in [4, 5, 6, 9] {
        if !is_same_after(Field::new(x, (size + i) * 3), rule, period, offset) {
            return None;
        }
    }

    Some((
        minimize2(Field::new(x, (size + 5) * 3), rule, period),
        period,
        offset,
    ))
}

// Находит всех уникальных глайдеров и осцилляторов для данного правила, перебирая все числа до max_count и переводя их в битовое представление
fn get_gliders(rule: &Rule, max_count: u64, use_progress: bool) -> BTreeSet<(u64, u64, i32)> {
    let mut gliders = BTreeSet::new();
    let mut used: HashMap<u64, (u64, u64, i32)> = HashMap::new(); // val, (period, next, offset)
    let progress = ProgressBar::new(max_count).with_style(
        ProgressStyle::default_bar()
            .template("[elapsed: {elapsed:>6} | remaining: {eta:>6}] {wide_bar}"),
    );
    for x in 1..max_count {
        if use_progress {
            progress.inc(1);
        }
        if let Some((min, period, offset)) = is_this_glider(x, rule) {
            if !check_used(min, period, rule, &used) {
                add_used(min, period, rule, &mut used);
                gliders.insert((min.val, period, offset));
            }
        }
    }
    gliders
}

fn show_gliders_for_all_rules() {
    let mut table = File::create("data/table.html").unwrap();
    let mut csv = File::create("data/table.csv").unwrap();
    writeln!(
        table,
        "<style>
table {{
    border-top: 1px solid black;
    border-right: 1px solid black;
    border-collapse: separate;
    border-spacing: 0px 0px;
}}
td {{
    border-left: 1px solid black;
    border-top: 1px solid black;
}}
td:nth-child(2) {{ background-color: #e3e3ff; }}
td:nth-child(3) {{ background-color: #e3e3ff; }}
td:nth-child(4) {{ background-color: #cbf9cb; }}
td:nth-child(5) {{ background-color: #cbf9cb; }}
td:nth-child(6) {{ background-color: #cbf9cb; }}
td:nth-child(7) {{ background-color: #cbf9cb; }}
</style>"
    )
    .unwrap();
    writeln!(table, "<table>").unwrap();

    macro_rules! cell {
        ($a:expr) => {
            writeln!(table, "<td>{}</td>", $a).unwrap();
        };
    }

    macro_rules! row {
        ($($a:tt)*) => {
            writeln!(table, "<tr>").unwrap();
            $($a)*
            writeln!(table, "</tr>").unwrap();
        };
    }

    row! {
        cell!("Rule");
        cell!("Osc #");
        cell!("Osc uniq");
        cell!("Gldr #");
        cell!("Gldr < #");
        cell!("Gldr > #");
        cell!("Gldr uniq");
    }

    writeln!(csv, "rule,osc_n,osc_uniq,gldr,gldr_l,gldr_r,gldr_uniq").unwrap();

    macro_rules! cell {
        ($a:expr) => {
            if $a != 0 {
                writeln!(table, "<td>{}</td>", $a).unwrap();
            } else {
                writeln!(table, "<td></td>").unwrap();
            }
        };
    }

    let progress = ProgressBar::new(5100).with_style(
        ProgressStyle::default_bar()
            .template("[elapsed: {elapsed:>6} | remaining: {eta:>6}] {wide_bar}"),
    );
    for hundreds in 0..51 {
        let current_gliders_file =
            format!("gliders{}-{}.html", hundreds * 100, hundreds * 100 + 100);
        let mut gliders_file = File::create(&format!("data/{}", current_gliders_file)).unwrap();
        let current_oscillators_file = format!(
            "oscillators{}-{}.html",
            hundreds * 100,
            hundreds * 100 + 100
        );
        let mut oscillators_file =
            File::create(&format!("data/{}", current_oscillators_file)).unwrap();
        writeln!(gliders_file, "{}", BEFORE).unwrap();
        writeln!(oscillators_file, "{}", BEFORE).unwrap();
        let size = 100;
        for ni in (0..size).map(|x| x + hundreds * 100) {
            progress.inc(1);

            writeln!(gliders_file, "<div id='{}'></div>", ni).unwrap();
            writeln!(oscillators_file, "<div id='{}'></div>", ni).unwrap();

            let rule = Rule::num_to_rule(ni);

            let gliders = get_gliders(&rule, 10_000, false);

            writeln!(gliders_file, "{}", CONTAINER_START).unwrap();
            writeln!(oscillators_file, "{}", CONTAINER_START).unwrap();
            for (x, period, offset) in &gliders {
                let field = Field::new(*x, 63);
                if *offset != 0 {
                    print_col(
                        &mut gliders_file,
                        ni,
                        &rule.show_field(field.centralize(), period * 3 * 2, 1, ni, "all_gliders"),
                        &format!("n{} p{} o{}", x, period, offset),
                    );
                } else {
                    print_col(
                        &mut oscillators_file,
                        ni,
                        &rule.show_field(field.centralize(), period * 3 * 2, 1, ni, "all_gliders"),
                        &format!("n{} p{}", x, period),
                    );
                }
            }
            writeln!(gliders_file, "{}", CONTAINER_END).unwrap();
            writeln!(oscillators_file, "{}", CONTAINER_END).unwrap();

            let mut oscillators_count = 0;
            let mut gliders_left_count = 0;
            let mut gliders_right_count = 0;
            let mut oscillators_p_o_uniq = HashSet::new();
            let mut gliders_p_o_uniq = HashSet::new();
            for (_, period, offset) in &gliders {
                if *offset == 0 {
                    oscillators_count += 1;
                    oscillators_p_o_uniq.insert(period);
                } else {
                    if *offset < 1 {
                        gliders_right_count += 1;
                    } else {
                        gliders_left_count += 1;
                    }
                    gliders_p_o_uniq.insert((period, offset));
                }
            }

            row! {
                cell!(ni);
                if oscillators_count == 0 {
                    cell!(0);
                } else {
                    writeln!(table, "<td><a href='{}#{}'>{}</a></td>", current_oscillators_file, ni, oscillators_count).unwrap();
                }
                cell!(oscillators_p_o_uniq.len());
                if gliders_left_count + gliders_right_count == 0 {
                    cell!(0);
                } else {
                    writeln!(table, "<td><a href='{}#{}'>{}</a></td>", current_gliders_file, ni, gliders_left_count + gliders_right_count).unwrap();
                }
                cell!(gliders_left_count);
                cell!(gliders_right_count);
                cell!(gliders_p_o_uniq.len());
            }

            writeln!(
                csv,
                "{},{},{},{},{},{},{}",
                ni,
                oscillators_count,
                oscillators_p_o_uniq.len(),
                gliders_left_count + gliders_right_count,
                gliders_left_count,
                gliders_right_count,
                gliders_p_o_uniq.len()
            )
            .unwrap();
        }
        writeln!(gliders_file, "{}", AFTER).unwrap();
        writeln!(oscillators_file, "{}", AFTER).unwrap();
    }
    writeln!(table, "</table>").unwrap();
}

fn find_more_gliders(ni: u64) {
    let mut file = File::create(&format!("data/gliders{}.html", ni)).unwrap();
    let rule = Rule::num_to_rule(ni);
    let gliders = get_gliders(&rule, 100_000_000, true);

    writeln!(file, "{}", BEFORE).unwrap();
    writeln!(file, "{}", CONTAINER_START).unwrap();
    for (x, period, offset) in &gliders {
        let field = Field::new(*x, 63);
        print_col(
            &mut file,
            ni,
            &rule.show_field(
                field.centralize(),
                period * 3 * 2,
                1,
                ni,
                &format!("more_gliders_{}", ni),
            ),
            &format!("n{} p{} o{}", x, period, offset),
        );
    }
    writeln!(file, "{}", CONTAINER_END).unwrap();
    writeln!(file, "{}", AFTER).unwrap();
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TableElem {
    rule: u64,
    #[serde(rename = "osc_n")]
    oscillators_count: u64,
    #[serde(rename = "osc_uniq")]
    oscillators_uniq: u64,
    #[serde(rename = "gldr")]
    gliders_count: u64,
    #[serde(rename = "gldr_l")]
    gliders_left: u64,
    #[serde(rename = "gldr_r")]
    gliders_right: u64,
    #[serde(rename = "gldr_uniq")]
    gliders_uniq: u64,
}

impl std::fmt::Display for TableElem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:5}: o({:3} {:3}) g({:3} | {:3} | {:3} | {:3})",
            self.rule,
            self.oscillators_count,
            self.oscillators_uniq,
            self.gliders_count,
            self.gliders_left,
            self.gliders_right,
            self.gliders_uniq
        )
    }
}

fn rules_data() -> Vec<TableElem> {
    let mut rdr = csv::Reader::from_reader(File::open("data/table.csv").unwrap());
    rdr.deserialize().map(|r| r.unwrap()).collect()
}

fn find_cool_rules() {
    let mut elems = rules_data();

    elems = elems
        .into_iter()
        .filter(|x| {
            x.oscillators_count != 0
                && x.gliders_left > 0
                && x.gliders_right > 0
                && x.oscillators_count >= 3
        })
        .collect();
    elems.sort_by_key(|x| {
        // let a = x.gliders_left as f64;
        // let b = x.gliders_right as f64;
        // (a.max(b) / a.min(b) * 10000.) as i32

        -(x.oscillators_count as i64)
    });
    for i in elems.iter().take(20) {
        println!("{}", i);
    }
}

fn is_this_glider_gun(val: u64, rule: &Rule) -> Option<(u64, u64, u64, i32)> {
    let size = occupied_size3(val);
    let pat = (!0u64) >> (64 - size);
    let mut x = Field::new(val, 60);
    rule.steps(&mut x);
    let mut count = 1;

    while x.val != val && count < 15 {
        if (x.val & pat) == val && (x.val & !pat) != 0 {
            let x_without_gun = Field::new(x.val & !pat, x.size).minimize();

            let mut x0 = Field::new(x.val, 60);
            rule.steps_count(&mut x0, count);
            if (x0.val & pat) == val && (x0.val & !pat) != 0 {
                let x0_without_gun = Field::new(x0.val & !pat, x0.size).minimize();

                // let mut x1 = Field::new(val, 63);
                // rule.steps_count(&mut x1, count);
                // if (x1.val & pat) == val && (x1.val & !pat) != 0 {
                // let x1_without_gun = Field::new(x1.val & !pat, x1.size).minimize();

                // if x_without_gun.val == x1_without_gun.val {
                if let Some(((min, period, offset), (_, period0, offset0))) =
                    is_this_glider(x_without_gun.val, rule)
                        .zip(is_this_glider(x0_without_gun.val, rule))
                {
                    if period == period0 && offset == offset0 && offset != 0 && period <= count {
                        return Some((count, min.val, period, offset));
                    }
                }
                // }
                // }
            }
        }
        rule.steps(&mut x);
        count += 1;
    }
    None
}

fn find_all_glider_guns_rules() {
    let elems = rules_data()
        .into_iter()
        .filter(|x| x.gliders_count != 0)
        // .filter(|x| x.rule < 700)
        .map(|x| x.rule)
        .collect::<Vec<u64>>();

    let mut file = File::create(&"data/glider_guns.html").unwrap();
    writeln!(file, "{}", BEFORE).unwrap();

    let size = 10_000;
    let progress = ProgressBar::new(size * elems.len() as u64).with_style(
        ProgressStyle::default_bar()
            .template("[elapsed: {elapsed:>6} | remaining: {eta:>6}] {wide_bar}"),
    );
    for ni in elems {
        let mut max_guns = 2;
        let rule = Rule::num_to_rule(ni);

        let mut guns = Vec::new();
        for val in 10_000..size {
            progress.inc(1);
            if let Some((gun_period, glider_val, glider_period, glider_offset)) =
                is_this_glider_gun(val, &rule)
            {
                guns.push((val, gun_period, glider_val, glider_period, glider_offset));
                max_guns -= 1;
                if max_guns == 0 {
                    writeln!(file, "{}", CONTAINER_END).unwrap();
                    break;
                }
            }
        }

        if !guns.is_empty() {
            writeln!(file, "{}", CONTAINER_START).unwrap();
            for (val, gun_period, glider_val, glider_period, glider_offset) in guns {
                print_col(
                    &mut file,
                    ni,
                    &rule.show_field(
                        Field::new(val, 63).centralize(),
                        gun_period * 3 * 5,
                        1,
                        ni,
                        "guns",
                    ),
                    &format!(
                        "n{} p{} | n{} p{}, o{}",
                        val, gun_period, glider_val, glider_period, glider_offset
                    ),
                );
            }
            writeln!(file, "{}", CONTAINER_END).unwrap();
        }
    }

    writeln!(file, "{}", AFTER).unwrap();
}

fn find_commute_rules_2d_time() {
    let mut random = Random::new(25025069121927896);
    let starts = (0..10)
        .map(|_| Field::new(random.get() & !(1 << 63), 63))
        .collect::<Vec<_>>();

    let commute_with_everyone = [
        137, 2210, 2298, 2932, 2943, 4203, 4934, 13382, 14751, 16566, 16703, 18419, 19786, 21900,
        23753, 25568, 26301, 26937, 28783, 32341, 35152, 35385, 36116, 37376, 40182,
    ];

    let rules = rules_data()
        .into_iter()
        .filter(|x| x.gliders_count > 2)
        .map(|x| x.rule)
        .filter(|ni| !commute_with_everyone.contains(ni))
        .map(|ni| (ni, Rule::num_to_rule(ni)))
        .collect::<Vec<_>>();

    for (i, (nia, a)) in rules.iter().enumerate() {
        let mut first = true;
        for (nib, b) in rules.iter().skip(i + 1) {
            assert!(nib > nia);
            if starts.iter().all(|start| {
                let mut x1 = *start;
                let mut x2 = *start;

                a.steps(&mut x1);
                b.steps(&mut x1);

                b.steps(&mut x2);
                a.steps(&mut x2);

                x1.val == x2.val
            }) {
                if first {
                    println!("---------------------- {}", nia);
                    first = false;
                }
                println!("{} <-> {}", nia, nib);
            }
        }
    }
}

fn main() {
    show_gliders_for_all_rules();
    find_more_gliders(633);
    find_cool_rules();
    find_all_glider_guns_rules();
    find_commute_rules_2d_time();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glider_guns() {
        color_backtrace::install();

        let rule = Rule::num_to_rule(116);
        assert_eq!(is_this_glider_gun(7, &rule), None);

        let rule = Rule::num_to_rule(160);
        assert_eq!(is_this_glider_gun(1, &rule), None);
        assert_eq!(is_this_glider_gun(305, &rule), None);
        assert_eq!(is_this_glider_gun(437, &rule), None);
        assert_eq!(is_this_glider_gun(601, &rule), None);

        let rule = Rule::num_to_rule(170);
        assert_eq!(is_this_glider_gun(16781, &rule), None);

        let rule = Rule::num_to_rule(173);
        assert_eq!(is_this_glider_gun(7, &rule), None);
        assert_eq!(is_this_glider_gun(59, &rule), None);
        assert_eq!(is_this_glider_gun(484, &rule), None);
        assert_eq!(is_this_glider_gun(9403, &rule), None);
        assert_eq!(is_this_glider_gun(1250, &rule), Some((5, 33, 2, -1)));
        assert_eq!(is_this_glider_gun(3813, &rule), Some((5, 33, 2, -1)));
        assert_eq!(is_this_glider_gun(3911, &rule), Some((5, 33, 2, -1)));

        let rule = Rule::num_to_rule(194);
        assert_eq!(is_this_glider_gun(2, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(5, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(6, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(12, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(13, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(13, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(18, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(50, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(146, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(1170, &rule), Some((2, 1, 2, -1)));
        assert_eq!(is_this_glider_gun(9362, &rule), Some((2, 1, 2, -1)));

        let rule = Rule::num_to_rule(207);
        assert_eq!(is_this_glider_gun(52, &rule), None);
        assert_eq!(is_this_glider_gun(244, &rule), None);
        assert_eq!(is_this_glider_gun(3328, &rule), None);
        assert_eq!(is_this_glider_gun(14068, &rule), None);

        let rule = Rule::num_to_rule(208);
        assert_eq!(is_this_glider_gun(505, &rule), None);
        assert_eq!(is_this_glider_gun(862, &rule), None);
        assert_eq!(is_this_glider_gun(3328, &rule), None);
        assert_eq!(is_this_glider_gun(14068, &rule), None);

        let rule = Rule::num_to_rule(214);
        assert_eq!(is_this_glider_gun(32, &rule), None);
        assert_eq!(is_this_glider_gun(220, &rule), None);

        let rule = Rule::num_to_rule(218);
        assert_eq!(is_this_glider_gun(2, &rule), Some((2, 1, 2, -1)));

        let rule = Rule::num_to_rule(219);
        assert_eq!(is_this_glider_gun(99, &rule), Some((2, 1, 2, -1)));

        let rule = Rule::num_to_rule(268);
        assert_eq!(is_this_glider_gun(3, &rule), Some((2, 1, 2, -1)));

        let rule = Rule::num_to_rule(275);
        assert_eq!(is_this_glider_gun(81920, &rule), None);

        let rule = Rule::num_to_rule(369);
        assert_eq!(is_this_glider_gun(6, &rule), None);

        let rule = Rule::num_to_rule(627);
        assert_eq!(is_this_glider_gun(1699, &rule), Some((5, 33, 2, -1)));
        assert_eq!(is_this_glider_gun(3237, &rule), Some((5, 33, 2, -1)));

        let rule = Rule::num_to_rule(876);
        assert_eq!(is_this_glider_gun(2, &rule), Some((1, 1, 1, 1)));
        assert_eq!(is_this_glider_gun(3, &rule), Some((1, 1, 1, 1)));
        assert_eq!(is_this_glider_gun(4, &rule), Some((1, 1, 1, 1)));
        assert_eq!(is_this_glider_gun(10, &rule), Some((1, 1, 1, 1)));
        assert_eq!(is_this_glider_gun(11, &rule), Some((1, 1, 1, 1)));
    }

    #[test]
    fn test1() {
        color_backtrace::install();

        let rule = Rule::num_to_rule(633);
        let mut used: HashMap<u64, (u64, u64, i32)> = HashMap::new(); // val, (period, next, offset)

        add_used(Field::new(1, 9), 2, &rule, &mut used);
        add_used(Field::new(2, 9), 2, &rule, &mut used);
        add_used(Field::new(4, 9), 2, &rule, &mut used);
        assert!(!check_used(Field::new(3, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(5, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(10, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(45, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(765, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(1215, 30), 28, &rule, &used));
        assert!(!check_used(Field::new(6125, 30), 13, &rule, &used));
        assert!(check_used(Field::new(4, 30), 4, &rule, &used));
        assert!(check_used(Field::new(17, 30), 4, &rule, &used));
        assert!(check_used(Field::new(23, 30), 4, &rule, &used));

        add_used(Field::new(3, 9), 2, &rule, &mut used);
        add_used(Field::new(6, 9), 2, &rule, &mut used);
        add_used(Field::new(12, 9), 2, &rule, &mut used);
        assert!(!check_used(Field::new(5, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(10, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(45, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(765, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(1215, 30), 28, &rule, &used));
        assert!(!check_used(Field::new(6125, 30), 13, &rule, &used));
        assert!(check_used(Field::new(12, 30), 4, &rule, &used));
        assert!(check_used(Field::new(19, 30), 4, &rule, &used));
        assert!(check_used(Field::new(27, 30), 4, &rule, &used));

        add_used(Field::new(5, 12), 4, &rule, &mut used);
        assert!(!check_used(Field::new(45, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(765, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(1215, 30), 28, &rule, &used));
        assert!(!check_used(Field::new(6125, 30), 13, &rule, &used));
        assert!(check_used(Field::new(2565, 30), 4, &rule, &used));
        assert!(check_used(Field::new(2805, 30), 4, &rule, &used));
        assert!(check_used(Field::new(10405, 30), 4, &rule, &used));
        assert!(check_used(Field::new(89775, 30), 4, &rule, &used));

        add_used(Field::new(10, 12), 2, &rule, &mut used);
        assert!(!check_used(Field::new(45, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(765, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(1215, 30), 28, &rule, &used));
        assert!(!check_used(Field::new(6125, 30), 13, &rule, &used));
        assert!(check_used(Field::new(490, 30), 2, &rule, &used));
        assert!(check_used(Field::new(650, 30), 2, &rule, &used));
        assert!(check_used(Field::new(3850, 30), 2, &rule, &used));

        add_used(Field::new(45, 15), 13, &rule, &mut used);
        assert!(!check_used(Field::new(765, 30), 13, &rule, &used));
        assert!(!check_used(Field::new(1215, 30), 28, &rule, &used));
        assert!(!check_used(Field::new(6125, 30), 13, &rule, &used));
        assert!(check_used(Field::new(23085, 30), 13, &rule, &used));
        assert!(check_used(Field::new(23275, 30), 13, &rule, &used));
        assert!(check_used(Field::new(92255, 30), 13, &rule, &used));
    }

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

    #[test]
    fn find_pattern_start_test() {
        color_backtrace::install();

        assert_eq!(find_pattern_start(Field::new(0b111_100_000_000, 12)), -2);

        assert_eq!(
            find_pattern_start(Field::new(0b001_101_100, 10 * 3).rotate_left(3 * 3)),
            3
        );
        assert_eq!(
            find_pattern_start(Field::new(0b001_101_100, 10 * 3).rotate_left(3 * 2)),
            2
        );
        assert_eq!(
            find_pattern_start(Field::new(0b001_101_100, 10 * 3).rotate_left(3)),
            1
        );
        assert_eq!(
            find_pattern_start(Field::new(0b001_101_100, 10 * 3).rotate_right(3)),
            -1
        );
        assert_eq!(
            find_pattern_start(Field::new(0b001_101_100, 10 * 3).rotate_right(3 * 2)),
            -2
        );
        assert_eq!(
            find_pattern_start(Field::new(0b001_101_100, 10 * 3).rotate_right(3 * 3)),
            -3
        );

        assert_eq!(
            find_pattern_start(Field::new(0b001_101_100, 4 * 3).rotate_left(3)),
            1
        );
        assert_eq!(
            find_pattern_start(Field::new(0b001_101_100, 4 * 3).rotate_right(3)),
            -1
        );
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
