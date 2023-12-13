use super::*;

fn prepare_test(response: Vec<u8>) -> cpc_nvm3_handle_t {
    let _ = init_logger("", CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG, None).ok();

    let handle = init().unwrap();
    open(handle, "cpcd_0", true).unwrap();
    let instance_arc_mutex = get_instance(handle).unwrap();
    let mut instance = instance_arc_mutex.lock().unwrap();

    let cpc_endpoint = instance.cpc_endpoint.as_mut().unwrap();
    cpc_endpoint.push_rx(response);

    handle
}

fn finalize_test(sl_cpc_nvm3_handle: cpc_nvm3_handle_t) -> Result<(), CpcNvm3Error> {
    close(sl_cpc_nvm3_handle)?;
    deinit(sl_cpc_nvm3_handle)?;
    Ok(())
}

#[test]
fn test_nvm3_close() {
    let _ = init_logger("", CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG, None).ok();

    let handle = init().unwrap();
    open(handle, "cpcd_0", true).unwrap();
    close(handle).unwrap();
}

#[test]
fn test_nvm3_double_init_unique_handles() {
    let _ = init_logger("", CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG, None).ok();

    let handle_1 = init().unwrap();
    let handle_2 = init().unwrap();
    let handle_3 = init().unwrap();
    assert_ne!(handle_1, handle_2);
    assert_ne!(handle_2, handle_3);
}

#[test]
fn test_nvm3_write_success() {
    let response = vec![
        0x02, // cmd
        0x05, // len
        0x00, // len
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x03, // transaction_id
        0x00, // response_type sl_status
        0x00, // status byte
        0x00, // status byte
        0x00, // status byte
        0x00, // status byte
    ];
    let handle = prepare_test(response);

    let data: &[u8] = &[0x1, 0x2];
    write_data(handle, 1234, data).unwrap();
    finalize_test(handle).unwrap();
}

#[test]
fn test_nvm3_write_invalid_key_response() {
    let response = vec![
        0x02, // cmd
        0x05, // len
        0x00, // len
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x03, // transaction_id
        0x01, // response_type ecode
        0x0A, // status byte
        0xE0, // status byte
        0x00, // status byte
        0xF0, // status byte
    ];
    let handle = prepare_test(response);

    let data: &[u8] = &[0x1, 0x2];
    match write_data(handle, 1234, data) {
        Ok(_) => {
            panic!("Expected failure with invalid key error");
        }
        Err(err) => match err {
            CpcNvm3Error::ErrorCodeWithContext(err, context) => {
                log::error!("{}", context);
                assert_eq!(err, CpcNvm3ErrorCodes::CPC_NVM3_INVALID_OBJECT_KEY);
            }
        },
    }
    finalize_test(handle).unwrap();
}

#[test]
fn test_nvm3_write_unknown_response() {
    let response = vec![
        0x02, // cmd
        0x05, // len
        0x00, // len
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x03, // transaction_id
        0x01, // response_type ecode
        0x01, // status byte
        0x00, // status byte
        0x00, // status byte
        0x00, // status byte
    ];
    let handle = prepare_test(response);

    let data: &[u8] = &[0x1, 0x2];
    match write_data(handle, 1234, data) {
        Ok(_) => {
            panic!("Expected failure with invalid key error");
        }
        Err(err) => match err {
            CpcNvm3Error::ErrorCodeWithContext(err, context) => {
                log::error!("{}", context);
                assert_eq!(err, CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR);
            }
        },
    }
    finalize_test(handle).unwrap();
}

#[test]
fn test_nvm3_read_success_small() {
    let response = vec![
        0x09, // cmd
        0x0B, // len
        0x00, // len
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x03, // transaction_id
        0x01, // last_frag
        0x01, // data 1
        0x02, // data 2
        0x03, // data 3
        0x04, // data 4
        0x05, // data 5
        0x06, // data 6
        0x07, // data 7
        0x08, // data 8
        0x09, // data 9
        0x0a, // data 10
    ];
    let handle = prepare_test(response);
    let mut buffer = [0u8; 10];
    let mut data_size: u16 = 0;

    read_data(handle, 1234, &mut buffer, &mut data_size).unwrap();
    finalize_test(handle).unwrap();
}

#[test]
fn test_nvm3_read_fail_with_status() {
    let response = vec![
        0x02, // cmd
        0x05, // len
        0x00, // len
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x00, // unique_id
        0x03, // transaction_id
        0x00, // response_type sl_status
        0x01, // status byte
        0x00, // status byte
        0x00, // status byte
        0x00, // status byte
    ];
    let handle = prepare_test(response);
    let mut buffer = [0u8; 10];
    let mut data_size: u16 = 0;

    match read_data(handle, 1234, &mut buffer, &mut data_size) {
        Ok(_) => {
            panic!("Should have failed")
        }
        Err(err) => match err {
            CpcNvm3Error::ErrorCodeWithContext(_, context) => log::error!("{}", context),
        },
    }
    finalize_test(handle).unwrap();
}
