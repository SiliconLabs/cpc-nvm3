/*******************************************************************************
* @file
 * @brief Co-Processor Communication Protocol(CPC) NVM3 - Library Header
 *******************************************************************************
 * # License
 * <b>Copyright 2023 Silicon Laboratories Inc. www.silabs.com</b>
 *******************************************************************************
 *
 * The licensor of this software is Silicon Laboratories Inc. Your use of this
 * software is governed by the terms of Silicon Labs Master Software License
 * Agreement (MSLA) available at
 * www.silabs.com/about-us/legal/master-software-license-agreement. This
 * software is distributed to you in Source Code format and is governed by the
 * sections of the MSLA applicable to Source Code.
 *
 ******************************************************************************/

pub mod nvm3;
pub mod protocol;
use std::ffi::{c_char, c_void, CStr};
use std::fmt;

#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum CpcNvm3ErrorCodes {
    /// Generic CPC NVM3 API failure, refer to logs for more details
    CPC_NVM3_FAILURE = -1,
    /// CPC NVM3 instance was not initialized
    CPC_NVM3_NOT_INITIALIZED = -2,
    /// An operation failed because the NVM3 instance was not opened
    CPC_NVM3_NOT_OPEN = -3,
    /// An operation failed because the NVM3 instance was not closed
    CPC_NVM3_NOT_CLOSED = -4,
    /// An unknown error occured, refer to logs for more details
    CPC_NVM3_UNKNOWN_ERROR = -5,
    /// An invalid argument was provided
    CPC_NVM3_INVALID_ARG = -6,
    /// There is a version mismatch between the lib CPC NVM3 and the CPC NVM3 component on the remote device
    CPC_NVM3_INVALID_VERSION = -7,
    /// An invalid NVM3 object key was provided
    CPC_NVM3_INVALID_OBJECT_KEY = -8,
    /// The NVM3 instance is not ready yet, the API should be called again
    CPC_NVM3_TRY_AGAIN = -9,
    /// A CPC endpoint error occured, refer to logs for more details
    CPC_NVM3_CPC_ENDPOINT_ERROR = -10,
    /// The read provided buffer is too small
    CPC_NVM3_BUFFER_TOO_SMALL = -11,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum CpcNvm3ObjectType {
    /// NVM3 entity is a counter
    CPC_NVM3_OBJECT_TYPE_COUNTER,
    /// NVM3 entity is a data object
    CPC_NVM3_OBJECT_TYPE_DATA,
    /// NVM3 entity is of an unknown type
    CPC_NVM3_OBJECT_TYPE_UNKNOWN,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum CpcNvm3LogLevel {
    CPC_NVM3_LOG_OFF,
    CPC_NVM3_LOG_ERROR,
    CPC_NVM3_LOG_WARNING,
    CPC_NVM3_LOG_INFO,
    CPC_NVM3_LOG_DEBUG,
    CPC_NVM3_LOG_TRACE,
}

impl fmt::Display for CpcNvm3ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant_str = match self {
            CpcNvm3ObjectType::CPC_NVM3_OBJECT_TYPE_COUNTER => "COUNTER",
            CpcNvm3ObjectType::CPC_NVM3_OBJECT_TYPE_DATA => "DATA",
            CpcNvm3ObjectType::CPC_NVM3_OBJECT_TYPE_UNKNOWN => "UNKNOWN",
        };
        write!(f, "{}", variant_str)
    }
}

/// @brief Initialize the logger for the CPC NVM3 library. The user must provide
///        the desired log level and the output destination file.
///
/// @param[in]  prefix     A prefix string to add to every logs. This can be used
///                        as an identifier when multiple processes log to a same file.
///                        If a prefix is not required, this argument can be NULL.
///
/// @param[in]  level      The desired log level. This is a value from the CpcNvm3LogLevel
///                        enumeration.
/// @param[in]  file_path  The output destination file. If this parameter is NULL,
///                        the logs will be written to the standard output.
///
/// @param[in]  append     A boolean that indicates whether to append to the log file if provided
///                        or to overwrite the existing content (if false).
///
/// @note The logger can only be initialized once. Attempting to initialize the logger
///       when it has already been initialized will be ignored.
#[no_mangle]
pub extern "C" fn cpc_nvm3_init_logger(
    prefix: *const c_char,
    level: CpcNvm3LogLevel,
    file_path: *const c_char,
    append: bool,
) -> i32 {
    let mut file_path_string_option = None;
    let mut prefix_string = None;

    if !prefix.is_null() {
        let prefix_c_str = unsafe { CStr::from_ptr(prefix) };
        prefix_string = Some(match prefix_c_str.to_str() {
            Ok(name) => name,
            Err(err) => {
                log::error!("Failed to convert prefix to string. {}", err.to_string());
                return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
            }
        });
    }

    if !file_path.is_null() {
        let file_path_c_str = unsafe { CStr::from_ptr(file_path) };
        file_path_string_option = Some(match file_path_c_str.to_str() {
            Ok(name) => name,
            Err(err) => {
                log::error!("Failed to convert file path to string. {}", err.to_string());
                return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
            }
        });
    }

    match nvm3::init_logger(prefix_string, level, file_path_string_option, append) {
        Ok(_) => 0,
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Initialize a new CPC NVM3 instance.
///
/// @param[out] handle A pointer to where the CPC NVM3 Handle will be stored.
///                     This handle is a positive integer and it's unique for each instance. It's used to
///                    refer to the corresponding instance in subsequent function calls.
///
/// @return On success, the function returns 0.
///         On error, the function returns a negative value, corresponding to a specific
///         CpcNvm3ErrorCodes, indicating the type of error that occurred.
#[no_mangle]
pub extern "C" fn cpc_nvm3_init(handle: *mut nvm3::cpc_nvm3_handle_t) -> i32 {
    if handle.is_null() {
        log::error!("handle must not be NULL");
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }
    match nvm3::init() {
        Ok(nvm3_handle) => {
            unsafe { *handle = nvm3_handle };
            0
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Deinitialize the specified CPC NVM3 instance, effectively releasing the associated resources.
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance to be deinitialized.
///
/// @return On success, the function returns 0.
///         On error, the function returns a negative value, corresponding to a specific
///         CpcNvm3ErrorCodes, indicating the type of error that occurred.
///
/// @note Before calling this function, the user must ensure the CPC NVM3 instance is not connected
///       to a CPC daemon. To do this, `cpc_nvm3_close` should be called first. If the instance
///       is still open, the function will return an error.
#[no_mangle]
pub extern "C" fn cpc_nvm3_deinit(cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t) -> i32 {
    match nvm3::deinit(cpc_nvm3_handle) {
        Ok(_) => 0,
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Initialize the CPC NVM3 library.
///        Upon success the user will get a handle that must be passed to subsequent calls.
///
/// @param[in]  instance_name    The name of the daemon instance. It will be the value of the instance_name in the config file of the daemon.
///                              This value can be NULL, and so the default "cpcd_0" value will be used. If running a single instance, this can
///                              be left to NULL, but when running simultaneous instances, it will need to be supplied.
/// @param[in]  instance_id      The id of the NVM3 instance.
/// @param[in]  enable_tracing   Enable tracing
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
/// @note Only one opened instance per process is allowed
#[no_mangle]
pub extern "C" fn cpc_nvm3_open(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpcd_instance_name: *const c_char,
    enable_cpc_traces: bool,
) -> i32 {
    let c_str = unsafe { CStr::from_ptr(cpcd_instance_name) };
    let instance_name = match c_str.to_str() {
        Ok(name) => name,
        Err(err) => {
            log::error!(
                "Failed to convert cpcd_instance_name to string. {}",
                err.to_string()
            );
            return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
        }
    };

    match nvm3::open(cpc_nvm3_handle, instance_name, enable_cpc_traces) {
        Ok(_) => return 0,
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Close the CPC NVM3 library.
///        Upon success the handle be considered invalid and cannot be used on
///        subsequent calls to the library
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
///
/// @return On success, the function returns 0.
///         On error, the function returns a negative value, corresponding to a specific
///         CpcNvm3ErrorCodes, indicating the type of error that occurred.
#[no_mangle]
pub extern "C" fn cpc_nvm3_close(cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t) -> i32 {
    match nvm3::close(cpc_nvm3_handle) {
        Ok(_) => {
            log::debug!("Closed instance #{}", cpc_nvm3_handle);
            0
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Write data to the specified object in the CPC NVM3 library.
///        The user must provide a valid handle obtained from the initialization process.
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
/// @param[in]  cpc_nvm3_object_key  The key of the object to write data to.
/// @param[in]  data_ptr                A pointer to the data buffer to be written.
/// @param[in]  data_length             The length of the data to be written.
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
///
/// @note The buffer is not copied. The user must ensure the data buffer is not modified during the write operation.
/// @note This API will return CPC_NVM3_TRY_AGAIN if another process is writing to the same object.
#[no_mangle]
pub extern "C" fn cpc_nvm3_write_data(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpc_nvm3_object_key: nvm3::cpc_nvm3_object_key_t,
    data_ptr: *const u8,
    data_length: u16,
) -> i32 {
    if data_length == 0 {
        log::error!("data_length must not be 0");
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }
    if data_ptr.is_null() {
        log::error!("data_ptr must not be NULL");
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }
    let data: &[u8] = unsafe { std::slice::from_raw_parts(data_ptr, data_length as usize) };

    match nvm3::write_data(cpc_nvm3_handle, cpc_nvm3_object_key, data) {
        Ok(_) => {
            log::debug!(
                "Successfully wrote to NVM3 data object {:?}",
                cpc_nvm3_object_key
            );
            return 0;
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Read data from the specified object in the CPC NVM3 library.
///        The user must provide a valid handle obtained from the initialization process.
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
/// @param[in]  cpc_nvm3_object_key  The key of the object to read data from.
/// @param[out] buffer_ptr              A pointer to the buffer where the read data will be stored.
/// @param[in]  buffer_size             The size of the provided buffer.
/// @param[out] object_size             A pointer to a variable where the actual size of the NVM3 object will be stored.
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
///
/// @note The user must ensure the provided buffer is large enough to hold the read data.
#[no_mangle]
pub extern "C" fn cpc_nvm3_read_data(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpc_nvm3_object_key: nvm3::cpc_nvm3_object_key_t,
    buffer_ptr: *mut c_void,
    buffer_size: u16,
    object_size: *mut u16,
) -> i32 {
    if buffer_ptr.is_null() || object_size.is_null() {
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }

    let buffer =
        unsafe { std::slice::from_raw_parts_mut(buffer_ptr as *mut u8, buffer_size as usize) };
    let data_size_ref: &mut u16 = unsafe { &mut *object_size };

    match nvm3::read_data(cpc_nvm3_handle, cpc_nvm3_object_key, buffer, data_size_ref) {
        Ok(_) => {
            log::debug!("Successfully read NVM3 object");
            return 0;
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Retrieve the count of objects stored in the specified CPC NVM3 instance.
///
/// @param[in]  cpc_nvm3_handle     The handle to the CPC NVM3 instance.
/// @param[out] object_count        Pointer to a variable where the total count
///                                 of stored objects will be written.
///                                 The value at this pointer will be updated
///                                 only if the function is successful.
///
/// @return On success, the function returns 0 and the object count is written
///         to the variable pointed to by the `object_count` parameter.
///         On error, it returns a negative value. This negative number corresponds
///         to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred.
#[no_mangle]
pub extern "C" fn cpc_nvm3_get_object_count(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    object_count: *mut u16,
) -> i32 {
    if object_count.is_null() {
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }
    let object_count_ref: &mut u16 = unsafe { &mut *object_count };

    match nvm3::get_object_count(cpc_nvm3_handle) {
        Ok(count) => {
            log::debug!("Successfully obtained NVM3 object count {:?}", count);
            *object_count_ref = count;
            return 0;
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Get a list of objects available on the CPC NVM3 instance
///
/// This function retrieves a list of keys for the objects stored in the NVM3 instance.
///
/// @param[in]  cpc_nvm3_handle             The handle to the CPC NVM3 instance.
/// @param[in]  cpc_nvm3_object_keys_ptr    Pointer to an array where the object keys will be stored.
/// @param[in]  max_key_count               Maximum number of keys that can be stored in the array.
/// @param[out] object_count                Pointer to a variable where the actual count of keys will be stored.
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
#[no_mangle]
pub extern "C" fn cpc_nvm3_list_objects(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpc_nvm3_object_keys_ptr: *const nvm3::cpc_nvm3_object_key_t,
    max_key_count: u16,
    object_count: *mut u16,
) -> i32 {
    if cpc_nvm3_object_keys_ptr.is_null() || object_count.is_null() || max_key_count == 0 {
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }

    let buffer = unsafe {
        std::slice::from_raw_parts_mut(
            cpc_nvm3_object_keys_ptr as *mut nvm3::cpc_nvm3_object_key_t,
            max_key_count as usize,
        )
    };

    let object_count_ref: &mut u16 = unsafe { &mut *object_count };

    match nvm3::list_objects(cpc_nvm3_handle, buffer, object_count_ref) {
        Ok(count) => {
            log::debug!("Successfully listed {:?} NVM3 objects", count);
            return 0;
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Write a value to the specified counter.
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
/// @param[in]  cpc_nvm3_object_key  The key of the counter.
/// @param[in]  value                   The value to write.
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
#[no_mangle]
pub extern "C" fn cpc_nvm3_write_counter(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpc_nvm3_object_key: nvm3::cpc_nvm3_object_key_t,
    value: u32,
) -> i32 {
    match nvm3::write_counter(cpc_nvm3_handle, cpc_nvm3_object_key, value) {
        Ok(_) => {
            log::debug!(
                "Successfully wrote to NVM3 counter {:?}",
                cpc_nvm3_object_key
            );
            0
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Read data from the specified counter.
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
/// @param[in]  cpc_nvm3_object_key  The key of the counter object to read data from.
/// @param[out] value                   A pointer to the variable where the counter data will be stored.
///                                     This value is optional, when a NULL pointer is provided, it
///                                     will be ignored.
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
#[no_mangle]
pub extern "C" fn cpc_nvm3_read_counter(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpc_nvm3_object_key: nvm3::cpc_nvm3_object_key_t,
    value: *mut u32,
) -> i32 {
    if value.is_null() {
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }
    match nvm3::read_counter(cpc_nvm3_handle, cpc_nvm3_object_key) {
        Ok(read_value) => {
            unsafe { *value = read_value };
            log::debug!("Successfully read NVM3 counter object");
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                return error_code as i32;
            }
        },
    }
    0
}

/// @brief Increment the specified counter.
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
/// @param[in]  cpc_nvm3_object_key  The key of the counter object to increment data from.
/// @param[out] new_value            A pointer to the variable where the counter new value will be stored.
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
#[no_mangle]
pub extern "C" fn cpc_nvm3_increment_counter(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpc_nvm3_object_key: nvm3::cpc_nvm3_object_key_t,
    new_value: *mut u32,
) -> i32 {
    match nvm3::increment_counter(cpc_nvm3_handle, cpc_nvm3_object_key) {
        Ok(read_value) => {
            log::debug!("Successfully incremented NVM3 counter");
            if !new_value.is_null() {
                unsafe { *new_value = read_value };
            }
            0
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Retrieve the maximum allowable size for an object that can be written
///        to the NVM3 instance on the remote device. The user must provide a
///        valid handle obtained from the initialization process.
///
/// @param[in]  cpc_nvm3_handle   The handle to the CPC NVM3 instance.
///
/// @return On success, the function returns 0.
///         On error, the function returns a negative value, corresponding to a specific
///         CpcNvm3ErrorCodes, indicating the type of error that occurred.
///
/// @note Make sure to verify that the CPC NVM3 instance is opened and functional
///       before calling this function, as it will fail otherwise.
#[no_mangle]
pub extern "C" fn cpc_nvm3_get_maximum_write_size(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    max_write: *mut u16,
) -> i32 {
    if max_write.is_null() {
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }
    match nvm3::get_maximum_write_size(cpc_nvm3_handle) {
        Ok(maximum_write_size) => {
            log::info!("Maximum write size is {} bytes", maximum_write_size);
            unsafe { *max_write = maximum_write_size };
            0
        }

        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Query additional information about the NVM3 object
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
/// @param[in]  cpc_nvm3_object_key  The key of the NVM3 object to query information from
/// @param[out] object_size             A pointer to the variable where the object size will be stored.
/// @param[out] object_type             A pointer to the variable where the object type will be stored.
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
#[no_mangle]
pub extern "C" fn cpc_nvm3_get_object_info(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpc_nvm3_object_key: nvm3::cpc_nvm3_object_key_t,
    object_size: *mut u16,
    object_type: *mut CpcNvm3ObjectType,
) -> i32 {
    if object_size.is_null() || object_type.is_null() {
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }

    match nvm3::get_object_info(cpc_nvm3_handle, cpc_nvm3_object_key) {
        Ok((rxd_object_size, rxd_object_type)) => {
            log::debug!(
                "Successfully obtained NVM3 object information for object. Key:{} Type:{} Size:{}",
                cpc_nvm3_object_key,
                rxd_object_type,
                rxd_object_size
            );
            unsafe { *object_size = rxd_object_size };
            unsafe { *object_type = rxd_object_type };
            0
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Delete an NVM3 object
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
/// @param[in]  cpc_nvm3_object_key  The key of the NVM3 object to query information from
///
/// @return On success, the function returns 0. On error, it returns a negative value.
///         This negative number corresponds to a specific CpcNvm3ErrorCodes,
///         indicating the type of error that occurred. If the connection to the CPC
///         endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.
#[no_mangle]
pub extern "C" fn cpc_nvm3_delete_object(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    cpc_nvm3_object_key: nvm3::cpc_nvm3_object_key_t,
) -> i32 {
    match nvm3::delete_object(cpc_nvm3_handle, cpc_nvm3_object_key) {
        Ok(_) => {
            log::debug!("Successfully deleted NVM3 object.");
            0
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Set the timeout on CPC operations. The timeout is the sum
/// of the provided arguments.
///
/// @param[in]  cpc_nvm3_handle         The handle to the CPC NVM3 instance.
/// @param[in]  seconds                 How many seconds to block.
/// @param[in]  microseconds            How many microseconds to block.
///
/// @return On success, the function returns 0.
///         On error, the function returns a negative value, corresponding to a specific
///         CpcNvm3ErrorCodes, indicating the type of error that occurred.
#[no_mangle]
pub extern "C" fn cpc_nvm3_set_cpc_timeout(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    seconds: i32,
    microseconds: i32,
) -> i32 {
    match nvm3::set_timeout(cpc_nvm3_handle, seconds, microseconds) {
        Ok(_) => 0,
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}

/// @brief Get the timeout on CPC operations.
///
/// @param[in]  cpc_nvm3_handle      The handle to the CPC NVM3 instance.
/// @param[out] seconds                 How many seconds to block.
/// @param[out] microseconds            How many microseconds to block.
///
/// @return On success, the function returns 0.
///         On error, the function returns a negative value, corresponding to a specific
///         CpcNvm3ErrorCodes, indicating the type of error that occurred.
#[no_mangle]
pub extern "C" fn cpc_nvm3_get_cpc_timeout(
    cpc_nvm3_handle: nvm3::cpc_nvm3_handle_t,
    seconds: *mut i32,
    microseconds: *mut i32,
) -> i32 {
    if seconds.is_null() || microseconds.is_null() {
        return CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG as i32;
    }
    match nvm3::get_timeout(cpc_nvm3_handle) {
        Ok((configured_seconds, configured_microseconds)) => {
            unsafe { *seconds = configured_seconds };
            unsafe { *microseconds = configured_microseconds };
            0
        }
        Err(err) => match err {
            nvm3::CpcNvm3Error::ErrorCodeWithContext(error_code, context) => {
                log::error!("{}", context);
                error_code as i32
            }
        },
    }
}
