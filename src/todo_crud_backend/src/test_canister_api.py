import requests
import pytest
import time

#SAME TESTS AS IN lib.rs for RUST 
#Replace the below with the local canister ID 
#If any changes made to dfx.json regards to port of replica then change the port as well 
#The ID returned is next id not the id with which the corresponding record was inserted with

"""
Test can be run via -

1)pip install requests(if not pre-installed)
2)pip install pytest(if not pre-installed)
3)pytest -v 
4)For a particular test case - pytest -v test_canister_api.py::test_pagination_with_invalid_params
"""
CANISTER_ID = "uxrrr-q7777-77774-qaaaq-cai"
BASE_URL = f"http://{CANISTER_ID}.localhost:4943"

#defining fixture for initializing the Session 
@pytest.fixture(scope="module")
def session():
    return requests.Session()
#defining feature that will be executed after each of the correspnding test
#defined below to return the clean canister state after tessting 
@pytest.fixture(scope="function", autouse=True)
def canister_state_cleanup(session):
    yield  

    try:
        #Deleting all the data from the canister
        response = session.get(f"{BASE_URL}/allTodos")
        todos = response.json()

        for todo in todos:
            todo_id = todo.get("id")
            if todo_id is not None:
                delete_url = f"{BASE_URL}/deleteTodo/{todo_id}"
                session.delete(delete_url)
    except requests.RequestException as e:
        print(f"An error occurred while cleaning the canister state {e}")


# a helper function to create random todo tasks for testing
def create_todo(session, text: str) -> dict:
    url = f"{BASE_URL}/createnewTodo"
    response = session.post(url, json={"new_text": text})
    return response.json()

# testcase for empty tasks
def test_get_all_todos_empty(session):
    response = session.get(f"{BASE_URL}/allTodos")
    assert response.status_code == 200
    assert response.json() == []

def test_create_and_get_todo(session):
    create_response = create_todo(session, "Random task")
    assert "new_job_id" in create_response
    assert isinstance(create_response["new_job_id"], int)
    new_id = create_response["new_job_id"]-1

    get_response = session.get(f"{BASE_URL}/getTodo/{new_id}")
    assert get_response.status_code == 200
    todo = get_response.json()
    assert todo["id"] == new_id
    assert todo["text"] == "Random task"
#Empty string as input 
def test_create_todo_with_empty_string(session):
    create_response = create_todo(session, "")
    new_id = create_response["new_job_id"]-1

    get_response = session.get(f"{BASE_URL}/getTodo/{new_id}")
    assert get_response.status_code == 200
    todo = get_response.json()
    assert todo["text"] == ""
#Random unicode text as input
def test_create_todo_with_special_chars(session):
    special_text = "!@#$%^&*() and unicode üñîçødé"
    create_response = create_todo(session, special_text)
    new_id = create_response["new_job_id"]-1

    get_response = session.get(f"{BASE_URL}/getTodo/{new_id}")
    assert get_response.status_code == 200
    assert get_response.json()["text"] == special_text

def test_get_nonexistent_todo(session):
    non_existent_id = 99999
    response = session.get(f"{BASE_URL}/getTodo/{non_existent_id}")
    assert response.status_code == 404

def test_update_todo(session):
    create_response = create_todo(session, "Random text")
    todo_id = create_response["new_job_id"]-1

    update_url = f"{BASE_URL}/updateTodo/{todo_id}"
    update_payload = {"new_text": "Random text updated"}
    update_response = session.put(update_url, json=update_payload)
    assert update_response.status_code == 200
    assert update_response.json() is True

    get_response = session.get(f"{BASE_URL}/getTodo/{todo_id}")
    assert get_response.status_code == 200
    updated_todo = get_response.json()
    assert updated_todo["id"] == todo_id
    assert updated_todo["text"] == "Random text updated"

def test_update_nonexistent_todo(session):
    non_existent_id = 88888
    update_url = f"{BASE_URL}/updateTodo/{non_existent_id}"
    update_payload = {"new_text": "Test text"}

    update_response = session.put(update_url, json=update_payload)
    assert update_response.status_code == 404

    get_response = session.get(f"{BASE_URL}/getTodo/{non_existent_id}")
    assert get_response.status_code == 404

def test_delete_todo(session):
    create_response = create_todo(session, "Todo to be deleted")
    todo_id = create_response["new_job_id"]-1

    delete_url = f"{BASE_URL}/deleteTodo/{todo_id}"
    delete_response = session.delete(delete_url)
    assert delete_response.status_code == 200
    assert delete_response.json() is True

    get_response = session.get(f"{BASE_URL}/getTodo/{todo_id}")
    assert get_response.status_code == 404

def test_delete_nonexistent_todo(session):
    non_existent_id = 77777
    delete_url = f"{BASE_URL}/deleteTodo/{non_existent_id}"
    delete_response = session.delete(delete_url)
    assert delete_response.status_code == 404

def test_pagination(session):
    for i in range(5):
        create_todo(session, f"Todo item {i+1}")
        time.sleep(0.1) 

    response1 = session.get(f"{BASE_URL}/getpaginatedTodos/1/2")
    assert response1.status_code == 200
    data1 = response1.json()
    assert len(data1) == 2
    assert data1[0]["text"] == "Todo item 1"
    assert data1[1]["text"] == "Todo item 2"

    response2 = session.get(f"{BASE_URL}/getpaginatedTodos/2/2")
    assert response2.status_code == 200
    data2 = response2.json()
    assert len(data2) == 2
    assert data2[0]["text"] == "Todo item 3"
    assert data2[1]["text"] == "Todo item 4"

    response3 = session.get(f"{BASE_URL}/getpaginatedTodos/3/2")
    assert response3.status_code == 200
    data3 = response3.json()
    assert len(data3) == 1
    assert data3[0]["text"] == "Todo item 5"

    response4 = session.get(f"{BASE_URL}/getpaginatedTodos/4/2")
    assert response4.status_code == 200
    assert response4.json() == []

def test_pagination_edge_cases(session):
    for i in range(3):
        create_todo(session, f"Edge item {i+1}")
        time.sleep(0.1)

    response1 = session.get(f"{BASE_URL}/getpaginatedTodos/1/10")
    assert response1.status_code == 200
    data1 = response1.json()
    assert len(data1) == 3

    response2 = session.get(f"{BASE_URL}/getpaginatedTodos/2/1")
    assert response2.status_code == 200
    data2 = response2.json()
    assert len(data2) == 1
    assert data2[0]["text"] == "Edge item 2"

def test_invalid_id_format(session):
    response = session.get(f"{BASE_URL}/getTodo/not-a-number")
    # breakpoint()
    assert response.status_code == 500
#invalid json body passed in request
def test_invalid_request_body(session):
    url = f"{BASE_URL}/createnewTodo"
    headers = {"Content-Type": "application/json"}
    malformed_json_response = session.post(url, data='{"new_text": "missing quote}')
    assert malformed_json_response.status_code == 500

    wrong_field_response = session.post(url, json={"wrong_field": "some text"})
    assert wrong_field_response.status_code == 500
    
#Paging limit
def test_pagination_with_invalid_params(session):
    page = 0
    limit = 5
    response_1 = session.get(f"{BASE_URL}/getpaginatedTodos/{page}/{limit}")
    assert response_1.status_code == 500
    page = 1
    limit = 0
    response_2 = session.get(f"{BASE_URL}/getpaginatedTodos/{page}/{limit}")
    assert response_2.status_code == 500
    
#A random concurrent test execution   
def test_concurrent_creations(session):
    import concurrent.futures

    num_requests = 5
    with concurrent.futures.ThreadPoolExecutor(max_workers=num_requests) as executor:
        future_to_text = {executor.submit(create_todo, session, f"Concurrent todo {i}"): i for i in range(num_requests)}
        
        created_ids = set()
        for future in concurrent.futures.as_completed(future_to_text):
            try:
                response = future.result()
                created_ids.add(response["new_job_id"])
            except Exception as exc:
                pytest.fail(f'Concurrent creation generated an exception: {exc}')

    assert len(created_ids) == num_requests, "Duplicate IDs were generated during concurrent creation."

#Paging limit
def test_create_todo_with_large_payload(session):
    large_text = "HELLO YRAL" * 6262
    create_response = create_todo(session, large_text)
    new_id = create_response["new_job_id"]-1

    get_response = session.get(f"{BASE_URL}/getTodo/{new_id}")
    assert get_response.status_code == 200
    todo = get_response.json()
    assert todo["text"] == large_text

