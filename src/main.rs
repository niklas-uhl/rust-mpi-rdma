/*
Copyright (c) 2023 Tim Niklas Uhl

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */
mod rusty_kamping {
    use std::{mem::size_of, ffi::{c_int, c_void}, ptr::null_mut, ops::Index, marker::PhantomData};

    use mpi_sys::*;

    pub fn init() {
        unsafe {
            MPI_Init(std::ptr::null_mut(), std::ptr::null_mut());
        }
    }
    pub fn finalize() {
        unsafe {
            MPI_Finalize();
        }
    }

    pub struct Communicator {
        comm: MPI_Comm,
    }
    impl Communicator {
        pub fn new() -> Communicator {
            unsafe {
                Communicator { comm: RSMPI_COMM_WORLD }
            }
        }
        pub fn rank(&self) -> usize {
            let mut rank = 0;
            unsafe {
                MPI_Comm_rank(self.comm, &mut rank as *mut c_int);
            }
            rank as usize
        }
        pub fn size(&self) -> usize {
            let mut size = 0;
            unsafe {
                MPI_Comm_size(self.comm, &mut size as *mut c_int);
            }
            size as usize
        }
        pub fn barrier(&self) {
            unsafe {
                MPI_Barrier(self.comm);
            }
        }
    }
    

    pub struct Win<'a, T: Sized>{
        win: MPI_Win,
        data: &'a mut [T],
        comm: &'a Communicator,
    }

    pub struct LocalWinLock<'a, T> {
        win: &'a MPI_Win,
        data: &'a [T],
        rank: usize,
    }

    impl<'a, T> LocalWinLock<'a, T> {
        fn new(win: &'a MPI_Win, data: &'a [T], rank: usize) -> LocalWinLock<'a, T> {
            unsafe {
                MPI_Win_lock(MPI_LOCK_SHARED as c_int, rank as c_int, 0, *win);
            }
            LocalWinLock { win, data, rank}
        }
    }

    impl<'a, T> Drop for LocalWinLock<'a, T> {
        fn drop(&mut self) {
            unsafe {
                MPI_Win_unlock(self.rank as c_int, *self.win);
            }
        }
    }
    impl<'a, T> Index<usize> for LocalWinLock<'a, T> {
        type Output = T;

        fn index(&self, index: usize) -> &Self::Output {
            &self.data[index]
        }
    }

    pub struct RemoteWinLock<'a, T> {
        win: &'a MPI_Win,
        rank: usize,
        phantom: PhantomData<T>,
    }
    
    impl<'a, T> RemoteWinLock<'a, T> {
        pub fn new(win: &MPI_Win, rank: usize) -> RemoteWinLock<T> {
            unsafe {
                MPI_Win_lock(MPI_LOCK_SHARED as c_int, rank as c_int, 0, *win);
            }
            RemoteWinLock { win, rank, phantom: PhantomData }
        }

        pub fn put<'b: 'a>(&mut self, value: &'b T, index: usize) {
            let origin_addr = value as *const T as *const c_void;
            unsafe {
                //TODO: Remove the hardcoded type here
                MPI_Put(origin_addr, 1, RSMPI_DOUBLE, self.rank as c_int, index as MPI_Aint, 1, RSMPI_DOUBLE, *self.win);
            }
        }
        
    }

    impl<'a, T> Drop for RemoteWinLock<'a, T> {
        fn drop(&mut self) {
            unsafe {
                MPI_Win_unlock(self.rank as c_int, *self.win);
            }
        }
    }

    impl<'a, T: Sized> Win<'a, T> {
        pub fn new(size: usize, comm: &'a Communicator) -> Win<T> {
            let mut base_ptr: *mut T = null_mut();
            let mut win = unsafe { RSMPI_WIN_NULL };
            let base_ptr_addr = &mut base_ptr as *mut *mut _;
            unsafe {
                MPI_Win_allocate(
                    (size_of::<T>()  * size ) as MPI_Aint,
                    size_of::<T>() as c_int,
                    RSMPI_INFO_NULL,
                    comm.comm,
                    base_ptr_addr as *mut c_void,
                    &mut win as *mut MPI_Win
                );
            }
            let data = unsafe {std::slice::from_raw_parts_mut(base_ptr, size)};
            Win {
                win,
                data,
                comm,
            }
        }

        pub fn lock_local(&mut self) -> LocalWinLock<T> {
            LocalWinLock::new(&self.win, self.data, self.comm.rank())
        }
        // pub fn lock_remote(&'a self, target: usize) -> LocalWinLock<'a, T> {
        //    LocalWinLock::new(&self.win, self.data, self.comm.rank())
        //}
        pub fn lock_remote_mut(&self, target: usize) -> RemoteWinLock<T> {
            RemoteWinLock::new(&self.win, target)
        }
    }

    impl<'a, T: Sized> Drop for Win<'a, T> {
        fn drop(&mut self) {
            unsafe { MPI_Win_free(&mut self.win as *mut MPI_Win); }
        }
    }
}

fn main() {
    rusty_kamping::init();
    let comm = rusty_kamping::Communicator::new();
    println!("Hello world from rank {} out of {}!", comm.rank(), comm.size());
    {
        let mut win = rusty_kamping::Win::new(1, &comm);
        if comm.rank() == 0 {
            for target_rank in 0..comm.size() {
                let mut remote_win = win.lock_remote_mut(target_rank);
                remote_win.put(&std::f64::consts::PI, 0);
            }
        }
        comm.barrier();
        {
            let local_win = win.lock_local();
            println!("{}", local_win[0]);
        }
    }
    rusty_kamping::finalize();
}
