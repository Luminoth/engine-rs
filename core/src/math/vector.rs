use std::ops::*;

use serde::{Deserialize, Serialize};

// TODO: be cleaner to have a derive macro for this
pub trait Vector: Sub<Output = Self> + Div<f32, Output = Self> + DivAssign<f32> + Sized {
    fn magnitude(&self) -> f32 {
        self.magnitude_squared().sqrt()
    }

    fn magnitude_squared(&self) -> f32;

    fn normalize(&mut self) {
        *self /= self.magnitude();
    }

    fn normalized(self) -> Self {
        let mag = self.magnitude();
        self / mag
    }

    fn distance(self, other: Self) -> f32 {
        (other - self).magnitude()
    }

    fn distance_squared(self, other: Self) -> f32 {
        (other - self).magnitude()
    }

    fn dot(&self, other: &Self) -> f32;

    // TODO: fn lerp(&mut self, other: &Self, t: f32);
}

//#region Vector2

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    pub fn left() -> Self {
        Self { x: -1.0, y: 0.0 }
    }

    pub fn up() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    pub fn down() -> Self {
        Self { x: 0.0, y: -1.0 }
    }
}

impl Vector for Vector2 {
    fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl Neg for Vector2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Add<Vector2> for Vector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub<Vector2> for Vector2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign<Vector2> for Vector2 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul<f32> for Vector2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<f32> for Vector2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0.0 {
            return Self::default();
        }

        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<f32> for Vector2 {
    fn div_assign(&mut self, rhs: f32) {
        if rhs == 0.0 {
            *self = Self::default();
            return;
        }

        self.x /= rhs;
        self.y /= rhs;
    }
}

//#endregion

//#region Vector3

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn right() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn left() -> Self {
        Self {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn up() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    pub fn down() -> Self {
        Self {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        }
    }

    pub fn foward() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    pub fn back() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        }
    }

    // TODO: fn cross(&self, other: &Self) -> Self;
}

impl Vector for Vector3 {
    fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Add<Vector3> for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign<Vector3> for Vector3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub<Vector3> for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign<Vector3> for Vector3 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0.0 {
            return Self::default();
        }

        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl DivAssign<f32> for Vector3 {
    fn div_assign(&mut self, rhs: f32) {
        if rhs == 0.0 {
            *self = Self::default();
            return;
        }

        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

//#endregion

//#region Vector4

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector for Vector4 {
    fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
}

impl Neg for Vector4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl Add<Vector4> for Vector4 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl AddAssign<Vector4> for Vector4 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub<Vector4> for Vector4 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl SubAssign<Vector4> for Vector4 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul<f32> for Vector4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl MulAssign<f32> for Vector4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl Div<f32> for Vector4 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0.0 {
            return Self::default();
        }

        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

impl DivAssign<f32> for Vector4 {
    fn div_assign(&mut self, rhs: f32) {
        if rhs == 0.0 {
            *self = Self::default();
            return;
        }

        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self.w /= rhs;
    }
}

//#endregion
