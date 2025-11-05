use crate::{
    instructions::{disable_uirqs, enable_uirqs, uirqs_enabled},
    msr::*,
    uitte::UittEntry,
};
use core::fmt::{Debug, Formatter, Result};

use core::slice;
use tock_registers::{LocalRegisterCopy, register_structs};

register_structs! {
    /// State component 14 is supervisor state used for User Interrupts state.
    /// The size of this state is 48 bytes.
    #[repr(C)]
    pub UintrState {
        /// UIHANDLER: user-interrupt handler.
        /// This is the linear address of the user-interrupt handler.
        /// User-interrupt delivery loads this address into RIP.
        (0 => pub handler: LocalRegisterCopy<u64>),
        (8 => stack_adjust: StackAdjustLocal),
        (16 => misc: MiscLocal),
        (24 => post_desc: PostDescLocal),
        /// UIRR: user-interrupt request register.
        /// This value includes one bit for each of the 64 user-interrupt vectors.
        /// If UIRR[i] = 1, a user interrupt with vector i is requesting service.
        /// The notation UIRRV is used to refer to the position of the
        /// most significant bit set in UIRR; if UIRR = 0, UIRRV = 0.
        (32 => pub uirr: LocalRegisterCopy<u64>),
        (40 => target_table: TargetTableLocal),
        (48 => @END),
    }
}

impl UintrState {
    pub const fn default() -> Self {
        let zero = LocalRegisterCopy::new(0);
        Self {
            handler: zero,
            stack_adjust: StackAdjustLocal::new(0),
            misc: MiscLocal::new(0),
            post_desc: PostDescLocal::new(0),
            uirr: zero,
            target_table: TargetTableLocal::new(0),
        }
    }

    pub fn new(
        uitt_addr: u64,
        uitt_sz: u64,
        sender_enabled: bool,
        handler_addr: u64,
        stack_addr: u64,
        stack_mode: StackAdjustMode,
        notif_vector: u64,
        receiver_enabled: bool,
        post_desc_addr: u64,
    ) -> Self {
        Self {
            handler: LocalRegisterCopy::new(handler_addr),
            stack_adjust: StackAdjustLocal::new(
                (stack_addr & StackAdjust::ADDR::SET.mask())
                    | StackAdjustFieldValue::from(stack_mode).value,
            ),
            misc: MiscLocal::new(
                Misc::UITTSZ.val(uitt_sz).value
                    | Misc::UINV.val(notif_vector).value
                    | Misc::UIF.val(receiver_enabled as _).value,
            ),
            post_desc: PostDescLocal::new(post_desc_addr & PostDesc::UPIDADDR::SET.mask()),
            uirr: LocalRegisterCopy::new(0),
            target_table: TargetTableLocal::new(
                TargetTable::SEND_ENABLED.val(sender_enabled as u64).value
                    | (uitt_addr & TargetTable::UITTADDR::SET.mask()),
            ),
        }
    }

    pub fn set_sender(&mut self, uitt_addr: u64, uitt_sz: u64, enabled: bool) {
        self.misc.modify(Misc::UITTSZ.val(uitt_sz));
        self.target_table.set(
            TargetTable::SEND_ENABLED.val(enabled as u64).value
                | (uitt_addr & TargetTable::UITTADDR::SET.mask()),
        );
    }

    /// Store UINTR receiver related states into the struct.
    /// This function does not write to the MSRs.
    pub fn set_receiver(
        &mut self,
        handler_addr: u64,
        stack_addr: u64,
        stack_mode: StackAdjustMode,
        notif_vector: u64,
        enabled: bool,
        post_desc_addr: u64,
    ) {
        self.handler.set(handler_addr);
        self.stack_adjust.set(
            (stack_addr & StackAdjust::ADDR::SET.mask())
                | StackAdjustFieldValue::from(stack_mode).value,
        );
        self.misc.modify(Misc::UINV.val(notif_vector));
        self.misc.modify(Misc::UIF.val(enabled as _));
        self.post_desc
            .set(post_desc_addr & PostDesc::UPIDADDR::SET.mask());
    }

    /// Read UITT and UITTSZ from MSR
    #[inline]
    pub fn save_sender(&mut self) {
        self.target_table.set(UintrMsr::IA32_UINTR_TT.read());
        self.read_misc();
    }

    /// Read handler, stack adjust, UINV, UIF, UPID, and UIRR from MSR
    #[inline]
    pub fn save_receiver(&mut self) {
        self.handler.set(UintrMsr::IA32_UINTR_HANDLER.read());
        self.stack_adjust
            .set(UintrMsr::IA32_UINTR_STACKADJUST.read());
        self.read_misc();
        self.post_desc.set(UintrMsr::IA32_UINTR_PD.read());
        self.uirr.set(UintrMsr::IA32_UINTR_RR.read());
    }

    /// Read all UINTR states from MSR
    #[inline]
    pub fn save_all(&mut self) {
        self.handler.set(UintrMsr::IA32_UINTR_HANDLER.read());
        self.stack_adjust
            .set(UintrMsr::IA32_UINTR_STACKADJUST.read());
        self.read_misc();
        self.post_desc.set(UintrMsr::IA32_UINTR_PD.read());
        self.uirr.set(UintrMsr::IA32_UINTR_RR.read());
        self.target_table.set(UintrMsr::IA32_UINTR_TT.read());
    }

    #[inline]
    fn read_misc(&mut self) {
        self.misc.set(UintrMsr::IA32_UINTR_MISC.read());
        self.misc.modify(Misc::UIF.val(uirqs_enabled() as u64));
    }

    #[inline]
    fn write_misc(&self) {
        if self.misc.is_set(Misc::UIF) {
            enable_uirqs();
        } else {
            disable_uirqs();
        }
        let mut misc_msr = self.misc;
        misc_msr.modify(Misc::UIF::CLEAR);
        unsafe {
            UintrMsr::IA32_UINTR_MISC.write(misc_msr.get());
        }
    }

    /// Write UITT and UITTSZ to MSR
    #[inline]
    pub fn restore_sender(&self) {
        self.write_misc();
        unsafe {
            UintrMsr::IA32_UINTR_TT.write(self.target_table.get());
        }
    }

    /// Write handler, stack adjust, UINV, UIF, UPID, and UIRR to MSR
    #[inline]
    pub fn restore_receiver(&self) {
        self.write_misc();
        unsafe {
            UintrMsr::IA32_UINTR_HANDLER.write(self.handler.get());
            UintrMsr::IA32_UINTR_STACKADJUST.write(self.stack_adjust.get());
            UintrMsr::IA32_UINTR_PD.write(self.post_desc.get());
            UintrMsr::IA32_UINTR_RR.write(self.uirr.get());
        }
    }

    /// Write all UINTR states to MSR
    #[inline]
    pub fn restore_all(&self) {
        self.write_misc();
        unsafe {
            UintrMsr::IA32_UINTR_HANDLER.write(self.handler.get());
            UintrMsr::IA32_UINTR_STACKADJUST.write(self.stack_adjust.get());
            UintrMsr::IA32_UINTR_PD.write(self.post_desc.get());
            UintrMsr::IA32_UINTR_RR.write(self.uirr.get());
            UintrMsr::IA32_UINTR_TT.write(self.target_table.get());
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the UITTADDR[0, UITTSZ] point to valid
    /// memory addresses containing UITT entries.
    pub unsafe fn uitt(&self) -> &[UittEntry] {
        let uitt_sz = self.misc.read(Misc::UITTSZ);
        unsafe { slice::from_raw_parts(self.target_table.get() as *const _, uitt_sz as usize) }
    }

    /// # Safety
    ///
    /// The caller must ensure that the UITTADDR[0, UITTSZ] point to valid
    /// memory addresses containing UITT entries.
    pub unsafe fn uitt_mut(&mut self) -> &mut [UittEntry] {
        let uitt_sz = self.misc.read(Misc::UITTSZ);
        unsafe { slice::from_raw_parts_mut(self.target_table.get() as *mut _, uitt_sz as usize) }
    }
}

impl Debug for UintrState {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("UintrState")
            .field("handler", &format_args!("{:#x}", self.handler.get()))
            .field(
                "stack_mode",
                &(if self.stack_adjust.read(StackAdjust::MODE) == StackAdjust::MODE::Subtract.value
                {
                    "substract"
                } else {
                    "load"
                }),
            )
            .field(
                "stack_addr",
                &format_args!("{:#x}", self.stack_adjust.get()),
            )
            .field("UITTSZ", &(self.misc.read(Misc::UITTSZ)))
            .field("UINV", &(format_args!("{:#x}", self.misc.read(Misc::UINV))))
            .field("UIF", &(self.misc.is_set(Misc::UIF)))
            .field("UPID addr", &(format_args!("{:#x}", self.post_desc.get())))
            .field("UIRR", &(format_args!("{:#x}", self.uirr.get())))
            .field(
                "send_enabled",
                &(self.target_table.is_set(TargetTable::SEND_ENABLED)),
            )
            .field(
                "UITT addr",
                &format_args!(
                    "{:#x}",
                    self.target_table.get()
                        & (TargetTable::UITTADDR.mask << TargetTable::UITTADDR.shift)
                ),
            )
            .finish()
    }
}
