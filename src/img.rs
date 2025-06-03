///Implementation of Converting PNG images similar to img2xterm
///
///This is based off of the original:https://github.com/rossy/img2xterm/tree/master
///
// use core::f64;

use rgb::{FromSlice, RGB8, Rgb};

use crate::parser::TerminalCharacter;

// pub fn basename2(string: &str) {
//     todo!()
// }

// const CHROMA_WEIGHT: f64 = 1.0;
// const VALUERANGE: [u8; 6] = [0x00u8, 0x5fu8, 0x87u8, 0xafu8, 0xd7u8, 0xffu8];
// ///Direct rewrite of the srgb2lab method in img2xterm. Converts SRGB color to an CIELAB color space.
// /// Requires mut refs to initialized values for the CIELAB color representation
// pub fn srgb2lab(red: u8, green: u8, blue: u8, lightness: &mut f64, aa: &mut f64, bb: &mut f64) {
//     let r = red as f64 / 255.0;
//     let g = green as f64 / 255.0;
//     let b = blue as f64 / 255.0;

//     let rl = match r <= 0.4045 {
//         true => r / 12.92,
//         false => ((r + 0.055) / 1.055).powf(2.4),
//     };
//     let gl = match g <= 0.4045 {
//         true => g / 12.92,
//         false => ((g + 0.055) / 1.055).powf(2.4),
//     };
//     let bl = match b <= 0.4045 {
//         true => b / 12.92,
//         false => ((b + 0.055) / 1.055).powf(2.4),
//     };

//     let x = 0.4124564 * rl + 0.3575761 * gl + 0.1804375 * bl;
//     let y = 0.2126729 * rl + 0.7151522 * gl + 0.0721750 * bl;
//     let z = 0.0193339 * rl + 0.1191920 * gl + 0.9503041 * bl;

//     let xn = x / 0.95047;
//     let yn = y;
//     let zn = z / 1.08883;

//     let fxn = match xn > (216.0 / 24389.0) {
//         true => xn.powf(1.0 / 3.0),
//         false => (841.0 / 108.0) * xn + (4.0 / 29.0),
//     };
//     let fyn = match yn > (216.0 / 24389.0) {
//         true => yn.powf(1.0 / 3.0),
//         false => (841.0 / 108.0) * yn + (4.0 / 29.0),
//     };
//     let fzn = match zn > (216.0 / 24389.0) {
//         true => zn.powf(1.0 / 3.0),
//         false => (841.0 / 108.0) * zn + (4.0 / 29.0),
//     };

//     *lightness = 116.0 * fyn - 16.0;
//     *aa = (500.0 * (fxn - fyn)) * CHROMA_WEIGHT;
//     *bb = (200.0 * (fyn - fzn)) * CHROMA_WEIGHT;
// }

// ///Direct rewrite of the srgb2yiq method in img2xterm. Converts SRGB color to an YIQ (NTSC TV) color space.
// /// Requires mut refs to initialized values for the YIQ color representation
// pub fn srgb2yiq(red: u8, green: u8, blue: u8, y: &mut f64, i: &mut f64, q: &mut f64) {
//     let r = red as f64 / 255.0;
//     let g = green as f64 / 255.0;
//     let b = blue as f64 / 255.0;

//     *y = 0.299 * r + 0.587 * g + 0.144 * b;
//     *i = (0.595716 * r + -0.274453 * g + -0.321263 * b) * CHROMA_WEIGHT;
//     *q = (0.211456 * r + -0.522591 * g + 0.311135 * b) * CHROMA_WEIGHT;
// }

// ///Direct rewrite of the cie94 method in img2xterm.
// /// This is likely doing a delta-E calculation (color diff)
// /// unlike the conversion methods actually uses return instead of refs
// pub fn cie94(l1: f64, a1: f64, b1: f64, l2: f64, a2: f64, b2: f64) -> f64 {
//     const KL: f64 = 1.0;
//     const K1: f64 = 0.045;
//     const K2: f64 = 0.015;

//     let c1 = (a1 * a1 + b1 * b1).sqrt();
//     let c2 = (a2 * a2 + b2 * b2).sqrt();
//     let dl = l1 - l2;
//     let dc = c1 - c2;
//     let da = a1 - a2;
//     let db = b1 - b2;
//     let dh = (da * da + db * db - dc * dc).sqrt();

//     let t1 = dl / KL;
//     let t2 = dc / (1.0 + K1 * c1);
//     let t3 = dh / (1.0 + K2 * c1);

//     return (t1 * t1 + t2 * t2 + t3 * t3).sqrt();
// }

// ///rewrite of the xterm2rgb method. Converts an XTerm Color code to the respective RGB
// /// requires mut ref to the array where RGB data should be placed
// /// Also array must be 3-usize because doy
// pub fn xterm2rgb(color: u8, rgb: [&mut u8; 3]) {
//     match color < 232 {
//         true => {
//             let color_mod = color - 16;
//             *rgb[0] = VALUERANGE[((color_mod / 36) % 6) as usize];
//             *rgb[1] = VALUERANGE[((color_mod / 6) % 6) as usize];
//             *rgb[2] = VALUERANGE[(color_mod % 6) as usize];
//         }
//         false => {
//             *rgb[0] = 8 + (color - 232) * 10;
//             *rgb[1] = 8 + (color - 232) * 10;
//             *rgb[2] = 8 + (color - 232) * 10;
//         }
//     }
// }

// ///rewrite of the rgb2xterm_cie94 method. probably does the cie94 function but on xterm colors?
// /// also returns u32? what were they cooking?
// pub fn rgb2xterm_cie94(r: u8, g: u8, b: u8) -> u32 {
//     let mut i: u32 = 16;
//     let mut ret: u32 = 0;

//     let mut d: f64 = f64::INFINITY;
//     let mut smallest_distance: f64 = f64::INFINITY;

//     let mut l: f64 = 0.0;
//     let mut aa: f64 = 0.0;
//     let mut bb: f64 = 0.0;

//     srgb2lab(r, g, b, &mut l, &mut aa, &mut bb);

//     while i < 256 {}

//     // ret
//     todo!()
// }

//TODO since png crate does the decoding for us, idk if we need to do color conversion

pub fn rgbify_buffer(buf: &[u8], info: png::Info<'_>) -> Vec<Rgb<u8>> {
    match info.color_type {
        png::ColorType::Rgb => {
            //direct rgb conversion in theory
            Vec::from(buf.as_rgb())
        }
        png::ColorType::Rgba => buf
            .as_rgba()
            .into_iter()
            .map(|item| Rgb::new(item.r, item.g, item.b))
            .collect(),
        png::ColorType::Grayscale => {
            let mut vec = Vec::with_capacity(buf.len() * 3);
            for g in buf {
                vec.push(Rgb::new(g.clone(), g.clone(), g.clone()));
            }
            vec
        }
        png::ColorType::GrayscaleAlpha => {
            let mut vec = Vec::with_capacity(buf.len() * 3);
            for ga in buf.chunks(2) {
                let g = ga[0];
                // let a = ga[1];
                vec.push(Rgb::new(g.clone(), g.clone(), g.clone()));
            }
            vec
        }
        _ => unreachable!("uncovered color type"),
    }
}

pub fn to_cow_string(pixels: &[RGB8]) -> String{
    let mut string = String::from("$the_cow = @\"\n");

    for pixel in pixels {
        string.push(TerminalCharacter::TerminalForegroundColorTruecolor(
            pixel.r, pixel.g, pixel.b,
        ));
    }

    string
}
