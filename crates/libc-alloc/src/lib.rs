//! Replaces wasi-libc's dlmalloc with a talc arena.
//!
//! wasi-libc's `sbrk` assumes 64 KiB wasm pages, so its dlmalloc traps when the
//! module is linked with `--page-size=1`. Defining every symbol that
//! `dlmalloc.c.obj` provides keeps the linker from pulling it (and `sbrk`) in.
//!
//! The talc arena lives here and backs `malloc`, rather than being the Rust
//! `#[global_allocator]`. `free(p)` has no size, so every block carries a
//! header. `cabi_realloc` (from std's `wasip2` crate) calls `__rust_alloc`,
//! and wasi-libc frees those canonical-ABI buffers with `free()` (e.g. the
//! list returned by `blocking-read` in `read()`). If `__rust_alloc` went
//! straight to talc, those blocks would have no header and `free` would trap.
//! Leaving Rust on the default `System` allocator, which calls `malloc`,
//! `posix_memalign` and `free` on wasi, gives every allocation the header.
//!
//! Users must therefore not set a `#[global_allocator]`; just `extern crate libc_alloc;`.
#![no_std]

#[cfg(target_os = "wasi")]
mod imp {
    use core::alloc::{GlobalAlloc, Layout};
    use core::ffi::c_void;
    use core::mem::MaybeUninit;
    use core::ptr::{self, null_mut};

    // The only heap in the module: both C (`malloc`) and Rust (`System` -> `malloc`)
    // allocate from it. A fixed arena avoids the 64 KiB page assumptions of
    // `sbrk` and talc's `memory.grow` source.
    static TALC: talc::wasm::WasmArenaTalc = {
        static mut MEMORY: [MaybeUninit<u8>; 0x80000] = [MaybeUninit::uninit(); 0x80000];
        // SAFETY: the memory for MEMORY is never modified externally. It's the allocator's.
        unsafe { talc::wasm::new_wasm_arena_allocator(&raw mut MEMORY) }
    };

    /// `max_align_t` on wasm32.
    const MIN_ALIGN: usize = 16;
    const EINVAL: i32 = 28;
    const ENOMEM: i32 = 48;

    // Each block is laid out as `[padding .. total_size, align][user data]`.
    // The header is `max(MIN_ALIGN, align)` bytes, so the user pointer keeps the
    // requested alignment, and the last 8 bytes hold what `free` needs.
    unsafe fn header(p: *mut u8) -> (*mut u8, Layout) {
        unsafe {
            let total = *(p.sub(8) as *const usize);
            let align = *(p.sub(4) as *const usize);
            (
                p.sub(align),
                Layout::from_size_align_unchecked(total, align),
            )
        }
    }

    unsafe fn write_header(base: *mut u8, layout: Layout) -> *mut u8 {
        unsafe {
            let p = base.add(layout.align());
            *(p.sub(8) as *mut usize) = layout.size();
            *(p.sub(4) as *mut usize) = layout.align();
            p
        }
    }

    unsafe fn alloc_aligned(size: usize, align: usize, zeroed: bool) -> *mut c_void {
        let align = align.max(MIN_ALIGN);
        let Some(total) = size.checked_add(align) else {
            return null_mut();
        };
        let Ok(layout) = Layout::from_size_align(total, align) else {
            return null_mut();
        };
        unsafe {
            let base = if zeroed {
                TALC.alloc_zeroed(layout)
            } else {
                TALC.alloc(layout)
            };
            if base.is_null() {
                return null_mut();
            }
            write_header(base, layout) as *mut c_void
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
        unsafe { alloc_aligned(size, MIN_ALIGN, false) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn calloc(n: usize, size: usize) -> *mut c_void {
        match n.checked_mul(size) {
            Some(total) => unsafe { alloc_aligned(total, MIN_ALIGN, true) },
            None => null_mut(),
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn free(p: *mut c_void) {
        if p.is_null() {
            return;
        }
        unsafe {
            let (base, layout) = header(p as *mut u8);
            TALC.dealloc(base, layout);
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn realloc(p: *mut c_void, size: usize) -> *mut c_void {
        unsafe {
            if p.is_null() {
                return malloc(size);
            }
            if size == 0 {
                free(p);
                return null_mut();
            }
            let (base, layout) = header(p as *mut u8);
            let align = layout.align();
            if align == MIN_ALIGN {
                let Some(total) = size.checked_add(align) else {
                    return null_mut();
                };
                if Layout::from_size_align(total, align).is_err() {
                    return null_mut();
                }
                let new_base = TALC.realloc(base, layout, total);
                if new_base.is_null() {
                    return null_mut();
                }
                return write_header(new_base, Layout::from_size_align_unchecked(total, align))
                    as *mut c_void;
            }
            let new = alloc_aligned(size, align, false);
            if !new.is_null() {
                let old_size = layout.size() - align;
                ptr::copy_nonoverlapping(p as *const u8, new as *mut u8, old_size.min(size));
                free(p);
            }
            new
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn aligned_alloc(align: usize, size: usize) -> *mut c_void {
        if !align.is_power_of_two() {
            return null_mut();
        }
        unsafe { alloc_aligned(size, align, false) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn posix_memalign(
        out: *mut *mut c_void,
        align: usize,
        size: usize,
    ) -> i32 {
        if !align.is_power_of_two() || align % size_of::<usize>() != 0 {
            return EINVAL;
        }
        unsafe {
            let p = alloc_aligned(size, align, false);
            if p.is_null() {
                return ENOMEM;
            }
            *out = p;
        }
        0
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn malloc_usable_size(p: *mut c_void) -> usize {
        if p.is_null() {
            return 0;
        }
        let (_, layout) = unsafe { header(p as *mut u8) };
        layout.size() - layout.align()
    }

    // Internal aliases used by wasi-libc's locale and atexit code.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn __libc_malloc(size: usize) -> *mut c_void {
        unsafe { malloc(size) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn __libc_calloc(n: usize, size: usize) -> *mut c_void {
        unsafe { calloc(n, size) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn __libc_free(p: *mut c_void) {
        unsafe { free(p) }
    }
}
