/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![macro_use]

/// Implements a single unary operator for a vector type. Only used for `Neg` at the moment.
macro_rules! impl_vector_unary_operator {
    (
        // Name of the vector type.
        $Vector:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($components:ident),*),
        // Name of the operator trait, for example `Neg`.
        $Operator:ident,
        // Name of the function on the operator trait, for example `neg`.
        $func:ident
    ) => {
        impl std::ops::$Operator for $Vector {
            type Output = Self;
            fn $func(mut self) -> Self::Output {
                $(
                    self.$components = self.$components.$func();
                )*
                self
            }
        }
    }
}

/// Implements a component-wise single infix binary operator between two vectors.
macro_rules! impl_vector_vector_binary_operator {
    (
        // Name of the vector type.
        $Vector:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($components:ident),*),
        // Name of the operator trait, for example `Add`.
        $Operator:ident,
        // Name of the function on the operator trait, for example `add`.
        $func:ident
    ) => {
        impl std::ops::$Operator for $Vector {
            type Output = Self;
            fn $func(mut self, rhs: $Vector) -> Self::Output {
                $(
                    self.$components = self.$components.$func(rhs.$components);
                )*
                self
            }
        }
    }
}

/// Implements a component-wise single infix binary operator between a vector on the left and a
/// scalar on the right-hand side.
macro_rules! impl_vector_scalar_binary_operator {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of each individual component, for example `i32`.
        $Scalar:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($components:ident),*),
        // Name of the operator trait, for example `Add`.
        $Operator:ident,
        // Name of the function on the operator trait, for example `add`.
        $func:ident
    ) => {
        impl std::ops::$Operator<$Scalar> for $Vector {
            type Output = Self;
            fn $func(mut self, rhs: $Scalar) -> Self::Output {
                $(
                    self.$components = self.$components.$func(rhs);
                )*
                self
            }
        }
    }
}

/// Implements a component-wise single infix binary operator between a scalar on the left and a
/// vector on the right-hand side.
macro_rules! impl_scalar_vector_binary_operator {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of each individual component, for example `i32`.
        $Scalar:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($components:ident),*),
        // Name of the operator trait, for example `Add`.
        $Operator:ident,
        // Name of the function on the operator trait, for example `add`.
        $func:ident
    ) => {
        impl std::ops::$Operator<$Vector> for $Scalar {
            type Output = $Vector;
            fn $func(self, mut rhs: $Vector) -> Self::Output {
                $(
                    rhs.$components = rhs.$components.$func(self);
                )*
                rhs
            }
        }
    }
}

/// Implements a single arithmetic assignment operator for a vector type, with a vector on the
/// right-hand side.
macro_rules! impl_vector_vector_assign_operator {
    (
        // Name of the vector type.
        $Vector:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($components:ident),*),
        // Name of the operator trait, for example `AddAssign`.
        $Operator:ident,
        // Name of the function on the operator trait, for example `add_assign`.
        $func:ident
    ) => {
        impl std::ops::$Operator for $Vector {
            fn $func(&mut self, rhs: $Vector) {
                $(
                    self.$components.$func(rhs.$components);
                )*
            }
        }
    }
}

/// Implements a single arithmetic assignment operator for a vector type, with a scalar on the
/// right-hand side.
macro_rules! impl_vector_scalar_assign_operator {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of each individual component, for example `i32`.
        $Scalar:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($components:ident),*),
        // Name of the operator trait, for example `AddAssign`.
        $Operator:ident,
        // Name of the function on the operator trait, for example `add_assign`.
        $func:ident
    ) => {
        impl std::ops::$Operator<$Scalar> for $Vector {
            fn $func(&mut self, rhs: $Scalar) {
                $(
                    self.$components.$func(rhs);
                )*
            }
        }
    }
}

/// Implements a reduction (sum or product) over an iterator of vectors.
macro_rules! impl_iter_vector_reduction {
    (
        // Name of the vector type.
        $Vector:ty,
        // Name of the reduction trait: `Sum` or `Product`.
        $Operator:ident,
        // Name of the function on the operator trait, for example `add`.
        $func:ident
    ) => {
        impl std::iter::$Operator<Self> for $Vector {
            #[doc = concat!("Element-wise ", stringify!($func), " of all vectors in the iterator.")]
            fn $func<I>(iter: I) -> Self
            where
                I: Iterator<Item = Self>,
            {
                Self::from_glam(iter.map(Self::to_glam).$func())
            }
        }

        impl<'a> std::iter::$Operator<&'a Self> for $Vector {
            #[doc = concat!("Element-wise ", stringify!($func), " of all vectors in the iterator.")]
            fn $func<I>(iter: I) -> Self
            where
                I: Iterator<Item = &'a Self>,
            {
                Self::from_glam(iter.map(|x| Self::to_glam(*x)).$func())
            }
        }
    };
}

/// Implements all common arithmetic operators on a built-in vector type.
macro_rules! impl_vector_operators {
    (
        // Name of the vector type to be implemented, for example `Vector2`.
        $Vector:ty,
        // Type of each individual component, for example `real`.
        $Scalar:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($components:ident),*)
    ) => {
        impl_vector_unary_operator!($Vector, ($($components),*), Neg, neg);
        impl_vector_vector_binary_operator!($Vector, ($($components),*), Add, add);
        impl_vector_vector_binary_operator!($Vector, ($($components),*), Sub, sub);
        impl_vector_vector_binary_operator!($Vector, ($($components),*), Mul, mul);
        impl_vector_scalar_binary_operator!($Vector, $Scalar, ($($components),*), Mul, mul);
        impl_scalar_vector_binary_operator!($Vector, $Scalar, ($($components),*), Mul, mul);
        impl_vector_vector_binary_operator!($Vector, ($($components),*), Div, div);
        impl_vector_scalar_binary_operator!($Vector, $Scalar, ($($components),*), Div, div);
        impl_iter_vector_reduction!($Vector, Sum, sum);
        impl_iter_vector_reduction!($Vector, Product, product);
        impl_vector_vector_assign_operator!($Vector, ($($components),*), AddAssign, add_assign);
        impl_vector_vector_assign_operator!($Vector, ($($components),*), SubAssign, sub_assign);
        impl_vector_vector_assign_operator!($Vector, ($($components),*), MulAssign, mul_assign);
        impl_vector_scalar_assign_operator!($Vector, $Scalar, ($($components),*), MulAssign, mul_assign);
        impl_vector_vector_assign_operator!($Vector, ($($components),*), DivAssign, div_assign);
        impl_vector_scalar_assign_operator!($Vector, $Scalar, ($($components),*), DivAssign, div_assign);
    }
}

/// Implements `Index` and `IndexMut` for a vector type, using an enum to indicate the desired axis.
macro_rules! impl_vector_index {
    (
        // Name of the vector type to be implemented, for example `Vector2`.
        $Vector:ty,
        // Type of each individual component, for example `real`.
        $Scalar:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($( $components:ident ),*),
        // Name of the enum type for the axes, for example `Vector2Axis`.
        $AxisEnum:ty,
        // Names of the enum variants, with parenthes, for example `(X, Y)`.
        ($( $axis_variants:ident ),*)
    ) => {
        impl std::ops::Index<$AxisEnum> for $Vector {
            type Output = $Scalar;
            fn index(&self, axis: $AxisEnum) -> &$Scalar {
                match axis {
                    $(<$AxisEnum>::$axis_variants => &self.$components),*
                }
            }
        }

        impl std::ops::IndexMut<$AxisEnum> for $Vector {
            fn index_mut(&mut self, axis: $AxisEnum) -> &mut $Scalar {
                match axis {
                    $(<$AxisEnum>::$axis_variants => &mut self.$components),*
                }
            }
        }
    }
}

/// Implements constants that are present on floating-point and integer vectors.
macro_rules! impl_vector_consts {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of target component, for example `real`.
        $Scalar:ty
    ) => {
        impl $Vector {
            /// Zero vector, a vector with all components set to `0`.
            pub const ZERO: Self = Self::splat(0 as $Scalar);

            /// One vector, a vector with all components set to `1`.
            pub const ONE: Self = Self::splat(1 as $Scalar);
        }
    };
}

/// Implements constants that are present only on floating-point vectors.
macro_rules! impl_float_vector_consts {
    (
        // Name of the vector type.
        $Vector:ty
    ) => {
        impl $Vector {
            /// Infinity vector, a vector with all components set to `real::INFINITY`.
            pub const INF: Self = Self::splat(real::INFINITY);
        }
    };
}

/// Implements constants that are present only on integer vectors.
macro_rules! impl_integer_vector_consts {
    (
        // Name of the vector type.
        $Vector:ty
    ) => {
        impl $Vector {
            /// Min vector, a vector with all components equal to [`i32::MIN`]. Can be used as a negative integer equivalent of `real::INF`.
            pub const MIN: Self = Self::splat(i32::MIN);

            /// Max vector, a vector with all components equal to [`i32::MAX`]. Can be used as an integer equivalent of `real::INF`.
            pub const MAX: Self = Self::splat(i32::MAX);
        }
    };
}

/// Implements constants present on 2D vectors.
macro_rules! impl_vector2x_consts {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of target component, for example `real`.
        $Scalar:ty
    ) => {
        impl $Vector {
            /// Left unit vector. Represents the direction of left.
            pub const LEFT: Self = Self::new(-1 as $Scalar, 0 as $Scalar);

            /// Right unit vector. Represents the direction of right.
            pub const RIGHT: Self = Self::new(1 as $Scalar, 0 as $Scalar);

            /// Up unit vector. Y is down in 2D, so this vector points -Y.
            pub const UP: Self = Self::new(0 as $Scalar, -1 as $Scalar);

            /// Down unit vector. Y is down in 2D, so this vector points +Y.
            pub const DOWN: Self = Self::new(0 as $Scalar, 1 as $Scalar);
        }
    };
}

/// Implements constants present on 3D vectors.
macro_rules! impl_vector3x_consts {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of target component, for example `real`.
        $Scalar:ty
    ) => {
        impl $Vector {
            /// Unit vector in -X direction. Can be interpreted as left in an untransformed 3D world.
            pub const LEFT: Self = Self::new(-1 as $Scalar, 0 as $Scalar, 0 as $Scalar);

            /// Unit vector in +X direction. Can be interpreted as right in an untransformed 3D world.
            pub const RIGHT: Self = Self::new(1 as $Scalar, 0 as $Scalar, 0 as $Scalar);

            /// Unit vector in +Y direction. Typically interpreted as up in a 3D world.
            pub const UP: Self = Self::new(0 as $Scalar, 1 as $Scalar, 0 as $Scalar);

            /// Unit vector in -Y direction. Typically interpreted as down in a 3D world.
            pub const DOWN: Self = Self::new(0 as $Scalar, -1 as $Scalar, 0 as $Scalar);

            /// Unit vector in -Z direction. Can be interpreted as “into the screen” in an untransformed 3D world.
            pub const FORWARD: Self = Self::new(0 as $Scalar, 0 as $Scalar, -1 as $Scalar);

            /// Unit vector in +Z direction. Can be interpreted as “out of the screen” in an untransformed 3D world.
            pub const BACK: Self = Self::new(0 as $Scalar, 0 as $Scalar, 1 as $Scalar);
        }
    };
}

/// Implements functions that are present on floating-point and integer vectors.
macro_rules! impl_vector_fns {
    (
        // Name of the vector type.
        $Vector:ty,
        // Name of the glam vector type.
        $GlamVector:ty,
        // Type of target component, for example `real`.
        $Scalar:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($comp:ident),*)
    ) => {
        impl $Vector {
            /// Returns a vector with the given components.
            pub const fn new($($comp: $Scalar),*) -> Self {
                Self {
                    $( $comp ),*
                }
            }

            /// Returns a new vector with all components set to `v`.
            pub const fn splat(v: $Scalar) -> Self {
                Self {
                    $( $comp: v ),*
                }
            }

            /// Converts the corresponding `glam` type to `Self`.
            fn from_glam(v: $GlamVector) -> Self {
                Self::new(
                    $( v.$comp ),*
                )
            }

            /// Converts `self` to the corresponding `glam` type.
            fn to_glam(self) -> $GlamVector {
                <$GlamVector>::new(
                    $( self.$comp ),*
                )
            }

            /// Returns a new vector with all components in absolute values (i.e. positive or
            /// zero).
            #[inline]
            pub fn abs(self) -> Self {
                Self::from_glam(self.to_glam().abs())
            }

            /// Returns a new vector with all components clamped between the components of `min` and `max`.
            ///
            /// # Panics
            /// If `min` > `max`, `min` is NaN, or `max` is NaN.
            #[inline]
            pub fn clamp(self, min: Self, max: Self) -> Self {
                Self::from_glam(self.to_glam().clamp(min.to_glam(), max.to_glam()))
            }

            /// Returns the length (magnitude) of this vector.
            #[inline]
            pub fn length(self) -> real {
                // does the same as glam's length() but also works for integer vectors
                (self.length_squared() as real).sqrt()
            }

            /// Squared length (squared magnitude) of this vector.
            ///
            /// Runs faster than [`Self::length`], so prefer it if you need to compare vectors or need the
            /// squared distance for some formula.
            #[inline]
            pub fn length_squared(self) -> $Scalar {
                self.to_glam().length_squared()
            }

            /// Returns a new vector containing the minimum of the two vectors, component-wise.
            #[inline]
            pub fn coord_min(self, other: Self) -> Self {
                self.glam2(&other, |a, b| a.min(b))
            }

            /// Returns a new vector containing the maximum of the two vectors, component-wise.
            #[inline]
            pub fn coord_max(self, other: Self) -> Self {
                self.glam2(&other, |a, b| a.max(b))
            }

            /// Returns a new vector with each component set to 1 if it's positive, -1 if it's negative, and 0 if it's zero.
            #[inline]
            pub fn sign(self) -> Self {
                #[inline]
                fn f(x: i32) -> i32 {
                    match x.cmp(&0) {
                        Ordering::Equal => 0,
                        Ordering::Greater => 1,
                        Ordering::Less => -1,
                    }
                }

                Self::new(
                    $( f(self.$comp as i32) as $Scalar ),*
                )
            }
        }
    }
}

/// Implements functions that are present only on floating-point vectors.
macro_rules! impl_float_vector_fns {
    (
        // Name of the vector type.
        $Vector:ty,
        // Names of the components, with parentheses, for example `(x, y)`.
        ($($comp:ident),*)
    ) => {
        impl $Vector {
            /// Returns a new vector with all components rounded up (towards positive infinity).
            #[inline]
            pub fn ceil(self) -> Self {
                Self::from_glam(self.to_glam().ceil())
            }

            /// Performs a cubic interpolation between this vector and `b` using `pre_a` and `post_b` as handles,
            /// and returns the result at position `weight`.
            ///
            /// `weight` is on the range of 0.0 to 1.0, representing the amount of interpolation.
            #[inline]
            pub fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, weight: real) -> Self {
                Self::new(
                    $(
                        self.$comp.cubic_interpolate(b.$comp, pre_a.$comp, post_b.$comp, weight)
                    ),*
                )
            }

            /// Performs a cubic interpolation between this vector and `b` using `pre_a` and `post_b` as handles,
            /// and returns the result at position `weight`.
            ///
            /// `weight` is on the range of 0.0 to 1.0, representing the amount of interpolation.
            /// It can perform smoother interpolation than [`Self::cubic_interpolate`] by the time values.
            #[inline]
            #[allow(clippy::too_many_arguments)]
            pub fn cubic_interpolate_in_time(
                self,
                b: Self,
                pre_a: Self,
                post_b: Self,
                weight: real,
                b_t: real,
                pre_a_t: real,
                post_b_t: real,
            ) -> Self {
                Self::new(
                    $(
                        self.$comp.cubic_interpolate_in_time(
                            b.$comp, pre_a.$comp, post_b.$comp, weight, b_t, pre_a_t, post_b_t,
                        )
                    ),*
                )
            }

            /// Returns the normalized vector pointing from this vector to `to`.
            ///
            /// This is equivalent to using `(b - a).normalized()`.
            #[inline]
            pub fn direction_to(self, to: Self) -> Self {
                (to - self).normalized()
            }

            /// Returns the squared distance between this vector and `to`.
            ///
            /// This method runs faster than [`Self::distance_to`], so prefer it if you need to compare vectors or need the squared distance for some formula.
            #[inline]
            pub fn distance_squared_to(self, to: Self) -> real {
                (to - self).length_squared()
            }

            /// Returns the distance between this vector and `to`.
            #[inline]
            pub fn distance_to(self, to: Self) -> real {
                (to - self).length()
            }

            /// Returns the dot product of this vector and `with`.
            #[inline]
            pub fn dot(self, with: Self) -> real {
                self.to_glam().dot(with.to_glam())
            }

            /// Returns a new vector with all components rounded down (towards negative infinity).
            #[inline]
            pub fn floor(self) -> Self {
                Self::from_glam(self.to_glam().floor())
            }

            /// Returns true if each component of this vector is finite.
            #[inline]
            pub fn is_finite(self) -> bool {
                self.to_glam().is_finite()
            }

            /// Returns `true` if the vector is normalized, i.e. its length is approximately equal to 1.
            #[inline]
            pub fn is_normalized(self) -> bool {
                self.to_glam().is_normalized()
            }

            /// Returns `true` if this vector's values are approximately zero.
            ///
            /// This method is faster than using `approx_eq()` with one value as a zero vector.
            #[inline]
            pub fn is_zero_approx(self) -> bool {
                $( self.$comp.is_zero_approx() )&&*
            }

            /// Returns the result of the linear interpolation between this vector and `to` by amount `weight`.
            ///
            /// `weight` is on the range of `0.0` to `1.0`, representing the amount of interpolation.
            #[inline]
            pub fn lerp(self, other: Self, weight: real) -> Self {
                Self::new(
                    $( self.$comp.lerp(other.$comp, weight) ),*
                )
            }

            /// Returns the vector scaled to unit length. Equivalent to `self / self.length()`. See
            /// also `is_normalized()`.
            ///
            /// # Panics
            /// If called on a zero vector.
            #[inline]
            pub fn normalized(self) -> Self {
                assert_ne!(self, Self::ZERO, "normalized() called on zero vector");

                // Copy Godot's implementation since it's faster than using glam's normalize_or_zero().
                self / self.length()
            }

            /// Returns a vector composed of the [`FloatExt::fposmod`] of this vector's components and `pmod`.
            #[inline]
            pub fn posmod(self, pmod: real) -> Self {
                Self::new(
                    $( self.$comp.fposmod(pmod) ),*
                )
            }

            /// Returns a vector composed of the [`FloatExt::fposmod`] of this vector's components and `modv`'s components.
            #[inline]
            pub fn posmodv(self, modv: Self) -> Self {
                Self::new(
                    $( self.$comp.fposmod(modv.$comp) ),*
                )
            }

            /// Returns a new vector with all components rounded to the nearest integer, with halfway cases rounded away from zero.
            #[inline]
            pub fn round(self) -> Self {
                Self::from_glam(self.to_glam().round())
            }

            /// A new vector with each component snapped to the closest multiple of the corresponding
            /// component in `step`.
            // TODO: also implement for integer vectors
            #[inline]
            pub fn snapped(self, step: Self) -> Self {
                Self::new(
                    $(
                        self.$comp.snapped(step.$comp)
                    ),*
                )
            }
        }

        impl $crate::builtin::math::ApproxEq for $Vector {
            /// Returns `true` if this vector and `to` are approximately equal.
            #[inline]
            #[doc(alias = "is_equal_approx")]
            fn approx_eq(&self, other: &Self) -> bool {
                $( self.$comp.approx_eq(&other.$comp) )&&*
            }
        }
    };
}

/// Implements functions present on 2D vectors.
macro_rules! impl_vector2x_fns {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of target component, for example `real`.
        $Scalar:ty
    ) => {
        impl $Vector {
            /// Returns the aspect ratio of this vector, the ratio of [`Self::x`] to [`Self::y`].
            #[inline]
            pub fn aspect(self) -> real {
                self.x as real / self.y as real
            }

            /// Returns the axis of the vector's highest value. See [`Vector2Axis`] enum. If all components are equal, this method returns [`None`].
            ///
            /// To mimic Godot's behavior, unwrap this function's result with `unwrap_or(Vector2Axis::X)`.
            #[inline]
            #[doc(alias = "max_axis_index")]
            pub fn max_axis(self) -> Option<Vector2Axis> {
                match self.x.partial_cmp(&self.y) {
                    Some(Ordering::Less) => Some(Vector2Axis::Y),
                    Some(Ordering::Equal) => None,
                    Some(Ordering::Greater) => Some(Vector2Axis::X),
                    _ => None,
                }
            }

            /// Returns the axis of the vector's lowest value. See [`Vector2Axis`] enum. If all components are equal, this method returns [`None`].
            ///
            /// To mimic Godot's behavior, unwrap this function's result with `unwrap_or(Vector2Axis::Y)`.
            #[inline]
            #[doc(alias = "min_axis_index")]
            pub fn min_axis(self) -> Option<Vector2Axis> {
                match self.x.partial_cmp(&self.y) {
                    Some(Ordering::Less) => Some(Vector2Axis::X),
                    Some(Ordering::Equal) => None,
                    Some(Ordering::Greater) => Some(Vector2Axis::Y),
                    _ => None,
                }
            }
        }

        impl $crate::builtin::SwizzleToVector for ($Scalar, $Scalar) {
            type Output = $Vector;
            fn swizzle_to_vector(self) -> $Vector {
                <$Vector>::new(self.0, self.1)
            }
        }
    };
}

/// Implements functions present on 3D vectors.
macro_rules! impl_vector3x_fns {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of target component, for example `real`.
        $Scalar:ty
    ) => {
        impl $Vector {
            /// Returns the axis of the vector's highest value. See [`Vector3Axis`] enum. If all components are equal, this method returns [`None`].
            ///
            /// To mimic Godot's behavior, unwrap this function's result with `unwrap_or(Vector3Axis::X)`.
            #[inline]
            #[doc(alias = "max_axis_index")]
            pub fn max_axis(self) -> Option<Vector3Axis> {
                match self.x.partial_cmp(&self.y) {
                    Some(Ordering::Less) => match self.y.partial_cmp(&self.z) {
                        Some(Ordering::Less) => Some(Vector3Axis::Z),
                        Some(Ordering::Equal) => None,
                        Some(Ordering::Greater) => Some(Vector3Axis::Y),
                        _ => None,
                    },
                    Some(Ordering::Equal) => match self.x.partial_cmp(&self.z) {
                        Some(Ordering::Less) => Some(Vector3Axis::Z),
                        _ => None,
                    },
                    Some(Ordering::Greater) => match self.x.partial_cmp(&self.z) {
                        Some(Ordering::Less) => Some(Vector3Axis::Z),
                        Some(Ordering::Equal) => None,
                        Some(Ordering::Greater) => Some(Vector3Axis::X),
                        _ => None,
                    },
                    _ => None,
                }
            }

            /// Returns the axis of the vector's lowest value. See [`Vector3Axis`] enum. If all components are equal, this method returns [`None`].
            ///
            /// To mimic Godot's behavior, unwrap this function's result with `unwrap_or(Vector3Axis::Z)`.
            #[inline]
            #[doc(alias = "min_axis_index")]
            pub fn min_axis(self) -> Option<Vector3Axis> {
                match self.x.partial_cmp(&self.y) {
                    Some(Ordering::Less) => match self.x.partial_cmp(&self.z) {
                        Some(Ordering::Less) => Some(Vector3Axis::X),
                        Some(Ordering::Equal) => None,
                        Some(Ordering::Greater) => Some(Vector3Axis::Z),
                        _ => None,
                    },
                    Some(Ordering::Equal) => match self.x.partial_cmp(&self.z) {
                        Some(Ordering::Greater) => Some(Vector3Axis::Z),
                        _ => None,
                    },
                    Some(Ordering::Greater) => match self.y.partial_cmp(&self.z) {
                        Some(Ordering::Less) => Some(Vector3Axis::Y),
                        Some(Ordering::Equal) => None,
                        Some(Ordering::Greater) => Some(Vector3Axis::Z),
                        _ => None,
                    },
                    _ => None,
                }
            }
        }

        impl $crate::builtin::SwizzleToVector for ($Scalar, $Scalar, $Scalar) {
            type Output = $Vector;
            fn swizzle_to_vector(self) -> $Vector {
                <$Vector>::new(self.0, self.1, self.2)
            }
        }
    };
}

/// Implements functions present on 4D vectors.
macro_rules! impl_vector4x_fns {
    (
        // Name of the vector type.
        $Vector:ty,
        // Type of target component, for example `real`.
        $Scalar:ty
    ) => {
        impl $Vector {
            /// Returns the axis of the vector's highest value. See [`Vector4Axis`] enum. If all components are equal, this method returns [`None`].
            ///
            /// To mimic Godot's behavior, unwrap this function's result with `unwrap_or(Vector4Axis::X)`.
            #[inline]
            #[doc(alias = "max_axis_index")]
            pub fn max_axis(self) -> Option<Vector4Axis> {
                let mut max_axis = Vector4Axis::X;
                let mut previous = None;
                let mut max_value = self.x;

                let components = [
                    (Vector4Axis::Y, self.y),
                    (Vector4Axis::Z, self.z),
                    (Vector4Axis::W, self.w),
                ];

                for (axis, value) in components {
                    if value >= max_value {
                        max_axis = axis;
                        previous = Some(max_value);
                        max_value = value;
                    }
                }

                (Some(max_value) != previous).then_some(max_axis)
            }

            /// Returns the axis of the vector's lowest value. See [`Vector4Axis`] enum. If all components are equal, this method returns [`None`].
            ///
            /// To mimic Godot's behavior, unwrap this function's result with `unwrap_or(Vector4Axis::W)`.
            #[inline]
            #[doc(alias = "min_axis_index")]
            pub fn min_axis(self) -> Option<Vector4Axis> {
                let mut min_axis = Vector4Axis::X;
                let mut previous = None;
                let mut min_value = self.x;

                let components = [
                    (Vector4Axis::Y, self.y),
                    (Vector4Axis::Z, self.z),
                    (Vector4Axis::W, self.w),
                ];

                for (axis, value) in components {
                    if value <= min_value {
                        min_axis = axis;
                        previous = Some(min_value);
                        min_value = value;
                    }
                }

                (Some(min_value) != previous).then_some(min_axis)
            }
        }

        impl $crate::builtin::SwizzleToVector for ($Scalar, $Scalar, $Scalar, $Scalar) {
            type Output = $Vector;
            fn swizzle_to_vector(self) -> $Vector {
                <$Vector>::new(self.0, self.1, self.2, self.3)
            }
        }
    };
}

/// Implements functions present on floating-point 2D and 3D vectors.
macro_rules! impl_vector2_vector3_fns {
    (
        // Name of the vector type.
        $Vector:ty,
        // Names of the components, with parentheses, for example `(x, y, z, w)`.
        ($($comp:ident),*)
    ) => {
        impl $Vector {
            /// Returns the angle to the given vector, in radians.
            #[inline]
            pub fn angle_to(self, to: Self) -> real {
                self.glam2(&to, |a, b| a.angle_between(b))
            }

           /// Returns the derivative at the given `t` on the [Bézier](https://en.wikipedia.org/wiki/B%C3%A9zier_curve)
           /// curve defined by this vector and the given `control_1`, `control_2`, and `end` points.
           #[inline]
           pub fn bezier_derivative(self, control_1: Self, control_2: Self, end: Self, t: real) -> Self {
               Self::new(
                    $(
                        self.$comp.bezier_derivative(control_1.$comp, control_2.$comp, end.$comp, t)
                    ),*
               )
           }

            /// Returns the point at the given `t` on the [Bézier](https://en.wikipedia.org/wiki/B%C3%A9zier_curve)
            /// curve defined by this vector and the given `control_1`, `control_2`, and `end` points.
            #[inline]
            pub fn bezier_interpolate(self, control_1: Self, control_2: Self, end: Self, t: real) -> Self {
                Self::new(
                    $(
                        self.$comp.bezier_interpolate(control_1.$comp, control_2.$comp, end.$comp, t)
                    ),*
                )
            }

            /// Returns a new vector "bounced off" from a plane defined by the given normal.
            ///
            /// # Panics
            /// If `n` is not normalized.
            #[inline]
            pub fn bounce(self, n: Self) -> Self {
                assert!(n.is_normalized(), "n is not normalized!");
                -self.reflect(n)
            }

            /// Returns the vector with a maximum length by limiting its length to `length`.
            #[inline]
            pub fn limit_length(self, length: Option<real>) -> Self {
                let length = length.unwrap_or(1.0);

                Self::from_glam(self.to_glam().clamp_length_max(length))

            }

            /// Returns a new vector moved toward `to` by the fixed `delta` amount. Will not go past the final value.
            #[inline]
            pub fn move_toward(self, to: Self, delta: real) -> Self {
                Self::from_glam(self.to_glam().move_towards(to.to_glam(), delta))
            }

            /// Returns the result of projecting the vector onto the given vector `b`.
            #[inline]
            pub fn project(self, b: Self) -> Self {
                Self::from_glam(self.to_glam().project_onto(b.to_glam()))
            }

            /// Returns the result of reflecting the vector defined by the given direction vector `n`.
            ///
            /// # Panics
            /// If `n` is not normalized.
            #[inline]
            pub fn reflect(self, n: Self) -> Self {
                assert!(n.is_normalized(), "n is not normalized!");
                2.0 * n * self.dot(n) - self
            }

            /// Returns a new vector slid along a plane defined by the given normal.
            ///
            /// # Panics
            /// If `n` is not normalized.
            #[inline]
            pub fn slide(self, n: Self) -> Self {
                assert!(n.is_normalized(), "n is not normalized!");
                self - n * self.dot(n)
            }
        }
    };
}

/// Implements functions present on floating-point 3D and 4D vectors.
macro_rules! impl_vector3_vector4_fns {
    (
        // Name of the vector type.
        $Vector:ty,
        // Names of the components, with parentheses, for example `(x, y, z, w)`.
        ($($comp:ident),*)
    ) => {
        impl $Vector {
            /// Returns the reciprocal (inverse) of the vector. This is the same as `1.0/n` for each component.
            #[inline]
            #[doc(alias = "inverse")]
            pub fn recip(self) -> Self {
                Self::from_glam(self.to_glam().recip())
            }
        }
    };
}
