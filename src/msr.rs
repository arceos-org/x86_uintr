use tock_registers::{LocalRegisterCopy, fields::FieldValue, register_bitfields};
use x86::msr::{rdmsr, wrmsr};

/// User Interrupts support
pub const X86_FEATURE_UINTR: u32 = 18 * 32 + 5;
/// enable User Interrupts support
pub const X86_CR4_UINTR_BIT: u32 = 25;
pub const X86_CR4_UINTR: u32 = 1 << X86_CR4_UINTR_BIT;

// User Interrupt interface
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types, dead_code)]
pub enum UintrMsr {
    IA32_UINTR_RR = 0x985,
    IA32_UINTR_HANDLER = 0x986,
    IA32_UINTR_STACKADJUST = 0x987,
    IA32_UINTR_MISC = 0x988, // 39:32-UINV, 31:0-UITTSZ
    IA32_UINTR_PD = 0x989,
    IA32_UINTR_TT = 0x98a,
}

impl UintrMsr {
    /// Read 64 bits msr register.
    #[inline(always)]
    pub fn read(self) -> u64 {
        unsafe { rdmsr(self as _) }
    }

    /// Write 64 bits to msr register.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this write operation has no unsafe side
    /// effects.
    #[inline(always)]
    pub unsafe fn write(self, value: u64) {
        unsafe { wrmsr(self as _, value) }
    }
}

register_bitfields! [u64,
    /// UISTACKADJUST: user-interrupt stack adjustment.
    /// This value controls adjustment to the stack pointer (RSP) prior to
    /// user-interrupt delivery. It can account for an OS ABI’s “red zone”
    /// or be configured to load RSP with an alternate stack pointer.
    /// The value UISTACKADJUST must be canonical.
    /// If bit 0 is 1, user-interrupt delivery loads RSP with UISTACKADJUST;
    /// otherwise, it subtracts UISTACKADJUST from RSP.
    /// Either way, user-interrupt delivery then aligns RSP to a 16-byte boundary.
    /// See Section 11.4.2 for details.
    pub StackAdjust [
        MODE OFFSET(0) NUMBITS(1) [
            Subtract = 0,
            Load = 1
        ],
        ADDR OFFSET(1) NUMBITS(63) [],
    ],

    /// This definition adheres to the xstate component and contains an extra
    /// UIF field, which is not part of the IA32_UINTR_MISC MSR and marked as reserved.
    pub Misc [
        /// UITTSZ: user-interrupt target table size.
        /// This value is the highest index of a valid entry in the UITT (see Section 11.7).
        UITTSZ OFFSET(0) NUMBITS(32),

        /// UINV: user-interrupt notification vector.
        /// This is the vector of those ordinary interrupts that are treated as
        /// user-interrupt notifications (Section 11.5.1).
        /// When the logical processor receives user-interrupt notification,
        /// it processes the user interrupts in the user posted-interrupt descriptor (UPID)
        /// referenced by UPIDADDR (see below and Section 11.5.2).
        UINV OFFSET(32) NUMBITS(8),

        /// UIF: user-interrupt flag.
        /// If UIF = 0, user-interrupt delivery is blocked;
        /// if UIF = 1, user interrupts may be delivered.
        /// User-interrupt delivery clears UIF, and the new UIRET instruction sets it.
        /// Section 11.6 defines other new instructions for accessing UIF.
        ///
        /// Because bit 7 of byte 23 is for UIF (which is not part of the IA32_UINTR_MISC MSR),
        /// software that reads a value from bytes 23:16 should clear bit 63
        /// of that 64-bit value before attempting to write it to the IA32_UINTR_MISC MSR.
        UIF OFFSET(63) NUMBITS(1),
    ],

    pub PostDesc [
        /// UPIDADDR: user posted-interrupt descriptor address.
        /// This is the linear address of the UPID that the logical processor
        /// consults upon receiving an ordinary interrupt with vector UINV.
        UPIDADDR OFFSET(6) NUMBITS(58),
    ],

    pub TargetTable [
        /// Bit 0 of this MSR determines whether the SENDUIPI instruction is enabled.
        /// WRMSR may set it to either 0 or 1.
        SEND_ENABLED OFFSET(0) NUMBITS(1),
        /// UITTADDR: user-interrupt target table address.
        /// This is the linear address of user-interrupt target table (UITT),
        /// which the logical processor consults when software invokes
        /// the SENDUIPI instruction (see Section 11.7).
        UITTADDR OFFSET(4) NUMBITS(60)
    ]
];

pub type StackAdjustLocal = LocalRegisterCopy<u64, StackAdjust::Register>;
pub type StackAdjustFieldValue = FieldValue<u64, StackAdjust::Register>;
pub type StackAdjustMode = StackAdjust::MODE::Value;
pub type MiscLocal = LocalRegisterCopy<u64, Misc::Register>;
pub type PostDescLocal = LocalRegisterCopy<u64, PostDesc::Register>;
pub type TargetTableLocal = LocalRegisterCopy<u64, TargetTable::Register>;
