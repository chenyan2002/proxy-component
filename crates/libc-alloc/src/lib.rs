//! Makes wasi-libc's malloc work in modules linked with `--page-size=1`.
//!
//! wasi-libc's dlmalloc gets memory only through `sbrk`, and `sbrk` is the
//! only place that assumes 64 KiB wasm pages: it aborts unless the increment
//! is a multiple of 65536, and converts between pages and bytes with `<< 16`.
//! dlmalloc itself locates the heap through `__heap_base`/`__heap_end`, which
//! are byte addresses. Defining `sbrk` here keeps the linker from pulling in
//! wasi-libc's `sbrk.o`, so everything else stays stock wasi-libc.
//!
//! This keeps a single allocator for the whole module: Rust's default `System`
//! allocator calls `malloc`/`free` on wasi, and `cabi_realloc` (from std's
//! `wasip2` crate) calls `__rust_alloc`, so C code can `free()` canonical-ABI
//! buffers. Users must therefore not set a `#[global_allocator]`; just
//! `extern crate libc_alloc;`.
//!
//! Only link this into modules built with 1-byte pages (the components'
//! `one-byte-page` feature); regular builds use wasi-libc's own `sbrk`.
#![no_std]

#[cfg(all(target_os = "wasi", target_arch = "wasm32"))]
mod imp {
    use core::arch::wasm32::{memory_grow, memory_size, unreachable};
    use core::ffi::c_void;

    unsafe extern "C" {
        // End of static data, in bytes. Provided by wasm-ld.
        static __heap_end: u8;
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn sbrk(increment: isize) -> *mut c_void {
        // With 1-byte pages, `memory.size` (in pages) is in bytes and covers at
        // least the static data. With 64 KiB pages it is far smaller than
        // `__heap_end`, which means the module wasn't linked with
        // `--page-size=1`; trap instead of corrupting memory.
        if memory_size(0) < &raw const __heap_end as usize {
            unreachable();
        }
        if increment == 0 {
            return memory_size(0) as *mut c_void;
        }
        // dlmalloc never trims (wasi-libc sets MORECORE_CANNOT_TRIM).
        if increment < 0 {
            unreachable();
        }
        match memory_grow(0, increment as usize) {
            // dlmalloc treats `(void *)-1` as failure and sets errno itself.
            usize::MAX => usize::MAX as *mut c_void,
            old => old as *mut c_void,
        }
    }
}
