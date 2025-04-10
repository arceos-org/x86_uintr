# x86_uintr

This crate provides support for Intel User Interrupt (UINTR) extensions, including:

- Core Definitions:
  - MSR specifications with reserved bits taken care of
  - Wrappers around instructions: `UIRET, TESTUI, CLUI, STUI, SENDUIPI`
  - In-memory structures: User Interrupt Target Table Entry (UITTE) and User Posted-Interrupt Descriptor (UPID)
  - XSTATE Component: `UintrState` struct with memory layout aligned with the supervisor user-interrupt state component for XSAVES/XRSTORS compatibility
- Interrupt Handling (`handler` feature):
  - General-purpose register preservation trampoline
  - Optional x87/SSE state management via FXSAVE/FXRSTOR (`fp_simd` feature)
  - Supports for custom handler written in Rust, without need for inline assembly, `#[no_mangle]` or `extern "C"` (via `x86_uintr::handler::set_handler()`)
  - UINTR handler entry address for writing to the IA32_UINTR_HANDLER MSR (via `x86_uintr::handler::handler_entry_addr()`)

The users may disable the `fp_simd` feature if they need finer control over the XSTATE components, or if they do not use the related registers at all.

## Example

```rust
use core::sync::atomic::{AtomicBool, Ordering};
use x86_uintr::handler::{UintrHandler, UintrTrapframe, handler_entry_addr, set_handler};
use x86_uintr::instructions::send_uipi;

static INTERRUPT_RECEIVED: AtomicBool = AtomicBool::new(false);

pub fn uintr_handler(tf: &mut UintrTrapframe) {
    INTERRUPT_RECEIVED.store(true, Ordering::SeqCst);
    println!("Received interrupt in user mode");
    println!("Trap Frame: {:#x?}", tf);
}

pub fn main() -> i32 {
    set_handler(UintrHandler(uintr_handler));
    let handler_address = handler_entry_addr();
    // Register the UITT and the handler to the kernel
    // Assume this UITTE points to the process itself
    let uitte_index = ...;
    unsafe { send_uipi(uitte_index) };
    while !INTERRUPT_RECEIVED.load(Ordering::SeqCst) {}
    println!("UINTR received!");
    0
}
```