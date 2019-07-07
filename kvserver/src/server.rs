extern crate protobuf;
extern crate futures;
extern crate grpcio;

pub mod kvprotos;

use std::io::Read;
use std::sync::{Arc, RwLock};
use std::{io, thread};
use std::collections::{BTreeMap, HashMap};

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};
use kvprotos::kvserver::{ResponseStatus, GetRequest, GetResponse, PutRequest, PutResponse,  DeleteRequest, DeleteResponse, ScanRequest, ScanResponse};
use kvprotos::kvserver_grpc::{self, Kvdb};

#[derive(Clone)]
struct KvService{
	engine: Arc<RwLock<BTreeMap<String,String>>>,
}

impl KvService {
	pub fn new() -> Self {
		println!("new KvService");
		KvService{
			engine: Arc::new(RwLock::new(BTreeMap::new())),
		}
	}
}

impl Kvdb for KvService {
	fn get(&mut self, ctx: RpcContext, req: GetRequest, sink: UnarySink<GetResponse>) {
        let mut response = GetResponse::new();
        println!("Received GetRequest {{ {:?} }}", req);
        
        let mutengine = &self.engine;
        let req_key = &req.key;
        let tmp = mutengine.read().unwrap();
        let res = tmp.get(req_key);
        println!("====>getcheck {:?}", self.engine);
        match res {
        	Some(s) => {
        		response.set_status(ResponseStatus::kSuccess);
        		response.set_value(s.clone());
        	},
        	None => response.set_status(ResponseStatus::kNotFound),
        }

        let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }

    fn put(&mut self, ctx: RpcContext, req: PutRequest, sink: UnarySink<PutResponse>){
    	let mut response = PutResponse::new();
    	println!("Received PutRequest {{ {:?} }}", req);

    	let mutengine = &mut self.engine;
    	let req_key = &req.key;
    	let req_val = &req.value;
    	let _res = mutengine.write().unwrap().insert(req_key.clone(), req_val.clone());

    	println!("====>putcheck {:?}", self.engine);

    	response.set_status(ResponseStatus::kSuccess);

    	let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }

    fn delete(&mut self, ctx: RpcContext, req: DeleteRequest, sink: UnarySink<DeleteResponse>){
    	let mut response = DeleteResponse::new();
    	println!("Received DeleteRequest {{ {:?} }}", req);

    	let mutengine = &mut self.engine;
    	let req_key = &req.key;
    	let res = mutengine.write().unwrap().remove(req_key);

    	match res {
    		Some(_) => response.set_status(ResponseStatus::kSuccess),
    		None => response.set_status(ResponseStatus::kNotFound),
    	}

    	let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }

    fn scan(&mut self, ctx: RpcContext, req: ScanRequest, sink: UnarySink<ScanResponse>){
    	let mut response = ScanResponse::new();
    	println!("Received ScanRequest {{ {:?} }}", req);

    	let mutengine = &self.engine;
    	let key_start = &req.key_start;
    	let key_end = &req.key_end;
    	let mut resmap = HashMap::new();
    	for (k, v) in mutengine.read().unwrap().range(key_start.clone()..key_end.clone()){
    		resmap.insert(k.clone(), v.clone());
    	}

    	if resmap.len() != 0 {
    		response.set_status(ResponseStatus::kSuccess);
    		response.set_key_value(resmap);
    	}else{
    		response.set_status(ResponseStatus::kNotFound);
    	}

    	let f = sink.success(response.clone())
            .map(move |_| println!("Responded with  {{ {:?} }}", response))
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
}

fn main(){
	let env = Arc::new(Environment::new(1));
    let service  = kvserver_grpc::create_kvdb(KvService::new());
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 20001)
        .build()
        .unwrap();

    server.start();

    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = rx.wait();
    let _ = server.shutdown().wait();
}

