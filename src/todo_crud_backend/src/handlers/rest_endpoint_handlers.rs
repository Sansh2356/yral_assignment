use ic_http_certification::HttpResponse;
use matchit::Params;
use serde_json::json;

use crate::responses::{create_not_found_response, json_response_get};
use crate::types::UpdateRequestBody;
use crate::types::{HttpRequest, Todo};
use crate::{add_todo, error::TodoApiError, list_todos_paginated, TODOS};


pub fn get_task_handler(
    request: &HttpRequest,
    params: &Params,
) -> Result<HttpResponse<'static>, TodoApiError> {
    let task_id_parsed: u64 = match params.get("id").unwrap().parse() {
        Ok(parsed_val) => parsed_val,
        Err(error) => {
            return Err(TodoApiError::RequestParamNotParsed(
                "GET task handler".to_string(),
                "id".to_string(),
                error.to_string(),
            ));
        }
    };
    println!("Parsed id: {}", task_id_parsed);

    if let Some(fetched_task) = TODOS.with(|todo_task| todo_task.borrow().get(&task_id_parsed)) {
        println!("Found todo: {:?}", fetched_task);
        let json_str = serde_json::to_string(&fetched_task)?;
        return Ok(json_response_get(200, json_str, false));
    }
    println!(
        "No todo found in existing storage for id: {}",
        task_id_parsed
    );
    return create_not_found_response();
}
pub fn paginated_read_task_handler(
    request: &HttpRequest,
    params: &Params,
) -> Result<HttpResponse<'static>, TodoApiError> {
    let page = match params.get("page").unwrap().parse::<u64>() {
        Ok(parsed_value) => parsed_value,
        Err(error) => {
            return Err(TodoApiError::RequestParamNotParsed(
                "paginated GET task handler".to_string(),
                "page".to_string(),
                error.to_string(),
            ));
        }
    };
    let limit = match params.get("limit").unwrap().parse::<u64>() {
        Ok(parsed_value) => parsed_value,
        Err(error) => {
            return Err(TodoApiError::RequestParamNotParsed(
                "paginated GET task handler".to_string(),
                "limit".to_string(),
                error.to_string(),
            ));
        }
    };
    //Limiting conditions before generating a response
    if limit == 0 {
        return Err(TodoApiError::InvalidLimit(
            "paginated GET task handler".to_string(),
            limit,
        ));
    }
    if page == 0 {
        return Err(TodoApiError::InvalidPage(
            "paginated GET task handler".to_string(),
            page,
        ));
    }
    let fetched_todo_vec = list_todos_paginated(page, limit);
    let resp_body = match serde_json::to_string(&fetched_todo_vec) {
        Ok(body) => body,
        Err(error) => {
            return Err(TodoApiError::ResponseBodyNotConvertedToJsonStr(
                "paginated GET task handler".to_string(),
                error.to_string(),
            ));
        }
    };

    Ok(json_response_get(200, resp_body, false))
}
pub fn delete_task_handler(
    request: &HttpRequest,
    params: &Params,
) -> Result<HttpResponse<'static>, TodoApiError> {
    let task_id_parsed: u64 = match params.get("id").unwrap().parse() {
        Ok(parsed_value) => parsed_value,
        Err(error) => {
            return Err(TodoApiError::RequestParamNotParsed(
                " DELETE task handler".to_string(),
                "id".to_string(),
                error.to_string(),
            ));
        }
    };
    println!("Parsed id: {}", task_id_parsed);
    if let Some(_) = TODOS.with(|todo_tasks| todo_tasks.borrow_mut().remove(&task_id_parsed)) {
        println!("Deleted todo with id: {}", task_id_parsed);
        return Ok(json_response_get(200, true.to_string(), false));
    } else {
        println!("No todo found to delete for id: {}", task_id_parsed);
        return create_not_found_response();
    }
}
pub fn update_task_handler(
    request: &HttpRequest,
    params: &Params,
) -> Result<HttpResponse<'static>, TodoApiError> {
    let task_id_parsed: u64 = match params.get("id").unwrap().parse() {
        Ok(parsed_value) => parsed_value,
        Err(error) => {
            return Err(TodoApiError::RequestParamNotParsed(
                "UPDATE task handler".to_string(),
                "id".to_string(),
                error.to_string(),
            ));
        }
    };
    println!("Parsed id: {}", task_id_parsed);
    let body_json = String::from_utf8_lossy(&request.body);
    println!("Request body for update: {}", body_json);
    let val: UpdateRequestBody = match serde_json::from_str(&body_json) {
        Ok(body) => body,
        Err(error) => {
            return Err(TodoApiError::ResponseBodyNotConvertedToJsonStr(
                "UPDATE task handler".to_string(),
                error.to_string(),
            ));
        }
    };
    if let Some(present_or_not) = TODOS.with(|todo_tasks| {
        println!(
            "Updating todo with id {} to new text {}",
            task_id_parsed, val.new_text
        );
        if todo_tasks.borrow().contains_key(&task_id_parsed) == true {
            todo_tasks.borrow_mut().insert(
                task_id_parsed,
                Todo {
                    id: task_id_parsed,
                    text: val.new_text,
                },
            )
        } else {
            //No id found to be updated returning none
            None
        }
    }) {
        println!("Todo updated successfully for id: {}", task_id_parsed);
        println!("Task updated successfully if present");
        return Ok(json_response_get(200, true.to_string(), false));
    }
    println!("Task update failed");
    return create_not_found_response();
}
pub fn get_all_todos_handler(
    request: &HttpRequest,
    params: &Params,
) -> Result<HttpResponse<'static>, TodoApiError> {
    println!("Matched route: GET /allTodos");
    let todos = TODOS.with(|todos| todos.borrow().values().collect::<Vec<Todo>>());
    println!("Fetched todos: {:?}", todos);
    let body = match serde_json::to_string(&todos) {
        Ok(bdy) => bdy,
        Err(error) => {
            return Err(TodoApiError::ResponseBodyNotConvertedToJsonStr(
                "GET all todos".to_string(),
                error.to_string(),
            ));
        }
    };
    Ok(json_response_get(200, body, false))
}
pub fn add_new_task_handler(
    request: &HttpRequest,
    params: &Params,
) -> Result<HttpResponse<'static>, TodoApiError> {
    let body_json = String::from_utf8_lossy(&request.body);
    println!("Request body for update: {}", body_json);
    let val: UpdateRequestBody = match serde_json::from_str(&body_json) {
        Ok(body) => body,
        Err(error) => {
            return Err(TodoApiError::ResponseBodyNotConvertedToJsonStr(
                "POST new task handler".to_string(),
                error.to_string(),
            ));
        }
    };
    let response_new_id = add_todo(val.new_text);
    let body_json = json!({
        "new_job_id":response_new_id,
    });
    let jsonify_body = serde_json::to_string(&body_json);
    Ok(json_response_get(200, jsonify_body.unwrap(), false))
}
