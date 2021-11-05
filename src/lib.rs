pub fn draw_image(filename: &str, array: Vec<Vec<bool>>) {
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

pub struct Random {
    state: u64,
}

impl Random {
    pub fn get(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_in_range(&mut self, from: u64, to: u64) -> u64 {
        assert!(from < to);
        from + self.get() % (to - from)
    }

    pub fn next_double(&mut self) -> f64 {
        (self.get() as f64) / (std::u64::MAX as f64)
    }

    pub fn new(seed: u64) -> Self {
        assert_ne!(seed, 0);
        Self { state: seed }
    }
}
