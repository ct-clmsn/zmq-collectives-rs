=== zmq-collectives-rs ===

Library implemented SPMD collective communication algorithms
using Robert van de Deijn's Binomial Tree scheduling in Rust
using 0MQ (zeromq). Provides log2(N) algorithmic performance
for each collective operation over N compute hosts.

These algorithms are used in HPC (Supercomputing) libraries
and runtime systems like MPI and OpenSHMEM.

== Algorithms Implemented ==

* Broadcast
* Reduction
* Scatter
* Gather

== Notes ==

Configurable with environment variables

ZMQ_COLLECTIVES_NRANKS
ZMQ_COLLECTIVES_RANK
ZMQ_COLLECTIVES_UUID
ZMQ_COLLECTIVES_ADDRESSES

Configuring 0MQ inproc, ipc, tcp backends

inproc is provided for threads
ipc is provided for single-host processes
tcp is provided for multi-host processes

			     backend
ZMQ_COLLECTIVES_NRANKS       inproc, ipc, tcp
ZMQ_COLLECTIVES_RANK         inproc, ipc, tcp
ZMQ_COLLECTIVES_UUID         ipc, tcp
ZMQ_COLLECTIVES_ADDRESSES    tcp

ZMQ_COLLECTIVES_ADDRESSES should contain a ',' delimited
list of ip addresses and ports. The list length should be
equal to the integer value of ZMQ_COLLECTIVES_NRANKS. An
example for a 2 rank application is below:

ZMQ_COLLECTIVES_NRANKS=2 ZMQ_COLLECTIVES_RANK=0
ZMQ_COLLECTIVES_ADDRESSES=127.0.0.1:5555,127.0.0.1:5556

ZMQ_COLLECTIVES_NRANKS=2 ZMQ_COLLECTIVES_RANK=1
ZMQ_COLLECTIVES_ADDRESSES=127.0.0.1:5555,127.0.0.1:5556

In this example, Rank 0 maps to 127.0.0.1:5555 and Rank 1
maps to 127.0.0.1:5556.

0MQ uses file descriptors to handle communication and
asynchrony control. There is a GNU/Linux kernel
configurable ~2063 limit on the number of file
descriptors a user process is authorized to open during
it's lifetime. This places a hard limitation on the
ability of 0MQ to handle thread counts (inproc backend)
over 1024. ipc and tcp backends can scale further
because both utilize 2 file descriptors.

== License ==

Boost Version 1.0

== Date ==

03MAY2021

== Author ==

Christopher Taylor

== Dependencies ==

* rust
* uuid
* serde
* bincode
* zmq
