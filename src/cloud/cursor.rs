// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use super::{field_offset, offset_range, PointCloudPacked};
use std::{
    marker::PhantomData,
    mem::size_of,
    ops::{Add, AddAssign},
};

/// Borrowed native-endian field cursor. Copies retain their position independently.
///
/// Reads use the same addressing as [`super::read_native`]. Advancement counts
/// points; `get` counts contiguous values. Advancing beyond the complete-record
/// count clamps to the end. Missing fields and zero strides/sized types are empty.
/// Malformed fields return None on unreadable accesses. No references into packed
/// bytes are created; writes require the exclusive indexed API instead.
#[derive(Clone, Copy, Debug)]
pub struct NativeFieldCursor<'a, T: bytemuck::Pod> {
    cloud: &'a PointCloudPacked,
    offset: Option<usize>,
    point: usize,
    count: usize,
    value: PhantomData<T>,
}
impl<'a, T: bytemuck::Pod> NativeFieldCursor<'a, T> {
    pub fn new(cloud: &'a PointCloudPacked, name: &str) -> Self {
        let offset = field_offset(cloud, name);
        let count = if offset.is_some() && cloud.point_step != 0 && size_of::<T>() != 0 {
            cloud.data.len() / cloud.point_step as usize
        } else {
            0
        };
        Self {
            cloud,
            offset,
            point: 0,
            count,
            value: PhantomData,
        }
    }
    pub fn get(&self, element: usize) -> Option<T> {
        let range = offset_range::<T>(self.cloud, self.offset?, self.point, element)?;
        bytemuck::try_pod_read_unaligned(&self.cloud.data[range]).ok()
    }
    /// End sentinel for the same field and backing message.
    pub fn end(mut self) -> Self {
        self.point = self.count;
        self
    }
    fn address(&self) -> Option<*const u8> {
        let offset = self
            .point
            .checked_mul(self.cloud.point_step as usize)?
            .checked_add(self.offset?)?;
        Some(self.cloud.data.as_ptr().wrapping_add(offset))
    }
}
impl<T: bytemuck::Pod> PartialEq for NativeFieldCursor<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}
impl<T: bytemuck::Pod> Eq for NativeFieldCursor<'_, T> {}
impl<T: bytemuck::Pod> AddAssign<usize> for NativeFieldCursor<'_, T> {
    fn add_assign(&mut self, points: usize) {
        self.point = self.point.saturating_add(points).min(self.count);
    }
}
impl<T: bytemuck::Pod> Add<usize> for NativeFieldCursor<'_, T> {
    type Output = Self;
    fn add(mut self, points: usize) -> Self {
        self += points;
        self
    }
}
impl<T: bytemuck::Pod> Iterator for NativeFieldCursor<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let value = self.get(0)?;
        *self += 1;
        Some(value)
    }
}
