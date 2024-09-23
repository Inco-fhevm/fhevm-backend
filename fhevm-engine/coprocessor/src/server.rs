use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use std::str::FromStr;

use crate::db_queries::{
    check_if_api_key_is_valid, check_if_ciphertexts_exist_in_db, fetch_tenant_server_key,
};
use crate::server::coprocessor::GenericResponse;
use crate::types::{CoprocessorError, TfheTenantKeys};
use crate::utils::sort_computations_by_dependencies;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use alloy::sol_types::SolStruct;
use coprocessor::async_computation_input::Input;
use coprocessor::{
    FetchedCiphertext, GetCiphertextSingleResponse, InputCiphertextResponse,
    InputCiphertextResponseHandle, InputUploadBatch, InputUploadResponse,
};
use fhevm_engine_common::tfhe_ops::{
    check_fhe_operand_types, current_ciphertext_version, trivial_encrypt_be_bytes,
    try_expand_ciphertext_list, validate_fhe_type,
};
use fhevm_engine_common::types::{FhevmError, SupportedFheCiphertexts, SupportedFheOperations};
use sha3::{Digest, Keccak256};
use sqlx::{query, Acquire};
use tonic::transport::Server;

pub mod common {
    tonic::include_proto!("fhevm.common");
}

pub mod coprocessor {
    tonic::include_proto!("fhevm.coprocessor");
}

pub struct CoprocessorService {
    pool: sqlx::Pool<sqlx::Postgres>,
    args: crate::cli::Args,
    tenant_key_cache: std::sync::Arc<tokio::sync::RwLock<lru::LruCache<i32, TfheTenantKeys>>>,
    signer: PrivateKeySigner,
}

pub async fn run_server(
    args: crate::cli::Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        if let Err(e) = run_server_iteration(args.clone()).await {
            println!("Error running server, retrying shortly: {:?}", e);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
    }
}

pub async fn run_server_iteration(
    args: crate::cli::Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = args
        .server_addr
        .parse()
        .expect("Can't parse server address");
    let db_url = crate::utils::db_url(&args);

    let coprocessor_key_file = tokio::fs::read_to_string(&args.coprocessor_private_key).await?;

    let signer = PrivateKeySigner::from_str(coprocessor_key_file.trim())?;
    println!("Coprocessor signer address: {}", signer.address());

    println!("Coprocessor listening on {}", addr);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(args.pg_pool_max_connections)
        .connect(&db_url)
        .await?;

    let tenant_key_cache: std::sync::Arc<tokio::sync::RwLock<lru::LruCache<i32, TfheTenantKeys>>> =
        std::sync::Arc::new(tokio::sync::RwLock::new(lru::LruCache::new(
            NonZeroUsize::new(args.tenant_key_cache_size as usize).unwrap(),
        )));

    let service = CoprocessorService {
        pool,
        args,
        tenant_key_cache,
        signer,
    };

    Server::builder()
        .add_service(
            crate::server::coprocessor::fhevm_coprocessor_server::FhevmCoprocessorServer::new(
                service,
            ),
        )
        .serve(addr)
        .await?;

    Ok(())
}

// for EIP712 signature
alloy::sol! {
    struct CiphertextVerification {
        address aclAddress;
        bytes32 hashOfCiphertext;
        uint256[] handlesList;
        address contractAddress;
        address callerAddress;
    }
}

// copied from go coprocessor
// theData := signerApi.TypedData{
//     Types: signerApi.Types{
//         "EIP712Domain": domainType,
//         "CiphertextVerification": []signerApi.Type{
//             {Name: "handlesList", Type: "uint256[]"},
//             {Name: "contractAddress", Type: "address"},
//             {Name: "callerAddress", Type: "address"},
//         },
//     },
//     Domain: signerApi.TypedDataDomain{
//         Name:              "FHEVMCoprocessor",
//         Version:           "1",
//         ChainId:           chainId,
//         VerifyingContract: verifyingContract.Hex(),
//     },
//     PrimaryType: "CiphertextVerification",
//     Message: signerApi.TypedDataMessage{
//         "handlesList":     hexInputs,
//         "contractAddress": contractAddress.Hex(),
//         "callerAddress":   callerAddress.Hex(),
//     },
// }

#[tonic::async_trait]
impl coprocessor::fhevm_coprocessor_server::FhevmCoprocessor for CoprocessorService {
    async fn upload_inputs(
        &self,
        request: tonic::Request<InputUploadBatch>,
    ) -> std::result::Result<tonic::Response<InputUploadResponse>, tonic::Status> {
        let tenant_id = check_if_api_key_is_valid(&request, &self.pool).await?;

        let req = request.get_ref();
        if req.input_ciphertexts.len() > self.args.maximimum_compact_inputs_upload {
            return Err(tonic::Status::from_error(Box::new(
                CoprocessorError::MoreThanMaximumCompactInputCiphertextsUploaded {
                    input_count: req.input_ciphertexts.len(),
                    maximum_allowed: self.args.maximimum_compact_inputs_upload,
                },
            )));
        }

        let mut response = InputUploadResponse {
            upload_responses: Vec::with_capacity(req.input_ciphertexts.len()),
        };
        if req.input_ciphertexts.is_empty() {
            return Ok(tonic::Response::new(response));
        }

        let fetch_key_response = {
            fetch_tenant_server_key(tenant_id, &self.pool, &self.tenant_key_cache)
                .await
                .map_err(|e| tonic::Status::from_error(e))?
        };
        let chain_id = fetch_key_response.chain_id;
        let server_key = fetch_key_response.server_key;
        let verifying_contract_address = fetch_key_response.verifying_contract_address;
        let verifying_contract_address =
            alloy::primitives::Address::from_str(&verifying_contract_address).map_err(|e| {
                tonic::Status::from_error(Box::new(
                    CoprocessorError::CannotParseTenantEthereumAddress {
                        bad_address: verifying_contract_address.clone(),
                        parsing_error: e.to_string(),
                    },
                ))
            })?;
        let acl_contract_address =
            alloy::primitives::Address::from_str(&fetch_key_response.acl_contract_address).map_err(|e| {
                tonic::Status::from_error(Box::new(
                    CoprocessorError::CannotParseTenantEthereumAddress {
                        bad_address: fetch_key_response.acl_contract_address.clone(),
                        parsing_error: e.to_string(),
                    },
                ))
            })?;

        let eip_712_domain = alloy::sol_types::eip712_domain! {
            name: "FHEVMCoprocessor",
            version: "1",
            chain_id: chain_id as u64,
            verifying_contract: verifying_contract_address,
        };

        let mut tfhe_work_set = tokio::task::JoinSet::new();

        // server key is biiig, clone the pointer
        let server_key = std::sync::Arc::new(server_key);
        let mut contract_addresses = Vec::with_capacity(req.input_ciphertexts.len());
        let mut caller_addresses = Vec::with_capacity(req.input_ciphertexts.len());
        for ci in &req.input_ciphertexts {
            // parse addresses
            contract_addresses.push(
                alloy::primitives::Address::from_str(&ci.contract_address).map_err(|e| {
                    CoprocessorError::CannotParseEthereumAddress {
                        bad_address: ci.contract_address.clone(),
                        parsing_error: e.to_string(),
                    }
                })?,
            );
            caller_addresses.push(
                alloy::primitives::Address::from_str(&ci.caller_address).map_err(|e| {
                    CoprocessorError::CannotParseEthereumAddress {
                        bad_address: ci.contract_address.clone(),
                        parsing_error: e.to_string(),
                    }
                })?,
            );
        }

        for (idx, ci) in req.input_ciphertexts.iter().enumerate() {
            let cloned_input = ci.clone();
            let server_key = server_key.clone();
            tfhe_work_set.spawn_blocking(
                move || -> Result<_, (Box<(dyn std::error::Error + Send + Sync)>, usize)> {
                    let expanded =
                        try_expand_ciphertext_list(&cloned_input.input_payload, &server_key)
                            .map_err(|e| {
                                let err: Box<(dyn std::error::Error + Send + Sync)> = Box::new(e);
                                (err, idx)
                            })?;

                    Ok((expanded, idx))
                },
            );
        }

        let mut results: BTreeMap<usize, Vec<SupportedFheCiphertexts>> = BTreeMap::new();
        while let Some(output) = tfhe_work_set.join_next().await {
            let (cts, idx) = output
                .map_err(|e| {
                    let err: Box<(dyn std::error::Error + Sync + Send)> = Box::new(e);
                    tonic::Status::from_error(err)
                })?
                .map_err(|e| tonic::Status::from_error(e.0))?;

            if cts.len() > self.args.maximum_handles_per_input as usize {
                return Err(tonic::Status::from_error(Box::new(
                    CoprocessorError::CompactInputCiphertextHasMoreCiphertextThanLimitAllows {
                        input_blob_index: idx,
                        input_ciphertexts_in_blob: cts.len(),
                        input_maximum_ciphertexts_allowed: self.args.maximum_handles_per_input
                            as usize,
                    },
                )));
            }

            assert!(
                results.insert(idx, cts).is_none(),
                "fresh map, we passed vector ordered by indexes before"
            );
        }

        assert_eq!(
            results.len(),
            req.input_ciphertexts.len(),
            "We should have all the ciphertexts now"
        );

        let mut trx = self
            .pool
            .begin()
            .await
            .map_err(Into::<CoprocessorError>::into)?;
        for (idx, input_blob) in req.input_ciphertexts.iter().enumerate() {
            let mut state = Keccak256::new();
            state.update(&input_blob.input_payload);
            let blob_hash = state.finalize().to_vec();
            assert_eq!(blob_hash.len(), 32, "should be 32 bytes");

            let corresponding_unpacked = results
                .get(&idx)
                .expect("we should have all results computed now");

            // save blob for audits and historical reference
            let _ = sqlx::query!(
                "
              INSERT INTO input_blobs(tenant_id, blob_hash, blob_data, blob_ciphertext_count)
              VALUES($1, $2, $3, $4)
              ON CONFLICT (tenant_id, blob_hash) DO NOTHING
            ",
                tenant_id,
                &blob_hash,
                &input_blob.input_payload,
                corresponding_unpacked.len() as i32
            )
            .execute(trx.as_mut())
            .await
            .map_err(Into::<CoprocessorError>::into)?;

            let mut hash_of_ciphertext: [u8; 32] = [0; 32];
            hash_of_ciphertext.copy_from_slice(&blob_hash);

            let mut ct_verification = CiphertextVerification {
                hashOfCiphertext: alloy::primitives::FixedBytes(hash_of_ciphertext),
                aclAddress: acl_contract_address,
                contractAddress: contract_addresses[idx],
                callerAddress: caller_addresses[idx],
                handlesList: Vec::with_capacity(corresponding_unpacked.len()),
            };

            let mut ct_resp = InputCiphertextResponse {
                input_handles: Vec::with_capacity(corresponding_unpacked.len()),
                eip712_signature: Vec::new(),
                eip712_contract_address: contract_addresses[idx].to_string(),
                eip712_caller_address: caller_addresses[idx].to_string(),
                eip712_signer_address: self.signer.address().to_string(),
            };

            for (ct_idx, the_ct) in corresponding_unpacked.iter().enumerate() {
                let (serialized_type, serialized_ct) = the_ct.serialize();
                let ciphertext_version = current_ciphertext_version();
                let mut handle_hash = Keccak256::new();
                handle_hash.update(&blob_hash);
                handle_hash.update(&[ct_idx as u8]);
                let mut handle = handle_hash.finalize().to_vec();
                assert_eq!(handle.len(), 32);
                // idx cast to u8 must succeed because we don't allow
                // more handles than u8 size
                handle[29] = ct_idx as u8;
                handle[30] = serialized_type as u8;
                handle[31] = ciphertext_version as u8;

                let _ = sqlx::query!(
                    "
                    INSERT INTO ciphertexts(
                        tenant_id,
                        handle,
                        ciphertext,
                        ciphertext_version,
                        ciphertext_type,
                        input_blob_hash,
                        input_blob_index
                    )
                    VALUES($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (tenant_id, handle, ciphertext_version) DO NOTHING
                ",
                    tenant_id,
                    &handle,
                    &serialized_ct,
                    ciphertext_version,
                    serialized_type,
                    &blob_hash,
                    ct_idx as i32
                )
                .execute(trx.as_mut())
                .await
                .map_err(Into::<CoprocessorError>::into)?;

                ct_verification
                    .handlesList
                    .push(alloy::primitives::U256::from_be_slice(&handle));
                ct_resp.input_handles.push(InputCiphertextResponseHandle {
                    handle: handle.to_vec(),
                    ciphertext_type: serialized_type as i32,
                });
            }

            let signing_hash = ct_verification.eip712_signing_hash(&eip_712_domain);
            let eip_712_signature = self.signer.sign_hash_sync(&signing_hash).map_err(|e| {
                CoprocessorError::Eip712SigningFailure {
                    error: e.to_string(),
                }
            })?;

            ct_resp.eip712_signature = eip_712_signature.as_bytes().to_vec();

            response.upload_responses.push(ct_resp);
        }

        trx.commit().await.map_err(Into::<CoprocessorError>::into)?;

        Ok(tonic::Response::new(response))
    }

    async fn async_compute(
        &self,
        request: tonic::Request<coprocessor::AsyncComputeRequest>,
    ) -> std::result::Result<tonic::Response<coprocessor::GenericResponse>, tonic::Status> {
        let req = request.get_ref();
        if req.computations.len() > self.args.server_maximum_ciphertexts_to_schedule {
            return Err(tonic::Status::from_error(Box::new(
                CoprocessorError::TooManyCiphertextsInBatch {
                    maximum_allowed: self.args.server_maximum_ciphertexts_to_schedule,
                    got: req.computations.len(),
                },
            )));
        }

        let tenant_id = check_if_api_key_is_valid(&request, &self.pool).await?;

        if req.computations.is_empty() {
            return Ok(tonic::Response::new(GenericResponse { response_code: 0 }));
        }

        // computations are now sorted based on dependencies or error should have
        // been returned if there's circular dependency
        let (sorted_computations, handles_to_check_in_db) =
            sort_computations_by_dependencies(&req.computations)?;

        // to insert to db
        let mut ct_types =
            check_if_ciphertexts_exist_in_db(handles_to_check_in_db, tenant_id, &self.pool).await?;
        let mut computations_inputs: Vec<Vec<Vec<u8>>> =
            Vec::with_capacity(sorted_computations.len());
        let mut computations_outputs: Vec<Vec<u8>> = Vec::with_capacity(sorted_computations.len());
        let mut are_comps_scalar: Vec<bool> = Vec::with_capacity(sorted_computations.len());
        for comp in &sorted_computations {
            computations_outputs.push(comp.output_handle.clone());
            let mut handle_types = Vec::with_capacity(comp.inputs.len());
            let mut is_computation_scalar = false;
            let mut this_comp_inputs: Vec<Vec<u8>> = Vec::with_capacity(comp.inputs.len());
            let mut is_scalar_op_vec: Vec<bool> = Vec::with_capacity(comp.inputs.len());
            for (idx, ih) in comp.inputs.iter().enumerate() {
                let fhe_op: SupportedFheOperations = comp
                    .operation
                    .try_into()
                    .map_err(|e| CoprocessorError::FhevmError(e))?;
                if let Some(input) = &ih.input {
                    match input {
                        Input::InputHandle(ih) => {
                            let ct_type = ct_types
                                .get(ih)
                                .expect("this must be found if operand is non scalar");
                            handle_types.push(*ct_type);
                            this_comp_inputs.push(ih.clone());
                            is_scalar_op_vec.push(false);
                        }
                        Input::Scalar(sc) => {
                            is_computation_scalar = true;
                            handle_types.push(-1);
                            this_comp_inputs.push(sc.clone());
                            is_scalar_op_vec.push(true);
                            assert!(idx == 1 || fhe_op.does_have_more_than_one_scalar(), "we should have checked earlier that only second operand can be scalar");
                        }
                    }
                }
            }

            // check before we insert computation that it has
            // to succeed according to the type system
            let output_type = check_fhe_operand_types(
                comp.operation,
                &handle_types,
                &this_comp_inputs,
                &is_scalar_op_vec,
            )
            .map_err(|e| CoprocessorError::FhevmError(e))?;

            computations_inputs.push(this_comp_inputs);
            are_comps_scalar.push(is_computation_scalar);
            // fill in types with output handles that are computed as we go
            assert!(ct_types
                .insert(comp.output_handle.clone(), output_type)
                .is_none());
        }

        let mut trx = self
            .pool
            .begin()
            .await
            .map_err(Into::<CoprocessorError>::into)?;

        let mut new_work_available = false;
        for (idx, comp) in sorted_computations.iter().enumerate() {
            let output_type = ct_types
                .get(&comp.output_handle)
                .expect("we should have collected all output result types by now with check_fhe_operand_types");
            let fhe_operation: i16 = comp.operation.try_into().map_err(|_| {
                CoprocessorError::FhevmError(FhevmError::UnknownFheOperation(comp.operation))
            })?;
            let res = query!(
                "
                    INSERT INTO computations(
                        tenant_id,
                        output_handle,
                        dependencies,
                        fhe_operation,
                        is_completed,
                        is_scalar,
                        output_type
                    )
                    VALUES($1, $2, $3, $4, false, $5, $6)
                    ON CONFLICT (tenant_id, output_handle) DO NOTHING
                ",
                tenant_id,
                comp.output_handle,
                &computations_inputs[idx],
                fhe_operation,
                are_comps_scalar[idx],
                output_type
            )
            .execute(trx.as_mut())
            .await
            .map_err(Into::<CoprocessorError>::into)?;
            if res.rows_affected() > 0 {
                new_work_available = true;
            }
        }
        if new_work_available {
            query!("NOTIFY work_available")
                .execute(trx.as_mut())
                .await
                .map_err(Into::<CoprocessorError>::into)?;
        }
        trx.commit().await.map_err(Into::<CoprocessorError>::into)?;
        return Ok(tonic::Response::new(GenericResponse { response_code: 0 }));
    }

    async fn wait_computations(
        &self,
        _request: tonic::Request<coprocessor::AsyncComputeRequest>,
    ) -> std::result::Result<tonic::Response<coprocessor::FhevmResponses>, tonic::Status> {
        return Err(tonic::Status::unimplemented("not implemented"));
    }

    async fn trivial_encrypt_ciphertexts(
        &self,
        request: tonic::Request<coprocessor::TrivialEncryptBatch>,
    ) -> std::result::Result<tonic::Response<coprocessor::GenericResponse>, tonic::Status> {
        let tenant_id = check_if_api_key_is_valid(&request, &self.pool).await?;
        let req = request.get_ref();

        let mut unique_handles: BTreeSet<&[u8]> = BTreeSet::new();
        for val in &req.values {
            validate_fhe_type(val.output_type).map_err(|e| CoprocessorError::FhevmError(e))?;
            if !unique_handles.insert(&val.handle) {
                return Err(CoprocessorError::DuplicateOutputHandleInBatch(format!(
                    "0x{}",
                    hex::encode(&val.handle)
                ))
                .into());
            }
        }

        let mut sks = sqlx::query!(
            "
                SELECT sks_key
                FROM tenants
                WHERE tenant_id = $1
            ",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::<CoprocessorError>::into)?;

        assert_eq!(sks.len(), 1);

        let sks = sks.pop().unwrap();
        let cloned = req.values.clone();
        let out_cts = tokio::task::spawn_blocking(move || {
            let server_key: tfhe::ServerKey = bincode::deserialize(&sks.sks_key).unwrap();
            tfhe::set_server_key(server_key);

            // single threaded implementation, we can optimize later
            let mut res: Vec<(Vec<u8>, i16, Vec<u8>)> = Vec::with_capacity(cloned.len());
            for v in cloned {
                let ct = trivial_encrypt_be_bytes(v.output_type as i16, &v.be_value);
                let (ct_type, ct_bytes) = ct.serialize();
                res.push((v.handle, ct_type, ct_bytes));
            }

            res
        })
        .await
        .unwrap();

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(Into::<CoprocessorError>::into)?;
        let mut trx = conn.begin().await.map_err(Into::<CoprocessorError>::into)?;

        for (handle, db_type, db_bytes) in out_cts {
            sqlx::query!("
                    INSERT INTO ciphertexts(tenant_id, handle, ciphertext, ciphertext_version, ciphertext_type)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (tenant_id, handle, ciphertext_version) DO NOTHING
                ",
                tenant_id, handle, db_bytes, current_ciphertext_version(), db_type as i16
            )
            .execute(trx.as_mut()).await.map_err(Into::<CoprocessorError>::into)?;
        }

        trx.commit().await.map_err(Into::<CoprocessorError>::into)?;

        return Ok(tonic::Response::new(GenericResponse { response_code: 0 }));
    }

    async fn get_ciphertexts(
        &self,
        request: tonic::Request<coprocessor::GetCiphertextBatch>,
    ) -> std::result::Result<tonic::Response<coprocessor::GetCiphertextResponse>, tonic::Status>
    {
        let tenant_id = check_if_api_key_is_valid(&request, &self.pool).await?;
        let req = request.get_ref();

        if req.handles.len() > self.args.server_maximum_ciphertexts_to_get {
            return Err(tonic::Status::from_error(Box::new(
                CoprocessorError::MoreThanMaximumCiphertextsAttemptedToDownload {
                    input_count: req.handles.len(),
                    maximum_allowed: self.args.server_maximum_ciphertexts_to_get,
                },
            )));
        }

        let mut result = coprocessor::GetCiphertextResponse {
            responses: Vec::new(),
        };
        let mut set = BTreeSet::new();

        for h in &req.handles {
            let _ = set.insert(h.clone());
        }

        let cts: Vec<Vec<u8>> = set.into_iter().collect();

        let db_cts = query!(
            "
                SELECT handle, ciphertext_type, ciphertext_version, ciphertext
                FROM ciphertexts
                WHERE tenant_id = $1
                AND handle = ANY($2::BYTEA[])
            ",
            tenant_id,
            &cts
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::<CoprocessorError>::into)?;

        let mut the_map: BTreeMap<Vec<u8>, _> = BTreeMap::new();
        for ct in db_cts {
            let _ = the_map.insert(ct.handle.clone(), ct);
        }

        for h in &req.handles {
            result.responses.push(GetCiphertextSingleResponse {
                handle: h.clone(),
                ciphertext: the_map.get(h).map(|res| FetchedCiphertext {
                    ciphertext_bytes: res.ciphertext.clone(),
                    ciphertext_type: res.ciphertext_type as i32,
                    ciphertext_version: res.ciphertext_version as i32,
                }),
            });
        }

        return Ok(tonic::Response::new(result));
    }
}
