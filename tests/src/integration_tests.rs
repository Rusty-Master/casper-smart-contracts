#[cfg(test)]
mod tests {
    use std::arch::x86_64::_MM_EXCEPT_INEXACT;

    // Outlining aspects of the Casper test support crate to include.
    use casper_engine_test_support::{
        ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
        DEFAULT_RUN_GENESIS_REQUEST,
    };
    // Custom Casper types that will be used within this test.
    use casper_types::{runtime_args, ContractHash, RuntimeArgs};

    // Constants for values within the contract.
    const CONTRACT_KEY: &str = "counter"; // Named key referencing this contract
    const COUNT_KEY: &str = "count"; // Named key referencing the value to increment/decrement
    const CONTRACT_VERSION_KEY: &str = "version"; // Key maintaining the version of a contract package

    // Wasm file names
    const COUNTER_V1_WASM: &str = "contract.wasm";
    const COUNTER_CALL_WASM: &str = "counter-call.wasm";

    //Entry Points
    const ENTRY_POINT_COUNTER_INC: &str = "counter_inc";
    const ENTRY_POINT_COUNTER_DEC: &str = "counter_dec";

    #[test]
    fn install_v1_and_check_entry_points() {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST).commit();

        let contract_installation_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            COUNTER_V1_WASM,
            runtime_args! {},
        )
        .build();

        builder
            .exec(contract_installation_request)
            .expect_success()
            .commit();

        let contract_hash = builder
            .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
            .named_keys()
            .get(CONTRACT_KEY)
            .expect("must have contract hash key")
            .into_hash()
            .map(ContractHash::new)
            .expect("must get contract hash");

        // verify version is 1
        let account = builder
            .get_account(*DEFAULT_ACCOUNT_ADDR)
            .expect("account exists");

        let version_key = *account
            .named_keys()
            .get(CONTRACT_VERSION_KEY)
            .expect("version uref should exist");

        let version = builder
            .query(None, version_key, &[])
            .expect("should be stored value")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<u32>()
            .expect("should be u32");

        assert_eq!(version, 1);

        //verify initial value of count variable is 0
        let contract = builder
            .get_contract(contract_hash)
            .expect("contract exists");

        let count_key = *contract
            .named_keys()
            .get(COUNT_KEY)
            .expect("count uref should exist in contract named keys");

        let count = builder
            .query(None, count_key, &[])
            .expect("Should be stored value")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<i32>()
            .expect("should be i32");

        assert_eq!(count, 0);

        // Needs session code to call entry points

        // let session_code_request = ExecuteRequestBuilder::standard(
        //     *DEFAULT_ACCOUNT_ADDR,
        //     COUNTER_CALL_WASM,
        //     runtime_args! {CONTRACT_KEY => contract_hash},
        // )
        // .build();

        // builder.exec(session_code_request).expect_success().commit();

        // try calling increment by hash not session code

        let contract_increment_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            contract_hash,
            ENTRY_POINT_COUNTER_INC,
            runtime_args! {},
        )
        .build();

        builder
            .exec(contract_increment_request)
            .expect_success()
            .commit();

        let incremented_count = builder
            .query(None, count_key, &[])
            .expect("Should be stored value")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<i32>()
            .expect("should be i32");

        assert_eq!(incremented_count, 1);

        // try decrement counter
        let contract_decrement_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            contract_hash,
            ENTRY_POINT_COUNTER_DEC,
            runtime_args! {},
        )
        .build();

        builder
            .exec(contract_decrement_request)
            .expect_failure()
            .commit();

        let current_count = builder
            .query(None, count_key, &[])
            .expect("Should be stored value")
            .as_cl_value()
            .expect("should be cl value")
            .clone()
            .into_t::<i32>()
            .expect("should be i32");

        assert_eq!(current_count, 1);
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
