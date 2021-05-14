=== zmq-collectives-rs ===

Library implements SPMD (single program many data) collective
communication algorithms (Robert van de Deijn's Binomial Tree)
in Rust using 0MQ. Provides log2(N) algorithmic performance
for each collective operation over N compute hosts.

These algorithms are used in HPC (high peformance computing/
supercomputing) and runtime systems like MPI and OpenSHMEM.

== Algorithms Implemented ==

* Broadcast
* Reduction
* Scatter
* Gather
* Barrier

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
the process instance this program represents.

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

0MQ uses sockets/file descriptors (same thing) to
handle communication and asynchrony control. There
is a GNU/Linux kernel configurable ~2063 default
limit on the number of file descriptors a user
process is authorized to open during execution. The
TcpBackend uses 2 file descriptors/sockets. In 0MQ
terms these sockets are ZMQ_ROUTER.

tcp is a "chatty" protocol; tcp requires round trips
between clients and servers during the data transmission
exchange to ensure data is communicated correctly. The
use of this protocol makes it less than ideal for jobs
requiring high performance. However, tcp is provided in
0MQ and is universally accessible (tcp is a commodity
protocol) and makes for a reasonable place to plant a
flag for providing an implementation.

This library requires libzmq. LD_LIBRARY_FLAGS and
PKG_CONFIG_PATH needs to point to the directories that
the libzmq library has been is installed. As an example,
let's say a user has installed libzmq into a directory
with the environment variable named:

$LIBZMQ_INSTALL_PREFIX_PATH 

libzmq.a or libzmq.so would be installed in the directory:
    $LIBZMQ_INSTALL_PREFIX_PATH/lib

libzmq.pc can be found in the directory:
    $LIBZMQ_INSTALL_PREFIX_PATH/lib/pkgconfig

== License ==

Boost Version 1.0

== Date ==

03MAY2021-11MAY2021

== Author ==

Christopher Taylor

== Dependencies ==

* pkg-config
* libzmq
* uuid
* serde
* bincode
* zmq
