use std::str::FromStr;
use std::io::Write;
use std::fs::File;
extern crate num;
use num::Complex;
extern crate image;
use image::ColorType;
use image::ImageEncoder;
use image::codecs::png::PngEncoder;


fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index+1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<f32>("", ','), None);
    assert_eq!(parse_pair::<f32>("1.2", ','), None);
    assert_eq!(parse_pair::<f32>(",", ','), None);
    assert_eq!(parse_pair::<f32>(",3.7", ','), None);
    assert_eq!(parse_pair::<i32>("400x800", 'x'), Some((400, 800)));
    assert_eq!(parse_pair::<f64>("1.24,-0.6048", ','), Some((1.24, -0.6048)));
    assert_eq!(parse_pair::<f32>("3.14,25.1", ','), Some((3.14, 25.1)));
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex{re, im}),
        None => None
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("3.14,2.71"), Some(Complex{re: 3.14, im: 2.71}));
    assert_eq!(parse_complex("1.2,"), None);
}

fn pixel_to_point(bounds: (usize, usize), pixel: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>) -> Complex<f64> {
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);

    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100,100), (25,75), Complex{re: -1.0, im: 1.0}, Complex{re: 1.0, im: -1.0}), Complex{re: -0.5, im: -0.5});
}

fn mandelbrot(c: Complex<f64>, limit :u32) -> Option<u32> {
    let mut z = Complex{re: 0.0, im: 0.0};
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 2.0 {
            return Some(i);
        }
    }
    
    None
}

#[repr(C, packed)]
#[derive(Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

fn colors_to_u8s(pixels: &[Color]) -> &[u8] {
    let p = pixels.as_ptr() as *const u8;
    let n = 3 * pixels.len();
    unsafe { std::slice::from_raw_parts(p, n) }
}

fn color_from_value(value: u32) -> Color {

    if  value <= 30 {
        Color{r: 50, g: 60, b: 50}
    } else if value > 30 && value <= 90 {
        Color{r: 255 - value as u8, g: 255 - value as u8, b: 20 }
    } else if value > 90 && value <= 200 {
        Color{r: 40, g: 255 - value as u8, b: 255 - value as u8 }
    } else {
        Color{r: 10, g: 20, b: 255 - value as u8 }
    }
}


fn render(pixels: &mut [Color], bounds: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    for y in 0..bounds.1 {
        for x in 0..bounds.0 {
            let point = pixel_to_point(bounds, (x, y), upper_left, lower_right);
            pixels[y * bounds.0 + x] =
                match mandelbrot(point, 255) {
                    None => Color{r: 0, g: 0, b: 0},
                    Some(value) => color_from_value(value)
                };
        }
    }
}

fn write_png(filename: &str, pixels: &[Color], bounds: (usize, usize)) {
    let file = File::create(filename).unwrap();
    let encoder = PngEncoder::new(file);

    let bytes = colors_to_u8s(&pixels); 

    encoder.write_image(&bytes, bounds.0 as u32, bounds.1 as u32, ColorType::Rgb8);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        writeln!(std::io::stderr(), "Usage: {} <png file> <image size> <upper left> <lower right>", args[0]).unwrap();
        writeln!(std::io::stderr(), "Example: {} mandel.png 800x600 -1.20,0.35 -1,0.20", args[0]).unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair::<usize>(&args[2], 'x').expect("failed to parse image size");
    let upper_left = parse_complex(&args[3]).expect("failed to parse upper left");
    let lower_right = parse_complex(&args[4]).expect("failed to parse lower right");

    let mut pixels = vec![Color{r: 0, g: 0, b: 0}; bounds.0 * bounds.1];

    render(&mut pixels, bounds, upper_left, lower_right);
    write_png(&args[1], &pixels, bounds);
}
