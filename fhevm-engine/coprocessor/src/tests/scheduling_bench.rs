use crate::db_queries::query_tenant_keys;
use crate::server::common::FheOperation;
use crate::server::coprocessor::{async_computation_input::Input, AsyncComputationInput};
use crate::server::coprocessor::{
    fhevm_coprocessor_client::FhevmCoprocessorClient, AsyncComputation, AsyncComputeRequest,
    InputToUpload, InputUploadBatch, TrivialEncryptBatch, TrivialEncryptRequestSingle,
};
use crate::tests::utils::{
    decrypt_ciphertexts, default_api_key, default_tenant_id, random_handle, setup_test_app,
    wait_until_all_ciphertexts_computed,
};
use alloy::primitives::keccak256;
use bigdecimal::num_bigint::BigInt;
use fhevm_engine_common::utils::safe_serialize;
use std::str::FromStr;
use std::time::SystemTime;
use tonic::metadata::MetadataValue;

pub fn test_random_user_address() -> String {
    let _private_key = "bd2400c676871534a682ca1c5e4cd647ec9c3e122f188c6e3f54e6900d586c7b";
    let public_key = "0x1BdA2a485c339C95a9AbfDe52E80ca38e34C199E";
    public_key.to_string()
}

pub fn test_random_contract_address() -> String {
    "0x76c222560Db6b8937B291196eAb4Dad8930043aE".to_string()
}

fn supported_bits_to_bit_type_in_db(inp: i32) -> i32 {
    match inp {
        1 => 0, // 1 bit - boolean
        4 => 1,
        8 => 2,
        16 => 3,
        32 => 4,
        64 => 5,
        128 => 6,
        160 => 7,
        256 => 8,
        512 => 9,
        1024 => 10,
        2048 => 11,
        other => panic!("unknown supported bits: {other}"),
    }
}

#[tokio::test]
async fn schedule_multi_erc20() -> Result<(), Box<dyn std::error::Error>> {
    let app = setup_test_app().await?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(app.db_url())
        .await?;
    let mut client = FhevmCoprocessorClient::connect(app.app_url().to_string()).await?;

    let mut handle_counter: u64 = random_handle();
    let mut next_handle = || {
        let out: u64 = handle_counter;
        handle_counter += 1;
        out.to_be_bytes().to_vec()
    };
    let api_key_header = format!("bearer {}", default_api_key());

    let mut output_handles = vec![];
    //let mut enc_request_payload = vec![];
    let mut async_computations = vec![];
    let mut num_samples: usize = 2;
    let samples = std::env::var("FHEVM_TEST_NUM_SAMPLES");
    if let Ok(samples) = samples {
        num_samples = samples.parse::<usize>().unwrap();
    }

    let keys = query_tenant_keys(vec![default_tenant_id()], &pool)
        .await
        .map_err(|e| {
            let e: Box<dyn std::error::Error> = e;
            e
        })?;
    let keys = &keys[0];
    let mut builder = tfhe::ProvenCompactCiphertextList::builder(&keys.pks);
    let the_list = builder
        .push(100_u64) // Balance source
        .push(10_u64) // Transfer amount
        .push(20_u64) // Balance destination
        .push(0_u64) // 0
        .build_with_proof_packed(&keys.public_params, &[], tfhe::zk::ZkComputeLoad::Proof)
        .unwrap();

    let serialized = safe_serialize(&the_list);
    let input_bytes = keccak256(&serialized);
    println!("Encrypting inputs...");
    let mut input_request = tonic::Request::new(InputUploadBatch {
        input_ciphertexts: vec![InputToUpload {
            input_payload: serialized,
            signatures: Vec::new(),
            user_address: test_random_user_address(),
            contract_address: test_random_contract_address(),
        }],
    });
    input_request.metadata_mut().append(
        "authorization",
        MetadataValue::from_str(&api_key_header).unwrap(),
    );
    let resp = client.upload_inputs(input_request).await?;
    let resp = resp.get_ref();
    assert_eq!(resp.upload_responses.len(), 1);
    let first_resp = &resp.upload_responses[0];
    assert_eq!(first_resp.input_handles.len(), 4);

    let handle_bals = first_resp.input_handles[0].handle.clone();
    let bals = AsyncComputationInput {
        input: Some(Input::InputHandle(handle_bals.clone())),
    };
    let handle_trxa = first_resp.input_handles[1].handle.clone();
    let trxa = AsyncComputationInput {
        input: Some(Input::InputHandle(handle_trxa.clone())),
    };
    let handle_bald = first_resp.input_handles[2].handle.clone();
    let bald = AsyncComputationInput {
        input: Some(Input::InputHandle(handle_bald.clone())),
    };
    let handle_zero = first_resp.input_handles[3].handle.clone();
    let zero = AsyncComputationInput {
        input: Some(Input::InputHandle(handle_zero.clone())),
    };

    for _ in 0..=(num_samples - 1) as u32 {
        let le_handle = next_handle();
        output_handles.push(le_handle.clone());
        let ite_handle = next_handle();
        output_handles.push(ite_handle.clone());
        let sub_handle = next_handle();
        output_handles.push(sub_handle.clone());
        let add_handle = next_handle();
        output_handles.push(add_handle.clone());

        async_computations.push(AsyncComputation {
            operation: FheOperation::FheSub.into(),
            output_handle: sub_handle,
            inputs: vec![
                bals.clone(),
                AsyncComputationInput {
                    input: Some(Input::InputHandle(ite_handle.clone())),
                },
            ],
        });
        async_computations.push(AsyncComputation {
            operation: FheOperation::FheAdd.into(),
            output_handle: add_handle,
            inputs: vec![
                bald.clone(),
                AsyncComputationInput {
                    input: Some(Input::InputHandle(ite_handle.clone())),
                },
            ],
        });
        async_computations.push(AsyncComputation {
            operation: FheOperation::FheLe.into(),
            output_handle: le_handle.clone(),
            inputs: vec![trxa.clone(), bals.clone()],
        });
        async_computations.push(AsyncComputation {
            operation: FheOperation::FheIfThenElse.into(),
            output_handle: ite_handle.clone(),
            inputs: vec![
                AsyncComputationInput {
                    input: Some(Input::InputHandle(le_handle.clone())),
                },
                trxa.clone(),
                zero.clone(),
            ],
        });
    }

    println!("Scheduling computations...");
    let mut compute_request = tonic::Request::new(AsyncComputeRequest {
        computations: async_computations,
    });
    compute_request.metadata_mut().append(
        "authorization",
        MetadataValue::from_str(&api_key_header).unwrap(),
    );
    let _resp = client.async_compute(compute_request).await?;

    println!("Computations scheduled, waiting upon completion...");
    let now = SystemTime::now();

    wait_until_all_ciphertexts_computed(&app).await?;
    println!("Execution time: {}", now.elapsed().unwrap().as_millis());

    let decrypt_request = output_handles.clone();
    let resp = decrypt_ciphertexts(&pool, 1, decrypt_request).await?;

    assert_eq!(
        resp.len(),
        output_handles.len(),
        "Outputs length doesn't match"
    );
    for (i, r) in resp.iter().enumerate() {
        match r.value.as_str() {
            "true" if i % 4 == 0 => (), // trxa <= bals true
            "10" if i % 4 == 1 => (),   // select trxa
            "90" if i % 4 == 2 => (),   // bals - trxa
            "30" if i % 4 == 3 => (),   // bald + trxa
            s => panic!("unexpected result: {} for output {i}", s),
        }
    }
    Ok(())
}
