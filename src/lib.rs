//  Cloneright (c) 2021 Christopher Taylor
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
        fn broadcast< Data : serde::ser::Serialize + serde::de::DeserializeOwned + Copy >(&self, data : &mut Data);
        fn reduce< DataItem, Data, F >(&self, init : DataItem, f : F, data : Data ) -> Result<DataItem, std::io::Error>
            where DataItem : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Copy,
                  Data : std::iter::IntoIterator<Item = DataItem> + serde::ser::Serialize + serde::de::DeserializeOwned + Clone,
                  F : Fn(DataItem, Data::Item) -> DataItem + Clone;
        fn barrier(&self);
        /*
        fn gather< Data : serde::ser::Serialize + serde::de::DeserializeOwned + Clone + Iterator >(&self, data : Data);
        fn scatter< Data : serde::ser::Serialize + serde::de::DeserializeOwned >(&self, data : Data);
        */
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
            let req_ = ctx_.socket(zmq::ROUTER).unwrap(); //: Vec<zmq::Socket> = (0..addresses_vec.len()).into_iter().map(| _ | ctx_.socket(zmq::REQ).unwrap()).collect();

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

                            // clears the server's ZMQ_ROUTER_ID from ZMQ_PROBE
                            //
                            self.req.recv_bytes(0).unwrap(); 
                            self.req.recv_bytes(0).unwrap();
                        }
                    }
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
        fn broadcast< Data : serde::ser::Serialize + serde::de::DeserializeOwned + Copy >(&self, data : &mut Data) {
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
            let mut v : i32 = 1;
            self.broadcast(&mut v);
            let v : Vec<i32> = vec![1,1];
            let rv = self.reduce(0, |x,y| x + y, v);
            if !rv.is_ok() {
                panic!("TcpBackend::barrier error on rank!");
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
