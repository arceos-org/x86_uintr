use core::arch::asm;

/// User-Interrupt Return
///
/// UIRET returns from the handling of a user interrupt.
/// It can be executed regardless of CPL.
/// It loads RIP, RFLAGS and RSP from the stack and sets the UIF.
///
/// # Safety
///
/// The caller must ensure that the RIP stored on the stack is canonical.
#[inline]
pub unsafe fn uiret() {
    unsafe { asm!(".byte 0xf3", ".byte 0x0f", ".byte 0x01", ".byte 0xec") }
}

/// Determine User Interrupt Flag
#[inline]
pub fn testui() -> bool {
    unsafe {
        asm!(".byte 0xf3", ".byte 0x0f", ".byte 0x01", ".byte 0xed");
        #[cfg(target_arch = "x86_64")]
        {
            use x86::current::rflags::{RFlags, read};
            read().contains(RFlags::FLAGS_CF)
        }
        #[cfg(target_arch = "x86")]
        {
            use x86::current::eflags::{EFlags, read};
            read().contains(EFlags::FLAGS_CF)
        }
    }
}

/// Clear User Interrupt Flag
#[inline]
pub fn clui() {
    unsafe {
        asm!(
            ".byte 0xf3",
            ".byte 0x0f",
            ".byte 0x01",
            ".byte 0xee",
            options(nostack, nomem)
        )
    }
}

/// Set User Interrupt Flag
#[inline]
pub fn stui() {
    unsafe {
        asm!(
            ".byte 0xf3",
            ".byte 0x0f",
            ".byte 0x01",
            ".byte 0xef",
            options(nostack, nomem)
        )
    }
}

#[inline]
pub fn uirqs_enabled() -> bool {
    testui()
}

#[inline]
pub fn disable_uirqs() {
    clui();
}

#[inline]
pub fn enable_uirqs() {
    stui();
}

/// Send User Interprocessor Interrupt
///
/// The SENDUIPI instruction sends the user interprocessor interrupt (IPI)
/// indicated by its register operand.
///
/// SENDUIPI uses a data structure called the user-interrupt target table (UITT).
/// This table is located at the linear address UITTADDR (in the IA32_UINTR_TT MSR);
/// it comprises UITTSZ+1 16-byte entries, where UITTSZ = IA32_UINT_MISC[31:0].
/// SENDUIPI uses the UITT entry (UITTE) indexed by the instruction's register operand.
///
/// Although SENDUIPI may be executed at any privilege level, all of the instruction’s
/// memory accesses (to a UITTE and a UPID) are performed with supervisor privilege.
///
/// SENDUIPI sends a user interrupt by posting a user interrupt with vector V
/// in the UPID referenced by UPIDADDR and then sending, as an ordinary IPI,
/// any notification interrupt specified in that UPID.
///
/// # Safety
///
/// The caller must ensure that uitte_index does not exceed UITTSZ and
/// UITT[uitte_index] is a valit UITT entry.
#[inline]
pub unsafe fn send_uipi(uitte_index: u64) {
    unsafe {
        asm!(
            ".byte 0xf3",
            ".byte 0x0f",
            ".byte 0xc7",
            ".byte 0xf0",
            in("rax") uitte_index,
            options(nostack)
        )
    };
}
