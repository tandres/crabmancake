use nalgebra::Vector3;

pub struct Attenuator {
    val: [f32; 3],
}

impl Attenuator {
    pub fn new(constant: f32, linear: f32, quadratic: f32) -> Self {
        let val: [f32; 3] = [constant, linear, quadratic];
        Self { val }
    }

    pub fn new_7m() -> Self {
        Self::new(1.0, 0.7, 1.8)
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.val
    }
}

pub struct Light {
    pub color: Vector3<f32>,
    pub location: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub target: Vector3<f32>,
    pub inner_limit: f32,
    pub outer_limit: f32,
    pub intensity: f32,
    pub attenuator: Attenuator,
}

impl Light {
    pub fn new_point(location: [f32; 3], color: [f32; 3], intensity: f32, attenuator: Attenuator) -> Self {
        Self::new_spot(location, [0.; 3], color, 180.0, 180.0, intensity, attenuator)
    }

    pub fn new_spot(location: [f32; 3], pointing_at: [f32; 3], color: [f32; 3], inner_limit: f32, outer_limit: f32, intensity: f32, attenuator: Attenuator) -> Self {
        let location = Vector3::from(location);
        let target = Vector3::from(pointing_at);
        let direction = target - location;
        let color = Vector3::from(color);
        let outer_limit = f32::cos(std::f32::consts::PI * outer_limit / 180.);
        let inner_limit = f32::cos(std::f32::consts::PI * inner_limit / 180.);
        Light { location, color, direction, target, inner_limit, outer_limit, intensity, attenuator }
    }

    pub fn set_location(&mut self, location: [f32; 3]) {
        self.location = Vector3::from(location);
        self.direction = self.target - self.location;
    }
}
