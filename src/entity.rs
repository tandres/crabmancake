use nalgebra::Vector3;

pub struct Entity {
    pub location: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub rotation_rate: Vector3<f32>,
}

impl Entity {
    pub fn new(loc: Vector3<f32>, rot: Vector3<f32>, vel: Vector3<f32>, rot_rate: Vector3<f32>) -> Self {
        Entity {
            location: loc,
            rotation: rot,
            velocity: vel,
            rotation_rate: rot_rate,
        }
    }
    #[allow(unused)]
    pub fn new_stationary() -> Self {
        Entity::new(Vector3::zeros(), Vector3::zeros(), Vector3::zeros(), Vector3::zeros())
    }

    pub fn new_at(loc: Vector3<f32>) -> Self {
        Entity::new(loc, Vector3::zeros(), Vector3::zeros(), Vector3::zeros())
    }
}

pub fn set_rotation(entity: &mut Entity, new_rot: Vector3<f32>) {
    entity.rotation = new_rot;
}

#[allow(unused)]
pub fn set_rot_rate(entity: &mut Entity, new_rate: Vector3<f32>) {
    entity.rotation_rate = new_rate;
}

pub fn update(entity: &mut Entity, elapsed: f32) {
    let elapsed = elapsed / 1000.;
    let delta_loc = entity.velocity * elapsed;
    entity.location = delta_loc + entity.location;
    let delta_rot = entity.rotation_rate * elapsed;
    entity.rotation = delta_rot + entity.rotation;
}
