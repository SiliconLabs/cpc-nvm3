/*******************************************************************************
* @file
 * @brief Co-Processor Communication Protocol(CPC) NVM3 - NVM3 Module
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
#[cfg(test)]
mod libcpc_mock;
#[cfg(test)]
mod tests;

use crate::protocol;
use crate::protocol::*;
use crate::CpcNvm3ErrorCodes;
use crate::CpcNvm3LogLevel;
use crate::CpcNvm3ObjectType;
use chrono::Local;
use libc::STDOUT_FILENO;
use log::{LevelFilter, Log, Metadata, Record};
use nom::multi::many0;
use nom::number::complete::le_u32;
use std::collections::HashMap;
use std::convert::From;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;

// Configure the mock CPC endpoint and handle if we are running tests
#[cfg(not(test))]
use libcpc as cpc;
#[cfg(test)]
use libcpc_mock as cpc;

const CPC_NVM3_MAJOR_VERSION: u8 = pkg_version::pkg_version_major!();
const CPC_NVM3_MINOR_VERSION: u8 = pkg_version::pkg_version_minor!();
const CPC_NVM3_PATCH_VERSION: u8 = pkg_version::pkg_version_patch!();

const CPC_NVM3_OBJECT_KEY_SIZE: usize = std::mem::size_of::<cpc_nvm3_object_key_t>();

const CPC_NVM3_READ_TIMEOUT_S: i32 = 5;
const CPC_ENDPOINT_TX_WINDOW: u8 = 1;

lazy_static::lazy_static! {
    static ref LOGGER_INITIALIZED: Mutex<bool> = Mutex::new(false);
    static ref CPC_NVM_LIB_INSTANCE_KEY: Mutex<u32> = Mutex::new(1);

    // We use Arc<Mutex<...>> to safely share the mutable instances across multiple threads.
    // Arc is an atomic reference count that manages the lifetime and shared ownership of the instances
    static ref CPC_NVM3_LIB_INSTANCES: Mutex<HashMap<cpc_nvm3_handle_t, Arc<Mutex<CpcNvm3Instance>>>> = Mutex::new(HashMap::new());
}

#[derive(Error, Debug)]
pub enum CpcNvm3Error {
    #[error("CPC NVM3 Error")]
    ErrorCodeWithContext(CpcNvm3ErrorCodes, String),
}

impl From<cpc::Error> for CpcNvm3Error {
    fn from(error: cpc::Error) -> Self {
        match error {
            cpc::Error::Errno(errno) => {
                if errno.kind() == std::io::ErrorKind::WouldBlock {
                    return CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_TRY_AGAIN,
                        format!("libcpc error: {} Try again", errno),
                    );
                }
                CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                    format!("libcpc error: {}", errno),
                )
            }
            error => CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                format!("libcpc error: {}", error),
            ),
        }
    }
}

impl From<CpcNvm3LogLevel> for log::LevelFilter {
    fn from(level: CpcNvm3LogLevel) -> Self {
        match level {
            CpcNvm3LogLevel::CPC_NVM3_LOG_OFF => log::LevelFilter::Off,
            CpcNvm3LogLevel::CPC_NVM3_LOG_ERROR => log::LevelFilter::Error,
            CpcNvm3LogLevel::CPC_NVM3_LOG_WARNING => log::LevelFilter::Warn,
            CpcNvm3LogLevel::CPC_NVM3_LOG_INFO => log::LevelFilter::Info,
            CpcNvm3LogLevel::CPC_NVM3_LOG_DEBUG => log::LevelFilter::Debug,
            CpcNvm3LogLevel::CPC_NVM3_LOG_TRACE => log::LevelFilter::Trace,
        }
    }
}

impl From<ProtocolError> for CpcNvm3Error {
    fn from(error: ProtocolError) -> Self {
        match error {
            ProtocolError::Bug(context) => CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                format!("Bug: {}", context),
            ),
            ProtocolError::UnknownProcotolError => CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                "Unknown protocol error".to_string(),
            ),
            ProtocolError::InvalidTransactionId(expected_id, received_id) => {
                let context = format!(
                    "Received a response with an invalid transaction id: expected {}, received {}",
                    expected_id, received_id
                );
                CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, context)
            }
            ProtocolError::InvalidCommandId => {
                let context = format!("Received a response with an invalid command id");
                CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, context)
            }
            ProtocolError::InvalidUniqueId(expected_id, received_id) => {
                let context = format!(
                    "Received a response with an invalid unique id: expected {}, received {}",
                    expected_id, received_id
                );
                CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, context)
            }
            ProtocolError::InvalidResponseLen(expected_id, received_id) => {
                let context = format!(
                    "Received a response with an invalid length field: expected {}, received {}",
                    expected_id, received_id
                );
                CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, context)
            }
            ProtocolError::SerializationError(context) => {
                CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, context)
            }
            ProtocolError::DeserializationError(context) => {
                CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, context)
            }
        }
    }
}

#[cfg(not(test))]
extern "C" {
    pub fn cpc_deinit(handle: *mut libcpc::cpc_handle_t) -> ::std::os::raw::c_int;
}

enum RxParseOutcome<R, E> {
    Parsed(R),
    Retry,
    Error(E),
}

struct CpcNvm3Instance {
    transaction_id: u8,
    unique_id: u32,
    maximum_write_fragment_size: Option<u16>,
    maximum_write_size: Option<u16>,
    cpc_endpoint: Option<cpc::cpc_endpoint>,
    cpc_handle: Option<cpc::cpc_handle>,
}

impl CpcNvm3Instance {
    pub fn new() -> Self {
        Self {
            unique_id: 0,
            transaction_id: 0,
            maximum_write_fragment_size: None,
            maximum_write_size: None,
            cpc_endpoint: None,
            cpc_handle: None,
        }
    }

    #[cfg(test)]
    fn reconnect(&mut self) -> Result<(), CpcNvm3Error> {
        Ok(())
    }

    #[cfg(not(test))]
    fn reconnect(&mut self) -> Result<(), CpcNvm3Error> {
        log::info!("Attempting to reconnect to libcpc");

        // Close the endpoint if it was not done previously
        match self.cpc_endpoint {
            Some(mut cpc_endpoint) => {
                log::debug!("Closing CPC endpoint in reconnection attempt");
                cpc_endpoint.close()?;
                self.cpc_endpoint = None;
            }
            None => {}
        }

        // Attempt to reconnect to libcpc
        match &mut self.cpc_handle {
            Some(cpc_handle) => {
                log::debug!("Restarting libcpc");
                // Give cpc_restart two attempts
                if let Err(_) = cpc_handle.restart() {
                    cpc_handle.restart()?;
                }

                // Attempt to connect to the NVM3 endpoint
                let ep_id = cpc::cpc_endpoint_id::Service(
                    cpc::sl_cpc_service_endpoint_id_t_enum::SL_CPC_ENDPOINT_NVM3,
                );
                log::debug!("Opening libcpc endpoint in reconnection attempt");
                match cpc_handle.open_endpoint(ep_id, CPC_ENDPOINT_TX_WINDOW) {
                    Ok(cpc_endpoint) => self.cpc_endpoint = Some(cpc_endpoint),
                    Err(err) => match err {
                        libcpc::Error::Errno(_) => return Err(err.into()),
                        libcpc::Error::NulError(_) => return Err(err.into()),
                        libcpc::Error::InvalidEndpointStateType(_) => return Err(err.into()),
                        libcpc::Error::InvalidEndpointEventType(_) => return Err(err.into()),
                    },
                }
                log::debug!("Successfully reconnected to libcpc");
                Ok(())
            }
            None => Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
                format!("Can't reconnect to a closed CPC endpoint"),
            )),
        }
    }

    fn open(
        &mut self,
        cpcd_instance_name: &str,
        enable_cpc_traces: bool,
    ) -> Result<(), CpcNvm3Error> {
        log::info!(
            "Opening [CPC NVM3 v{}.{}.{}]",
            CPC_NVM3_MAJOR_VERSION,
            CPC_NVM3_MINOR_VERSION,
            CPC_NVM3_PATCH_VERSION
        );

        if self.cpc_handle.is_some() || self.cpc_endpoint.is_some() {
            return Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_NOT_CLOSED,
                format!("Tried to open already opened instance"),
            ));
        }

        unsafe extern "C" fn reset_callback() {
            log::debug!("LibCPC reset received");
        }

        let mut result = || {
            let cpc_handle = match cpc::init(
                cpcd_instance_name,
                enable_cpc_traces,
                Some(reset_callback),
            ) {
                Ok(cpc_handle) => cpc_handle,
                Err(err) => {
                    Err(CpcNvm3Error::ErrorCodeWithContext(
                                CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                                format!("Failed to init libCPC. CPCd with ({}) needs to run and be connected to a secondary. {}", cpcd_instance_name, err.to_string()),
                            ))
                }?,
            };
            self.cpc_handle = Some(cpc_handle);

            // Attempt to connect to the NVM3 endpoint
            let ep_id = cpc::cpc_endpoint_id::Service(
                cpc::sl_cpc_service_endpoint_id_t_enum::SL_CPC_ENDPOINT_NVM3,
            );
            let cpc_endpoint = match cpc_handle.open_endpoint(ep_id, CPC_ENDPOINT_TX_WINDOW) {
                Ok(cpc_endpoint) => cpc_endpoint,
                Err(err) => match err {
                    libcpc::Error::Errno(_) => return Err(err.into()),
                    libcpc::Error::NulError(_) => return Err(err.into()),
                    libcpc::Error::InvalidEndpointStateType(_) => return Err(err.into()),
                    libcpc::Error::InvalidEndpointEventType(_) => return Err(err.into()),
                },
            };
            log::debug!("Connected to the NVM3 endpoint");

            // Get the maximum write fragment size
            let cpc_max_write_size = cpc_endpoint.get_max_write_size()? as u16;
            let nvm3_write_overhead = protocol::CmdWriteData::get_overhead();
            self.maximum_write_fragment_size = Some(cpc_max_write_size - nvm3_write_overhead);
            log::debug!(
                "Maximum fragment size is {} bytes",
                self.maximum_write_fragment_size.unwrap_or(0)
            );

            // Configure the timeout on the endpoint
            let timeout = cpc::cpc_timeval_t {
                seconds: CPC_NVM3_READ_TIMEOUT_S,
                microseconds: 0,
            };
            cpc_endpoint.set_read_timeout(timeout)?;

            // Configuration is completed, we can assign the endpoint to the instance
            self.cpc_endpoint = Some(cpc_endpoint);

            // Get the version of the NVM3 protocol on the secondary
            let get_version_command = GetVersion::new(self.unique_id, &mut self.transaction_id);

            self.write(&get_version_command.serialize()?)?;
            log::debug!("Queried the NVM3 protocol version from the secondary");

            let secondary_version = self.get_response(&get_version_command)?;

            log::info!(
                "[CPC Secondary NVM3 API v{}.{}.{}]",
                secondary_version.major_version,
                secondary_version.minor_version,
                secondary_version.patch_version
            );

            // Make sure the major version matches
            if secondary_version.major_version != CPC_NVM3_MAJOR_VERSION {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_INVALID_VERSION,
                    "Major versions do not match".to_string(),
                ));
            }

            // Get the maximum write size
            log::debug!("Fetching maximum write size");
            let get_maximum_write_command = PropValueGet::new(
                self.unique_id,
                &mut self.transaction_id,
                protocol::PropertyType::MaxWriteSize,
            );

            let bytestream = get_maximum_write_command.serialize()?;
            self.write(&bytestream)?;

            let response = self.get_response(&get_maximum_write_command)?;
            match response {
                PropValueGetResponse::Value(property_value) => match property_value {
                    PropertyValue::MaxWriteSize(property_value) => {
                        log::debug!("Maximum write size is {} bytes", property_value);
                        self.maximum_write_size = Some(property_value)
                    }
                    _ => {
                        return Err(CpcNvm3Error::ErrorCodeWithContext(
                            CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                            format!("Unexpected property value {}", property_value),
                        ));
                    }
                },
                PropValueGetResponse::StatusCode(err) => {
                    return Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        err.to_string(),
                    ));
                }
            };
            log::info!("Successfuly opened NVM3 instance");
            Ok(())
        };

        match result() {
            Ok(_) => Ok(()),
            Err(err) => {
                #[cfg(not(test))]
                if let Some(cpc_handle) = &mut self.cpc_handle {
                    let err =
                        unsafe { cpc_deinit(&mut cpc_handle.cpc as *mut libcpc::cpc_handle_t) };
                    if err != 0 {
                        return Err(CpcNvm3Error::ErrorCodeWithContext(
                            CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                            format!("Failed to deinit libcpc errno {}", err),
                        ));
                    }
                }
                self.cpc_endpoint = None;
                self.cpc_handle = None;
                self.maximum_write_fragment_size = None;
                self.maximum_write_size = None;
                Err(err)
            }
        }
    }

    fn handle_libcpc_error(&mut self, err: libcpc::Error) -> CpcNvm3Error {
        match err {
            libcpc::Error::Errno(err) => match err.kind() {
                std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::Interrupted => {
                    log::debug!("libcpc errno {} occured, attempting to reconnect", err);
                    if let Err(err) = self.reconnect() {
                        return err;
                    }
                    return CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_TRY_AGAIN,
                        "reconnected to libcpc try again".to_string(),
                    );
                }
                std::io::ErrorKind::WouldBlock => {
                    return CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_TRY_AGAIN,
                        "CPC communication timed out, try again.".to_string(),
                    );
                }
                _ => {
                    return CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                        format!("libcpc encountered an unexpected error {:?}", err),
                    );
                }
            },
            libcpc::Error::NulError(_) => {
                return CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                    format!("libcpc returned an unexpected error {:?}", err),
                )
            }
            libcpc::Error::InvalidEndpointStateType(_) => {
                return CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                    format!("libcpc returned an unexpected error {:?}", err),
                )
            }
            libcpc::Error::InvalidEndpointEventType(_) => {
                return CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                    format!("libcpc returned an unexpected error {:?}", err),
                )
            }
        }
    }

    fn write(&mut self, data: &Vec<u8>) -> Result<(), CpcNvm3Error> {
        // Check if the endpoint was previously disconnected
        if self.cpc_endpoint.is_none() {
            if self.cpc_handle.is_none() {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
                    format!(
                        "CPC Write failed. The CPC is not initialized. Call cpc_nvm3_open first."
                    ),
                ));
            }
            self.reconnect()?;
        }

        match &self.cpc_endpoint {
            Some(cpc_endpoint) => {
                let write_flags =
                    [cpc::cpc_endpoint_write_flags_t_enum::CPC_ENDPOINT_WRITE_FLAG_NONE];
                if let Err(err) = cpc_endpoint.write(data, &write_flags) {
                    return Err(self.handle_libcpc_error(err));
                }
                log::debug!("Wrote {:?} ", data);
            }
            None => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
                    format!("The CPC endpoint is not initialized. Call cpc_nvm3_open first."),
                ))
            }
        }
        Ok(())
    }

    fn read(&mut self) -> Result<Vec<u8>, CpcNvm3Error> {
        // Check if the endpoint was previously disconnected
        if self.cpc_endpoint.is_none() {
            if self.cpc_handle.is_none() {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
                    format!(
                        "CPC Write failed. The CPC is not initialized. Call cpc_nvm3_open first."
                    ),
                ));
            }
            self.reconnect()?;
        }

        match &self.cpc_endpoint {
            Some(cpc_endpoint) => {
                let read_flags = [cpc::cpc_endpoint_read_flags_t_enum::CPC_ENDPOINT_READ_FLAG_NONE];
                let data = match cpc_endpoint.read(&read_flags) {
                    Ok(data) => data,
                    Err(err) => return Err(self.handle_libcpc_error(err)),
                };

                log::debug!("Read {:?} ", data);
                Ok(data)
            }
            None => Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
                format!("The CPC endpoint is not initialized. Call cpc_nvm3_open first."),
            )),
        }
    }

    pub fn close(&mut self) -> Result<(), CpcNvm3Error> {
        match &mut self.cpc_endpoint {
            Some(cpc_endpoint) => {
                cpc_endpoint.close()?;
                #[cfg(not(test))]
                if let Some(cpc_handle) = &mut self.cpc_handle {
                    let err =
                        unsafe { cpc_deinit(&mut cpc_handle.cpc as *mut libcpc::cpc_handle_t) };
                    if err != 0 {
                        return Err(CpcNvm3Error::ErrorCodeWithContext(
                            CpcNvm3ErrorCodes::CPC_NVM3_CPC_ENDPOINT_ERROR,
                            format!("Failed to deinit libcpc errno {}", err),
                        ));
                    }
                }
            }
            None => {
                Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
                    format!("The CPC endpoint is not initialized. Call cpc_nvm3_open first."),
                ))
            }?,
        }
        self.cpc_endpoint = None;
        self.cpc_handle = None;
        Ok(())
    }

    pub fn get_maximum_write_size(&mut self) -> Result<u16, CpcNvm3Error> {
        match self.maximum_write_size {
            Some(maximum_write_size) => Ok(maximum_write_size),
            None => {
                Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
                        format!(
                            "Could not get maximum write size since the CPC NVM3 instance is not opened"
                        ),
                    ))
            }?,
        }
    }

    pub fn get_maximum_write_fragment_size(&mut self) -> Result<u16, CpcNvm3Error> {
        match self.maximum_write_fragment_size {
            Some(maximum_write_fragment_size) => Ok(maximum_write_fragment_size),
            None => {
                Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
                        format!(
                            "Could not get maximum write size since the CPC NVM3 instance is not opened"
                        ),
                    ))
            }?,
        }
    }

    fn parse_response<C: Command>(
        &mut self,
        command: &C,
        input: &[u8],
    ) -> RxParseOutcome<C::Response, CpcNvm3Error> {
        match command.parse_response(input) {
            Ok(response) => RxParseOutcome::Parsed(response),
            Err(err) => match err {
                ProtocolError::InvalidCommandId => {
                    log::debug!("Dropping response with invalid command ID");
                    RxParseOutcome::Retry
                }
                ProtocolError::InvalidTransactionId(expected, actual) => {
                    log::debug!(
                        "Dropping response with invalid transaction ID {}. Expected {}",
                        actual,
                        expected
                    );
                    RxParseOutcome::Retry
                }
                ProtocolError::InvalidUniqueId(expected, actual) => {
                    log::debug!(
                        "Dropping response with invalid unique ID {}. Expected {}",
                        actual,
                        expected
                    );
                    RxParseOutcome::Retry
                }
                _ => RxParseOutcome::Error(err.into()),
            },
        }
    }

    pub fn get_response<C: Command>(&mut self, command: &C) -> Result<C::Response, CpcNvm3Error> {
        loop {
            let rx_packet = self.read()?;
            match self.parse_response(command, &rx_packet) {
                RxParseOutcome::Parsed(response) => return Ok(response),
                RxParseOutcome::Retry => continue,
                RxParseOutcome::Error(err) => return Err(err),
            }
        }
    }
}

#[allow(non_camel_case_types)] // This will be used in a generated a C header file
pub type cpc_nvm3_handle_t = u32;
#[allow(non_camel_case_types)] // This will be used in a generated a C header file
pub type cpc_nvm3_object_key_t = u32;

fn find_next_available_handle() -> Result<cpc_nvm3_handle_t, CpcNvm3Error> {
    match CPC_NVM_LIB_INSTANCE_KEY.lock() {
        Ok(mut id) => {
            if *id == u32::MAX {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    format!("Instance key could not be incremented. Limit reached."),
                ));
            }
            *id += 1;
            Ok(*id)
        }
        Err(err) => Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
            format!("Failed to lock NVM3 instance map. Error: {}", err),
        )),
    }
}

fn get_instance(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
) -> Result<Arc<Mutex<CpcNvm3Instance>>, CpcNvm3Error> {
    let instances = match CPC_NVM3_LIB_INSTANCES.lock() {
        Ok(guard) => guard,
        Err(err) => {
            Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            ))
        }?,
    };

    let instance_mutex = match instances.get(&cpc_nvm3_handle) {
        Some(instance) => instance,
        None => {
            Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_NOT_INITIALIZED,
                format!("Could not find the provided instance"),
            ))
        }?,
    };

    Ok(Arc::clone(instance_mutex))
}

pub struct FileLogger {
    level: log::LevelFilter,
    prefix: String,
    file: Mutex<File>,
}

impl FileLogger {
    pub fn new(level: log::LevelFilter, prefix: String, file: File) -> Self {
        FileLogger {
            level,
            prefix,
            file: Mutex::new(file),
        }
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut file_guard = self.file.lock().unwrap();

            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            write!(
                file_guard,
                "{} {} - {}: {}\n",
                timestamp,
                self.prefix,
                record.level(),
                record.args()
            )
            .unwrap();

            file_guard.flush().unwrap();
        }
    }

    fn flush(&self) {
        let mut file_guard = self.file.lock().unwrap();
        file_guard.flush().unwrap();
    }
}

pub fn init_logger(
    prefix: Option<&str>,
    level: CpcNvm3LogLevel,
    file_path: Option<&str>,
    append: bool,
) -> Result<(), CpcNvm3Error> {
    let mut logger_initialized = LOGGER_INITIALIZED.lock().map_err(|_| {
        CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
            "Failed to lock logger initialization status".to_string(),
        )
    })?;

    if !*logger_initialized {
        let log_file = if let Some(path) = file_path {
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(append) // This will set the file to append mode.
                .open(path) // Open or create the file at the provided path.
                .map_err(|e| {
                    CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        format!("Failed to open or create log file: {:?}", e),
                    )
                })?
        } else {
            // Fall back to STDOUT if no file path is provided.
            unsafe { File::from_raw_fd(STDOUT_FILENO) }
        };
        log::set_boxed_logger(Box::new(FileLogger::new(
            level.into(),
            prefix.unwrap_or("").to_string(),
            log_file,
        )))
        .map_err(|_| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                "Failed to set logger".to_string(),
            )
        })?;

        log::set_max_level(LevelFilter::from(level));
        *logger_initialized = true;
    }
    Ok(())
}

pub fn init() -> Result<cpc_nvm3_handle_t, CpcNvm3Error> {
    let handle = find_next_available_handle()?;
    let mut cpc_nvm3_instance = CpcNvm3Instance::new();
    #[cfg(not(test))]
    {
        cpc_nvm3_instance.unique_id = std::process::id();
    }

    // Push key/value to the instance map
    let mut map = match CPC_NVM3_LIB_INSTANCES.lock() {
        Ok(m) => m,
        Err(err) => {
            Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("Failed to NVM3 instance map. Err{}", err),
            ))
        }?,
    };
    map.insert(handle, Arc::new(Mutex::new(cpc_nvm3_instance)));

    log::debug!("cpc_nvm3_init was successful, assigned handle {}", handle);

    Ok(handle)
}

pub fn open(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpcd_instance_name: &str,
    enable_cpc_traces: bool,
) -> Result<(), CpcNvm3Error> {
    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut cpc_nvm3_instance = instance_arc_mutex.lock().map_err(|err| {
        CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, format!("{}", err))
    })?;

    cpc_nvm3_instance.open(cpcd_instance_name, enable_cpc_traces)?;

    log::debug!(
        "cpc_nvm3_open was successful, on handle {}",
        cpc_nvm3_handle
    );

    Ok(())
}

pub fn write_data(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpc_nvm3_object_key: cpc_nvm3_object_key_t,
    data: &[u8],
) -> Result<(), CpcNvm3Error> {
    log::debug!("Writing to NVM3 instance");

    let mut last_fragment = false;
    let mut offset = 0;
    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;
    let fragment_size = instance.get_maximum_write_fragment_size()? as usize;

    if data.len() as u16 > instance.get_maximum_write_size()? {
        return Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_INVALID_ARG,
            format!(
                "Requested a write ({}) that is larger than the maximum write size ({})",
                data.len(),
                instance.get_maximum_write_size()?
            ),
        ));
    }

    while !last_fragment {
        if data.len() - offset <= fragment_size {
            last_fragment = true;
        }

        log::debug!("Writing at offset {}", offset);

        let data_fragment = &data[offset..(offset + fragment_size).min(data.len())];
        let mut write_data_command = CmdWriteData::new(
            instance.unique_id,
            &mut instance.transaction_id,
            cpc_nvm3_object_key,
            offset as u16,
            last_fragment as u8,
            data_fragment.to_vec(),
        );
        let write_data = write_data_command.serialize()?;
        instance.write(&write_data)?;
        let response = instance.get_response(&write_data_command)?;

        match response {
            StatusCode::SlStatus(sl_status) => match sl_status {
                SlStatus::Ok => log::debug!("Received write complete acknowledgement"),
                SlStatus::Fail => {
                    return Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        "Writing to NVM3 instance failed".to_string(),
                    ))
                }
                SlStatus::Busy => {
                    return Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_TRY_AGAIN,
                        "NVM3 is busy with another write operation, try again".to_string(),
                    ))
                }
                SlStatus::Unknown => {
                    return Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        format!("Received an unexpected sl_status code {}", sl_status),
                    ))
                }
            },
            StatusCode::ECode(ecode) => match ecode {
                ECode::KeyInvalid => {
                    return Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_INVALID_OBJECT_KEY,
                        format!("{}", ecode.to_string()),
                    ))
                }
                _ => {
                    return Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                        format!("{}", ecode.to_string()),
                    ))
                }
            },
            StatusCode::Unknown => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                    format!("Unknown response type received"),
                ))
            }
        }
        offset += fragment_size;
    }
    Ok(())
}

pub fn deinit(cpc_nvm3_handle: cpc_nvm3_handle_t) -> Result<(), CpcNvm3Error> {
    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance = instance_arc_mutex.lock().map_err(|err| {
        CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, format!("{}", err))
    })?;

    log::debug!("Deinit NVM3 instance");
    // About to de-init the instance, make sure the cpc endpoint is also closed.
    if instance.cpc_endpoint.is_some() || instance.cpc_handle.is_some() {
        return Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_NOT_CLOSED,
            format!(
                "Failed to de-init NVM3 instance. It is still opened. Call cpc_nvm3_close first."
            ),
        ));
    };

    instance.transaction_id = 0;
    instance.maximum_write_fragment_size = None;
    instance.maximum_write_size = None;

    match CPC_NVM3_LIB_INSTANCES.lock() {
        Ok(mut map) => {
            map.remove(&cpc_nvm3_handle);
            Ok(())
        }
        Err(err) => Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
            format!("Failed to lock NVM3 instance map. Err{}", err),
        )),
    }
}

pub fn close(cpc_nvm3_handle: cpc_nvm3_handle_t) -> Result<(), CpcNvm3Error> {
    // Close the CPC endpoint
    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance = instance_arc_mutex.lock().map_err(|err| {
        CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, format!("{}", err))
    })?;
    instance.close()?;
    instance.cpc_endpoint = None;
    Ok(())
}

pub fn get_object_count(cpc_nvm3_handle: cpc_nvm3_handle_t) -> Result<u16, CpcNvm3Error> {
    log::debug!("Getting objects count from NVM3 instance");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;

    let get_object_count_command =
        CmdGetObjectCount::new(instance.unique_id, &mut instance.transaction_id);
    let write_data = get_object_count_command.serialize()?;
    instance.write(&write_data)?;

    let response = instance.get_response(&get_object_count_command)?;
    match response {
        CmdGetObjectCountResponse::StatusCode(status_code) => match status_code {
            StatusCode::SlStatus(sl_status) => match sl_status {
                SlStatus::Ok | SlStatus::Fail | SlStatus::Busy | SlStatus::Unknown => {
                    Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        format!("Received an unexpected sl_status code {}", status_code),
                    ))
                }
            },

            StatusCode::ECode(e_code) => match e_code {
                ECode::KeyNotFound => Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_INVALID_OBJECT_KEY,
                    format!("{}", status_code),
                )),
                _ => Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    format!("Get object count failed with status code: {}", status_code),
                )),
            },

            StatusCode::Unknown => Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                format!("Unknown response type received"),
            )),
        },
        CmdGetObjectCountResponse::ObjectCount { object_count } => Ok(object_count),
    }
}

pub fn extract_object_keys(input: &[u8]) -> nom::IResult<&[u8], Vec<cpc_nvm3_object_key_t>> {
    many0(le_u32)(input)
}

pub fn list_objects(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpc_nvm3_object_keys_ptr: &mut [cpc_nvm3_object_key_t],
    object_count: &mut u16,
) -> Result<(), CpcNvm3Error> {
    log::debug!("Listing objects from NVM3 instance");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance = instance_arc_mutex.lock().map_err(|err| {
        CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, format!("{}", err))
    })?;

    log::debug!(
        "Sending object enumeration request with a limit of {} objects",
        cpc_nvm3_object_keys_ptr.len()
    );
    let mut enumerate_objects_command = CmdEnumerateObjects::new(
        instance.unique_id,
        &mut instance.transaction_id,
        cpc_nvm3_object_keys_ptr.len() as u16,
    );

    instance.write(&enumerate_objects_command.serialize()?)?;

    let mut continue_reading = true;
    let mut data = vec![];

    while continue_reading {
        let response = instance.get_response(&enumerate_objects_command)?;

        // Response can either be an error (StatusIs) or a success with the data
        let received_data = match response {
            CmdEnumerateObjectsResponse::Data(segment, last_fragment) => {
                continue_reading = !last_fragment;
                if !last_fragment {
                    log::debug!(
                          "Received {} bytes. Another fragment is available, fetching object list again",
                          segment.len()
                      );
                }
                Ok(segment)
            }
            CmdEnumerateObjectsResponse::StatusCode(status_code) => match status_code {
                StatusCode::SlStatus(sl_status) => match sl_status {
                    SlStatus::Ok | SlStatus::Fail | SlStatus::Unknown => {
                        Err(CpcNvm3Error::ErrorCodeWithContext(
                            CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                            format!("Received an unexpected sl_status code {}", status_code),
                        ))
                    }
                    SlStatus::Busy => {
                        return Err(CpcNvm3Error::ErrorCodeWithContext(
                            CpcNvm3ErrorCodes::CPC_NVM3_TRY_AGAIN,
                            "NVM3 is busy with another operation, try again".to_string(),
                        ))
                    }
                },

                StatusCode::ECode(e_code) => match e_code {
                    _ => Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        format!("list_objects failed with status code: {}", status_code),
                    )),
                },

                StatusCode::Unknown => Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                    format!("Unknown response type received"),
                )),
            },
        }?;
        data.extend(received_data);
    }
    if data.len() > cpc_nvm3_object_keys_ptr.len() * CPC_NVM3_OBJECT_KEY_SIZE {
        return Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_BUFFER_TOO_SMALL,
            "list_objects failed, provided buffer is too small".to_string(),
        ));
    };

    let num_objects = data.len() / CPC_NVM3_OBJECT_KEY_SIZE;
    if num_objects * CPC_NVM3_OBJECT_KEY_SIZE != data.len() {
        return Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
            "The data length is not a multiple of the object size".to_string(),
        ));
    }

    match extract_object_keys(&data) {
        Ok((remaining, keys)) => {
            if keys.len() != num_objects || remaining.len() != 0 {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    "Number of deserialized keys doesn't match the expected number.".to_string(),
                ));
            }
            cpc_nvm3_object_keys_ptr.copy_from_slice(&keys);
        }
        Err(e) => {
            return Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("Failed to deserialize keys: {:?}", e),
            ));
        }
    }

    *object_count = (data.len() / CPC_NVM3_OBJECT_KEY_SIZE) as u16;
    Ok(())
}

pub fn read_data(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpc_nvm3_object_key: cpc_nvm3_object_key_t,
    buffer: &mut [u8],
    data_size: &mut u16,
) -> Result<(), CpcNvm3Error> {
    log::debug!("Reading data from NVM3 instance");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance = instance_arc_mutex.lock().map_err(|err| {
        CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, format!("{}", err))
    })?;

    let mut read_command = CmdReadData::new(
        instance.unique_id,
        &mut instance.transaction_id,
        cpc_nvm3_object_key,
        buffer.len() as u16,
    );

    instance.write(&read_command.serialize()?)?;

    let mut continue_reading = true;
    let mut data = vec![];

    while continue_reading {
        let response = instance.get_response(&read_command)?;

        // Response can either be an error (StatusIs) or a success with the data
        let received_data = match response {
            CmdReadDataResponse::Data(segment, last_fragment) => {
                continue_reading = !last_fragment;
                if !last_fragment {
                    log::debug!(
                        "Received {} bytes. Another fragment is available, reading again",
                        segment.len()
                    );
                }
                Ok(segment)
            }
            CmdReadDataResponse::StatusCode(status_code) => match status_code {
                StatusCode::SlStatus(sl_status) => match sl_status {
                    SlStatus::Ok | SlStatus::Fail | SlStatus::Unknown => {
                        Err(CpcNvm3Error::ErrorCodeWithContext(
                            CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                            format!("Received an unexpected sl_status code {}", status_code),
                        ))
                    }
                    SlStatus::Busy => {
                        return Err(CpcNvm3Error::ErrorCodeWithContext(
                            CpcNvm3ErrorCodes::CPC_NVM3_TRY_AGAIN,
                            "NVM3 is busy with another operation, try again".to_string(),
                        ))
                    }
                },

                StatusCode::ECode(e_code) => match e_code {
                    ECode::KeyNotFound => Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_INVALID_OBJECT_KEY,
                        format!("{}", status_code),
                    )),
                    ECode::ReadDataSize => Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_BUFFER_TOO_SMALL,
                        format!("{}", status_code),
                    )),
                    ECode::SizeTooSmall => Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_BUFFER_TOO_SMALL,
                        format!("{}", status_code),
                    )),
                    _ => Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        format!("Read failed with status code: {}", status_code),
                    )),
                },

                StatusCode::Unknown => Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                    format!("Unknown response type received"),
                )),
            },
        }?;
        data.extend(received_data);
    }
    if data.len() > buffer.len() {
        return Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_BUFFER_TOO_SMALL,
            "Read failed, provided buffer is too small".to_string(),
        ));
    };

    buffer[..data.len()].copy_from_slice(&data);
    *data_size = data.len() as u16;

    Ok(())
}

pub fn write_counter(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpc_nvm3_object_key: cpc_nvm3_object_key_t,
    value: u32,
) -> Result<(), CpcNvm3Error> {
    log::debug!("Writing to NVM3 counter");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;

    let write_counter_command = CmdWriteCounter::new(
        instance.unique_id,
        &mut instance.transaction_id,
        cpc_nvm3_object_key,
        value,
    );
    let write_data = write_counter_command.serialize()?;
    instance.write(&write_data)?;
    let response = instance.get_response(&write_counter_command)?;

    match response {
        StatusCode::SlStatus(sl_status) => match sl_status {
            SlStatus::Ok => log::debug!("Received write counter acknowledgement"),
            SlStatus::Fail => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    "Writing counter to NVM3 instance failed".to_string(),
                ))
            }
            SlStatus::Unknown | SlStatus::Busy => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    format!("Received an unexpected sl_status code {}", sl_status),
                ))
            }
        },
        StatusCode::ECode(ecode) => match ecode {
            ECode::KeyInvalid => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_INVALID_OBJECT_KEY,
                    format!("{}", ecode.to_string()),
                ))
            }
            _ => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                    format!("{}", ecode.to_string()),
                ))
            }
        },
        StatusCode::Unknown => {
            return Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                format!("Unknown response type received"),
            ))
        }
    }
    Ok(())
}

fn process_read_counter_response(response: CmdCounterValueResponse) -> Result<u32, CpcNvm3Error> {
    // Response can either be an error (StatusIs) or a success with the data
    match response {
        CmdCounterValueResponse::Data(data) => Ok(data),
        CmdCounterValueResponse::StatusCode(status_code) => match status_code {
            StatusCode::SlStatus(sl_status) => match sl_status {
                SlStatus::Ok | SlStatus::Fail | SlStatus::Unknown | SlStatus::Busy => {
                    Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        format!("Received an unexpected sl_status code {}", status_code),
                    ))
                }
            },

            StatusCode::ECode(e_code) => match e_code {
                ECode::KeyNotFound => Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_INVALID_OBJECT_KEY,
                    format!("{}", status_code),
                )),
                _ => Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    format!("Read failed with status code: {}", status_code),
                )),
            },

            StatusCode::Unknown => Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                format!("Unknown response type received"),
            )),
        },
    }
}

pub fn read_counter(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpc_nvm3_object_key: cpc_nvm3_object_key_t,
) -> Result<u32, CpcNvm3Error> {
    log::debug!("Reading counter from NVM3 instance");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance = instance_arc_mutex.lock().map_err(|err| {
        CpcNvm3Error::ErrorCodeWithContext(CpcNvm3ErrorCodes::CPC_NVM3_FAILURE, format!("{}", err))
    })?;

    let read_counter_command = CmdReadCounter::new(
        instance.unique_id,
        &mut instance.transaction_id,
        cpc_nvm3_object_key,
    );
    instance.write(&read_counter_command.serialize()?)?;
    let response = instance.get_response(&read_counter_command)?;

    Ok(process_read_counter_response(response)?)
}

pub fn increment_counter(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpc_nvm3_object_key: cpc_nvm3_object_key_t,
) -> Result<u32, CpcNvm3Error> {
    log::debug!("Incrementing NVM3 counter");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;

    let increment_counter_command = CmdIncrementCounter::new(
        instance.unique_id,
        &mut instance.transaction_id,
        cpc_nvm3_object_key,
    );
    let write_data = increment_counter_command.serialize()?;
    instance.write(&write_data)?;
    let response = instance.get_response(&increment_counter_command)?;
    Ok(process_read_counter_response(response)?)
}

pub fn get_maximum_write_size(cpc_nvm3_handle: cpc_nvm3_handle_t) -> Result<u16, CpcNvm3Error> {
    log::debug!("Fetching NVM3 maximum write size");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;

    Ok(instance.get_maximum_write_size()?)
}

pub fn get_object_info(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpc_nvm3_object_key: cpc_nvm3_object_key_t,
) -> Result<(u16, CpcNvm3ObjectType), CpcNvm3Error> {
    log::debug!("Fetching NVM3 object info");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;

    let get_object_info_command = CmdGetObjectInfo::new(
        instance.unique_id,
        &mut instance.transaction_id,
        cpc_nvm3_object_key,
    );
    let write_data = get_object_info_command.serialize()?;
    instance.write(&write_data)?;

    let response = instance.get_response(&get_object_info_command)?;
    match response {
        CmdGetObjectInfoResponse::StatusCode(status_code) => match status_code {
            StatusCode::SlStatus(sl_status) => match sl_status {
                SlStatus::Ok | SlStatus::Fail | SlStatus::Busy | SlStatus::Unknown => {
                    Err(CpcNvm3Error::ErrorCodeWithContext(
                        CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                        format!("Received an unexpected sl_status code {}", status_code),
                    ))
                }
            },

            StatusCode::ECode(e_code) => match e_code {
                ECode::KeyNotFound => Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_INVALID_OBJECT_KEY,
                    format!("{}", status_code),
                )),
                _ => Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    format!("Read failed with status code: {}", status_code),
                )),
            },

            StatusCode::Unknown => Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                format!("Unknown response type received"),
            )),
        },
        CmdGetObjectInfoResponse::ObjectInfo {
            object_type,
            object_size,
        } => Ok((object_size, object_type)),
    }
}

pub fn delete_object(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    cpc_nvm3_object_key: cpc_nvm3_object_key_t,
) -> Result<(), CpcNvm3Error> {
    log::debug!("Deleting NVM3 object #{:?}", cpc_nvm3_object_key);

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let mut instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;

    let delete_object_command = CmdDeleteObject::new(
        instance.unique_id,
        &mut instance.transaction_id,
        cpc_nvm3_object_key,
    );
    let write_data = delete_object_command.serialize()?;
    instance.write(&write_data)?;

    let parsed_response = instance.get_response(&delete_object_command)?;
    match parsed_response {
        StatusCode::SlStatus(sl_status) => match sl_status {
            SlStatus::Ok => log::debug!("Received delete object acknowledgement"),
            SlStatus::Fail => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    "Deletion of NVM3 object failed".to_string(),
                ))
            }
            SlStatus::Unknown | SlStatus::Busy => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                    format!("Received an unexpected sl_status code {}", sl_status),
                ))
            }
        },
        StatusCode::ECode(ecode) => match ecode {
            ECode::KeyInvalid | ECode::KeyNotFound => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_INVALID_OBJECT_KEY,
                    format!("{}", ecode.to_string()),
                ))
            }
            _ => {
                return Err(CpcNvm3Error::ErrorCodeWithContext(
                    CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                    format!("{}", ecode.to_string()),
                ))
            }
        },
        StatusCode::Unknown => {
            return Err(CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_UNKNOWN_ERROR,
                format!("Unknown response type received"),
            ))
        }
    }

    Ok(())
}

pub fn set_timeout(
    cpc_nvm3_handle: cpc_nvm3_handle_t,
    seconds: i32,
    microseconds: i32,
) -> Result<(), CpcNvm3Error> {
    log::debug!(
        "Configuring blocking timeout to {} seconds and {} microseconds",
        seconds,
        microseconds
    );

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;

    let set_timeout = libcpc::cpc_timeval_t {
        seconds: seconds,
        microseconds: microseconds,
    };

    match &instance.cpc_endpoint {
        Some(endpoint) => {
            endpoint.set_read_timeout(set_timeout)?;
            Ok(())
        }
        None => Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
            format!("CPC Write failed. The CPC is not initialized. Call cpc_nvm3_open first."),
        )),
    }
}

pub fn get_timeout(cpc_nvm3_handle: cpc_nvm3_handle_t) -> Result<(i32, i32), CpcNvm3Error> {
    log::debug!("Obtaining configured timeout");

    let instance_arc_mutex = get_instance(cpc_nvm3_handle)?;
    let instance: std::sync::MutexGuard<CpcNvm3Instance> =
        instance_arc_mutex.lock().map_err(|err| {
            CpcNvm3Error::ErrorCodeWithContext(
                CpcNvm3ErrorCodes::CPC_NVM3_FAILURE,
                format!("{}", err),
            )
        })?;

    match &instance.cpc_endpoint {
        Some(endpoint) => {
            let timeout = endpoint.get_read_timeout()?;
            log::debug!(
                "Configured timeout is {} seconds and {} microseconds",
                timeout.seconds,
                timeout.microseconds
            );
            Ok((timeout.seconds, timeout.microseconds))
        }
        None => Err(CpcNvm3Error::ErrorCodeWithContext(
            CpcNvm3ErrorCodes::CPC_NVM3_NOT_OPEN,
            format!("CPC Write failed. The CPC is not initialized. Call cpc_nvm3_open first."),
        )),
    }
}
