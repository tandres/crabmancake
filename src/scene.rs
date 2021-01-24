use crate::key_state::KeyState;
use nalgebra::{Isometry3, Perspective3, Matrix4, Point3, Unit, UnitQuaternion, Vector3};

pub const FIELD_OF_VIEW: f32 = 45. * std::f32::consts::PI / 180.; //in radians
pub const Z_FAR: f32 = 1000.;
pub const Z_NEAR: f32 = 1.0;

const MAX_SPEED: f32 = 0.25;

#[derive(Clone, PartialEq)]
pub struct Scene {
    //TJATODO: Change these to vectors
    eye: Point3<f32>,
    target: Point3<f32>,
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
        let target = eye + look_dir;
        Self {
            eye, look_dir, look_dir_left, look_dir_up, width, height, target,
        }
    }

    pub fn look_at(&mut self, target: [f32; 3]) {
        let target = Point3::from(target);
        self.target = target;
        self.target_refresh();
    }

    fn target_refresh(&mut self) {
        let target = self.target;
        self.look_dir = Vector3::from(target - self.eye).normalize();
        self.look_dir_left = self.look_dir.cross(&Vector3::y());
        self.look_dir_up = self.look_dir.cross(&self.look_dir_left);
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

    pub fn scale(&mut self, amount: f32) {
        let adjusted_vector = Vector3::from(self.eye - self.target);
        let adjusted_vector: Vector3<f32> = Matrix4::new_scaling(amount).transform_vector(&adjusted_vector);
        let adjusted_vector = adjusted_vector + Vector3::new(self.target.x, self.target.y, self.target.z);
        self.move_absolute([adjusted_vector.x, adjusted_vector.y, adjusted_vector.z]);
    }

    pub fn move_absolute(&mut self, position: [f32; 3]) {
        self.eye = Point3::from(position);
        self.target_refresh();
    }

    pub fn strafe(&mut self, x: f32, y: f32) {
        let ud = y * self.look_dir_up;
        let lr = x * self.look_dir_left;
        let movement_vec = Vector3::from(ud + lr);

        self.eye += movement_vec;
        self.target += movement_vec;
    }

    pub fn rotate_2d_about_target(&mut self, x_rot: f32, y_rot: f32) {
        let relative_position = self.eye - self.target;
        let uq_x = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::y()), x_rot);
        let uq_y = UnitQuaternion::from_axis_angle(&Unit::new_normalize(self.look_dir_left), y_rot);
        let new_position : Vector3<f32> = (uq_y * uq_x * relative_position).xyz();
        let new_position : Vector3<f32> = new_position + Vector3::new(self.target.x, self.target.y, self.target.z);
        self.eye = Point3::from(new_position);
        self.look_at([self.target.x, self.target.y, self.target.z]);
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
}
