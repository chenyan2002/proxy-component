//! Global allocator for the generated crates, which are linked with `--page-size=1`.
//!
//! std's default allocator on wasm32-unknown-unknown is dlmalloc, but its memory
//! source assumes 64 KiB pages. This uses dlmalloc with a source that grows
//! memory by bytes instead.
//!
//! Spliced into the generated `lib.rs` (inside `mod one_byte_page_alloc`) by
//! `src/codegen/mod.rs`; it is not compiled on its own.

use core::alloc::{GlobalAlloc, Layout};
use core::arch::wasm32::memory_grow;
use core::cell::UnsafeCell;
use core::ptr::null_mut;

struct OneBytePage;

unsafe impl dlmalloc::Allocator for OneBytePage {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        // With 1-byte pages, `memory.grow` counts bytes.
        match memory_grow(0, size) {
            usize::MAX => (null_mut(), 0, 0),
            prev => (prev as *mut u8, size, 0),
        }
    }
    fn remap(&self, _: *mut u8, _: usize, _: usize, _: bool) -> *mut u8 {
        null_mut()
    }
    fn free_part(&self, _: *mut u8, _: usize, _: usize) -> bool {
        false
    }
    fn free(&self, _: *mut u8, _: usize) -> bool {
        false
    }
    fn can_release_part(&self, _: u32) -> bool {
        false
    }
    fn allocates_zeros(&self) -> bool {
        true
    }
    // Only used by dlmalloc for rounding; `alloc` accepts any size.
    fn page_size(&self) -> usize {
        64 * 1024
    }
}

struct Dlmalloc(UnsafeCell<dlmalloc::Dlmalloc<OneBytePage>>);

// SAFETY: wasm without atomics is single-threaded.
unsafe impl Sync for Dlmalloc {}

unsafe impl GlobalAlloc for Dlmalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { (*self.0.get()).malloc(layout.size(), layout.align()) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { (*self.0.get()).calloc(layout.size(), layout.align()) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { (*self.0.get()).free(ptr, layout.size(), layout.align()) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { (*self.0.get()).realloc(ptr, layout.size(), layout.align(), new_size) }
    }
}

#[global_allocator]
static ALLOC: Dlmalloc = Dlmalloc(UnsafeCell::new(dlmalloc::Dlmalloc::new_with_allocator(
    OneBytePage,
)));
