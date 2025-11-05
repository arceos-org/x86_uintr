//! UPID: User Posted-Interrupt Descriptor

use core::fmt::{Debug, Formatter, Result};

use tock_registers::{LocalRegisterCopy, register_bitfields};

register_bitfields![u64,
    NotificationControl [
        /// If this bit is set, there is a notification outstanding for one or
        /// more user interrupts in PIR.
        OUTSTANDING OFFSET(0) NUMBITS(1),
        /// If this bit is set, agents (including SENDUIPI) should not send
        /// notifications when posting user interrupts in this descriptor.
        SUPPRESSED OFFSET(1) NUMBITS(1),
        /// Used by SENDUIPI
        VECTOR OFFSET(16) NUMBITS(8),
        /// Target physical APIC ID – used by SENDUIPI.
        /// In xAPIC mode, bits 47:40 are the 8-bit APIC ID.
        /// In x2APIC mode, the entire field forms the 32-bit APIC ID.
        DESTINATION OFFSET(32) NUMBITS(32)
    ]
];

pub type NotificationControlLocal = LocalRegisterCopy<u64, NotificationControl::Register>;

#[repr(C, align(64))]
pub struct Upid {
    pub control: NotificationControlLocal,
    /// One bit for each user-interrupt vector.
    /// There is a user-interrupt request for a vector if the corresponding bit is 1.
    pub posted_uirq: LocalRegisterCopy<u64>,
}

impl Upid {
    pub fn new(outstanding: bool, suppressed: bool, notif_vector: u8, destination: u32) -> Self {
        Self {
            control: NotificationControlLocal::new(
                NotificationControl::OUTSTANDING.val(outstanding as _).value
                    | NotificationControl::SUPPRESSED.val(suppressed as _).value
                    | NotificationControl::VECTOR.val(notif_vector as _).value
                    | NotificationControl::DESTINATION.val(destination as _).value,
            ),
            posted_uirq: LocalRegisterCopy::new(0),
        }
    }

    pub fn set_notification_enabled(&mut self, enabled: bool) {
        self.control
            .modify(NotificationControl::SUPPRESSED.val(!enabled as _));
    }

    pub fn set_outstanding_notification(&mut self, outstanding: bool) {
        self.control
            .modify(NotificationControl::OUTSTANDING.val(outstanding as _));
    }
}

impl Debug for Upid {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Upid")
            .field(
                "outstanding",
                &(self.control.is_set(NotificationControl::OUTSTANDING)),
            )
            .field(
                "suppressed",
                &(self.control.is_set(NotificationControl::SUPPRESSED)),
            )
            .field(
                "vector",
                &(format_args!("{:#x}", self.control.read(NotificationControl::VECTOR))),
            )
            .field(
                "destination",
                &(self.control.read(NotificationControl::DESTINATION)),
            )
            .field("UPIR", &format_args!("{:#x}", self.posted_uirq.get()))
            .finish()
    }
}
