# wrapped-mpi
Simple demo for calling MPI directly from Rust (without any potentially incomplete wrapper)

# Building and Running
```sh
cargo build
mpirun -n <num_pes> target/debug/wrapped-mpi
```
