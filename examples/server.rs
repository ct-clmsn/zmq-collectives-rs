//  Copyright (c) 2021 Christopher Taylor
//
//  SPDX-License-Identifier: BSL-1.0
//  Distributed under the Boost Software License, Version 1.0. (See accompanying
//  file LICENSE_1_0.txt or copy at http://www.boost.org/LICENSE_1_0.txt)
//
use std::vec;
use zmq_collectives_rs::zmq_collectives::{Params, Backend, Collectives, TcpBackend};

/*
Run command:

ZMQ_COLLECTIVES_ADDRESSES=127.0.0.1:5555,127.0.0.1:5556 ZMQ_COLLECTIVES_NRANKS=2 ZMQ_COLLECTIVES_RANK=0 cargo run --example server
ZMQ_COLLECTIVES_ADDRESSES=127.0.0.1:5555,127.0.0.1:5556 ZMQ_COLLECTIVES_NRANKS=2 ZMQ_COLLECTIVES_RANK=1 cargo run --example server
*/
fn main() {
    
    let p = Params::new();
    let be = TcpBackend::new(&p);
    be.initialize(&p);

    {
        println!("broadcast");
        let mut val : i32 = 0;

        if be.rank() == 0 {
            val = 1;
        }

        be.broadcast(&mut val);
        println!("{}", val);

        be.broadcast(&mut val);
        println!("{}", val);
    }

    be.barrier();

    {
        println!("broadcast");
        let mut val : i32 = 0;

        if be.rank() == 0 {
            val = 1;
        }

        be.broadcast(&mut val);
        println!("{}", val);
    }

    be.barrier();

    {
        println!("reduce");
        let values : Vec<i32> = vec![1,1,1,1];
        let reduc_vec = be.reduce(0, |x, y| x + y, values);
        
        if be.rank() == 0 {
            println!("{}", reduc_vec.unwrap());
        }
    }

    be.barrier();

    {
        println!("reduce");
        let values : Vec<i32> = vec![1,1,1,1];
        let reduc_vec = be.reduce(0, |x, y| x + y, values);
        
        if be.rank() == 0 {
            println!("{}", reduc_vec.unwrap());
        }
    }

    be.barrier();

    {
        println!("scatter");
        let ivalues : Vec<i32> = vec![1,1,1,1];
        let mut ovalues : Vec<i32> = vec![0,0];
        be.scatter(ivalues.iter(), ivalues.len(), &mut ovalues.iter_mut());
        
        for iv in ovalues {
            println!("{}", iv);
        }
    }

    be.barrier();

    {
        println!("gather");
        let ivalues : Vec<i32> = vec![1,1];
        let mut ovalues : Vec<i32> = vec![0,0,0,0];
        be.gather(ivalues.iter(), ivalues.len(), &mut ovalues.iter_mut());
        
        for iv in ovalues {
            println!("{}", iv);
        }
    }

    be.finalize();
}
