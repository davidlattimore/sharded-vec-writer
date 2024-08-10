use std::error::Error;
use std::fmt::Display;
use std::marker::PhantomData;

/// Builds a `Vec<T>`, with each variable-sized chunk of the Vec being initialised separately, most
/// likely from a separate thread.
pub struct VecWriter<'vec, T> {
    storage: &'vec mut Vec<T>,
    taken: usize,
}

/// A mutable borrow of part of a `Vec`. Can be used to initialise that part of the `Vec` before
/// returning it. Dropping a shard without returning it to the writer will drop any values that were
/// written into it.
pub struct Shard<'vec, T> {
    /// Pointer to the start off `storage` on the builder.
    storage: *mut T,

    /// The start offset within the original builder that we're responsible for.
    start_offset: usize,

    /// The exclusive end-offset up to which we're responsible for.
    end_offset: usize,

    /// The exclusive offset up to which we have initialised.
    initialised_up_to: usize,

    _phantom: PhantomData<&'vec mut T>,
}

impl<'vec, T> Drop for Shard<'vec, T> {
    fn drop(&mut self) {
        // We've been dropped without being returned to the writer, clean up any values that were
        // written so that they don't leak.
        for offset in self.start_offset..self.initialised_up_to {
            unsafe { self.storage.add(offset).read() };
        }
    }
}

unsafe impl<T: Send> Send for Shard<'_, T> {}
unsafe impl<T: Sync> Sync for Shard<'_, T> {}

impl<'vec, T> VecWriter<'vec, T> {
    /// Creates a new writer that will write into the supplied `Vec`.
    pub fn new(storage: &'vec mut Vec<T>) -> Self {
        let taken = storage.len();
        Self { storage, taken }
    }

    /// Takes the next `n` elements of the vector or panics if there is insufficient capacity.
    pub fn take_shard(&mut self, n: usize) -> Shard<'vec, T> {
        self.try_take_shard(n).unwrap_or_else(|| {
            panic!(
                "Tried to take {n} when only {} available",
                self.storage.capacity() - self.taken
            );
        })
    }

    /// Takes the next `n` elements of the vector or returns None if there is insufficient capacity.
    pub fn try_take_shard(&mut self, n: usize) -> Option<Shard<'vec, T>> {
        let end_offset = self.taken.saturating_add(n);
        if end_offset > self.storage.capacity() {
            return None;
        }
        let shard = Shard {
            storage: self.storage.as_mut_ptr(),
            start_offset: self.taken,
            initialised_up_to: self.taken,
            end_offset,
            _phantom: Default::default(),
        };
        self.taken = end_offset;
        Some(shard)
    }

    /// Returns a shard to the vector, increasing the initialised length of the vector by the size
    /// of the shard. The shard must have been fully initialised before being returned. Shards must
    /// be returned in order. Panics on failure.
    #[track_caller]
    pub fn return_shard(&mut self, shard: Shard<T>) {
        self.try_return_shard(shard).unwrap()
    }

    /// As for `return_shard`, but returns an error on failure rather than panicking.
    pub fn try_return_shard(&mut self, shard: Shard<T>) -> Result<(), InitError> {
        if self.storage.as_mut_ptr() != shard.storage {
            return Err(InitError::WrongVec);
        }
        if shard.initialised_up_to != shard.end_offset {
            return Err(InitError::UninitElements);
        }
        if self.storage.len() != shard.start_offset {
            return Err(InitError::OutOfOrder);
        }
        // Safety: All values between the previous length and the new length were set by writes in
        // `try_push`.
        unsafe { self.storage.set_len(shard.initialised_up_to) };

        // The values written into the shard are now owned by the vec, so forget the shard without
        // dropping it, otherwise it'll double-free the values in the shard.
        core::mem::forget(shard);
        Ok(())
    }
}

impl<'builder, T> Shard<'builder, T> {
    /// Appends a value to the shard. Panics if the shard has already been fully used.
    #[track_caller]
    pub fn push(&mut self, value: T) {
        self.try_push(value).unwrap();
    }

    /// Appends a value to the shard or returns an error if it has already been fully used.
    pub fn try_push(&mut self, value: T) -> Result<(), InsufficientCapacity> {
        if self.initialised_up_to == self.end_offset {
            return Err(InsufficientCapacity);
        }
        // Safety: The memory we're writing to was allocated by the Vec that we're writing. It's
        // currently uninitialised (not that that matters for safety). It doesn't alias, since all
        // shards are created non-overlapping.
        unsafe { self.storage.add(self.initialised_up_to).write(value) };
        self.initialised_up_to += 1;
        Ok(())
    }

    /// Returns the offset in the output vector at which the next push will write.
    pub fn output_offset(&self) -> usize {
        self.initialised_up_to
    }
}

/// Insufficient capacity for operation.
#[derive(Debug, PartialEq, Eq)]
pub struct InsufficientCapacity;
impl Error for InsufficientCapacity {}
impl Display for InsufficientCapacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Insufficient capacity")
    }
}

/// An error that can occur when returning a shard to a writer.
#[derive(Debug, PartialEq, Eq)]
pub enum InitError {
    /// One or more elements weren't initialised.
    UninitElements,

    /// A shard was returned to a writer other than the one that created it.
    WrongVec,

    /// Shards were returned out-of-order or a shard was missing.
    OutOfOrder,
}
impl Error for InitError {}
impl Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::UninitElements => write!(f, "Elements not initialised"),
            InitError::WrongVec => write!(f, "Shard returned to wrong vec"),
            InitError::OutOfOrder => write!(f, "Shards returned out-of-order"),
        }
    }
}
