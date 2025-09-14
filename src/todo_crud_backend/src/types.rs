use std::borrow::Cow;

use candid::CandidType;
use ic_http_certification::HttpResponse;
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, Storable};
use matchit::Params;
use serde::{Deserialize, Serialize};

use crate::error::TodoApiError;

/*
For generating the candid files one can use the following instructions
1)cargo install candid-extractor
2)ic_cdk::export_candid!(); (Add this macro for the generation)
3)cargo install generate-did
4)generate-did <canister_name> (TO BE RUN FROM PARENT DIRECTOR OF CANISTER)
*/
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct HeaderField(pub String, pub String);

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
//Difference is in certificates of both though
pub struct HttpUpdateRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
}

pub type TodoId = u64;
//Virtual memory for stable structures where each structure will
//have separate memory instead of shared space which persists unlike wasm based on heap
pub type Memory = VirtualMemory<DefaultMemoryImpl>;
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Todo {
    pub id: TodoId,
    pub text: String,
}
//Implementing trait bound for todo for stable storage
impl Storable for Todo {
    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let todo_struct_bytes = match candid::encode_one(self) {
            Ok(bytes) => bytes,
            Err(error) => {
                panic!("Unable to parse the corresponding struct");
            }
        };
        Cow::Owned(todo_struct_bytes)
    }
    fn into_bytes(self) -> Vec<u8> {
        let todo_struct_bytes = match candid::encode_one(self) {
            Ok(bytes) => bytes,
            Err(error) => {
                panic!("Unable to parse the corresponding struct");
            }
        };
        todo_struct_bytes
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let serialized_struct: Todo = match candid::decode_one(&*bytes) {
            Ok(todo_serialized_struct) => todo_serialized_struct,
            Err(error) => {
                panic!(
                    "An error occurred while serializing from bytes - {:?}",
                    error
                );
            }
        };

        serialized_struct
    }
}
//Functional pointer mapping wrt the route handler for each of the route
pub type RouteHandler =
    for<'a> fn(&'a HttpRequest, &'a Params) -> Result<HttpResponse<'static>, TodoApiError>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRequestBody {
    pub new_text: String,
}
