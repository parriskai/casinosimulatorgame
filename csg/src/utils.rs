use nalgebra::{Matrix4, Vector3};

/// Converts from a cpu type to the raw gpu type
pub trait IntoGPUMatrix {
    type RAW;
    fn into_gmat(&self) -> Self::RAW;
}

// nalgebra impl
impl<T: Copy> IntoGPUMatrix for Matrix4<T> {
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

/// Basic Transformation type, shortcuts dealing with the most basic nalgebra ops
#[derive(Clone, Copy)]
pub enum Transform{
    /// Do nothing
    Identity,
    /// Move (x,y,z)
    Translate(f32, f32, f32),
    /// Scale (x,y,z)
    Scale(f32, f32, f32), 
    /// General type for other things (compound ops, rotation, etc)
    Complex(nalgebra::Matrix4<f32>)
} impl Transform{
    /// Converts to a nalgebra matrix
    pub fn as_matrix(&self) -> Matrix4<f32>{
        match self {
            Transform::Identity => Matrix4::identity(),

            Transform::Translate(x, y, z) => Matrix4::new(
                1., 0., 0., *x,
                0., 1., 0., *y,
                0., 0., 1., *z,
                0., 0., 0., 1.),
            Transform::Scale(x, y, z) => Matrix4::new(
                *x,0., 0., 0.,
                0.,*y, 0., 0.,
                0., 0.,*z, 0.,
                0., 0.,0., 1.),
            Transform::Complex(m) => m.clone()
        }
    }
    /// Chain a second transformation
    pub fn then(self, next: Transform) -> Self{
        match (self, next) {
            (Transform::Identity, n) => n,
            (s, Transform::Identity) => s,
            (Transform::Translate(x1, y1, z1), Transform::Translate(x2, y2, z2)) => {Transform::Translate(x1 + x2, y1 + y2, z1 + z2)},
            (Transform::Scale(x1, y1, z1), Transform::Scale(x2, y2, z2)) => {Transform::Scale(x1 * x2, y1 * y2, z1 * z2)},
            (s, n) => Transform::Complex(n.as_matrix() * s.as_matrix())
        }
    }

    /// The back most layer
    pub fn flatten_to_back() -> Self{
        Transform::Scale(1., 1., 0.).then(Transform::Translate(0., 0., 1.))
    }
} impl IntoGPUMatrix for Transform{
    type RAW = <Matrix4<f32> as IntoGPUMatrix>::RAW;

    fn into_gmat(&self) -> Self::RAW {self.as_matrix().into_gmat()}
}

pub fn coordinate_transform(src_min: Vector3<f32>, src_max: Vector3<f32>, dst_min: Vector3<f32>, dst_max: Vector3<f32>) -> Matrix4<f32> {
    let scale = (dst_max - dst_min).component_div(&(src_max - src_min));

    Matrix4::new_translation(&dst_min)
        * Matrix4::new_nonuniform_scaling(&scale)
        * Matrix4::new_translation(&-src_min)
}
