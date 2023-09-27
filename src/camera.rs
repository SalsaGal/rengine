use glam::{Mat4, Vec2, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Camera {
    pub translation: Vec2,
}

impl From<&Camera> for Mat4 {
    fn from(value: &Camera) -> Self {
        Self::look_to_rh(value.translation.extend(0.0), Vec3::NEG_Z, Vec3::Y)
    }
}
