// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! RISC-V related functions

use core::time::Duration;
use crate::panic;
use riscv::register::*;
use crate::symbols::*;

/// Get current time from MMIO
pub fn time() -> Duration {
    let mtime = crate::clint::CLINT_MTIME_BASE as *const u64;
    Duration::from_nanos(unsafe { mtime.read_volatile() } * 100)
}

/// Build satp value from mode, asid and page table base addr
pub fn build_satp(mode: usize, asid: usize, addr: usize) -> usize {
    if addr % PAGE_SIZE != 0 {
        panic!("satp not aligned!");
    }
    (mode as usize) << 60 | (asid & 0xffff) << 44 | (addr >> 12) & 0xff_ffff_ffff
}

/// Enable interrupt
pub fn intr_on() {
    unsafe {
        sie::set_sext();
        sie::set_ssoft();
        sie::set_stimer();
        sstatus::set_sie();
    }
}

/// Turn off interrupt
pub fn intr_off() {
    unsafe {
        sstatus::clear_sie();
    }
}

/// Check if interrupt is enabled
pub fn intr_get() -> bool {
    unsafe {
        sstatus::read().sie()
    }
}

#[inline(always)]
pub fn hart_id() -> usize {
    let mut hart_id: usize = 0;
    unsafe { asm!("mv $0, tp" : "=r"(hart_id) :: "volatile"); }
    hart_id
}

#[inline]
pub fn r_sip() -> usize {
    let mut sip: usize = 0;
    unsafe { asm!("csrr $0, sip" : "=r"(sip) :: "volatile"); }
    sip
}

#[inline]
pub fn w_sip(x: usize) {
    unsafe { asm!("csrw sip, $0" :: "r"(x) : "volatile"); }
}

#[inline]
pub fn r_sstatus() -> usize {
    let mut x: usize = 0;
    unsafe { asm!("csrr $0, sstatus" : "=r"(x) :: "volatile"); }
    x
}

#[inline]
pub fn r_satp() -> usize {
    let mut x: usize = 0;
    unsafe { asm!("csrr $0, satp" : "=r"(x) :: "volatile"); }
    x
}

#[inline]
pub fn w_sstatus(x: usize) {
    unsafe { asm!("csrw sstatus, $0" :: "r"(x) : "volatile"); }
}

extern "C" { fn __sp() -> usize; }

pub fn sp() -> usize {
    unsafe { __sp() }
}

pub fn wait_forever() -> ! {
    loop {
        unsafe {
            riscv::asm::wfi();
        }
    }
}
