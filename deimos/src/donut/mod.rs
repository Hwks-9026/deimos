pub mod scene;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{donut, print};

lazy_static! {
    static ref DONUT_SCENE: Mutex<scene::Scene3d> =
        Mutex::new(scene::Scene3d {
            theta_1: 0.0,
            theta_2: 0.0,
            position: [0.0, 0.0, 0.0]
        });
}

pub fn redraw_donut(dir: scene::RotateDirection) {
    let mut donut = DONUT_SCENE.lock();

    donut.rotate(dir);
    donut.current_frame();
}
