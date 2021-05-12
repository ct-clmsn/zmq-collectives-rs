//  Copyright (c) 2021 Christopher Taylor
//
//  SPDX-License-Identifier: BSL-1.0
//  Distributed under the Boost Software License, Version 1.0. (See accompanying
//  file LICENSE_1_0.txt or copy at http://www.boost.org/LICENSE_1_0.txt)
//
fn main() {
    let ctx_ = zmq::Context::new();
    let server = ctx_.socket(zmq::ROUTER).unwrap();
    server.set_identity("x".as_bytes());
    server.set_probe_router(true);
    server.bind("tcp://127.0.0.1:5555");

    let client = ctx_.socket(zmq::ROUTER).unwrap();
    client.set_identity("x".as_bytes());
    client.set_probe_router(true);
    client.connect("tcp://127.0.0.1:5555"); 

    server.recv_bytes(0);
    let sr = server.recv_bytes(0).unwrap();
    println!("srcv\t{}", String::from_utf8(sr).unwrap());

    client.recv_bytes(0);
    let car = client.recv_bytes(0).unwrap();
    println!("crcv\t{}", String::from_utf8(car).unwrap());

    server.send("x", zmq::SNDMORE);
    server.send("Hello", 0);

    println!("{}", client.recv_bytes(0).unwrap().len());
    let cbr = client.recv_bytes(0).unwrap();
    println!("{}", String::from_utf8(cbr).unwrap());

    server.send("x", zmq::SNDMORE);
    server.send("Hello", 0);

    println!("{}", client.recv_bytes(0).unwrap().len());
    let cbr = client.recv_bytes(0).unwrap();
    println!("{}", String::from_utf8(cbr).unwrap());
}
