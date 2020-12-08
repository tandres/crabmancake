use nalgebra::{Isometry3, Matrix3x1, Perspective3, Point3, Unit, UnitQuaternion, Vector3};

pub const FIELD_OF_VIEW: f32 = 45. * std::f32::consts::PI / 180.; //in radians
pub const Z_FAR: f32 = 1000.;
pub const Z_NEAR: f32 = 1.0;

#[derive(Clone)]
pub struct Scene {
    eye: Point3<f32>,
    look_dir: Vector3<f32>,
    width: f32,
    height: f32,
}

impl Scene {
    pub fn new(look_dir: [f32; 3], eye: [f32; 3], width: f32, height: f32) -> Self {
        let look_dir = Vector3::from(look_dir);
        let eye = Point3::from(eye);
        Self {
            eye, look_dir, width, height,
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
        self.eye += Vector3::from(offset)
    }

    pub fn move_absolute(&mut self, position: [f32; 3]) {
        self.eye = Point3::from(position)
    }

    pub fn mouse_rotate(&mut self, rotations: [f32; 3]) {
        let sensi = 1. / 100.;
        let x_rot_angle = sensi * rotations[1];
        let y_rot_angle = sensi * rotations[0];
        let x_axis = self.look_dir.cross(&Vector3::y());
        let uq_y = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::y()), y_rot_angle);
        let uq_x = UnitQuaternion::from_axis_angle(&Unit::new_normalize(x_axis), x_rot_angle);
        // let rot = Isometry3::rotation(Vector3::from(rotations));
        self.look_dir = uq_y * uq_x * self.look_dir;
    }

    pub fn update_aspect(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}
