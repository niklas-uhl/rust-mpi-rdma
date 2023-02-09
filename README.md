# rust-mpi-rdma
Simple demo for calling MPI directly from Rust (without any potentially incomplete wrapper).

Also, includes a proof-of-concept of safe MPI-RDMA communication. 

# Building and Running
```sh
cargo build
mpirun -n <num_pes> target/debug/wrapped-mpi
```
