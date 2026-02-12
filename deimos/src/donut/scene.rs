const PI: f64 = 3.1415926535897932;

const FRAC_PI_2: f64 = PI / 2.0;

use core::intrinsics;
use alloc::string::String;
use alloc::string::ToString;

use crate::hardware_interface::serial;
use crate::hardware_interface::vga_buffer::ScreenChar;
use crate::hardware_interface::vga_buffer::VGAWriter;
use crate::{print, println};
use crate::hardware_interface::vga_buffer::WRITER;

#[inline]
fn cos(val: f64) -> f64 {
    intrinsics::cosf64(val)
}

#[inline]
fn sin(val: f64) -> f64 {
    intrinsics::sinf64(val)
}

#[inline]
fn sqrt(val: f64) -> f64 {
    intrinsics::sqrtf64(val)
}

#[inline]
fn powf(val: f64, pow: f64) -> f64 {
    intrinsics::powf64(val, pow)
}

#[inline]
fn powi(val: f64, pow: i32) -> f64 {
    intrinsics::powif64(val, pow)
}


pub fn atan(mut x: f64) -> f64 {
    let negative = x < 0.0;
    if negative {
        x = -x;
    }

    let reciprocal = x > 1.0;
    if reciprocal {
        x = 1.0 / x;
    }
    
    //polynomial approximation
    let x2 = x * x;
    let mut res = x * (0.999999999998822
        + x2 * (-0.3333333324637782
        + x2 * (0.1999999557434313
        + x2 * (-0.1428562309199903
        + x2 * (0.11110022378413165
        + x2 * (-0.09081559103901618
        + x2 * (0.07548425255474668
        + x2 * (-0.05753069151608242
        + x2 * 0.024508930925243422))))))));

    if reciprocal {
        res = FRAC_PI_2 - res;
    }

    // 5. Restore sign
    if negative {
        -res
    } else {
        res
    }
}

pub(crate) enum RotateDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Default)]
pub(crate) struct Scene3d {
    pub(crate) theta_1: f64,
    pub(crate) theta_2: f64,
    pub(crate) position: [f64; 3],
}

const SCREEN_SIZE: (i32, i32) = (20, 12);
const GRADIENT: [char; 13] = [
    '.',  ':', '!', '-', '=', '+', '*', 'x', '%', '&', '$', '#', '@',
];
impl Scene3d {
    pub(crate) fn current_frame(&self) {
        let mut writer = WRITER.lock();
        writer.clear();
        for y in 0..(2*SCREEN_SIZE.1) {
            for x in (2*0..SCREEN_SIZE.0) {
                match calculate_sdf(
                    ((x - SCREEN_SIZE.0) as f64 / 37.5, (y - SCREEN_SIZE.1) as f64 / 37.5),
                    self.theta_1,
                    self.theta_2,
                ) {
                    Some(iterations) => {
                        let ch = match iterations {
                            1 | 2 => GRADIENT[12],
                            3 | 4 => GRADIENT[11],
                            5 | 6 => GRADIENT[10],
                            7 | 8 => GRADIENT[9],
                            9 | 10 => GRADIENT[8],
                            11 | 12 => GRADIENT[7],
                            13 | 14 => GRADIENT[6],
                            15 | 16 => GRADIENT[5],
                            17 | 18 => GRADIENT[4],
                            19 | 20 => GRADIENT[3],
                            21 | 22 => GRADIENT[2],
                            23 | 24 => GRADIENT[1],
                            _ => GRADIENT[0],
                        };
                        //print!("{}", ch);
                        writer.write_byte_at(ch as u8, (x as u8, y as u8));
                    }
                    None => {
                        //print!(" ");
                        writer.write_byte_at(' ' as u8, (x as u8, y as u8));
                    }
                }
            }
        }
    }

    pub(crate) fn rotate(&mut self, dir: RotateDirection) {
        match dir {
            RotateDirection::Left => {
                self.theta_1 += PI / 16.0;
            }
            RotateDirection::Right => {
                self.theta_1 -= PI / 16.0;
            }
            RotateDirection::Up => {
                self.theta_2 += PI / 16.0;
            }
            RotateDirection::Down => {
                self.theta_2 -= PI / 16.0;
            }
        }
    }
    pub(crate) fn translate(&mut self, pos: [f64; 3]) {
        self.position[0] += pos[0];
        self.position[1] += pos[1];
        self.position[2] += pos[2];
    }
}

const FOCAL_LENGTH: f64 = 3.0;
const SLIGHT_SCALE: f64 = 0.99;

fn calculate_sdf(
    screen_position: (f64, f64),
    theta_1: f64,
    theta_2: f64,
) -> Option<usize> {
    let mut ray_pos = [screen_position.0, screen_position.1, -10.0];
    let mut distance = torus_y(ray_pos, theta_1, theta_2);//new_sdf(ray_pos, theta_1, theta_2);
    let mut ray = set_magnitude(
        [screen_position.0, screen_position.1, FOCAL_LENGTH],
        distance * SLIGHT_SCALE,
    );
    let mut iters: usize = 1;
    loop {
        if distance > 25.0 {
            return None;
        }
        if distance < 1.0 / 10.0 {
            return Some(iters);
        }
        iters += 1;
        ray_pos = [
            ray_pos[0] + ray[0],
            ray_pos[1] + ray[1],
            ray_pos[2] + ray[2],
        ];
        distance = torus_y(ray_pos, theta_1, theta_2); //new_sdf(ray_pos, theta_1, theta_2);
        ray = set_magnitude(
            [screen_position.0, screen_position.1, FOCAL_LENGTH],
            distance * SLIGHT_SCALE,
        );
    }
}

fn set_magnitude(vec: [f64; 3], magnitude: f64) -> [f64; 3] {
    if magnitude == 0.0 {
        return [0.0, 0.0, 0.0];
    } else {
        let vec_magnitude = powi(vec[0], 2) + powi(vec[1], 2) + sqrt(powi(vec[2], 2));
        let factor = magnitude / vec_magnitude;
        return [vec[0] * factor, vec[1] * factor, vec[2] * factor];
    }
}

fn compute_light(ray: [f64; 3], pos: [f64; 3], steps: usize, theta_y: f64, theta_x: f64) -> usize {
    let ray = set_magnitude(ray, -1.0);
    let light: [f64; 3] = [(0.0 - pos[0]), 0.0 - pos[1], 0.0 - pos[2]];

    let transformed_pos = space_transform(pos, theta_y, theta_x);
    let two_d_pos: [f64; 2] = [
        1.40 - powf(transformed_pos[0] * transformed_pos[0] + transformed_pos[1] * transformed_pos[1], 0.5),
        transformed_pos[2],
    ];
    let theta = atan(transformed_pos[0] / transformed_pos[1]);
    let phi = (two_d_pos[1] / two_d_pos[0]);
    let out_or_in = two_d_pos[0].signum();

    let normal_vector = set_magnitude([cos(theta), sin(theta), sin(phi)], out_or_in);

    let dot_product_1 =
        normal_vector[0] * light[0] + normal_vector[1] * light[1] + normal_vector[2] * light[2];
    let dot_product_2 =
        normal_vector[0] * ray[0] + normal_vector[1] * ray[1] + normal_vector[2] * ray[2];

    let new_steps = steps as f64 * dot_product_1 * dot_product_2 * 1.0;

    new_steps as usize
}


fn space_transform(pos: [f64; 3], theta_y: f64, theta_x: f64) -> [f64; 3] {
    let (x, y, z) = (pos[0], pos[1], pos[2]);

    let (cos_y, sin_y) = (cos(theta_y), sin(theta_y));
    let (cos_x, sin_x) = (cos(theta_x), sin(theta_x));

    // Y-axis inverse
    let x1 = x * cos_y + z * sin_y;
    let z1 = -x * sin_y + z * cos_y;
    let y1 = y;

    // X-axis inverse
    let y2 = y1 * cos_x + z1 * sin_x;
    let z2 = -y1 * sin_x + z1 * cos_x;

    let p = [x1, y2, z2];
    p
}

fn torus_y(ray_pos: [f64; 3], theta_y: f64, theta_x: f64) -> f64 {
    let p = space_transform(ray_pos, theta_y, theta_x);
    let r_o = 1.40;
    let r_i = 0.5;
    let q1 = sqrt(p[0] * p[0] + p[2] * p[2]) - r_o;
    let q2 = p[1];
    return sqrt(q1 * q1 + q2 * q2) - r_i;
}


