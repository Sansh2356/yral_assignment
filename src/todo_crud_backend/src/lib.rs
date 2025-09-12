#![allow(unused)]
use candid::{CandidType, Deserialize};
use ic_cdk::api::{certified_data_set, data_certificate};
use ic_cdk::println;
use ic_http_certification::{
    utils::{add_skip_certification_header, skip_certification_certified_data},
    HttpResponse,
};
use ic_stable_structures::Cell;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, Storable,
};
use matchit::{Params, Router};
use regex::Regex;
use serde::Serialize;
use serde_json::json;
use std::ops::{Bound, DerefMut};
use std::{borrow::Cow, cell::RefCell, collections::HashMap};
pub mod error;
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
pub struct HttpUpdateRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
}

pub type TodoId = u64;
pub type Memory = VirtualMemory<DefaultMemoryImpl>;
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Todo {
    pub id: TodoId,
    pub text: String,
}
//Implementing trait bound for todo for stable storage #[derive(Clone, Debug, CandidType, Deserialize)]

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
pub type RouteHandler = for<'a> fn(&'a HttpRequest, &'a Params) -> HttpResponse<'static>;
//TODO: Memory will be changed after initial commits
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static TODOS: RefCell<StableBTreeMap<u64, Todo, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );
    static NEXT_ID: RefCell<Cell<u64, Memory>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            1,
        )
    );
    //Simple zero copy routers instead of static matching of regex expressions
    static QUERY_ROUTER: RefCell<Router<RouteHandler>> = RefCell::new(Router::new());
    static UPDATE_ROUTER: RefCell<Router<RouteHandler>> = RefCell::new(Router::new());

}
//Instead of parsing via regex using zero-copying simple router instead
fn build_query_router() {
    QUERY_ROUTER.with(|router| {
        router
            .borrow_mut()
            .insert("/allTodos", get_all_todos_handler);
        router
            .borrow_mut()
            .insert("/getTodo/{id}", get_task_handler);
        router.borrow_mut().insert(
            "/getpaginatedTodos/{page}/{limit}",
            paginated_read_task_handler,
        );
    });
}
fn build_update_router() {
    UPDATE_ROUTER.with(|router| {
        router
            .borrow_mut()
            .insert("/createnewTodo", add_new_task_handler);
        router
            .borrow_mut()
            .insert("/updateTodo/{id}", update_task_handler);
        router
            .borrow_mut()
            .insert("/deleteTodo/{id}", delete_task_handler);
    });
}
#[ic_cdk::init]
fn init() {
    certified_data_set(&skip_certification_certified_data());
    build_query_router();
    build_update_router();
    println!("Canister Initialization complete.");
}
#[ic_cdk::update]
fn add_todo(todo_text: String) -> u64 {
    println!("Adding new todo with text: {}", todo_text);
    NEXT_ID.with(|counter| {
        let mut ref_binding = counter.borrow_mut();
        let mut id = ref_binding.get();
        let mut new_id = *id;
        println!("Generated new todo id: {}", new_id);

        let todo = Todo {
            id: new_id,
            text: todo_text,
        };
        TODOS.with(|todos| {
            println!("Inserting todo into TODOS: {:?}", todo);
            todos.borrow_mut().insert(new_id, todo);
        });
        new_id += 1;
        ref_binding.set(new_id);
        println!("Updated NEXT_ID to: {}", new_id);
        new_id
    })
}
#[ic_cdk::query]
fn get_todo(id: u64) -> Option<Todo> {
    println!("Fetching todo with id: {}", id);
    TODOS.with(|todos| todos.borrow().get(&id))
}

#[ic_cdk::query]
fn list_todos_paginated(page: u64, limit: u64) -> Vec<Todo> {
    println!("Listing todos page: {}, limit: {}", page, limit);
    TODOS.with(|todos| {
        println!("Total tasks present in mapping {}", todos.borrow().len());
        //The offset factor will be ((page_size-1)*limit)
        let start = ((page - 1) * limit) as usize;
        //Handling the final case of exceeding the last page
        let end = std::cmp::min(start + (limit as usize), todos.borrow().len() as usize);
        println!("Pagination from {},{}", start, end);
        if start >= todos.borrow().len() as usize {
            println!("Start index exceeds todos length, returning empty list");
            vec![]
        } else {
            //Range based queries in `BTreeMap` instead
            todos
                .borrow()
                .values_range((Bound::Included(start as u64), Bound::Included(end as u64)))
                .collect()
        }
    })
}
#[ic_cdk::update]
fn update_todo(id: u64, new_text: String) -> bool {
    println!("Updating todo with id: {} to new text: {}", id, new_text);
    TODOS.with(|todos| {
        let mut todos = todos.borrow_mut();
        if let Some(todo) = todos.get(&id) {
            println!("Found todo: {:?}, updating text", todo);
            let new_todo = Todo {
                text: new_text,
                id: todo.id,
            };
            todos.remove(&todo.id);
            todos.insert(todo.id, new_todo);
            true
        } else {
            println!("Todo with id {} not found", id);
            false
        }
    })
}

#[ic_cdk::update]
fn delete_todo(id: u64) -> bool {
    println!("Deleting todo with id: {}", id);
    TODOS.with(|todos| {
        let mut todos = todos.borrow_mut();
        let result = todos.remove(&id).is_some();
        println!("Delete result for id {}: {}", id, result);
        result
    })
}

#[ic_cdk::query]
fn get_all_todos() -> Vec<Todo> {
    println!("Fetching all todos");
    TODOS.with(|todos| todos.borrow().values().collect())
}

// Function to provide random id corresponding to each todo-entry.
#[ic_cdk::query]
async fn get_randomness() -> Vec<u8> {
    println!("Fetching randomness from management canister");
    let randomness = ic_cdk::management_canister::raw_rand().await.unwrap();
    println!("Randomness received: {:?}", randomness);
    randomness
}
//Either updating the `POST/PATCH/PUT/DELETE` calls or `GET` calls only
fn json_response_GET(
    status: u16,
    body: String,
    update_call_or_not: bool,
) -> ic_http_certification::HttpResponse<'static> {
    println!(
        "Creating JSON response with status {} and body: {}",
        status, body
    );
    let byte_body = body.as_bytes();
    HttpResponse::ok(
        Cow::Owned(Vec::from(byte_body)),
        vec![
            ("content-type".to_string(), "application/json".to_string()),
            (
                "strict-transport-security".to_string(),
                "max-age=31536000; includeSubDomains".to_string(),
            ),
            ("x-content-type-options".to_string(), "nosniff".to_string()),
            ("referrer-policy".to_string(), "no-referrer".to_string()),
            (
                "cache-control".to_string(),
                "no-store, max-age=0".to_string(),
            ),
            ("pragma".to_string(), "no-cache".to_string()),
        ],
    )
    .with_upgrade(update_call_or_not)
    .build()
}
fn create_not_found_response() -> HttpResponse<'static> {
    println!("Creating Not Found response");
    HttpResponse::not_found(
        Cow::Borrowed(b"Not Found" as &[u8]),
        vec![("Content-Type".to_string(), "text/plain".to_string())],
    )
    .build()
}
fn get_task_handler(request: &HttpRequest, params: &Params) -> HttpResponse<'static> {
    let task_id_parsed: u64 = params.get("id").unwrap().parse().unwrap();
    println!("Parsed id: {}", task_id_parsed);

    println!("Parsed id: {}", task_id_parsed);
    if let Some(fetched_task) = TODOS.with(|todo_task| todo_task.borrow().get(&task_id_parsed)) {
        println!("Found todo: {:?}", fetched_task);
        let json_str = serde_json::to_string(&fetched_task).unwrap();
        return json_response_GET(200, json_str, false);
    }
    println!(
        "No todo found in existing storage for id: {}",
        task_id_parsed
    );
    return create_not_found_response();
}
fn paginated_read_task_handler(request: &HttpRequest, params: &Params) -> HttpResponse<'static> {
    let page = params.get("page").unwrap().parse::<u64>().unwrap();
    let limit = params.get("limit").unwrap().parse::<u64>().unwrap();
    let fetched_todo_vec = list_todos_paginated(page, limit);
    let resp_body = serde_json::to_string(&fetched_todo_vec).unwrap();

    json_response_GET(200, resp_body, false)
}
fn delete_task_handler(request: &HttpRequest, params: &Params) -> HttpResponse<'static> {
    let task_id_parsed: u64 = params.get("id").unwrap().parse().unwrap();
    println!("Parsed id: {}", task_id_parsed);
    if let Some(_) = TODOS.with(|todo_tasks| todo_tasks.borrow_mut().remove(&task_id_parsed)) {
        println!("Deleted todo with id: {}", task_id_parsed);
        return json_response_GET(200, true.to_string(), false);
    } else {
        println!("No todo found to delete for id: {}", task_id_parsed);
        return create_not_found_response();
    }
}
fn update_task_handler(request: &HttpRequest, params: &Params) -> HttpResponse<'static> {
    let task_id_parsed: u64 = params.get("id").unwrap().parse().unwrap();
    println!("Parsed id: {}", task_id_parsed);
    let body_json = String::from_utf8_lossy(&request.body);
    println!("Request body for update: {}", body_json);
    let val: UpdateRequestBody = serde_json::from_str(&body_json).unwrap();

    if let Some(_) = TODOS.with(|todo_tasks| {
        println!(
            "Updating todo with id {} to new text {}",
            task_id_parsed, val.new_text
        );
        todo_tasks.borrow_mut().insert(
            task_id_parsed,
            Todo {
                id: task_id_parsed,
                text: val.new_text,
            },
        )
    }) {
        println!("Todo updated successfully for id: {}", task_id_parsed);
        println!("Task updated successfully");
        return json_response_GET(200, true.to_string(), false);
    }
    println!("Task update failed");
    return create_not_found_response();
}
fn get_all_todos_handler(request: &HttpRequest, params: &Params) -> HttpResponse<'static> {
    println!("Matched route: GET /allTodos");
    let todos = TODOS.with(|todos| todos.borrow().values().collect::<Vec<Todo>>());
    println!("Fetched todos: {:?}", todos);
    let body = serde_json::to_string(&todos).unwrap();
    json_response_GET(200, body, false)
}
fn add_new_task_handler(request: &HttpRequest, params: &Params) -> HttpResponse<'static> {
    let body_json = String::from_utf8_lossy(&request.body);
    println!("Request body for update: {}", body_json);
    let val: UpdateRequestBody = serde_json::from_str(&body_json).unwrap();
    let response_new_id = add_todo(val.new_text);
    let body_json = json!({
        "new_job_id":response_new_id,
    });
    let jsonify_body = serde_json::to_string(&body_json);
    return json_response_GET(200, jsonify_body.unwrap(), false);
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRequestBody {
    new_text: String,
}
/*
Request urls can be the following -:
1)Get all todo - all_todos without any params GET - http://<canister-id>.localhost:<PORT NUMBER>/allTodos
2)Get a single todo on the basis of Id - fetch a single todo params ID - GET -  http://<canister-id>.localhost:<PORT NUMBER>/getTodo/id
3)Create a new todo task - adding a new todo task and incerment the global canister state with params of `Text:String`- POST -  http://<canister-id>.localhost:<PORT NUMBER>/newTodo
4)Deleting an existing todo task - Deleting the todo task according to the provided task-id params of `ID` - DELETE -  http://<canister-id>.localhost:<PORT_NUMBER>/deleteTodo/id
5)Updating an existing todo task - Updating an exisiting task according to the given task-id params of `ID` - PUT -  http://<canister-id>.localhost:<PORT_NUMBER>/UpdateTodo/id

I have currently skipped the certifications in `canister::init` and if it does fail to verify certification during `Response verification`
then try above endpoints via - <canister-d>.raw.localhost instead of <canister-id>.localhost
*/
#[ic_cdk::update]
//It will serve only the POST/PUT/PATCH/DELETE requests
fn http_request_update(request: HttpRequest) -> ic_http_certification::HttpResponse<'static> {
    ic_cdk::println!("Received request for DELETE/PUT/PATCH is - : {:?}", request);
    let path = request.url.as_str();
    let method = request.method.as_str();
    println!("Request method: {}, path: {}", method, path);
    let resp = UPDATE_ROUTER.with_borrow(|router| {
        match router.at(path) {
            Ok(route_handler_matching) => {
                let route_handler = route_handler_matching.value;
                let response = route_handler(&request, &route_handler_matching.params);
                ic_cdk::println!("samdvncxvnxcnvc - {:?}", response);
                return response;
            }
            Err(error) => {
                panic!("{:?}", error);
            }
        };
    });
    resp
}
#[ic_cdk::query]
//It will serve only GET requests i.e are the query ones
fn http_request(request: HttpRequest) -> ic_http_certification::HttpResponse<'static> {
    ic_cdk::println!("Received request for GET is - : {:?}", request);
    let path = request.url.as_str();
    let method = request.method.as_str();
    let resp = QUERY_ROUTER.with_borrow(|router| match router.at(path) {
        Ok(route_handler_matching) => {
            let route_handler = route_handler_matching.value;
            let mut response = route_handler(&request, &route_handler_matching.params);
            add_skip_certification_header(data_certificate().unwrap(), &mut response);
            response
        }
        Err(_) => UPDATE_ROUTER.with_borrow(|router| match router.at(path) {
            Ok(_) => {
                let mut upgrade_to_update_response = json_response_GET(200, "".to_string(), true);
                add_skip_certification_header(
                    data_certificate().unwrap(),
                    &mut upgrade_to_update_response,
                );
                upgrade_to_update_response
            }
            Err(_) => {
                let mut route_not_found = create_not_found_response();
                add_skip_certification_header(data_certificate().unwrap(), &mut route_not_found);
                route_not_found
            }
        }),
    });
    resp
}
ic_cdk::export_candid!();
