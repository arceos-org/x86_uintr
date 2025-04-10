//! UPID: User Posted-Interrupt Descriptor

use tock_registers::{LocalRegisterCopy, register_bitfields, register_structs};

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

register_structs! {
    pub Upid {
        (0x00 => control: NotificationControlLocal),
        /// One bit for each user-interrupt vector.
        /// There is a user-interrupt request for a vector if the corresponding bit is 1.
        (0x08 => posted_uirq: LocalRegisterCopy<u64>),
        (0x10 => @END),
    }
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
}
