use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct Inner<T> {
    data: *mut T,
    ref_count: AtomicUsize,
    borrow_count: AtomicUsize,
    borrow_mut_count: AtomicUsize,
}

impl<T> Inner<T> {
    fn new(data: T) -> Self {
        let data = Box::into_raw(Box::new(data));
        Self {
            data,
            ref_count: AtomicUsize::new(1),
            borrow_count: AtomicUsize::new(0),
            borrow_mut_count: AtomicUsize::new(0),
        }
    }
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.data));
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum ArcUnsafeRefCellError {
    BorrowError,
    BorrowMutError,
    UnableToDrop,
}

impl Display for ArcUnsafeRefCellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArcUnsafeRefCellError::BorrowError => write!(f, "BorrowError"),
            ArcUnsafeRefCellError::BorrowMutError => write!(f, "BorrowMutError"),
            ArcUnsafeRefCellError::UnableToDrop => write!(f, "UnableToDrop"),
        }
    }
}

pub(crate) struct UnsafeRef<T> {
    data: *mut T,
    inner: NonNull<Inner<T>>,
}

impl<T> Deref for UnsafeRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> Drop for UnsafeRef<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.inner.as_ref();
            inner
                .borrow_count
                .fetch_sub(1, Ordering::Release);
        }
    }
}

pub(crate) struct UnsafeRefMut<T> {
    data: *mut T,
    drop_mut: bool,
    inner: NonNull<Inner<T>>,
}

impl<T> Deref for UnsafeRefMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for UnsafeRefMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for UnsafeRefMut<T> {
    fn drop(&mut self) {
        unsafe {
            if self.drop_mut {
                let inner = self.inner.as_ref();
                inner
                    .borrow_mut_count
                    .fetch_sub(1, Ordering::Release);
            }
        }
    }
}

pub struct ArcUnsafeRefCellWrapper<T> {
    data: NonNull<Inner<T>>,
}

// 添加Send和Sync标记，使类型成为正确的Arc
unsafe impl<T: Send + Sync> Send for ArcUnsafeRefCellWrapper<T> {}
unsafe impl<T: Send + Sync> Sync for ArcUnsafeRefCellWrapper<T> {}

#[allow(dead_code)]
impl<T> ArcUnsafeRefCellWrapper<T> {
    pub fn new(data: T) -> Self {
        let inner = Box::into_raw(Box::new(Inner::new(data)));
        let data = NonNull::new(inner).unwrap();
        Self { data }
    }

    pub fn borrow(&self) -> Result<UnsafeRef<T>, ArcUnsafeRefCellError> {
        unsafe {
            let inner = self.data.as_ref();

            if inner
                .borrow_mut_count
                .load(Ordering::Acquire)
                > 0
            {
                return Err(ArcUnsafeRefCellError::BorrowError);
            }

            inner
                .borrow_count
                .fetch_add(1, Ordering::Acquire);

            Ok(UnsafeRef {
                data: inner.data,
                inner: self.data,
            })
        }
    }

    pub fn clone_ref(&self) -> Result<(), ArcUnsafeRefCellError> {
        unsafe {
            let inner = self.data.as_ref();
            if inner
                .borrow_mut_count
                .load(Ordering::Acquire)
                > 0
            {
                return Err(ArcUnsafeRefCellError::BorrowError);
            }
            inner
                .ref_count
                .fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    pub fn drop_ref(&self) -> Result<(), ArcUnsafeRefCellError> {
        unsafe {
            let inner = self.data.as_ref();
            if inner.ref_count.load(Ordering::Acquire) <= 1 {
                // 不能再减少引用计数
                return Err(ArcUnsafeRefCellError::UnableToDrop);
            }
            inner
                .ref_count
                .fetch_sub(1, Ordering::Release);
            Ok(())
        }
    }

    pub fn borrow_mut(&self) -> Result<UnsafeRefMut<T>, ArcUnsafeRefCellError> {
        unsafe {
            let inner = self.data.as_ref();

            if inner.borrow_count.load(Ordering::Acquire) > 0
                || inner
                    .borrow_mut_count
                    .load(Ordering::Acquire)
                    > 0
            {
                return Err(ArcUnsafeRefCellError::BorrowMutError);
            }

            inner
                .borrow_mut_count
                .fetch_add(1, Ordering::Acquire);

            Ok(UnsafeRefMut {
                data: inner.data,
                inner: self.data,
                drop_mut: true,
            })
        }
    }

    pub unsafe fn get_mut(&mut self) -> &mut T {
        unsafe {
            let inner = self.data.as_mut();
            &mut *inner.data
        }
    }

    pub unsafe fn get(&self) -> &T {
        unsafe {
            let inner = self.data.as_ref();
            &*inner.data
        }
    }

    /// Borrows the data without increasing the reference count.
    pub fn borrow_mut_unsafe(&self) -> Result<UnsafeRefMut<T>, ArcUnsafeRefCellError> {
        unsafe {
            // 借用但不增加引用计数
            let inner = self.data.as_ref();
            if inner.borrow_count.load(Ordering::Acquire) > 0
                || inner
                    .borrow_mut_count
                    .load(Ordering::Acquire)
                    > 0
            {
                return Err(ArcUnsafeRefCellError::BorrowMutError);
            }
            Ok(UnsafeRefMut {
                data: inner.data,
                inner: self.data,
                drop_mut: false,
            })
        }
    }

    pub fn get_inner(&self) -> *mut Inner<T> {
        self.data.as_ptr()
    }

    pub fn from_inner(inner: *mut Inner<T>) -> Self {
        let data = NonNull::new(inner).unwrap();
        if unsafe {data.as_ref().ref_count.load(Ordering::Acquire) == 0} {
            panic!("Invalid reference count");
        }
        unsafe {
            let inner = data.as_ref();
            inner
                .ref_count
                .fetch_add(1, Ordering::Relaxed);
        }
        Self { data }
    }
    
    /// 获取当前的引用计数
    pub fn ref_count(&self) -> usize {
        unsafe {
            self.data.as_ref().ref_count.load(Ordering::Acquire)
        }
    }
    
    /// 判断是否唯一持有者
    pub fn is_unique(&self) -> bool {
        self.ref_count() == 1
    }
}

impl<T> Clone for ArcUnsafeRefCellWrapper<T> {
    fn clone(&self) -> Self {
        unsafe {
            let inner = self.data.as_ref();
            inner
                .ref_count
                .fetch_add(1, Ordering::Relaxed);
        }
        Self { data: self.data }
    }
}

impl<T> Drop for ArcUnsafeRefCellWrapper<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.data.as_ref();
            // 使用正确的内存排序模式
            // 这里使用 Release 是因为我们释放内存可能要对其他线程可见
            if inner
                .ref_count
                .fetch_sub(1, Ordering::Release)
                == 1
            {
                // 需要一个额外的内存屏障，确保在释放内存前，所有线程不会再访问此数据
                std::sync::atomic::fence(Ordering::Acquire);
                drop(Box::from_raw(self.data.as_ptr()));
            }
        }
    }
}
