#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

#[cfg( target_arch = "wasm32")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
fn main() {}
