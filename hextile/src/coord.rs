use std::fmt;
use std::ops::{Add, Sub, Mul, Div};
use std::cmp::max;


pub static ZERO_PIXEL: Pixel = Pixel{ x: 0, y: 0 };

pub static HEX_SIZE: Pixel = Pixel{ x: 90, y: 80 };

pub static TILE_IMG_CENTER: Pixel = Pixel{ x: 90,  y: 80 };
pub static TILE_IMG_SIZE:   Pixel = Pixel{ x: 180, y: 160 };


pub static HEX_NEIGHBORS: [Hex; 6] = [
    Hex{ p: 0, q: 1, r: -1 },
    Hex{ p: 1, q: 0, r: -1 },
    Hex{ p: 1, q: -1, r: 0 },
    Hex{ p: 0, q: -1, r: 1 },
    Hex{ p: -1, q: 0, r: 1 },
    Hex{ p: -1, q: 1, r: 0 }];

static HEX_P_COORD: Pixel = Pixel{ x: 90, y: -40 };
static HEX_Q_COORD: Pixel = Pixel{ x: 0,  y: -80 };




#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    pub x: isize,
    pub y: isize
}
impl Pixel{
    pub fn new(x: isize, y: isize) -> Pixel {
        Pixel{ x: x, y: y }
    }

    pub fn to_hex(&self, origin: Pixel) -> Hex {
        let px = *self - origin;

        let (fx, fy) = (px.x as f64, px.y as f64);
        let (fp, fq) = (fx/90.0, -fx/180.0 - fy/80.0);
        let fr = -fp-fq;

        Hex::round(fp, fq, fr)
    }
}
impl Add<Pixel> for Pixel {
    type Output = Pixel;
    fn add(self, other: Pixel) -> Pixel {
        Pixel::new(self.x + other.x, self.y + other.y)
    }
}
impl Sub<Pixel> for Pixel {
    type Output = Pixel;
    fn sub(self, other: Pixel) -> Pixel {
        Pixel::new(self.x - other.x, self.y - other.y)
    }
}
impl Mul<isize> for Pixel {
    type Output = Pixel;
    fn mul(self, scalar: isize) -> Pixel {
        Pixel::new(self.x * scalar, self.y * scalar)
    }
}
impl Div<isize> for Pixel {
    type Output = Pixel;
    fn div(self, scalar: isize) -> Pixel {
        Pixel::new(self.x / scalar, self.y / scalar)
    }
}
impl fmt::Display for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "px({}, {})", self.x, self.y)
    }
}
impl fmt::Debug for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Hex {
    pub p: isize,
    pub q: isize,
    pub r: isize
}
impl Hex {
    pub fn new(p: isize, q: isize) -> Hex {
        Hex{ p: p, q: q, r: -p-q }
    }

    pub fn range(radius: usize) -> Vec<Hex> {
        let radius = radius as isize;
        let mut hexes = vec!();
        for p in -radius..radius+1 {
            for q in -radius..radius+1 {
                let r = -p-q;
                if r >= -radius && r <= radius {
                    hexes.push(Hex::new(p, q));
                }
            }
        }
        hexes
    }

    fn round(fp: f64, fq: f64, fr: f64) -> Hex {
        let mut p = fp.round();
        let mut q = fq.round();
        let mut r = fr.round();

        let dp = (p - fp).abs();
        let dq = (q - fq).abs();
        let dr = (r - fr).abs();
        
        if dp > dq && dp > dr {
            p = -q-r;
        } else if dq > dr {
            q = -p-r;
        } else {
            r = -p-q;
        }

        Hex{ p: p as isize,
             q: q as isize,
             r: r as isize }
    }

    pub fn to_pixel(&self, offset: Pixel) -> Pixel {
        HEX_P_COORD * self.p + HEX_Q_COORD * self.q + offset
    }

    pub fn max_coord(&self) -> usize {
        max(max(self.p.abs() as usize, self.q.abs() as usize), self.r.abs() as usize)
    }
}
impl Add<Hex> for Hex {
    type Output = Hex;
    fn add(self, other: Hex) -> Hex {
        Hex::new(self.p + other.p, self.q + other.q)
    }
}
impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hex({}, {}, {})", self.p, self.q, self.r)
    }
}
impl fmt::Debug for Hex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
