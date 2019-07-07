extern crate protobuf;
extern crate grpcio;
extern crate futures;

pub mod kvprotos;

use std::io;
use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};

use kvprotos::kvserver::{ResponseStatus, GetRequest, PutRequest, DeleteRequest, ScanRequest};
use kvprotos::kvserver_grpc::KvdbClient;

use std::collections::HashMap;

struct Client {
    client: KvdbClient,
}

impl Client {
	pub fn new(host: String, port: u16) -> Self {
        let addr = format!("{}:{}", host, port);
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(addr.as_ref());
        let kv_client = KvdbClient::new(ch);

        Client {
            client: kv_client,
        }
    }

    pub fn get(&self, key: String) -> Option<String> {
        let mut request = GetRequest::new();
        request.set_key(key);
        let ret = self.client.get(&request).expect("RPC failed");
        match ret.status {
            ResponseStatus::kSuccess => Some(ret.value),
            ResponseStatus::kNotFound | ResponseStatus::kFailed | ResponseStatus::kNoType => None
        }
    }

    pub fn put(&self, key: String, value: String) -> bool {
        let mut request = PutRequest::new();
        request.set_key(key);
        request.set_value(value);
        let ret = self.client.put(&request).expect("RPC failed");
        match ret.status {
            ResponseStatus::kSuccess => true,
            ResponseStatus::kNotFound | ResponseStatus::kFailed | ResponseStatus::kNoType => false
        }
    }

    pub fn delete(&self, key: String) -> bool {
        let mut request = DeleteRequest::new();
        request.set_key(key);
        let ret = self.client.delete(&request).expect("RPC failed");
        match ret.status {
            ResponseStatus::kSuccess => true,
            ResponseStatus::kNotFound | ResponseStatus::kFailed | ResponseStatus::kNoType => false
        }
    }

    pub fn scan(&self, key_start: String, key_end: String) -> Option<HashMap<String,String>> {
        let mut request = ScanRequest::new();
        request.set_key_start(key_start);
        request.set_key_end(key_end);
        let ret = self.client.scan(&request).expect("RPC failed");
        match ret.status {
            ResponseStatus::kSuccess => Some(ret.key_value),
            ResponseStatus::kNotFound | ResponseStatus::kFailed | ResponseStatus::kNoType => None
        }
    }

}

fn printusage(){
	println!("Usage:\n\t\tput <key> <value>\n\t\t
		      get <key>\n\t\t
		      delete <key>\n\t\t
		      scan <key> <key>\n\t\t
		      exit");
}

// put <key> <value>
// get <key>
// delete <key>
// scan <key> <key>
// exit
fn cmd(client : &Client){

	loop {
		let mut cmdline = String::new();
		io::stdin().read_line(&mut cmdline).expect("Failed to read line");

		let mut cmditer = cmdline.trim().split_whitespace();

		let mut str_vec = vec![];
		while let Some(cmdele) = cmditer.next() {
			str_vec.push(cmdele);
		}

		let len = str_vec.len();
		if len == 0 {
			printusage();
			continue;
		}

		if str_vec[0] == "put" {
			if len != 3 {
				printusage();
				continue;
			}
			let ret = client.put(str_vec[1].to_string(), str_vec[2].to_string());
			if ret {
				println!("put <{}, {}> success", str_vec[1], str_vec[2]);
			}else{
				println!("put <{}, {}> failed", str_vec[1], str_vec[2]);
			}
			continue;
		}

		if str_vec[0] == "get" {
			if len != 2 {
				printusage();
				continue;
			}
			let ret = client.get(str_vec[1].to_string());
			match ret {
        		Some(v) => println!("get:{}'s value:{}", str_vec[1], v),
        		None => println!("get None"),
    		}
    		continue;
		}

		if str_vec[0] == "delete" {
			if len != 2 {
				printusage();
				continue;
			}
			let ret = client.delete(str_vec[1].to_string());
			if ret {
				println!("delete key {} success", str_vec[1]);
			}else{
				println!("delete key {} failed", str_vec[1]);
			}
			continue;
		}

		if str_vec[0] == "scan" {
			if len != 3 {
				printusage();
				continue;
			}
			let ret = client.scan(str_vec[1].to_string(), str_vec[2].to_string());
		    match ret {
		        Some(v) => println!("scan {{ {:?} }}",v),
		        None => println!("scan None")
		    }
		    continue;
		}

		if str_vec[0] == "exit" {
			if len != 1 {
				printusage();
				continue;
			}
			break;
		}

		printusage();
	}
}

fn main() {
	let test_host = String::from("127.0.0.1");
    let test_port = 20001;

    let client = Client::new(test_host.clone(), test_port);

    cmd(&client);
    // client.put("aa".to_string(),"aaaaa".to_string());
    // client.put("bb".to_string(),"bbbbb".to_string());
    // client.put("cc".to_string(),"ccccc".to_string());
    // let ret = client.get("aa".to_string());
    // match ret {
    //     Some(v) => println!("get:aa's value:{}", v),
    //     None => println!("get None")
    // }
    // client.delete("aa".to_string());
    // client.put("dd".to_string(),"ccccc".to_string());
    // client.put("dd".to_string(),"ddddd".to_string());
    // let ret = client.scan("aa".to_string(),"ee".to_string());
    // match ret {
    //     Some(v) => println!("scan{{ {:?} }}",v),
    //     None => println!("scan None")
    // }
    // let ret = client.get("aa".to_string());
    // match ret {
    //     Some(v) => println!("get:aa's value:{}", v),
    //     None => println!("get None")
    // }
    // let ret = client.get("aa".to_string());
    // match ret {
    //     Some(v) => println!("get:aa's value:{}", v),
    //     None => println!("get None")
    // }
    // let ret = client.get("aa".to_string());
    // match ret {
    //     Some(v) => println!("get:aa's value:{}", v),
    //     None => println!("get None")
    // }
}