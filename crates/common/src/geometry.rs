//! Geometric primitives.

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};

/// A 2D point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn distance(&self, other: Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    #[inline]
    pub fn lerp(&self, other: Point, t: f32) -> Point {
        Point::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, rhs: f32) -> Point {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

/// A 2D size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size { width: 0.0, height: 0.0 };

    #[inline]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    #[inline]
    pub fn contains(&self, other: Size) -> bool {
        self.width >= other.width && self.height >= other.height
    }
}

/// A 2D rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Rect = Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    #[inline]
    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Self {
            x: origin.x,
            y: origin.y,
            width: size.width,
            height: size.height,
        }
    }

    #[inline]
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p1.x - p2.x).abs();
        let height = (p1.y - p2.y).abs();
        Self { x, y, width, height }
    }

    #[inline]
    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    #[inline]
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.y
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    #[inline]
    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    #[inline]
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    #[inline]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right > x && bottom > y {
            Some(Rect::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(x, y, right - x, bottom - y)
    }

    #[inline]
    pub fn translate(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    #[inline]
    pub fn inflate(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x - dx, self.y - dy, self.width + dx * 2.0, self.height + dy * 2.0)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    #[inline]
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Convert to integer pixel coordinates for rasterization.
    pub fn to_pixel_rect(&self) -> PixelRect {
        PixelRect {
            x: self.x.floor() as i32,
            y: self.y.floor() as i32,
            width: self.width.ceil() as u32,
            height: self.height.ceil() as u32,
        }
    }
}

/// Integer rectangle for pixel operations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    #[inline]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    #[inline]
    pub fn to_rect(&self) -> Rect {
        Rect::new(self.x as f32, self.y as f32, self.width as f32, self.height as f32)
    }
}

/// Edge sizes (for margin, padding, border).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeSizes {
    pub const ZERO: EdgeSizes = EdgeSizes {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    #[inline]
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    #[inline]
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    #[inline]
    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    #[inline]
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    #[inline]
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    #[inline]
    pub fn total_size(&self) -> Size {
        Size::new(self.horizontal(), self.vertical())
    }
}

impl Add for EdgeSizes {
    type Output = EdgeSizes;
    fn add(self, rhs: EdgeSizes) -> EdgeSizes {
        EdgeSizes::new(
            self.top + rhs.top,
            self.right + rhs.right,
            self.bottom + rhs.bottom,
            self.left + rhs.left,
        )
    }
}

/// Corner radii for rounded rectangles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    pub const ZERO: CornerRadii = CornerRadii {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };

    #[inline]
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }
}

/// A 2D affine transformation matrix.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub m31: f32,
    pub m32: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    pub const fn identity() -> Self {
        Self {
            m11: 1.0,
            m12: 0.0,
            m21: 0.0,
            m22: 1.0,
            m31: 0.0,
            m32: 0.0,
        }
    }

    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            m11: 1.0,
            m12: 0.0,
            m21: 0.0,
            m22: 1.0,
            m31: x,
            m32: y,
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            m11: sx,
            m12: 0.0,
            m21: 0.0,
            m22: sy,
            m31: 0.0,
            m32: 0.0,
        }
    }

    pub fn rotation(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            m11: cos,
            m12: sin,
            m21: -sin,
            m22: cos,
            m31: 0.0,
            m32: 0.0,
        }
    }

    pub fn skew(sx: f32, sy: f32) -> Self {
        Self {
            m11: 1.0,
            m12: sy.tan(),
            m21: sx.tan(),
            m22: 1.0,
            m31: 0.0,
            m32: 0.0,
        }
    }

    pub fn then(&self, other: &Transform) -> Transform {
        Transform {
            m11: self.m11 * other.m11 + self.m12 * other.m21,
            m12: self.m11 * other.m12 + self.m12 * other.m22,
            m21: self.m21 * other.m11 + self.m22 * other.m21,
            m22: self.m21 * other.m12 + self.m22 * other.m22,
            m31: self.m31 * other.m11 + self.m32 * other.m21 + other.m31,
            m32: self.m31 * other.m12 + self.m32 * other.m22 + other.m32,
        }
    }

    pub fn transform_point(&self, point: Point) -> Point {
        Point::new(
            self.m11 * point.x + self.m21 * point.y + self.m31,
            self.m12 * point.x + self.m22 * point.y + self.m32,
        )
    }

    pub fn transform_rect(&self, rect: Rect) -> Rect {
        let p1 = self.transform_point(Point::new(rect.x, rect.y));
        let p2 = self.transform_point(Point::new(rect.right(), rect.y));
        let p3 = self.transform_point(Point::new(rect.x, rect.bottom()));
        let p4 = self.transform_point(Point::new(rect.right(), rect.bottom()));

        let min_x = p1.x.min(p2.x).min(p3.x).min(p4.x);
        let min_y = p1.y.min(p2.y).min(p3.y).min(p4.y);
        let max_x = p1.x.max(p2.x).max(p3.x).max(p4.x);
        let max_y = p1.y.max(p2.y).max(p3.y).max(p4.y);

        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn determinant(&self) -> f32 {
        self.m11 * self.m22 - self.m12 * self.m21
    }

    pub fn inverse(&self) -> Option<Transform> {
        let det = self.determinant();
        if det.abs() < f32::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Transform {
            m11: self.m22 * inv_det,
            m12: -self.m12 * inv_det,
            m21: -self.m21 * inv_det,
            m22: self.m11 * inv_det,
            m31: (self.m21 * self.m32 - self.m22 * self.m31) * inv_det,
            m32: (self.m12 * self.m31 - self.m11 * self.m32) * inv_det,
        })
    }

    pub fn is_identity(&self) -> bool {
        (self.m11 - 1.0).abs() < f32::EPSILON
            && self.m12.abs() < f32::EPSILON
            && self.m21.abs() < f32::EPSILON
            && (self.m22 - 1.0).abs() < f32::EPSILON
            && self.m31.abs() < f32::EPSILON
            && self.m32.abs() < f32::EPSILON
    }
}
