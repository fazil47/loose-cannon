use bevy::prelude::{Mat3, Quat, Transform, Vec3};

pub trait TransformExt {
    /// Rotates this [`Transform`] so that its local negative `Y` direction is toward
    /// `target` and its local negative `Z` direction is toward `forward`.
    fn set_down(&mut self, target: Vec3, forward: Vec3);
}

impl TransformExt for Transform {
    fn set_down(&mut self, target: Vec3, forward: Vec3) {
        let up = Vec3::normalize(self.translation - target);
        let right = up.cross(forward).normalize();
        let forward = right.cross(up).normalize();

        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));
    }
}
