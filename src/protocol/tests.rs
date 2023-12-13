use super::*;
use crate::nvm3::init_logger;
use crate::CpcNvm3LogLevel;

#[test]
fn test_invalid_transaction_id_error() {
    let _ = init_logger(CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG, None).ok();

    let write_completed_response_with_invalid_transaction_id = vec![
        0x01, // cmd
        0x00, // len 1
        0x06, // len 2
        0x00, // transaction_id
        0x00, // response_type sl_status
        0x00, // status byte 1
        0x00, // status byte 2
        0x00, // status byte 3
        0x00, // status byte 4
    ];

    let mut transaction_id: u8 = 0;
    let object_key: u32 = 1234;
    let offset: u16 = 100;
    let last_frag = 1;
    let data = vec![0u8; 1024];
    let cmd_write_data =
        CmdWriteData::new(&mut transaction_id, object_key, offset, last_frag, data);

    match cmd_write_data.parse_response(&write_completed_response_with_invalid_transaction_id) {
        Err(ProtocolError::InvalidTransactionId(expected, actual)) => {
            assert_eq!(expected, 1);
            assert_eq!(actual, 0);
        }
        Err(err) => {
            log::error!("Error details: {:?}", err);
            panic!("Expected InvalidTransactionId error");
        }
        _ => panic!("Expected InvalidTransactionId error"),
    }
}

#[test]
fn test_invalid_command_id_error() {
    let _ = init_logger(CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG, None).ok();

    let write_completed_response_with_invalid_command_id = vec![
        0x00, // cmd
        0x00, // len 1
        0x06, // len 2
        0x01, // transaction_id
        0x00, // response_type sl_status
        0x00, // status byte 1
        0x00, // status byte 2
        0x00, // status byte 3
        0x00, // status byte 4
    ];

    let mut transaction_id: u8 = 0;
    let object_key: u32 = 1234;
    let offset: u16 = 100;
    let last_frag = 1;
    let data = vec![0u8; 1024];
    let cmd_write_data =
        CmdWriteData::new(&mut transaction_id, object_key, offset, last_frag, data);

    match cmd_write_data.parse_response(&write_completed_response_with_invalid_command_id) {
        Err(ProtocolError::InvalidCommandId) => {
            log::error!("Error Invalid command Id");
        }
        Err(err) => {
            log::error!("Error details: {:?}", err);
            panic!("Expected InvalidCommandId error");
        }
        _ => panic!("Expected InvalidCommandId error"),
    }
}
#[test]
fn test_invalid_transaction_id_wrap_around() {
    let _ = init_logger(CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG, None).ok();

    let write_completed_response_with_overflowed_transaction_id = vec![
        0x02, // cmd
        0x06, // len 1
        0x00, // len 2
        0x00, // transaction_id
        0x00, // response_type sl_status
        0x00, // status byte 1
        0x00, // status byte 2
        0x00, // status byte 3
        0x00, // status byte 4
    ];

    let mut transaction_id: u8 = 0xFF;
    let object_key: u32 = 1234;
    let offset: u16 = 100;
    let last_frag = 1;
    let data = vec![0u8; 1024];
    let cmd_write_data =
        CmdWriteData::new(&mut transaction_id, object_key, offset, last_frag, data);
    cmd_write_data
        .parse_response(&write_completed_response_with_overflowed_transaction_id)
        .unwrap();
}

#[test]
fn test_invalid_response_len() {
    let _ = init_logger(CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG, None).ok();

    let write_completed_response_with_invalid_len = vec![
        0x02, // cmd
        0x00, // len 1
        0x10, // len 2
        0x01, // transaction_id
        0x00, // response_type sl_status
        0x00, // status byte 1
        0x00, // status byte 2
        0x00, // status byte 3
        0x00, // status byte 4
    ];

    let mut transaction_id: u8 = 0;
    let object_key: u32 = 1234;
    let offset: u16 = 100;
    let last_frag = 1;
    let data = vec![0u8; 1024];
    let cmd_write_data =
        CmdWriteData::new(&mut transaction_id, object_key, offset, last_frag, data);

    match cmd_write_data.parse_response(&write_completed_response_with_invalid_len) {
        Err(ProtocolError::InvalidResponseLen(expected, actual)) => {
            assert_eq!(expected, 6);
            assert_eq!(actual, 4096);
        }
        Err(err) => {
            log::error!("Error details: {:?}", err);
            panic!("Expected InvalidResponseLen error");
        }
        _ => panic!("Expected InvalidResponseLen error"),
    }
}

#[test]
fn test_valid_write_completed_response() {
    let _ = init_logger(CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG, None).ok();

    let write_completed_response_with_invalid_len = vec![
        0x02, // cmd
        0x06, // len 1
        0x00, // len 2
        0x01, // transaction_id
        0x00, // response_type sl_status
        0x00, // status byte 1
        0x00, // status byte 2
        0x00, // status byte 3
        0x00, // status byte 4
    ];

    let mut transaction_id: u8 = 0;
    let object_key: u32 = 1234;
    let offset: u16 = 100;
    let last_frag = 1;
    let data = vec![0u8; 1024];
    let cmd_write_data =
        CmdWriteData::new(&mut transaction_id, object_key, offset, last_frag, data);

    cmd_write_data
        .parse_response(&write_completed_response_with_invalid_len)
        .unwrap();
}
