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
configurable ~2063 default limit on the number of file
descriptors a user process is authorized to open during
it's lifetime.

0MQ's dependency on limited operating system resources
means there is an upperbound on parallelism for thread
counts using the inproc backend over 1024. ipc and tcp
backends can scale further because both utilize 2 file
descriptors. The limitation gives users the opportunity
to determine how many processes and machines are necessary
to reach the scale or level of parallelism they require
in a application's execution.

0MQ's inproc backend uses shared memory to communicate.
For performance sensitive applications, the overhead of
serializing data, writing to shared memory, reading the
data from shared memory, deserializing the data read from
shared memory is unnecessary overhead when a pointer
dereference could be used to achieve the same outcome.

tcp is a "chatty" protocol; tcp requires round trips
between clients and servers during the data transmission
exchange to ensure data is communicated correctly. The
use of this protocol makes it less than ideal for jobs
requiring high performance. However, tcp is provided in
0MQ and is universally accessible (tcp is a commodity
protocol) and makes for a reasonable place to plant a
flag for providing an implementation.

The tcp backend provided by this library uses ZMQ_ROUTER
sockets to handle requests and responses and *should*
scale to sets of process and machine combinations greater
than 1024 or 2048.

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
