# `todo_crud`


If you want to start working on your project right away, you might want to try the following commands:

```bash
cd todo_crud/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background --clean

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```
## To upgrade canister 

```
dfx canister stop todo_crud_backend
# After stopping make changes to canister code and rebuild the canister
dfx build
dfx canister install todo_crud_backend --mode upgrade
dfx canister start todo_crud_backend
dfx deploy todo_crud_backend
```


Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

## Additional Testing

Further more testing for canister can be done via PocketIC testnet locally by following the steps - 

- Save the downloaded file as pocket-ic.gz, decompress it, and make it executable':
    * `gzip -d pocket-ic.gz`

    * `chmod +x pocket-ic`
    * `export POCKET_IC_BIN="$(pwd)/pocket-ic"`
- After executable has been downloaded and its PATH has been exported .

- A simple test can be made following this code snippet - 
  ```
  fn setup() -> (PocketIc, Principal) {
    std::env::set_var("POCKET_IC_BIN", "YOUR_EXPORTED_PATH"); 
    let pocket_ic_instance = PocketIc::new();

    let backend_canister = pocket_ic_instance.create_canister();
    pocket_ic_instance.add_cycles(backend_canister, 223213); 
    let wasm = fs::read(WASM_BUILT).expect("Wasm file not found, run 'dfx build'.");
    pocket_ic_instance.install_canister(backend_canister, wasm, vec![], None);
    (pocket_ic_instance, backend_canister)
    }

    fn test_fetch_empty_todos(){
        let (pic, backend_canister) = setup();

        let Ok(response) = pic.query_call(
            backend_canister,
            Principal::anonymous(),
            "get_all_todos",
            encode_one("").unwrap(),
        ) else {
            panic!("Expected reply");
        };
        let result: String = decode_one(&response).unwrap();
        assert_eq!(result,"[]");
    }

