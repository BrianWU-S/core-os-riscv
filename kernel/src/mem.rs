// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use core::ops::Range;
use crate::info;
pub use crate::symbols::*;

pub const MAX_PAGE: usize = 128 * 1024 * 1024 / (1 << 12);

pub struct Allocator {
    pub page_allocated: [usize; MAX_PAGE],
    pub base_addr: usize,
}

pub const fn align_val(val: usize, order: usize) -> usize {
    let o = (1usize << order) - 1;
    (val + o) & !o
}

pub const fn align_val_down(val: usize, order: usize) -> usize {
    val & !((1usize << order) - 1)
}

pub const fn page_down(val: usize) -> usize {
    align_val_down(val, PAGE_ORDER)
}

use crate::{println, panic};

impl Allocator {
    pub const fn new() -> Self {
        Allocator {
            base_addr: 0,
            page_allocated: [0; MAX_PAGE],
        }
    }

    fn offset_addr_of(&self, id: usize) -> usize {
        let addr = self.base_addr + id * PAGE_SIZE;
        addr
    }
    unsafe fn offset_id_of(&self, id: usize) -> *mut u8 {
        self.offset_addr_of(id) as *mut u8
    }

    fn offset_page_of(&self, page: *mut u8) -> usize {
        let id = (page as usize - self.base_addr) / PAGE_SIZE;
        id
    }

    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        let page_required = align_val(size, PAGE_ORDER) / PAGE_SIZE;
        for i in 0..MAX_PAGE {
            if self.page_allocated[i] == 0 {
                let mut found = true;
                for j in 0..page_required {
                    if self.page_allocated[i + j] != 0 {
                        found = false;
                        break;
                    }
                }
                if found {
                    for j in 0..page_required {
                        self.page_allocated[i + j] = page_required;
                    }
                    unsafe { return self.offset_id_of(i); }
                }
            }
        }
        panic!("no available page")
    }

    pub fn deallocate(&mut self, addr: *mut u8) {
        let id = self.offset_page_of(addr);
        let page_stride = self.page_allocated[id];
        for j in 0..page_stride {
            self.page_allocated[j + id] = 0;
        }
    }

    pub fn debug(&self) {
        let mut j = 0;
        loop {
            let size = self.page_allocated[j];
            let addr = &self.page_allocated as *const usize;
            let addr = unsafe { addr.add(j) };
            if size != 0 {
                let from = self.offset_addr_of(j);
                let to = self.offset_addr_of(j + size);
                println!("{} {:X} {:X}-{:X} (pages: {:X})", j, addr as usize, from, to, size);
                j += size;
            } else {
                j += 1;
            }
            if j == MAX_PAGE {
                break;
            }
        }
    }
}

use crate::nulllock::Mutex;

static __ALLOC: Mutex<Allocator> = Mutex::new(Allocator::new(), "alloc");

pub fn init() {
    unsafe {
        ALLOC().lock().base_addr = align_val(HEAP_START, PAGE_ORDER);
    }
    // workaround for non-zero data region
    let mut alloc = ALLOC().lock();
    for i in 0..MAX_PAGE {
        alloc.page_allocated[i] = 0;
    }
}

pub fn ALLOC() -> &'static Mutex<Allocator> { &__ALLOC }

use core::alloc::{GlobalAlloc, Layout};

struct OsAllocator {}

unsafe impl GlobalAlloc for OsAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        info!("{}", layout.size());
        ALLOC().lock().allocate(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        ALLOC().lock().deallocate(ptr);
    }
}

#[global_allocator]
static GA: OsAllocator = OsAllocator {};

#[alloc_error_handler]
pub fn alloc_error(l: Layout) -> ! {
    panic!(
        "Allocator failed to allocate {} bytes with {}-byte alignment.",
        l.size(),
        l.align()
    );
}

pub unsafe fn zero_volatile<T>(range: Range<*mut T>)
    where
        T: From<u8>,
{
    let mut ptr = range.start;

    while ptr < range.end {
        core::ptr::write_volatile(ptr, T::from(0));
        ptr = ptr.offset(1);
    }
}

pub fn debug() {
    for i in 0x8004f000 as u64..0x80093058 {
        let d = unsafe { core::ptr::read(i as *const u8) };
        if d != 0 {
            println!("0x{:x}: {:x}", i, d);
        }
        if i % 0x100000 == 0 {
            println!("{:x}", i);
        }
    }
}