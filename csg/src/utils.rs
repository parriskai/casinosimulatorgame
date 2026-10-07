//! `csg::utils`
//! Common file for all utils that arent specific to one particular module

use nalgebra::{
    Matrix4,
    Vector3
};

/// Converts a CPU-side matrix type into a byte representation needed for passing to
/// GPU APIs.
/// 
/// This is primarily intended for types whose CPU representation differs from 
/// the layout expected by shaders or GPU buffer APIs. For example nalgebra's [`Matrix4`]
pub trait IntoGpuMatrix {
    /// The raw representation used when storing the value in a GPU buffer.
    /// Likely needs to implement [`bytemuck::Pod`] and [`bytemuck::Zeroable`]
    /// However this trait does not enforce that
    type Raw;

    /// Performs a conversion from this value into its GPU-compatable representation
    fn into_gmat(&self) -> Self::Raw;
}


/// Converts a nalgebra [`Matrix4`] of type T into a column-major 2D array.
/// This implementation requires the matrix element type to be [`Copy`].
impl<T: Copy> IntoGpuMatrix for Matrix4<T> {
    type Raw = [[T; 4]; 4];

    fn into_gmat(&self) -> Self::Raw {
        [
            [self[(0, 0)], self[(1, 0)], self[(2, 0)], self[(3, 0)]],
            [self[(0, 1)], self[(1, 1)], self[(2, 1)], self[(3, 1)]],
            [self[(0, 2)], self[(1, 2)], self[(2, 2)], self[(3, 2)]],
            [self[(0, 3)], self[(1, 3)], self[(2, 3)], self[(3, 3)]]
        ]
    }
}

/// Creates an affine transformation that maps one axis-aligned coordinate
///  range to another. 
/// 
/// The resulting matrix maps `src_min` to `dst_min` and `src_max` to 
/// `dst_max`, with points between them scaled linearly. 
/// 
/// The transformation consists of three operations:
/// 1. Translate the source range so that `src_min` is at the origin. 
/// 2. Scale each axis independently to match the destination range. 
/// 3. Translate the result so that the origin is at `dst_min`.
/// 
/// In mathematical terms, for a point `p`: 
///
/// ```text
/// p' = dst_min + (p - src_min) * (dst_max - dst_min) / (src_max - src_min)
/// ```
/// 
/// # Important Notes
/// This function does not explicitly check for zero-sized source dimensions.
/// A zero component in `src_max - src_min` results in division by zero and 
/// therefore produces an infinite or NaN scale factor.
///
/// # Examples
/// 
/// Mapping a unit cube to the range `[-1, 1]`:
/// 
/// ```
/// # use nalgebra::Vector3;
/// # use your_crate::coordinate_transform;
/// let transform = coordinate_transform(
///     Vector3::new(0.0, 0.0, 0.0),
///     Vector3::new(1.0, 1.0, 1.0),
///     Vector3::new(-1.0, -1.0, -1.0),
///     Vector3::new(1.0, 1.0, 1.0)
/// );
/// ```
/// 
/// [`Matrix4`]: nalgebra::Matrix4
/// [`Copy`]: std::marker::Copy
pub fn coordinate_transform(src_min: Vector3<f32>, src_max: Vector3<f32>, dst_min: Vector3<f32>, dst_max: Vector3<f32>) -> Matrix4<f32> {
    let scale = (dst_max - dst_min).component_div(&(src_max - src_min));

    Matrix4::new_translation(&dst_min)
        * Matrix4::new_nonuniform_scaling(&scale)
        * Matrix4::new_translation(&-src_min)
}


pub trait IntoPair<A, B>{
    fn into_pair(self) -> (A, B);
}

impl<A, B> IntoPair<A, B> for (A, B){
    fn into_pair(self) -> (A, B) {self}
}

impl <A> IntoPair<A, ()> for A{
    fn into_pair(self) -> (A, ()) {(self, ())}
}