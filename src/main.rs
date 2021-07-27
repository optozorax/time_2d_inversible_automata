use rand::prelude::*;
use std::fmt;

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
            result[i] = self.0[i].invert();
        }
        Rule(result)
    }

    fn mirror(&self) -> Rule {
        let mut result = [Block(0); 4];
        for i in 0..4 {
            let (a, b) = Block(i).get();
            let (c, d) = self.0[i as usize].get();
            result[Block::new(b, a).0 as usize] = Block::new(d, c);
        }
        Rule(result)
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

    fn is_preserve_two(&self, array: &[bool], copy: &mut Vec<bool>) -> bool {
        copy_arr(array, copy);

        self.full_step(copy);
        self.full_step(copy);

        array == copy
    }

    fn is_time_symmetrical(
        &self,
        array: &[bool],
        copy1: &mut Vec<bool>,
        copy2: &mut Vec<bool>,
    ) -> bool {
        copy_arr(&array, copy1);
        copy_arr(&array, copy2);

        self.full_step(copy1);
        self.invert_time().full_step_like_invert(copy2);

        copy1 == copy2
    }

    fn is_self_mirrored(
        &self,
        array: &[bool],
        copy1: &mut Vec<bool>,
        copy2: &mut Vec<bool>,
    ) -> bool {
        copy_arr(&array, copy1);
        copy_arr(&array, copy2);

        self.full_step(copy1);
        mirror(copy2);
        self.full_step(copy2);
        mirror(copy2);

        copy1 == copy2
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

fn print_html(rules: &[Rule], array: &[bool], copy1: &mut Vec<bool>, copy2: &mut Vec<bool>) {
    for (ni, i) in rules.iter().enumerate() {
        println!(
            "<div class=\"col{}{}{}{}\">",
            if i.is_preserve(&array, copy1) {
                " trivial"
            } else {
                ""
            },
            if i.is_preserve_two(&array, copy1) {
                " trivial_two"
            } else {
                ""
            },
            if i.is_time_symmetrical(&array, copy1, copy2) {
                " time_symmetricale"
            } else {
                ""
            },
            if i.is_self_mirrored(&array, copy1, copy2) {
                " self_mirror"
            } else {
                ""
            }
        );
        println!("<tt><b>{}</b> - {}</tt><br>", ni, i);
        println!(
            "<img class=\"pixelated skip-img\" src=\"img/{}_skip.png\">",
            ni
        );
        println!(
            "<img class=\"pixelated both-img\" src=\"img/{}_both.png\">",
            ni
        );
        println!("</div>");
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let array = (0..50).map(|_| rng.gen::<bool>()).collect::<Vec<_>>();
    let mut copy1 = array.clone();
    let mut copy2 = array.clone();

    let rules = get_rules();

    enum ShowFilter {
        NotShow(Vec<usize>),
        Show(Vec<usize>),
        ShowWithBlue(Vec<usize>),
    }
    use ShowFilter::*;
    impl ShowFilter {
        fn filter(&self, n: usize) -> bool {
            match self {
                NotShow(arr) => arr.iter().all(|x| *x != n),
                Show(arr) | ShowWithBlue(arr) => arr.iter().any(|x| *x == n),
            }
        }
        fn should_blue(&self) -> bool {
            matches!(self, ShowWithBlue(_))
        }
    }

    for show in [
        NotShow(vec![0, 7, 16, 23]),
        Show(vec![1, 5, 6, 14]),
        Show(vec![2, 10, 13, 21]),
        Show(vec![3, 4, 8, 12]),
        Show(vec![9, 17, 18, 22]),
        Show(vec![11, 15, 19, 20]),
        ShowWithBlue(vec![3, 4, 8, 11, 12, 15, 19, 20]),
    ] {
        println!("-------------------------------\nRelations graph:");

        println!("digraph G {{\nedge [arrowhead=none];");
        for (ni, i) in rules.iter().enumerate().filter(|(ni, _)| show.filter(*ni)) {
            for (nj, j) in rules
                .iter()
                .enumerate()
                .skip(ni)
                .filter(|(nj, _)| show.filter(*nj))
            {
                if i.is_time_invert_by(j, &array, &mut copy1) {
                    println!("{} -> {};", ni, nj);
                }
                if i.is_color_invert_by(j, &array, &mut copy1, &mut copy2) {
                    println!("{} -> {} [color=red];", ni, nj);
                }
                if i.is_mirrored_by(j, &array, &mut copy1, &mut copy2) {
                    println!("{} -> {} [color=green];", ni, nj);
                }
                if i.is_commute_with(j, &array, &mut copy1, &mut copy2)
                    && ni != nj
                    && show.should_blue()
                {
                    println!("{} -> {} [color=blue];", ni, nj);
                }
            }
        }
        println!("}}");
    }

    let show = NotShow(vec![0, 7, 16, 23]);

    println!("-------------------------------\nCommute graph:");

    for (ni, i) in rules.iter().enumerate().filter(|(ni, _)| show.filter(*ni)) {
        for (nj, j) in rules
            .iter()
            .enumerate()
            .skip(ni)
            .filter(|(nj, _)| show.filter(*nj))
        {
            if i.is_commute_with(j, &array, &mut copy1, &mut copy2) && ni != nj {
                println!("{} -> {};", ni, nj);
            }
        }
    }

    println!("-------------------------------\nRule generation graph:");

    for (ni, i) in rules.iter().enumerate().filter(|(ni, _)| show.filter(*ni)) {
        let n = get_number(&rules, &i.invert_time());
        if n <= ni {
            println!("{} -> {};", ni, n);
        }

        let n = get_number(&rules, &i.invert_color());
        if n <= ni {
            println!("{} -> {} [color=red];", ni, n);
        }

        let n = get_number(&rules, &i.mirror());
        if n <= ni {
            println!("{} -> {} [color=green];", ni, n);
        }
    }

    println!("-------------------------------\nhtml:");

    print_html(&rules, &array, &mut copy1, &mut copy2);

    draw_all_images(&rules, &array, &mut copy1);
}
