use rand::prelude::*;
use std::fmt;
use std::io::prelude::*;
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Block(u8);

impl Block {
    fn new(a: bool, b: bool) -> Self {
        Block(a as u8 * 2 + b as u8)
    }

    fn get(self) -> (bool, bool) {
        match self.0 {
            0 => (false, false),
            1 => (false, true),
            2 => (true, false),
            3 => (true, true),
            _ => unreachable!(),
        }
    }

    fn invert(self) -> Block {
        let (a, b) = self.get();
        Block::new(!a, !b)
    }

    fn mirror(self) -> Block {
        let (a, b) = self.get();
        Block::new(b, a)
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
struct Rule([Block; 4]);

impl Rule {
    fn new(arr: &[u8]) -> Self {
        Rule([Block(arr[0]), Block(arr[1]), Block(arr[2]), Block(arr[3])])
    }

    fn step(&self, array: &mut Vec<bool>, first_step: bool) {
        let len = array.len();
        if !first_step {
            array.push(array[0]);
        }
        for i in ((!first_step as usize)..len).step_by(2) {
            let block = Block::new(array[i], array[i + 1]);
            let (a, b) = self.0[block.0 as usize].get();
            array[i] = a;
            array[i + 1] = b;
        }
        if !first_step {
            array[0] = array.pop().unwrap();
        }
    }

    fn full_step(&self, array: &mut Vec<bool>) {
        self.step(array, true);
        self.step(array, false);
    }

    fn full_step_like_invert(&self, array: &mut Vec<bool>) {
        self.step(array, false);
        self.step(array, true);
    }

    fn invert_time(&self) -> Rule {
        let mut result = [Block(0); 4];
        for i in 0..4 {
            result[self.0[i].0 as usize] = Block(i as u8);
        }
        Rule(result)
    }

    fn invert_color(&self) -> Rule {
        let mut result = [Block(0); 4];
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            result[Block(i as u8).invert().0 as usize] = self.0[i].invert();
        }
        Rule(result)
    }

    fn invert_half_color(&self) -> Rule {
        let mut result = [Block(0); 4];
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            result[i] = self.0[i].invert();
        }
        Rule(result)
    }

    fn mirror(&self) -> Rule {
        let mut result = [Block(0); 4];
        for i in 0..4 {
            result[Block(i as u8).mirror().0 as usize] = self.0[i].mirror();
        }
        Rule(result)
    }

    fn strange_change(&self) -> Rule {
        let mut result = *self;
        // let mut result = self.invert_color();
        // let mut result = self.mirror();
        // let mut result = self.invert_half_color();
        result.0.swap(0, 1);
        result.0.swap(2, 3);
        // result.0.swap(0, 2);
        // result.0.swap(1, 3);
        result

        // let mut result = [Block(0); 4];
        // for i in 0..4 {
        //     // result[self.0[i].mirror().0 as usize] = Block(i as u8).mirror(); // ничего не даёт
        //     // result[self.0[i].invert().0 as usize] = Block(i as u8).invert(); // ничего не даёт
        //
        //     // result[self.0[i].mirror().0 as usize] = Block(i as u8).invert(); // 0 - 21
        //     // result[self.0[i].invert().0 as usize] = Block(i as u8).mirror(); // 0 - 21
        //
        //     // result[self.0[i].mirror().0 as usize] = Block(i as u8); // 0 - 2
        //     // result[self.0[i].invert().0 as usize] = Block(i as u8); // ничего не даёт
        //
        //     // result[self.0[i].0 as usize] = Block(i as u8).mirror(); // 0 - 2
        //     // result[self.0[i].0 as usize] = Block(i as u8).invert(); // ничего не даёт
        //
        //     // result[Block(i as u8).invert().0 as usize] = self.0[i].mirror(); // 0 - 21
        //     // result[Block(i as u8).mirror().0 as usize] = self.0[i].invert(); // 0 - 21
        //
        //     // result[Block(i as u8).mirror().0 as usize] = self.0[i]; // 0 - 2
        //     // result[Block(i as u8).invert().0 as usize] = self.0[i]; // ничего не даёт
        // }
        // Rule(result)
    }
}

fn copy_arr(array: &[bool], copy: &mut [bool]) {
    copy.iter_mut()
        .zip(array.iter())
        .for_each(|(to, what)| *to = *what);
}

/// Properties of Rule
impl Rule {
    fn is_preserve(&self, array: &[bool], copy: &mut Vec<bool>) -> bool {
        copy_arr(array, copy);

        self.full_step(copy);

        array == copy
    }

    fn is_preserve_two_steps(&self, array: &[bool], copy: &mut Vec<bool>) -> bool {
        copy_arr(array, copy);

        self.full_step(copy);
        self.full_step(copy);

        array == copy
    }

    fn is_time_symmetrical(&self) -> bool {
        *self == self.invert_time()
    }

    fn is_self_inverse(&self) -> bool {
        *self == self.invert_color()
    }

    fn is_self_mirrored(&self) -> bool {
        *self == self.mirror()
    }

    fn is_save_count(&self, array: &[bool], copy: &mut Vec<bool>) -> bool {
        copy_arr(array, copy);

        self.full_step(copy);

        array.iter().filter(|x| **x).count() == copy.iter().filter(|x| **x).count()
    }
}

/// Rule relationship with others
impl Rule {
    /// Needed for 2D time
    fn is_commute_with(
        &self,
        other: &Rule,
        array: &[bool],
        copy1: &mut Vec<bool>,
        copy2: &mut Vec<bool>,
    ) -> bool {
        copy_arr(&array, copy1);
        copy_arr(&array, copy2);

        self.full_step(copy1);
        other.full_step(copy1);

        other.full_step(copy2);
        self.full_step(copy2);

        copy1 == copy2
    }

    /// Other can be invert rule for self
    fn is_time_invert_by(&self, other: &Rule, array: &[bool], copy1: &mut Vec<bool>) -> bool {
        copy_arr(&array, copy1);

        self.full_step(copy1);
        other.full_step_like_invert(copy1);

        copy1 == array
    }

    fn is_color_invert_by(
        &self,
        other: &Rule,
        array: &[bool],
        copy1: &mut Vec<bool>,
        copy2: &mut Vec<bool>,
    ) -> bool {
        copy_arr(&array, copy1);
        copy_arr(&array, copy2);

        self.full_step(copy1);

        invert_color(copy2);
        other.full_step(copy2);
        // other.full_step_like_invert
        invert_color(copy2);

        copy1 == copy2
    }

    fn is_mirrored_by(
        &self,
        other: &Rule,
        array: &[bool],
        copy1: &mut Vec<bool>,
        copy2: &mut Vec<bool>,
    ) -> bool {
        copy_arr(&array, copy1);
        copy_arr(&array, copy2);

        self.full_step(copy1);

        mirror(copy2);
        other.full_step(copy2);
        mirror(copy2);

        copy1 == copy2
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{} {} {} {}]",
            self.0[0].0, self.0[1].0, self.0[2].0, self.0[3].0
        )
    }
}

fn permutations<T, F: FnMut(&[T])>(a: &mut [T], mut f: F) {
    fn helper<T, F: FnMut(&[T])>(k: usize, a: &mut [T], f: &mut F) {
        if k == 1 {
            f(a);
        } else {
            helper(k - 1, a, f);
            for i in 0..k - 1 {
                if k % 2 == 0 {
                    a.swap(i, k - 1);
                } else {
                    a.swap(0, k - 1);
                }
                helper(k - 1, a, f);
            }
        }
    }
    helper(a.len(), a, &mut f);
}

fn simulate_both(rule: &Rule, array: &mut Vec<bool>, steps: usize) -> Vec<Vec<bool>> {
    let mut result = vec![];
    for _ in 0..steps / 2 {
        result.push(array.clone());
        rule.step(array, true);
        result.push(array.clone());
        rule.step(array, false);
    }
    result
}

fn simulate_skip(rule: &Rule, array: &mut Vec<bool>, steps: usize) -> Vec<Vec<bool>> {
    let mut result = vec![];
    for _ in 0..steps {
        result.push(array.clone());
        rule.step(array, true);
        rule.step(array, false);
    }
    result
}

fn draw_image(filename: &str, array: Vec<Vec<bool>>) {
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::Path;

    let path = Path::new(filename);
    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, array[0].len() as u32, array.len() as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    let data = array
        .iter()
        .map(|x| x.iter())
        .flatten()
        .map(|x| (!*x) as u8 * 255)
        .collect::<Vec<u8>>();
    writer.write_image_data(&data).unwrap();
}

fn get_rules() -> Vec<Rule> {
    let mut rule_base = [0, 1, 2, 3];
    let mut rules = vec![];
    permutations(&mut rule_base, |rule| {
        rules.push(Rule::new(rule));
    });
    rules.sort();

    rules
}

fn invert_color(a: &mut Vec<bool>) {
    for i in a {
        *i = !*i;
    }
}

fn mirror(a: &mut [bool]) {
    let len = a.len();
    for i in 0..len / 2 {
        a.swap(i, len - i - 1);
    }
}

fn draw_all_images(rules: &[Rule], array: &[bool], copy1: &mut Vec<bool>) {
    for (ni, i) in rules.iter().enumerate() {
        copy_arr(&array, copy1);
        draw_image(
            &format!("img/{}_skip.png", ni),
            simulate_skip(&i, copy1, 50),
        );

        copy_arr(&array, copy1);
        draw_image(
            &format!("img/{}_both.png", ni),
            simulate_both(&i, copy1, 50),
        );
    }
}

struct SpaceVec<T>(pub Vec<T>);

impl<T: fmt::Display> fmt::Display for SpaceVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len();
        for (index, i) in self.0.iter().enumerate() {
            write!(f, "{}", i)?;
            if index + 1 != len {
                write!(f, " ")?;
            }
        }
        Ok(())
    }
}

fn get_number(rules: &[Rule], rule: &Rule) -> usize {
    rules.iter().position(|x| x == rule).unwrap()
}

fn print_html(rules: &[Rule], array: &[bool], copy1: &mut Vec<bool>) {
    for (ni, i) in rules.iter().enumerate() {
        println!(
            "<div class=\"automata-col{}{}{}{}{}{}\">",
            if i.is_preserve(&array, copy1) {
                " trivial"
            } else {
                ""
            },
            if i.is_preserve_two_steps(&array, copy1) {
                " trivial_two"
            } else {
                ""
            },
            if i.is_time_symmetrical() {
                " time_symmetricale"
            } else {
                ""
            },
            if i.is_self_mirrored() {
                " self_mirror"
            } else {
                ""
            },
            if i.is_self_inverse() {
                " self_inverse"
            } else {
                ""
            },
            if i.is_save_count(&array, copy1) {
                " save_count"
            } else {
                ""
            }
        );
        println!("<span class=\"automata-name\"><b>{}</b></span><br>", ni);
        println!(
            "<img class=\"pixelated skip-img\" src=\"/assets/invertible-automata/img/{}_skip.png\">",
            ni
        );
        println!(
            "<img class=\"pixelated both-img\" src=\"/assets/invertible-automata/img/{}_both.png\">",
            ni
        );
        println!("<span class=\"automata-name\">{}</span>", i);
        println!("</div>");
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let array = (0..50).map(|_| rng.gen::<bool>()).collect::<Vec<_>>();
    let mut copy1 = array.clone();
    let mut copy2 = array.clone();

    let rules = get_rules();

    struct ShowFilter {
        arr: Vec<usize>,
        filter_type: bool,
        show_purple: bool,
        show_cyan: bool,
    }
    impl ShowFilter {
        fn filter(&self, n: usize) -> bool {
            if self.filter_type {
                self.arr.iter().all(|x| *x != n)
            } else {
                self.arr.iter().any(|x| *x == n)
            }
        }

        fn not_show(arr: Vec<usize>, show_purple: bool, show_cyan: bool) -> Self {
            Self {
                arr,
                filter_type: true,
                show_purple,
                show_cyan,
            }
        }

        fn show(arr: Vec<usize>, show_purple: bool, show_cyan: bool) -> Self {
            Self {
                arr,
                filter_type: false,
                show_purple,
                show_cyan,
            }
        }
    }

    for (n, show) in [
        ShowFilter::not_show(vec![], false, false),
        ShowFilter::not_show(vec![], true, false),
        ShowFilter::not_show(vec![], true, true),
        // 3..8
        ShowFilter::show(vec![0, 23, 7, 16], false, false),
        ShowFilter::show(vec![2, 21, 10, 13], false, false),
        ShowFilter::show(vec![1, 6, 5, 14], false, false),
        ShowFilter::show(vec![3, 4, 8, 12], false, false),
        ShowFilter::show(vec![9, 17, 18, 22], false, false),
        ShowFilter::show(vec![11, 15, 19, 20], false, false),
        // 9..12
        ShowFilter::show(vec![0, 23, 7, 16], true, false),
        ShowFilter::show(vec![2, 21, 10, 13], true, false),
        ShowFilter::show(vec![1, 6, 5, 14, 9, 17, 18, 22], true, false),
        ShowFilter::show(vec![11, 15, 19, 20, 3, 4, 8, 12], true, false),
        // 13..16
        ShowFilter::show(vec![0, 23, 7, 16], true, true),
        ShowFilter::show(vec![2, 21, 10, 13], true, true),
        ShowFilter::show(vec![1, 6, 5, 14, 9, 17, 18, 22], true, true),
        ShowFilter::show(vec![11, 15, 19, 20, 3, 4, 8, 12], true, true),
    ]
    .iter()
    .enumerate()
    {
        let mut file = std::fs::File::create("temp.dot").unwrap();

        write!(file, "digraph G {{\nedge [arrowhead=none];").unwrap();
        for (ni, i) in rules.iter().enumerate().filter(|(ni, _)| show.filter(*ni)) {
            let n = get_number(&rules, &i.invert_time());
            if ni <= n {
                write!(file, "{} -> {};", ni, n).unwrap();
            }

            let n = get_number(&rules, &i.invert_color());
            if ni <= n {
                write!(file, "{} -> {} [color=red];", ni, n).unwrap();
            }

            let n = get_number(&rules, &i.mirror());
            if ni <= n {
                write!(file, "{} -> {} [color=green];", ni, n).unwrap();
            }

            let n = get_number(&rules, &i.invert_half_color());
            if ni <= n && show.show_purple {
                write!(file, "{} -> {} [color=purple];", ni, n).unwrap();
            }

            let n = get_number(&rules, &i.strange_change());
            if ni <= n && show.show_cyan {
                write!(file, "{} -> {} [color=cyan];", ni, n).unwrap();
            }
        }
        write!(file, "}}").unwrap();
        drop(file);

        Command::new("dot")
            .args(&["-Tsvg", "temp.dot", "-o", &format!("svg/{}.svg", n)])
            .output()
            .unwrap();

        Command::new("rm").args(&["temp.dot"]).output().unwrap();
    }

    for (n, show) in [
        ShowFilter::show(vec![2, 10, 13, 21], false, false),
        ShowFilter::show(vec![3, 11, 12, 20], false, false),
        ShowFilter::show(vec![4, 8, 15, 19], false, false),
    ]
    .iter()
    .enumerate()
    {
        let mut file = std::fs::File::create("temp.dot").unwrap();
        write!(file, "digraph G {{\nedge [arrowhead=none];").unwrap();
        for (ni, i) in rules.iter().enumerate().filter(|(ni, _)| show.filter(*ni)) {
            for (nj, j) in rules
                .iter()
                .enumerate()
                .skip(ni)
                .filter(|(nj, _)| show.filter(*nj))
            {
                if i.is_commute_with(j, &array, &mut copy1, &mut copy2) && ni != nj {
                    write!(file, "{} -> {};", ni, nj).unwrap();
                }
            }
        }
        write!(file, "}}").unwrap();
        drop(file);

        Command::new("dot")
            .args(&["-Tsvg", "temp.dot", "-o", &format!("svg/commute{}.svg", n)])
            .output()
            .unwrap();

        Command::new("rm").args(&["temp.dot"]).output().unwrap();
    }

    println!("-------------------------------\nhtml:");

    print_html(&rules, &array, &mut copy1);

    // draw_all_images(&rules, &array, &mut copy1);
}
