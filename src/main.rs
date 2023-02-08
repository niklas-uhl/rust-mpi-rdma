mod rusty_kamping {
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
        pub fn rank(&self) -> i32 {
            let mut rank: i32 = 0;
            unsafe {
                MPI_Comm_rank(self.comm, &mut rank as *mut i32);
            }
            rank
        }
        pub fn size(&self) -> i32 {
            let mut size: i32 = 0;
            unsafe {
                MPI_Comm_size(self.comm, &mut size as *mut i32);
            }
            size
        }
    }

}

fn main() {
    rusty_kamping::init();
    let comm = rusty_kamping::Communicator::new();
    println!("Hello world from rank {} out of {}!", comm.rank(), comm.size());
    rusty_kamping::finalize();
}
