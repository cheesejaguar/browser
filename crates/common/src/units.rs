//! CSS units and length values.

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

/// An absolute length in CSS pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Px(pub f32);

impl Px {
    pub const ZERO: Px = Px(0.0);

    #[inline]
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    #[inline]
    pub fn get(self) -> f32 {
        self.0
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }

    #[inline]
    pub fn max(self, other: Px) -> Px {
        Px(self.0.max(other.0))
    }

    #[inline]
    pub fn min(self, other: Px) -> Px {
        Px(self.0.min(other.0))
    }

    #[inline]
    pub fn clamp(self, min: Px, max: Px) -> Px {
        Px(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub fn abs(self) -> Px {
        Px(self.0.abs())
    }
}

impl Add for Px {
    type Output = Px;
    fn add(self, rhs: Px) -> Px {
        Px(self.0 + rhs.0)
    }
}

impl Sub for Px {
    type Output = Px;
    fn sub(self, rhs: Px) -> Px {
        Px(self.0 - rhs.0)
    }
}

impl Mul<f32> for Px {
    type Output = Px;
    fn mul(self, rhs: f32) -> Px {
        Px(self.0 * rhs)
    }
}

impl Neg for Px {
    type Output = Px;
    fn neg(self) -> Px {
        Px(-self.0)
    }
}

impl fmt::Display for Px {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}px", self.0)
    }
}

/// A percentage value (0-100).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Percentage(pub f32);

impl Percentage {
    pub const ZERO: Percentage = Percentage(0.0);
    pub const HUNDRED: Percentage = Percentage(100.0);

    #[inline]
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    #[inline]
    pub fn get(self) -> f32 {
        self.0
    }

    /// Get as a unit fraction (0.0 - 1.0).
    #[inline]
    pub fn as_fraction(self) -> f32 {
        self.0 / 100.0
    }

    /// Apply percentage to a value.
    #[inline]
    pub fn of(self, value: f32) -> f32 {
        value * self.as_fraction()
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

/// A CSS length value.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Length {
    /// Absolute pixels.
    Px(f32),
    /// Relative to font-size of element.
    Em(f32),
    /// Relative to font-size of root element.
    Rem(f32),
    /// Viewport width percentage.
    Vw(f32),
    /// Viewport height percentage.
    Vh(f32),
    /// Viewport minimum dimension percentage.
    Vmin(f32),
    /// Viewport maximum dimension percentage.
    Vmax(f32),
    /// Physical centimeters.
    Cm(f32),
    /// Physical millimeters.
    Mm(f32),
    /// Physical inches.
    In(f32),
    /// Points (1/72 inch).
    Pt(f32),
    /// Picas (12 points).
    Pc(f32),
    /// Character width (width of '0').
    Ch(f32),
    /// x-height of font.
    Ex(f32),
    /// Line height.
    Lh(f32),
    /// Auto value.
    Auto,
    /// Zero.
    Zero,
}

impl Default for Length {
    fn default() -> Self {
        Length::Zero
    }
}

impl Length {
    /// Convert to pixels given context.
    pub fn to_px(&self, context: &LengthContext) -> f32 {
        match self {
            Length::Px(v) => *v,
            Length::Em(v) => v * context.font_size,
            Length::Rem(v) => v * context.root_font_size,
            Length::Vw(v) => v * context.viewport_width / 100.0,
            Length::Vh(v) => v * context.viewport_height / 100.0,
            Length::Vmin(v) => v * context.viewport_width.min(context.viewport_height) / 100.0,
            Length::Vmax(v) => v * context.viewport_width.max(context.viewport_height) / 100.0,
            Length::Cm(v) => v * 96.0 / 2.54,
            Length::Mm(v) => v * 96.0 / 25.4,
            Length::In(v) => v * 96.0,
            Length::Pt(v) => v * 96.0 / 72.0,
            Length::Pc(v) => v * 96.0 / 6.0,
            Length::Ch(v) => v * context.ch_width,
            Length::Ex(v) => v * context.ex_height,
            Length::Lh(v) => v * context.line_height,
            Length::Auto => 0.0,
            Length::Zero => 0.0,
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Length::Auto)
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Length::Zero => true,
            Length::Px(v)
            | Length::Em(v)
            | Length::Rem(v)
            | Length::Vw(v)
            | Length::Vh(v)
            | Length::Vmin(v)
            | Length::Vmax(v)
            | Length::Cm(v)
            | Length::Mm(v)
            | Length::In(v)
            | Length::Pt(v)
            | Length::Pc(v)
            | Length::Ch(v)
            | Length::Ex(v)
            | Length::Lh(v) => *v == 0.0,
            Length::Auto => false,
        }
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Length::Px(v) => write!(f, "{}px", v),
            Length::Em(v) => write!(f, "{}em", v),
            Length::Rem(v) => write!(f, "{}rem", v),
            Length::Vw(v) => write!(f, "{}vw", v),
            Length::Vh(v) => write!(f, "{}vh", v),
            Length::Vmin(v) => write!(f, "{}vmin", v),
            Length::Vmax(v) => write!(f, "{}vmax", v),
            Length::Cm(v) => write!(f, "{}cm", v),
            Length::Mm(v) => write!(f, "{}mm", v),
            Length::In(v) => write!(f, "{}in", v),
            Length::Pt(v) => write!(f, "{}pt", v),
            Length::Pc(v) => write!(f, "{}pc", v),
            Length::Ch(v) => write!(f, "{}ch", v),
            Length::Ex(v) => write!(f, "{}ex", v),
            Length::Lh(v) => write!(f, "{}lh", v),
            Length::Auto => write!(f, "auto"),
            Length::Zero => write!(f, "0"),
        }
    }
}

/// Length or percentage.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LengthPercentage {
    Length(Length),
    Percentage(Percentage),
}

impl Default for LengthPercentage {
    fn default() -> Self {
        LengthPercentage::Length(Length::Zero)
    }
}

impl LengthPercentage {
    pub fn px(value: f32) -> Self {
        LengthPercentage::Length(Length::Px(value))
    }

    pub fn percent(value: f32) -> Self {
        LengthPercentage::Percentage(Percentage::new(value))
    }

    pub fn auto() -> Self {
        LengthPercentage::Length(Length::Auto)
    }

    pub fn zero() -> Self {
        LengthPercentage::Length(Length::Zero)
    }

    /// Convert to pixels given context and containing block size.
    pub fn to_px(&self, context: &LengthContext, containing_size: f32) -> f32 {
        match self {
            LengthPercentage::Length(len) => len.to_px(context),
            LengthPercentage::Percentage(pct) => pct.of(containing_size),
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, LengthPercentage::Length(Length::Auto))
    }

    pub fn is_zero(&self) -> bool {
        match self {
            LengthPercentage::Length(len) => len.is_zero(),
            LengthPercentage::Percentage(pct) => pct.0 == 0.0,
        }
    }
}

impl fmt::Display for LengthPercentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LengthPercentage::Length(len) => write!(f, "{}", len),
            LengthPercentage::Percentage(pct) => write!(f, "{}", pct),
        }
    }
}

/// Context for resolving relative length units.
#[derive(Clone, Copy, Debug)]
pub struct LengthContext {
    pub font_size: f32,
    pub root_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub ch_width: f32,
    pub ex_height: f32,
    pub line_height: f32,
}

impl Default for LengthContext {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            root_font_size: 16.0,
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            ch_width: 8.0,
            ex_height: 8.0,
            line_height: 1.2 * 16.0,
        }
    }
}

/// CSS angle value.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Angle {
    Deg(f32),
    Rad(f32),
    Grad(f32),
    Turn(f32),
}

impl Angle {
    pub fn to_radians(&self) -> f32 {
        match self {
            Angle::Deg(v) => v.to_radians(),
            Angle::Rad(v) => *v,
            Angle::Grad(v) => v * std::f32::consts::PI / 200.0,
            Angle::Turn(v) => v * 2.0 * std::f32::consts::PI,
        }
    }

    pub fn to_degrees(&self) -> f32 {
        match self {
            Angle::Deg(v) => *v,
            Angle::Rad(v) => v.to_degrees(),
            Angle::Grad(v) => v * 0.9,
            Angle::Turn(v) => v * 360.0,
        }
    }
}

impl Default for Angle {
    fn default() -> Self {
        Angle::Deg(0.0)
    }
}

/// CSS time value.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Time {
    Seconds(f32),
    Milliseconds(f32),
}

impl Time {
    pub fn to_seconds(&self) -> f32 {
        match self {
            Time::Seconds(v) => *v,
            Time::Milliseconds(v) => v / 1000.0,
        }
    }

    pub fn to_milliseconds(&self) -> f32 {
        match self {
            Time::Seconds(v) => v * 1000.0,
            Time::Milliseconds(v) => *v,
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Time::Seconds(0.0)
    }
}

/// CSS resolution value.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Resolution {
    Dpi(f32),
    Dpcm(f32),
    Dppx(f32),
}

impl Resolution {
    pub fn to_dppx(&self) -> f32 {
        match self {
            Resolution::Dpi(v) => v / 96.0,
            Resolution::Dpcm(v) => v / (96.0 / 2.54),
            Resolution::Dppx(v) => *v,
        }
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution::Dppx(1.0)
    }
}
