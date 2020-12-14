use std::sync::Arc;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref APP_STATE: Mutex<Arc<AppState>> = Mutex::new(Arc::new(AppState::new()));
}

pub fn update(time: f32) -> f32 {

    let mut data = APP_STATE.lock().unwrap();
    let delta_t = time - data.time;

    *data = Arc::new(AppState {
        time: time,
        ..*data.clone()
    });
    delta_t
}

pub fn get_curr() -> Arc<AppState> {
    APP_STATE.lock().unwrap().clone()
}

pub struct AppState {
    pub mouse_down: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub rotation_x_axis: f32,
    pub rotation_y_axis: f32,
    pub time: f32,
    pub rotations: [f64; 3],
    pub limit: f32,
    pub light_location: [f32; 3],
}

impl AppState {
    fn new() -> Self {
        Self {
            mouse_down: false,
            mouse_x: -1.,
            mouse_y: -1.,
            rotation_x_axis: -0.5,
            rotation_y_axis: -0.5,
            time: 0.,
            rotations: [0.; 3],
            limit: 175.,
            light_location: [0.,2.,0.],
        }
    }
}

pub fn update_shape_rotation(index: usize, value: f64) {
    let mut data = APP_STATE.lock().unwrap();
    let mut rotations = data.rotations.clone();
    rotations[index] = value;
    *data = Arc::new(AppState {
        rotations,
        ..*data.clone()
    });
}

pub fn update_limit(value: f64) {
    let mut data = APP_STATE.lock().unwrap();
    let limit = value as f32;
    *data = Arc::new(AppState {
        limit,
        ..*data.clone()
    });
}

pub fn update_light_location(index: usize, value: f64) {
    let mut data = APP_STATE.lock().unwrap();
    let mut light_location = data.light_location.clone();
    light_location[index] = value as f32;
    log::info!("Light location: {}, {}, {}", light_location[0], light_location[1], light_location[2]);
    *data = Arc::new(AppState {
        light_location,
        ..*data.clone()
    });
}
