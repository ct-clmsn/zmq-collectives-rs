=== zmq-collectives-rs ===

Library implemented SPMD collective communication algorithms
using Robert van de Deijn's Binomial Tree scheduling in Rust
using 0MQ (zeromq). Provides log2(N) algorithmic performance
for each collective operation over N compute hosts.

These algorithms are used in HPC (supercomputing) libraries
and runtime systems like MPI and OpenSHMEM.

== Algorithms Implemented ==

* Broadcast
* Reduction
* Scatter
* Gather

== Configuring Distributed Programs ==

Environment variables are used to configure distributed
runs of SPMD applications using this library. Each of
these environment variables needs to be supplied to
correctly run programs.

* ZMQ_COLLECTIVES_NRANKS
* ZMQ_COLLECTIVES_RANK
* ZMQ_COLLECTIVES_ADDRESSES

ZMQ_COLLECTIVES_NRANKS unsigned integer value indicating
how many processes are running.

ZMQ_COLLECTIVES_RANK unsigned integer value indicating
the process this program represents.

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

== Notes ==

0MQ uses file descriptors to handle communication and
asynchrony control. There is a GNU/Linux kernel
configurable ~2063 limit on the number of file
descriptors a user process is authorized to open during
it's lifetime. This places a hard limitation on the
ability of 0MQ to handle thread counts (inproc backend)
over 1024. ipc and tcp backends can scale further
because both utilize 2 file descriptors.

The tcp backend provided by this library, ZMQ_ROUTER
sockets are used and *should* scale to a larger set of
machines than 1024 or 2048.

== License ==

Boost Version 1.0

== Date ==

03MAY2021-11MAY2021

== Author ==

Christopher Taylor

== Dependencies ==

* rust
* uuid
* serde
* bincode
* zmq
