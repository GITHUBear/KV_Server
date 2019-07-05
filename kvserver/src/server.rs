extern crate futures;
extern crate grpcio;
extern crate kvprotos;

use std::io::Read;
use std::sync::Arc;
use std::{io, thread};

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};

use kvprotos::src::kvserver::{ GetRequest, GetResponse, PutRequest, PutResponse, DeleteRequest, DeleteResponse};
use kvprotos::src::kvserver_grpc::{self, Kv};

#[derive(Clone)]
struct KvService;

impl Kv for KvService {
	fn get(&self, ctx: RpcContext, req: GetRequest, sink: UnarySink<GetResponse>) {
        let mut response = GetResponse::new();
        println!("Received GetRequest {{ {:?} }}", req);
        response.set_value(b"value-cd".to_vec());

        let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
}

