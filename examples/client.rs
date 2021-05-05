//  Copyright (c) 2021 Christopher Taylor
//
//  SPDX-License-Identifier: BSL-1.0
//  Distributed under the Boost Software License, Version 1.0. (See accompanying
//  file LICENSE_1_0.txt or copy at http://www.boost.org/LICENSE_1_0.txt)
//
use zmq_collectives_rs::zmq_collectives::{Backend, Collectives, TcpBackend};

fn main() {
    let be = TcpBackend::new();
    be.initialize();
    let mut val : i32 = 0;
    be.broadcast(&mut val);
    be.finalize();
}
