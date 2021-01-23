use crate::key_state::KeyState;
use nalgebra::{Isometry3, Perspective3, Point3, Unit, UnitQuaternion, Vector3};

pub const FIELD_OF_VIEW: f32 = 45. * std::f32::consts::PI / 180.; //in radians
pub const Z_FAR: f32 = 1000.;
pub const Z_NEAR: f32 = 1.0;

const MAX_SPEED: f32 = 0.25;

#[derive(Clone, PartialEq)]
pub struct Scene {
    eye: Point3<f32>,
    look_dir: Vector3<f32>,
    look_dir_left: Vector3<f32>,
    look_dir_up: Vector3<f32>,
    width: f32,
    height: f32,
}

impl Scene {
    pub fn new(eye: [f32; 3], width: f32, height: f32) -> Self {
        let look_dir = Vector3::from([1.,0.,0.]);
        let look_dir_left = Vector3::from([0.,0.,1.]);
        let look_dir_up = Vector3::from([0.,1.,0.]);

        let eye = Point3::from(eye);
        Self {
            eye, look_dir, look_dir_left, look_dir_up, width, height,
        }
    }

    pub fn get_view_as_vec(&self) -> Vec<f32> {
        // log::info!("Looking at: ({:?})", self.look_dir);
        let target = Point3::from(self.eye + self.look_dir);
        let view = Isometry3::look_at_rh(&self.eye, &target, &Vector3::y());
        view.to_homogeneous().as_slice().to_vec()
    }

    pub fn get_eye_as_vec(&self) -> Vec<f32> {
        self.eye.coords.as_slice().to_vec()
    }

    pub fn get_projection_as_vec(&self) -> Vec<f32> {
        let aspect: f32 = self.width / self.height;
        let projection = Perspective3::new(aspect, FIELD_OF_VIEW, Z_NEAR, Z_FAR);
        projection.to_homogeneous().as_slice().to_vec()
    }

    pub fn move_relative(&mut self, offset: [f32; 3]) {
        let new_position = self.eye + Vector3::from(offset);
        self.eye = new_position;
    }

    #[allow(dead_code)]
    pub fn move_absolute(&mut self, position: [f32; 3]) {
        self.eye = Point3::from(position)
    }

    pub fn mouse_rotate(&mut self, rotations: [f32; 3]) {
        let sensi = 1. / 100.;
        let min_angle = f32::from(10.).to_radians();
        let max_angle = f32::from(170.).to_radians();
        let x_rot_angle = sensi * rotations[1];
        let y_rot_angle = sensi * rotations[0];
        let up = Vector3::y();
        let up_angle = self.look_dir.angle(&up);
        let x_rot_angle = if up_angle > max_angle && x_rot_angle.is_sign_negative() {
            0.
        } else if up_angle < min_angle && x_rot_angle.is_sign_positive() {
            0.
        } else {
            x_rot_angle
        };
        let uq_y = UnitQuaternion::from_axis_angle(&Unit::new_normalize(up), y_rot_angle);
        let uq_x = UnitQuaternion::from_axis_angle(&Unit::new_normalize(self.look_dir_left), x_rot_angle);
        self.look_dir = uq_y * uq_x * self.look_dir;
        //min and max are swapped here on purpose remember
        // self.look_dir.y = nalgebra::clamp(self.look_dir.y, max_angle.cos(), min_angle.cos());
        self.look_dir_left = self.look_dir.cross(&up);
        self.look_dir_up = self.look_dir.cross(&self.look_dir_left);
    }

    pub fn update_aspect(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn update_from_key_state(&mut self, key_state: &KeyState) {
        let fwbw = match (key_state.forward, key_state.backward) {
            (true, true) | (false, false) => 0.,
            (true, false) => 1.,
            (false, true) => -1.,
        };
        let lr = match (key_state.left, key_state.right) {
            (true, true) | (false, false) => 0.,
            (true, false) => -1.,
            (false, true) => 1.,
        };
        if fwbw == 0. && lr == 0. {
            return;
        }
        let fwbw : Vector3<f32> = fwbw * self.look_dir;
        let lr = lr * self.look_dir_left;
        let movement_vec = Vector3::from(fwbw + lr).normalize();
        let movement_vec = MAX_SPEED * movement_vec;
        self.move_relative([movement_vec.x, movement_vec.y, movement_vec.z]);
    }
}
