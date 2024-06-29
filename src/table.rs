use std::{
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

use orx_concurrent_vec::ConcurrentVec;

use crate::Revision;

const PAGE_BITS: usize = 16;
const PAGE_SIZE: usize = 2 << PAGE_BITS;

#[derive(Default)]
pub struct Table {
    pages: ConcurrentVec<Page>,
}

pub struct Page {
    page_impl: Box<dyn PageOps>,
    initialized: AtomicUsize,
    data: NonNull<()>,
}

pub struct PageIndex(usize);

pub struct SlotIndex(usize);

/// Error from `Page::push` indicating that the page has reached
/// capacity and a new page is needed.
pub struct PageFull;

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_page<T: 'static>(&mut self) -> PageIndex {
        PageIndex(self.pages.push(Page::new::<T>()))
    }

    pub fn get_page(&self, index: PageIndex) -> &Page {
        self.pages.get(index.0).unwrap()
    }
}

impl Page {
    fn new<T: 'static>() -> Self {
        let page_impl = PageImpl::<T>::new();
        let data: NonNull<()> = page_impl.data.cast();
        Page {
            page_impl: Box::new(page_impl),
            initialized: AtomicUsize::new(0),
            data,
        }
    }

    /// Push a new value onto the page, giving it a unique index.
    /// Returns Err if the page is full.
    ///
    /// # Unsafety
    ///
    /// Must be same `T` that the `Page` was created with
    pub unsafe fn push<T>(&self, revision: Revision, value: T) -> Result<SlotIndex, PageFull> {
        // Find a unique slot within the page.
        // Be can't use `fetch_add` because we want to be sure we don't
        // store more than `PAGE_SIZE` slots.
        let mut index;
        loop {
            let initialized = self.initialized.load(Ordering::Relaxed);
            if initialized == PAGE_SIZE {
                return Err(PageFull);
            }

            index = initialized + 1;
            if self
                .initialized
                .compare_exchange(initialized, index, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        let data: NonNull<Slot<T>> = self.data.cast();
        let data_ptr = data.as_ptr().add(index);
        std::ptr::write(
            data_ptr,
            Slot {
                revision: Some(revision),
                value: MaybeUninit::new(value),
            },
        );

        Ok(SlotIndex(index))
    }

    /// Remove the data from the given slot.
    ///
    /// Requires an `&mut self` to ensure that any references previously given out
    /// to this data must have expired.
    ///
    /// # Unsafety
    ///
    /// Must be same `T` that the `Page` was created with.
    pub unsafe fn clear<T>(&mut self, slot: SlotIndex, revision: Revision) {
        let mut data = unsafe { self.slot_data::<T>(slot, Some(revision)) };
        let data = unsafe { data.as_mut() };
        data.revision = None;
        unsafe {
            data.value.assume_init_drop();
        }
    }

    /// Reinitialize the data in the given slot. The slot must not be initialized.
    ///
    /// # Unsafety
    ///
    /// Must be same `T` that the `Page` was created with.
    ///
    /// Must not race with any other calls to `reinitialize` the same slot.
    pub unsafe fn reinitialize<T>(&self, slot: SlotIndex, revision: Revision, value: T) {
        let mut data = unsafe { self.slot_data::<T>(slot, None) };
        let data = unsafe { data.as_mut() };
        data.value = MaybeUninit::new(value);
        data.revision = Some(revision);

        // FIXME: We are being a bit fast-and-loose with respect to the atomic model
        // and should fix it. There could be other threads with outdated pointers trying to call
        // `get` on this slot. Their revisions ought not to match (though of course we can't
        // prove that, have to add that to the unsafety conditions) but still the compiler needs to
        // know of the possibility of concurrent access. I'm not sure the easiest correct way to signal that.
    }

    /// # Unsafety
    ///
    /// Must be same `T` that the `Page` was created with.
    pub unsafe fn get<T>(&self, slot: SlotIndex, revision: Revision) -> &T {
        let data = unsafe { self.slot_data::<T>(slot, Some(revision)) };
        unsafe { data.as_ref().value.assume_init_ref() }
    }

    unsafe fn slot_data<T>(&self, slot: SlotIndex, revision: Option<Revision>) -> NonNull<Slot<T>> {
        assert!(
            slot.0 < self.initialized.load(Ordering::Relaxed),
            "trying to access uninitialized slot"
        );
        let data: NonNull<Slot<T>> = self.data.cast();
        let data: NonNull<Slot<T>> = unsafe { NonNull::new_unchecked(data.as_ptr().add(slot.0)) };

        assert_eq!(
            unsafe { data.as_ref().revision },
            revision,
            "revision does not match, pointer must have been leaked"
        );

        data
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        unsafe {
            let initialized = self.initialized.load(Ordering::Relaxed);
            self.page_impl.drop_page(initialized);
        }
    }
}

struct Slot<T> {
    revision: Option<Revision>,
    value: MaybeUninit<T>,
}

impl<T> Drop for Slot<T> {
    fn drop(&mut self) {
        if self.revision.is_some() {
            unsafe { self.value.assume_init_drop() };
        }
    }
}

struct PageImpl<T> {
    data: NonNull<Slot<T>>,
}

trait PageOps {
    unsafe fn drop_page(&self, initialized: usize);
}

impl<T> PageImpl<T> {
    fn new() -> Self {
        let mut vec: Vec<Slot<T>> = Vec::with_capacity(PAGE_SIZE);
        let data: NonNull<Slot<T>> = unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) };
        std::mem::forget(vec);
        Self { data }
    }
}

impl<T> PageOps for PageImpl<T> {
    unsafe fn drop_page(&self, initialized: usize) {
        // UNSAFE CONDITION:
        //
        // The pointer was created from a vec in `Page::new`.
        unsafe {
            std::mem::drop(Vec::<Slot<T>>::from_raw_parts(
                self.data.as_ptr(),
                initialized,
                PAGE_SIZE,
            ));
        }
    }
}
