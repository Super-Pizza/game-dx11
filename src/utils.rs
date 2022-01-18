use core::fmt;
use std::{ffi::OsStr, iter, ops, os::windows::prelude::OsStrExt};

/*use directx_math::{
    XMVectorGetX, XMVectorGetY, XMVectorGetZ, XMVectorScale, XMMATRIX, XMVECTOR, XMVECTORF32,
};*/

pub fn str_to_c16<T>(string: &T) -> Vec<u16>
where
    T: std::convert::AsRef<std::ffi::OsStr> + ?Sized,
{
    OsStr::new(string)
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<u16>>()
}
pub fn win32_to_hresult(code: u32) -> i32 {
    if code as i32 <= 0 {
        code as i32
    } else {
        ((code & 0x0000FFFF) | (7 << 16) | 0x80000000) as i32
    }
}
#[macro_export]
macro_rules! offset_of {
    ($Struct:path, $field:tt) => {{
        let s1 = MaybeUninit::<$Struct>::uninit();
        let s = s1.as_ptr();
        let f = unsafe { std::ptr::addr_of!((*(s as *const $Struct)).$field) };
        (f as usize) - (s as usize)
    }};
}
#[macro_export]
macro_rules! release {
    ($($tt:tt)*) => {
        unsafe { (&*($($tt)* as *mut _ as *mut IUnknown)).Release() }
    };
}
pub fn read_key(code: u16, keys: [u16; 16]) -> bool {
    let slot = code / 16;
    let bit = code % 16;
    keys[slot as usize] & 1 << bit > 0
} /*
  pub fn mat_to_quat(mat: XMMATRIX) -> XMVECTOR {
      unsafe {
          let mut q: XMVECTORF32 = XMVECTORF32 {
              f: [0.0, 0.0, 0.0, 0.0],
          };
          let t = if XMVectorGetZ(mat.r[2]) <= 0.0 {
              if XMVectorGetX(mat.r[0]) > XMVectorGetY(mat.r[1]) {
                  let t: f32 =
                      1.0 + XMVectorGetX(mat.r[0]) - XMVectorGetY(mat.r[1]) - XMVectorGetZ(mat.r[2]);
                  q.f[0] = t;
                  q.f[1] = XMVectorGetY(mat.r[0]) + XMVectorGetX(mat.r[1]);
                  q.f[2] = XMVectorGetZ(mat.r[0]) + XMVectorGetX(mat.r[2]);
                  q.f[3] = XMVectorGetZ(mat.r[1]) - XMVectorGetY(mat.r[2]);
                  t
              } else {
                  let t: f32 =
                      1.0 - XMVectorGetX(mat.r[0]) + XMVectorGetY(mat.r[1]) - XMVectorGetZ(mat.r[2]);
                  q.f[0] = XMVectorGetY(mat.r[0]) + XMVectorGetX(mat.r[1]);
                  q.f[1] = t;
                  q.f[2] = XMVectorGetZ(mat.r[1]) + XMVectorGetY(mat.r[2]);
                  q.f[3] = XMVectorGetX(mat.r[2]) - XMVectorGetZ(mat.r[0]);
                  t
              }
          } else if XMVectorGetX(mat.r[0]) < -XMVectorGetY(mat.r[1]) {
              let t: f32 =
                  1.0 - XMVectorGetX(mat.r[0]) - XMVectorGetY(mat.r[1]) + XMVectorGetZ(mat.r[2]);
              q.f[0] = XMVectorGetZ(mat.r[0]) + XMVectorGetX(mat.r[2]);
              q.f[1] = XMVectorGetZ(mat.r[1]) + XMVectorGetY(mat.r[2]);
              q.f[2] = t;
              q.f[3] = XMVectorGetY(mat.r[0]) - XMVectorGetX(mat.r[1]);
              t
          } else {
              let t: f32 =
                  1.0 + XMVectorGetX(mat.r[0]) + XMVectorGetY(mat.r[1]) + XMVectorGetZ(mat.r[2]);
              q.f[0] = XMVectorGetZ(mat.r[1]) - XMVectorGetY(mat.r[2]);
              q.f[1] = XMVectorGetX(mat.r[2]) - XMVectorGetZ(mat.r[0]);
              q.f[2] = XMVectorGetY(mat.r[0]) - XMVectorGetX(mat.r[1]);
              q.f[3] = t;
              t
          };
          XMVectorScale(q.v, 0.5 / f32::sqrt(t))
      }
  }*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coord<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: ops::Add<Output = T>> ops::Add<Self> for Coord<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Coord<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x: {}, y: {}, z: {}", self.x, self.y, self.z)
    }
}
