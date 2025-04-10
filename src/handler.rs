use atomic::Atomic;
use bytemuck::NoUninit;
use core::sync::atomic::Ordering;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GeneralRegisters {
    /// argument registers
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,

    /// callee-saved registers
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    /// return value
    pub rax: u64,
}

/// Pushed by CPU
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct UintrInfo {
    pub uirr_vector: u64,
    pub rip: u64,
    pub rflags: u64,
    pub rsp: u64,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct UintrTrapframe {
    pub regs: GeneralRegisters,
    pub info: UintrInfo,
}

/// # SAFETY
///
/// This function is the entry point of UINTR handler, and should not be called
/// by any user code. Its address can be filled into the IA32_UINTR_HANDLER MSR.
#[naked]
#[allow(dead_code)]
pub unsafe extern "C" fn uintr_handler_asm_entry() {
    unsafe {
        core::arch::naked_asm!(
            "
            // fill trapframe
            push   rax
            push   r15
            push   r14
            push   r13
            push   r12
            push   rbp
            push   rbx
            push   r11
            push   r10
            push   r9
            push   r8
            push   rcx
            push   rdx
            push   rsi
            push   rdi

            // set first argument to beginning of trapframe
            mov    rdi, rsp

            call     uintr_handler_rust_entry

            // restore trap frame
            pop   rdi;
            pop   rsi;
            pop   rdx;
            pop   rcx;
            pop   r8;
            pop   r9;
            pop   r10;
            pop   r11
            pop   rbx;
            pop   rbp;
            pop   r12;
            pop   r13;
            pop   r14;
            pop   r15;
            pop   rax;

            // skip UIRRV
            add   rsp, 8

            uiret
            nop
            "
        );
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "fp_simd")] {
        pub use xsave::XSaveLegacy;
        pub type HandlerType = fn(&mut UintrTrapframe, &mut XSaveLegacy);
        static HANDLER: Atomic<UintrHandler> = atomic::Atomic::new(UintrHandler(|_, _| {}));
    } else {
        pub type HandlerType = fn(&mut UintrTrapframe);

        static HANDLER: Atomic<UintrHandler> = atomic::Atomic::new(UintrHandler(|_| {}));
    }
}

/// Wrapping around the handler function pointer so that we can impl NoUninit trait for it
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct UintrHandler(pub HandlerType);

// Potential UB? https://github.com/Amanieu/atomic-rs/issues/35
unsafe impl NoUninit for UintrHandler {}

#[unsafe(no_mangle)]
pub extern "C" fn uintr_handler_rust_entry(utf: &mut UintrTrapframe) {
    cfg_if::cfg_if! {
        if #[cfg(feature = "fp_simd")] {
            // only save legacy xstate to save stack space and reduce latency
            let mut fxstate = XSaveLegacy::default();
            unsafe { core::arch::x86_64::_fxsave64(&mut fxstate as *mut _ as *mut u8) };
            HANDLER.load(Ordering::SeqCst).0(utf, &mut fxstate);
            unsafe { core::arch::x86_64::_fxrstor64(&fxstate as *const _ as *const u8);}
        } else {
            HANDLER.load(Ordering::SeqCst).0(utf);
        }
    };
}

#[allow(dead_code)]
pub fn handler_entry_addr() -> usize {
    uintr_handler_asm_entry as usize
}

#[allow(dead_code)]
pub fn set_handler(handler: UintrHandler) {
    HANDLER.store(handler, Ordering::SeqCst);
}
