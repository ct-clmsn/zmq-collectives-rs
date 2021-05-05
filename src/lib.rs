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

    pub trait Backend {
        fn initialize(&self);
        fn finalize(&self);
        fn send< D : serde::ser::Serialize >(&self, rnk : usize, data : D);
        fn recv< D : serde::de::DeserializeOwned >(&self, rnk : usize) -> Result<D, std::io::Error> ;
    }

    pub trait Collectives {
        fn broadcast< Data : serde::ser::Serialize + serde::de::DeserializeOwned >(&self, data : &mut Data);
        /*fn reduction< Data : serde::ser::Serialize + serde::de::DeserializeOwned >(&self, data : Data);
        fn scatter< Data : serde::ser::Serialize + serde::de::DeserializeOwned >(&self, data : Data);
        fn gather< Data : serde::ser::Serialize + serde::de::DeserializeOwned >(&self, data : Data);
        fn barrier< Data : serde::ser::Serialize + serde::de::DeserializeOwned >(&self, data : Data);*/
    }
 
    /*
    pub struct InprocBackend {
        nranks : usize,
        id : uuid::Uuid,
        ctx : zmq::Context,
        req : Vec<zmq::Socket>,
        rep : Vec<zmq::Socket>,
    }

    impl InprocBackend {
        pub fn new() -> Self {
            let nranks_ : usize = match std::env::var("ZMQ_COLLECTIVES_NRANKS") {
                Ok(nr) => nr.parse::<usize>().unwrap(),
                Err(_) => 0,
            };

            let ctx_ = zmq::Context::new();
            
            let req_ = (0..nranks_).map(| _ | ctx_.socket(zmq::ROUTER).unwrap() ).collect();
            let rep_ = (0..nranks_).map(| _ | ctx_.socket(zmq::REP).unwrap() ).collect();
            Self{nranks : nranks_, id : uuid::Uuid::new_v4(), ctx : ctx_, req : req_, rep : rep_ }
        }
    }

    pub struct IpcBackend {
        nranks : usize,
        rank : usize,
        id : String,
        ctx : zmq::Context,
        req : zmq::Socket,
        rep : zmq::Socket,
    }

    impl IpcBackend {
        pub fn new() -> Self {
            let nranks_ : usize = match std::env::var("ZMQ_COLLECTIVES_NRANKS") {
                Ok(nr) => nr.parse::<usize>().unwrap(),
                Err(_) => 0,
            };

            let rank_ : usize = match std::env::var("ZMQ_COLLECTIVES_RANK") {
                Ok(nr) => nr.parse::<usize>().unwrap(),
                Err(_) => 0,
            };

            let uid_ : String = match std::env::var("ZMQ_COLLECTIVES_UUID") {
                Ok(nr) => nr.parse::<String>().unwrap(),
                Err(_) => String::new(),
            };
      
            let ctx_ = zmq::Context::new();
            let req_ = ctx_.socket(zmq::ROUTER).unwrap();
            req_.set_identity(rank_.to_string().as_bytes()).unwrap();
            let rep_ = ctx_.socket(zmq::REP).unwrap();
            Self{nranks : nranks_, rank : rank_, id : uid_, ctx : ctx_, req : req_, rep : rep_ }
        }
    }

    impl Backend for IpcBackend {
        fn initialize(&self) {

            let bind_res = self.rep.bind( &("ipc://".to_owned() + &self.id + "_" + &self.rank.to_string()) );
            if bind_res.is_err() {
                panic!("IpcBackend::initialize bind failed");
            }

            for i in 0..self.nranks {
                if i != self.rank {
                    let mut sleep_counter : usize = 0;
                    let mut is_break = true;
                    while is_break {
                        let res = self.rep.connect( &("ipc://".to_owned() + &self.id + "_" + &i.to_string()) ) ;
                        match res {
                            Ok(_) => { is_break = false; }
                            Err(_) => {
                                let sleep_ms = std::time::Duration::from_millis( fibonacci(sleep_counter) as u64 );
                                std::thread::sleep(sleep_ms);
                                sleep_counter += 1; 
                            }
                        }
                    }
                }
            }

            for i in 0..self.nranks {
                if i != self.rank {
                    for j in 0..self.nranks {
                        if i != j {
                            self.req.send(0.to_string().as_bytes(), zmq::SNDMORE).unwrap();
                            self.req.send(0.to_string().as_bytes(), zmq::SNDMORE).unwrap();
                            self.req.send(0.to_string().as_bytes(), 0).unwrap();
                        }
                    }
                }
                else {
                    for j in 0..self.nranks {
                        if i != j {
                            self.rep.recv_bytes(0).unwrap();
                            self.rep.recv_bytes(0).unwrap();
                            self.rep.recv_bytes(0).unwrap();
                        }
                    }
                }
            }
        }

        fn finalize(&self) {
        }

        fn send< D : serde::ser::Serialize >(&self, rnk : usize, data : D) {
            let data_encode : Vec<u8> = bincode::serialize(&data).unwrap();

            let xmt_a = self.req.send(rnk.to_string().as_bytes(), zmq::SNDMORE);
            if xmt_a.is_err() { panic!("IpcBackend::send 1st send error!"); }

            let xmt_b = self.req.send("", zmq::SNDMORE);
            if xmt_b.is_err() { panic!("IpcBackend::send 2nd send error!"); }

            let xmt_c = self.req.send(data_encode, 0);
            if xmt_c.is_err() { panic!("IpcBackend::send 3rd send error!"); }
        }

        fn recv< D : serde::de::DeserializeOwned >(&self, _rnk : usize) -> Result<D, std::io::Error> {

            let rcv_a = self.rep.recv_bytes(0);
            if rcv_a.is_err() { panic!("IpcBackend::recv 1st error!"); }

            let rcv_b = self.rep.recv_bytes(0);
            if rcv_b.is_err() { panic!("IpcBackend::recv 2nd error!"); }

            let recv_bytes_result = self.rep.recv_bytes(0);
            if recv_bytes_result.is_err() { panic!("IpcBackend::recv 3rd error!"); }

            let d_bytes : Vec<u8> = recv_bytes_result.unwrap();
            let data : Result<D, bincode::Error> = bincode::deserialize(&d_bytes[..]);

            return match data {
                Ok (b) => Ok(b), 
                Err(_) => Err( std::io::Error::new(std::io::ErrorKind::Other, "Ipc::recv bincode::deserialize Error") )
            };
        }
    }

    impl Collectives for IpcBackend {
        fn broadcast< Data : serde::ser::Serialize + serde::de::DeserializeOwned >(&self, data : Data) {
            let depth = (self.nranks as f64).log2().ceil() as usize;
            for d in 0..depth {
            }
        }
    }
    */

    pub struct TcpBackend {
        nranks : usize,
        rank : usize,
        ctx : zmq::Context,
        rep : zmq::Socket,
        req : Vec<zmq::Socket>,
        addresses : Vec<String>,
    }

    impl TcpBackend {
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

            let ctx_ = zmq::Context::new();

            let rep_ = ctx_.socket(zmq::REP).unwrap();
            let req_ : Vec<zmq::Socket> = (0..addresses_vec.len()).into_iter().map(| _ | ctx_.socket(zmq::REQ).unwrap()).collect();

            Self{nranks : nranks_, rank : rank_, ctx : ctx_, rep : rep_, req : req_, addresses : addresses_vec }
        }
    }

    impl Backend for TcpBackend {
        fn initialize(&self) {

            let bind_address_str = self.addresses.get(self.rank);
            let bind_res = self.rep.bind( &("tcp://".to_string() + bind_address_str.unwrap() ) );

            if bind_res.is_err() {
                panic!("TcpBackend::initialize bind failed");
            }

            for orank in 0..self.addresses.len() {
                if orank != self.rank {
                    let mut sleep_counter : usize = 1;
                    let connect_address_str = self.addresses.get(orank).unwrap();
                    while sleep_counter > 0 {
                        let res = self.req.get(orank).unwrap().connect( &("tcp://".to_string() + connect_address_str) );
                        match res {
                            Ok(_) => { sleep_counter = 0; }
                            Err(_) => {
                                let sleep_ms = std::time::Duration::from_millis( fibonacci(sleep_counter) as u64 );
                                std::thread::sleep(sleep_ms);
                                sleep_counter += 1; 
                            }
                        }
                    }
                }
            }
        }

        fn finalize(&self) {
        }

        fn send< D : serde::ser::Serialize >(&self, rnk : usize, data : D) {
            if rnk > self.nranks { panic!("TcpBackend::send error on rank!"); }

            let data_encode : Vec<u8> = bincode::serialize(&data).unwrap();
            let mut sleep_counter : usize = 1;
            while sleep_counter > 0 {
                let xmt_c = self.req.get(rnk).unwrap().send(&data_encode, 0);
                if xmt_c.is_err() { 
                    let sleep_ms = std::time::Duration::from_millis( fibonacci(sleep_counter) as u64 );
                    std::thread::sleep(sleep_ms);
                    sleep_counter += 1;
                }
                else { sleep_counter = 0; }
            }
        }

        fn recv< D : serde::de::DeserializeOwned >(&self, rnk : usize) -> Result<D, std::io::Error> {
            if rnk > self.nranks { panic!("TcpBackend::send error on rank!"); }

            let mut data : Option< Result<D, bincode::Error> > = Option::None;
            let mut sleep_counter : usize = 1;

            while sleep_counter > 0 {
                let recv_bytes_result = self.rep.recv_bytes(0);
                if recv_bytes_result.is_err() { 
                    let sleep_ms = std::time::Duration::from_millis( fibonacci(sleep_counter) as u64 );
                    std::thread::sleep(sleep_ms);
                    sleep_counter += 1;
                }
                else {
                    let d_bytes : Vec<u8> = recv_bytes_result.unwrap();
                    data.replace( bincode::deserialize(&d_bytes[..]) );
                    sleep_counter = 0;
                }
            }

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
    }

    impl Collectives for TcpBackend {
        fn broadcast< Data : serde::ser::Serialize + serde::de::DeserializeOwned >(&self, data : &mut Data) {
            let depth : usize = (self.nranks as f64).log2().ceil() as usize;
            for d in 0..depth {
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
