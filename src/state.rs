use std::sync::Arc;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref APP_STATE: Mutex<Arc<AppState>> = Mutex::new(Arc::new(AppState::new()));
}

pub fn update(time: f32, canvas_height: f32, canvas_width: f32) -> f32 {
    let min_height_width = canvas_height.min(canvas_width);
    let display_size = 0.9 * min_height_width;
    let half_display_size = display_size / 2.;
    let half_canvas_height = canvas_height / 2.;
    let half_canvas_width = canvas_width / 2.;

    let mut data = APP_STATE.lock().unwrap();
    let delta_t = time - data.time;

    *data = Arc::new(AppState {
        canvas_height: canvas_height,
        canvas_width: canvas_width,

        control_bottom: half_canvas_height - half_display_size,
        control_top: half_canvas_height + half_display_size,
        control_left: half_canvas_width - half_display_size,
        control_right: half_canvas_width + half_display_size,

        time: time,
        ..*data.clone()
    });
    delta_t
}

pub fn get_curr() -> Arc<AppState> {
    APP_STATE.lock().unwrap().clone()
}

pub struct AppState {
    pub canvas_height: f32,
    pub canvas_width: f32,
    pub control_bottom: f32,
    pub control_top: f32,
    pub control_left: f32,
    pub control_right: f32,
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
            canvas_height: 0.,
            canvas_width: 0.,
            control_bottom: 0.,
            control_top: 0.,
            control_left: 0.,
            control_right: 0.,
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

pub fn update_mouse_down(x: f32, y: f32, is_down: bool) {
    let mut data = APP_STATE.lock().unwrap();
    *data = Arc::new(AppState {
        mouse_down: is_down,
        mouse_x: x,
        mouse_y: data.canvas_height - y,
        ..*data.clone()
    });
}

pub fn update_mouse_position(x: f32, y: f32) {
    let mut data = APP_STATE.lock().unwrap();
    let inverted_y = data.canvas_height - y;
    let x_delta = x - data.mouse_x;
    let y_delta = inverted_y - data.mouse_y;
    let rotation_x_delta = if data.mouse_down {
        std::f32::consts::PI * y_delta / data.canvas_height
    } else {
        0.
    };
    let rotation_y_delta = if data.mouse_down {
        std::f32::consts::PI * x_delta / data.canvas_width
    } else {
        0.
    };

    *data = Arc::new(AppState {
        mouse_x: x,
        mouse_y: inverted_y,
        rotation_x_axis: data.rotation_x_axis + rotation_x_delta,
        rotation_y_axis: data.rotation_y_axis - rotation_y_delta,
        ..*data.clone()
    });
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
