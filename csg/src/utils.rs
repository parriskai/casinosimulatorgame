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

pub fn coordinate_transform(src_min: Vector3<f32>, src_max: Vector3<f32>, dst_min: Vector3<f32>, dst_max: Vector3<f32>) -> Matrix4<f32> {
    let scale = (dst_max - dst_min).component_div(&(src_max - src_min));

    Matrix4::new_translation(&dst_min)
        * Matrix4::new_nonuniform_scaling(&scale)
        * Matrix4::new_translation(&-src_min)
}
