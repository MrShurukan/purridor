use crate::panic_buffer::PanicBuffer;
use crate::raylib::{App, Color};
use core::fmt::Write;
use core::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
    panic::PanicInfo,
};

unsafe extern "C" {
    fn memalign(alignment: usize, size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

struct NewlibAllocator;

unsafe impl GlobalAlloc for NewlibAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let alignment = layout
            .align()
            .max(core::mem::align_of::<usize>());

        unsafe { memalign(alignment, layout.size()) as *mut u8 }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { free(ptr.cast()) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: NewlibAllocator = NewlibAllocator;

#[alloc_error_handler]
fn allocation_error(_layout: Layout) -> ! {
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut buffer = PanicBuffer::new();

    let _ = writeln!(buffer, "PANIC!");

    if let Some(location) = info.location() {
        let _ = writeln!(
            buffer,
            "{}:{}:{}",
            location.file(),
            location.line(),
            location.column(),
        );
    }

    let _ = writeln!(buffer);
    let _ = write!(buffer, "{}", info.message());

    crate::raylib::panic_screen(
        buffer.as_bytes_with_nul()
    )
}