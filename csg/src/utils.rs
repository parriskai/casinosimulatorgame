use nalgebra::Matrix4;

use crate::utils::Transform::Translate;

pub trait IntoGPUMatrix {
    type RAW;
    fn into_gmat(&self) -> Self::RAW;
} impl<T: Copy> IntoGPUMatrix for Matrix4<T> {
    type RAW = [[T; 4]; 4];

    fn into_gmat(&self) -> Self::RAW {
        [
            [self[(0, 0)], self[(1, 0)], self[(2, 0)], self[(3, 0)]],
            [self[(0, 1)], self[(1, 1)], self[(2, 1)], self[(3, 1)]],
            [self[(0, 2)], self[(1, 2)], self[(2, 2)], self[(3, 2)]],
            [self[(0, 3)], self[(1, 3)], self[(2, 3)], self[(3, 3)]]
        ]
    }
}

#[derive(Clone, Copy)]
pub enum Transform{
    Identity,
    Translate(f32, f32, f32),
    Scale(f32, f32, f32), 
    Complex(nalgebra::Matrix4<f32>)
} impl Transform{
    pub fn as_matrix(&self) -> nalgebra::Matrix4<f32>{
        match self {
            Transform::Identity => nalgebra::Matrix4::identity(),

            Transform::Translate(x, y, z) => nalgebra::Matrix4::new(
                1., 0., 0., *x,
                0., 1., 0., *y,
                0., 0., 1., *z,
                0., 0., 0., 1.),
            Transform::Scale(x, y, z) => nalgebra::Matrix4::new(
                *x,0., 0., 0.,
                0.,*y, 0., 0.,
                0., 0.,*z, 0.,
                0., 0.,0., 1.),
            Transform::Complex(m) => m.clone()
        }
    }
    pub fn then(self, next: Transform) -> Self{
        match (self, next) {
            (Transform::Identity, n) => n,
            (s, Transform::Identity) => s,
            (Transform::Translate(x1, y1, z1), Transform::Translate(x2, y2, z2)) => {Transform::Translate(x1 + x2, y1 + y2, z1 + z2)},
            (Transform::Scale(x1, y1, z1), Transform::Scale(x2, y2, z2)) => {Transform::Scale(x1 * x2, y1 * y2, z1 * z2)},
            (s, n) => Transform::Complex(n.as_matrix() * s.as_matrix())
        }
    }

    pub fn flatten_to_back() -> Self{
        Transform::Scale(1., 1., 0.).then(Transform::Translate(0., 0., 1.))
    }

    pub fn flatten_to_layer(layer: usize, total_layer_count: usize) -> Self{
        Transform::Scale(1., 1., 0.).then(Transform::Translate(0., 0., (layer as f32 /  total_layer_count as f32)))
    }
} impl IntoGPUMatrix for Transform{
    type RAW = <nalgebra::Matrix4<f32> as IntoGPUMatrix>::RAW;

    fn into_gmat(&self) -> Self::RAW {self.as_matrix().into_gmat()}
}