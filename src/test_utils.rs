use std::ops::Sub;

use bevy::prelude::{Quat, Vec2, Vec3, Vec4};

/// Utility trait to compare floating-point values with a tolerance.
pub(crate) trait AbsDiffEq {
    /// Calculate the absolute value of the difference between two
    /// floating-point quantities. For non-scalar quantities, the maximum
    /// absolute difference for all components is returned.
    fn abs_diff(a: &Self, b: &Self) -> f32;

    /// Check if two floating-point quantities are approximately equal within a
    /// given tolerance. Non-scalar values are checked component-wise.
    fn abs_diff_eq(a: &Self, b: &Self, tol: f32) -> bool;
}

impl AbsDiffEq for f32 {
    fn abs_diff(a: &f32, b: &f32) -> f32 {
        (a - b).abs()
    }

    fn abs_diff_eq(a: &f32, b: &f32, tol: f32) -> bool {
        (a - b).abs() < tol
    }
}

impl AbsDiffEq for f64 {
    fn abs_diff(a: &f64, b: &f64) -> f32 {
        (a - b).abs() as f32
    }

    fn abs_diff_eq(a: &f64, b: &f64, tol: f32) -> bool {
        (a - b).abs() < tol as f64
    }
}

impl AbsDiffEq for Vec2 {
    fn abs_diff(a: &Vec2, b: &Vec2) -> f32 {
        a.sub(*b).abs().max_element()
    }

    fn abs_diff_eq(a: &Vec2, b: &Vec2, tol: f32) -> bool {
        a.abs_diff_eq(*b, tol)
    }
}

impl AbsDiffEq for Vec3 {
    fn abs_diff(a: &Vec3, b: &Vec3) -> f32 {
        a.sub(*b).abs().max_element()
    }

    fn abs_diff_eq(a: &Vec3, b: &Vec3, tol: f32) -> bool {
        a.abs_diff_eq(*b, tol)
    }
}

impl AbsDiffEq for Vec4 {
    fn abs_diff(a: &Vec4, b: &Vec4) -> f32 {
        a.sub(*b).abs().max_element()
    }

    fn abs_diff_eq(a: &Vec4, b: &Vec4, tol: f32) -> bool {
        a.abs_diff_eq(*b, tol)
    }
}

impl AbsDiffEq for Quat {
    fn abs_diff(a: &Quat, b: &Quat) -> f32 {
        Vec4::from(*a).sub(Vec4::from(*b)).abs().max_element()
    }

    fn abs_diff_eq(a: &Quat, b: &Quat, tol: f32) -> bool {
        a.abs_diff_eq(*b, tol)
    }
}

/// Assert that two floating-point quantities are approximately equal.
///
/// This macro asserts that the absolute difference between the two first
/// arguments is strictly less than a tolerance factor, which can be explicitly
/// passed as third argument or implicitly defaults to `1e-5`.
///
/// The two quantities must implement the [`AbsDiffEq`] helper trait. This trait
/// is implemented for common floating-point quantities like `f32` or `f64`, or
/// associated vector math types like `Vec3` or `Quat`.
///
/// # Usage
///
/// ```
/// let x = 3.500009;
/// assert_approx_eq!(x, 3.5); // default tolerance 1e-5
///
/// let x = 3.509;
/// assert_approx_eq!(x, 3.5, 0.01); // explicit tolerance
/// ```
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                assert!(
                    crate::test_utils::AbsDiffEq::abs_diff_eq(left_val, right_val, 1e-5),
                    "assertion failed: expected={} actual={} delta={} tol=1e-5(default)",
                    left_val,
                    right_val,
                    crate::test_utils::AbsDiffEq::abs_diff(left_val, right_val),
                );
            }
        }
    };
    ($left:expr, $right:expr, $tol:expr $(,)?) => {
        match (&$left, &$right, &$tol) {
            (left_val, right_val, tol_val) => {
                assert!(
                    crate::test_utils::AbsDiffEq::abs_diff_eq(left_val, right_val, *tol_val),
                    "assertion failed: expected={} actual={} delta={} tol={}",
                    left_val,
                    right_val,
                    crate::test_utils::AbsDiffEq::abs_diff(left_val, right_val),
                    tol_val
                );
            }
        }
    };
}

pub(crate) use assert_approx_eq;


/// The Slang every generated module starts with.
pub(crate) const HANABI_COMMON: &str = include_str!("shaders/hanabi_common.slang");

/// The newest `slangc` on `PATH` (as aurora picks it).
fn slangc() -> std::path::PathBuf {
    let version = |p: &std::path::Path| -> Option<Vec<u32>> {
        let out = std::process::Command::new(p).arg("-v").output().ok()?;
        let text = String::from_utf8_lossy(&out.stdout).to_string()
            + &String::from_utf8_lossy(&out.stderr);
        Some(
            text.split_whitespace()
                .next()?
                .trim_start_matches('v')
                .split('.')
                .map_while(|n| n.parse().ok())
                .collect(),
        )
    };
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .map(|dir| dir.join("slangc"))
        .filter(|p| p.exists())
        .filter_map(|p| version(&p).map(|v| (v, p)))
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, p)| p)
        .unwrap_or_else(|| "slangc".into())
}

/// Compile a Slang module to SPIR-V, as aurora does; the compiler's errors on failure.
pub(crate) fn slang_check(code: &str) -> Result<(), String> {
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let dir = std::env::temp_dir().join("hanabi-slang-tests");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let src = dir.join(format!("{}-{n}.slang", std::process::id()));
    let out = src.with_extension("spv");
    std::fs::write(&src, code).map_err(|e| e.to_string())?;
    let res = std::process::Command::new(slangc())
        .arg(&src)
        .args(["-target", "spirv", "-profile", "spirv_1_6", "-o"])
        .arg(&out)
        .output()
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&out);
    if res.status.success() {
        let _ = std::fs::remove_file(&src);
        Ok(())
    } else {
        Err(format!(
            "{}\n{}",
            src.display(),
            String::from_utf8_lossy(&res.stderr)
        ))
    }
}
