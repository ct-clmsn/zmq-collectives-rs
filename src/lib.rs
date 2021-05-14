//  Copyright (c) 2021 Christopher Taylor
//
//  SPDX-License-Identifier: BSL-1.0
//  Distributed under the Boost Software License, Version 1.0. (See accompanying
//  file LICENSE_1_0.txt or copy at http://www.boost.org/LICENSE_1_0.txt)
//
use std::env;
use std::result;
use std::time;
use std::thread;
use uuid;
use std::io::{Error, ErrorKind};
use serde::{Serialize, Deserialize};
use bincode::{deserialize, serialize};
use zmq;

pub mod zmq_collectives {

    fn fibonacci(n: usize) -> usize {
        match n {
            0 => 1,
            1 => 1,
            _ => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    pub struct Params {
        nranks : usize,
        rank : usize,
        addresses : Vec<String>,
    }   

    impl Params {
        pub fn new() -> Self {
            let nranks_ : usize = match std::env::var("ZMQ_COLLECTIVES_NRANKS") {
                Ok(nr) => nr.parse::<usize>().unwrap(),
                Err(_) => 0,
            };

            let rank_ : usize = match std::env::var("ZMQ_COLLECTIVES_RANK") {
                Ok(nr) => nr.parse::<usize>().unwrap(),
                Err(_) => 0,
            };

            let addresses_ : String = match std::env::var("ZMQ_COLLECTIVES_ADDRESSES") {
                Ok(nr) => nr.parse::<String>().unwrap(),
                Err(_) => String::new(),
            };

            let addresses_vec : Vec<String> = addresses_.split(',').into_iter().map(| a | String::from(a)).collect();
            if addresses_vec.len() < 1 {
                panic!("TcpBackend::new ZMQ_COLLECTIVES_ADDRESSES has 0 addresses");
            }

            Self{nranks : nranks_, rank : rank_, addresses : addresses_vec }
        }
    }

    pub trait Backend {
        fn initialize(&self, p : & Params);
        fn finalize(&self);
        fn rank(&self) -> usize;
        fn n_ranks(&self) -> usize;
        fn send< D : serde::ser::Serialize + Clone >(&self, rnk : usize, data : D);
        fn recv< D : serde::de::DeserializeOwned + Clone >(&self, rnk : usize) -> Result<D, std::io::Error> ;
    }

    pub trait Collectives {

        fn broadcast<Data>(&self, data : &mut Data)
           where Data : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy;  

        fn reduce<DataItem, Data, F >(&self, init : DataItem, f : F, data : Data ) -> Result<DataItem, std::io::Error>
            where DataItem : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy,
                  Data : std::iter::IntoIterator<Item = DataItem> + serde::ser::Serialize + serde::de::DeserializeOwned + Clone,
                  F : Fn(DataItem, Data::Item) -> DataItem + Clone;

        fn barrier(&self);

        fn scatter< 'a, DataItem >(&self, in_beg : std::slice::Iter<'a, DataItem>, in_size : usize, out : &mut std::slice::IterMut<DataItem> )
            where DataItem : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy;

        fn gather< 'a, DataItem >(&self, in_beg : &mut std::slice::Iter<'a, DataItem>, in_size : usize, out : &mut std::slice::IterMut<DataItem> )
            where DataItem : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy;
    }
 
    pub struct TcpBackend {
        nranks : usize,
        rank : usize,
        ctx : zmq::Context,
        rep : zmq::Socket,
        req : zmq::Socket,
    }

    impl TcpBackend {
        pub fn new(param : & Params) -> Self {
            let ctx_ = zmq::Context::new();
            let rep_ = ctx_.socket(zmq::ROUTER).unwrap();
            let req_ = ctx_.socket(zmq::ROUTER).unwrap();

            Self{nranks : param.nranks, rank : param.rank, ctx : ctx_, rep : rep_, req : req_}
        }
    }

    impl Backend for TcpBackend {
        fn initialize(&self, param : & Params) {

            let bind_address_str = param.addresses.get(self.rank);

	    self.rep.set_identity(self.rank.to_string().as_bytes()).unwrap();
            self.rep.set_probe_router(true);
            let bind_res = self.rep.bind( &("tcp://".to_string() + bind_address_str.unwrap() ) );

            if bind_res.is_err() {
                panic!("TcpBackend::initialize bind failed");
            }

	    self.req.set_identity(self.rank.to_string().as_bytes()).unwrap();
            self.req.set_probe_router(true);

            if param.addresses.len() != self.nranks {
                panic!("TcpBackend::initialize Params address list lenght is not equal to TcpBackend.nranks");
            }

            /*
            for orank in 0..param.addresses.len() {
                if orank == self.rank {
                    for irank in 0..param.addresses.len() {
                        if irank != self.rank {
                            self.rep.recv_bytes(0).unwrap();
                            self.rep.recv_bytes(0).unwrap();
                        }
                    }
                }
                else {
                    for irank in 0..param.addresses.len() {
			if irank != self.rank {
                            let connect_address_str = param.addresses.get(irank).unwrap();
                            self.req.connect( &("tcp://".to_string() + connect_address_str) ).unwrap();

                            // clears the server's ZMQ_ROUTER_ID data from ZMQ_PROBE
                            //
                            self.req.recv_bytes(0).unwrap(); 
                            self.req.recv_bytes(0).unwrap();
                        }
                    }
                }
            }
            */

            for orank in 0..param.addresses.len() {
                if orank == self.rank {
                    for irank in 0..param.addresses.len() {
			if irank != self.rank {
                            let connect_address_str = param.addresses.get(irank).unwrap();
                            self.req.connect( &("tcp://".to_string() + connect_address_str) ).unwrap();

                            // clears the server's ZMQ_ROUTER_ID data from ZMQ_PROBE
                            //
                            self.req.recv_bytes(0).unwrap(); 
                            self.req.recv_bytes(0).unwrap();
                        }
                    }

                }
                else {
                    self.rep.recv_bytes(0).unwrap();
                    self.rep.recv_bytes(0).unwrap();
                }
            }
        }

        fn finalize(&self) {
        }

        fn send< D : serde::ser::Serialize + Clone >(&self, rnk : usize, data : D) {
            if rnk > self.nranks { panic!("TcpBackend::send error on rank!"); }

            let data_encode : Vec<u8> = bincode::serialize(&data).unwrap();
            self.req.send(rnk.to_string().as_bytes(), zmq::SNDMORE).unwrap();
            self.req.send(&data_encode, 0).unwrap();
        }

        fn recv< D : serde::de::DeserializeOwned + Clone >(&self, rnk : usize) -> Result<D, std::io::Error> {
            if rnk > self.nranks { panic!("TcpBackend::send error on rank!"); }

            let mut data : Option< Result<D, bincode::Error> > = Option::None;
            self.rep.recv_bytes(0).unwrap();
            let recv_bytes_result = self.rep.recv_bytes(0);
            let d_bytes : Vec<u8> = recv_bytes_result.unwrap();
            data.replace( bincode::deserialize(&d_bytes[..]) );

            return match data {
                Some(sd) => {
                    match sd {
                        Ok(d) => Ok(d),
                        Err(_) =>  Err( std::io::Error::new(std::io::ErrorKind::Other, "TcpBackend::recv bincode::deserialize Error") )
                    }
                },
                None => Err( std::io::Error::new(std::io::ErrorKind::Other, "TcpBackend::recv bincode::deserialize Error") )
            };
        }

        fn rank(&self) -> usize { return self.rank; }
        fn n_ranks(&self) -> usize { return self.nranks; }
    }

    impl Collectives for TcpBackend {

        fn broadcast<Data>(&self, data : &mut Data)
           where Data : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy {

            let depth : usize = (self.nranks as f64).log2().ceil() as usize;
            let mut k : usize = self.nranks / 2;
            let mut not_recv : bool = true; 
            for _d in 0..depth {
                let twok : usize = 2 * k;
                if (self.rank % twok) == 0 {
                    self.send(self.rank+k, &data);
                }
                else if not_recv && ((self.rank % twok) == k) {
                    let mut res : Result< Data, std::io::Error > = self.recv(self.rank-k);
                    match res.as_mut() {
                        Ok(v) => { *data = *v; },
                        Err(_) => { panic!("TcpBackend::send error on rank!"); }
                    }

                    not_recv = false;
                }

                k >>= 1;
            }
        }

        fn reduce< DataItem, Data, F >(&self, init : DataItem, f : F, data : Data) -> Result<DataItem, std::io::Error>
            where DataItem : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy,
                  Data : std::iter::IntoIterator<Item = DataItem> + serde::ser::Serialize + serde::de::DeserializeOwned + Clone,
                  F : Fn(DataItem, Data::Item) -> DataItem + Clone {

            let depth : usize = (self.nranks as f64).log2().ceil() as usize;

            let mut not_sent : bool = true; 
            let mut mask : usize = 0x1;

            let mut local = data.into_iter().fold(init, &f);

            for _d in 0..depth {
                if (mask & self.rank) == 0 {
                    if (mask | self.rank) < self.nranks && not_sent {
                        let mut res : Result< Data::Item, std::io::Error > = self.recv(self.rank);
                        match res.as_mut() {
                            Ok(v) => { local = f(local, *v); },
                            Err(_) => { panic!("TcpBackend::send error on rank!"); }
                        }
                    }
                }
                else if not_sent {
                    let parent = self.rank & (!mask);
                    self.send(parent, &local);
                    not_sent = false;
                }

                mask <<= 1;
            }

            return Ok(local);
        }

        fn barrier(&self) {
            // barrier implementation consists of a broadcast
            // followed by a reduction
            //
            let mut v : i32 = 1;
            self.broadcast(&mut v);

            let v : Vec<i32> = vec![1,1];
            let rv = self.reduce(0, |x,y| x + y, v);
            if !rv.is_ok() {
                panic!("TcpBackend::barrier error on rank!");
            }
        }

        fn scatter< 'a, DataItem >(&self, in_beg : std::slice::Iter<'a, DataItem>, in_size : usize, out : &mut std::slice::IterMut<DataItem> )
            where DataItem : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy {

            let depth : usize = (self.nranks as f64).log2().ceil() as usize;
            let block_size : usize = in_size / self.nranks;
            let k : usize = self.nranks / 2;
            let mut not_received : bool = true; 

            for _d in 0..depth {
                let twok = 2 * k;
                if (self.rank % twok) == 0 {
                    let beg : usize = ((self.rank + k) % self.nranks) * block_size;
                    let end : usize = (self.nranks - (self.rank % self.nranks)) * block_size;

                    let mut itr = (&in_beg).clone().skip(beg);

                    let mut subdata : Vec<DataItem> = Vec::with_capacity(end-beg);
                    (beg..end).for_each(| _i | subdata.push(*itr.next().unwrap()) );

                    self.send(self.rank+k, subdata);
                }
                else if not_received && ((self.rank % twok) == k) {
                    let res : Result< Vec<DataItem>, std::io::Error > = self.recv(self.rank-k);

                    if res.is_err() {
                        panic!("TcpBackend::recv error on rank!");
                    }

                    let resv = &res.unwrap();
                    let mut resitr = resv.iter();

                    out.for_each(| oval | *oval = *resitr.next().unwrap() );

                    not_received = false;
                }
            }

            if self.rank() < 1 {
                let mut citr = in_beg.clone();
                out.take(block_size).for_each(| oval | *oval = *citr.next().unwrap() );
            }
        }

        fn gather< 'a, DataItem >(&self, in_beg : &mut std::slice::Iter<'a, DataItem>, in_size : usize, out : &mut std::slice::IterMut<DataItem> )
            where DataItem : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy {

            let depth : usize = (self.nranks as f64).log2().ceil() as usize;
            let mut mask : usize = 0x1;

            let mut buf : Vec<DataItem> = Vec::new();

            if self.rank() > 0 {
                (0..in_size).for_each( | _i | buf.push( *in_beg.next().unwrap() ) );
            }

            for _d in 0..depth {
                if (mask & self.rank()) == 0 {
                   let child : usize = self.rank() | mask;
                   if child < self.n_ranks() {
                       let recvbuf : Vec<DataItem> = self.recv(child).unwrap();
                       recvbuf.iter().for_each(| rb | buf.push(*rb) );
                   }
                }
                else {
                    let parent : usize = self.rank() & (!mask);
                    self.send(parent, &buf);
                }

                mask <<= 1;
            }

            if self.rank() < 1 {
                in_beg.for_each(| iv | buf.insert(0, *iv) );
                let mut bufitr = buf.iter();
                out.for_each(| ov | *ov = *bufitr.next().unwrap() );
            }
        }
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        //assert_eq!(2 + 2, 4);
        use crate::zmq_collectives::{Backend, IpcBackend};
        let be = IpcBackend::new();
        //be.initialize();
    }
}
