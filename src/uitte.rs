//! UITTE: User Interrupt Target Table Entry

use core::fmt::{Debug, Formatter, Result};

use crate::msr::{PostDesc, PostDescLocal};
use tock_registers::{LocalRegisterCopy, register_bitfields, register_structs};

register_bitfields![u64,
    VUV [
        VALID OFFSET(0) NUMBITS(1),
        UINTR_VECTOR OFFSET(8) NUMBITS(6)
    ]
];

pub type VuvLocal = LocalRegisterCopy<u64, VUV::Register>;

register_structs! {
    #[repr(C, align(16))]
    pub UittEntry {
        (0x00 => state: VuvLocal),
        (0x08 => upid_addr: PostDescLocal),
        (0x10 => @END),
    }
}

impl UittEntry {
    pub fn new(uintr_vector: u8, upid_addr: u64) -> Self {
        Self {
            state: VuvLocal::new(
                VUV::VALID::SET.value | VUV::UINTR_VECTOR.val(uintr_vector as _).value,
            ),
            upid_addr: PostDescLocal::new(upid_addr & PostDesc::UPIDADDR::SET.mask()),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.state.read(VUV::VALID) == 1
    }

    pub fn set_valid(&mut self, valid: bool) {
        self.state.modify(VUV::VALID.val(valid as _));
    }

    pub fn uintr_vector(&self) -> u64 {
        self.state.read(VUV::UINTR_VECTOR)
    }
}

impl Debug for UittEntry {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("UittEntry")
            .field("valid", &(self.is_valid()))
            .field(
                "UINV",
                &(format_args!("{:#x}", self.state.read(VUV::UINTR_VECTOR))),
            )
            .field("upid_addr", &(format_args!("{:#x}", self.upid_addr.get())))
            .finish()
    }
}
