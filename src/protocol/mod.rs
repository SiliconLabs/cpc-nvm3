/*******************************************************************************
* @file
 * @brief Co-Processor Communication Protocol(CPC) NVM3 - Protocol Module
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
mod tests;

use crate::CpcNvm3ObjectType;
use nom::error::{Error, ErrorKind};
use nom::Err;
use num_enum::TryFromPrimitive;
use std::fmt;
use std::num::NonZeroUsize;
use thiserror::Error;

#[derive(TryFromPrimitive, PartialEq, Copy, Clone, Debug)]
#[repr(u32)]
pub enum ECode {
    Ok = 0,
    AlignmentInvalid = 0xF000E001,
    SizeTooSmall = 0xF000E002,
    NoValidPages = 0xF000E003,
    PageSizeNotSupported = 0xF000E004,
    ObjectSizeNotSupported = 0xF000E005,
    StorageFull = 0xF000E006,
    NotOpened = 0xF000E007,
    OpenedWithOtherParameters = 0xF000E008,
    Parameter = 0xF000E009,
    KeyInvalid = 0xF000E00A,
    KeyNotFound = 0xF000E00B,
    ObjectIsNotData = 0xF000E00C,
    ObjectIsNotACounter = 0xF000E00D,
    EraseFailed = 0xF000E00E,
    WriteDataSize = 0xF000E00F,
    WriteFailed = 0xF000E010,
    ReadDataSize = 0xF000E011,
    ReadFailed = 0xF000E012,
    InitWithFullNvm = 0xF000E013,
    ResizeParameter = 0xF000E014,
    ResizeNotEnoughSpace = 0xF000E015,
    EraseCountError = 0xF000E016,
    AddressRange = 0xF000E017,
    NvmAccess = 0xF000E019,
    Unknown = u32::MAX,
}

#[derive(num_enum::TryFromPrimitive, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum StatusIsResponseType {
    ResponseTypeSlStatus = 0,
    ResponseTypeEcode = 1,
    ResponseTypeUnknown = u8::MAX,
}

#[repr(u32)]
pub enum StatusCode {
    SlStatus(SlStatus),
    ECode(ECode),
    Unknown,
}

#[derive(num_enum::TryFromPrimitive, PartialEq, Copy, Clone, Debug)]
#[repr(u32)]
pub enum SlStatus {
    Ok = 0,
    Fail = 1,
    Busy = 4,
    Unknown = u32::MAX,
}

impl From<u8> for CpcNvm3ObjectType {
    fn from(value: u8) -> Self {
        match value {
            0 => CpcNvm3ObjectType::CPC_NVM3_OBJECT_TYPE_DATA,
            1 => CpcNvm3ObjectType::CPC_NVM3_OBJECT_TYPE_COUNTER,
            _ => CpcNvm3ObjectType::CPC_NVM3_OBJECT_TYPE_UNKNOWN,
        }
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusCode::SlStatus(sl_status) => write!(f, "SlStatus: {}", sl_status),
            StatusCode::ECode(e_code) => write!(f, "ECode: {}", e_code),
            StatusCode::Unknown => write!(f, "Unknown status code"),
        }
    }
}

impl fmt::Display for SlStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlStatus::Ok => write!(f, "Success"),
            SlStatus::Fail => write!(f, "Failure"),
            SlStatus::Busy => write!(f, "Busy"),
            SlStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for ECode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ECode::Ok => write!(f, "Operation successful"),
            ECode::AlignmentInvalid => write!(f, "Invalid data alignment"),
            ECode::SizeTooSmall => write!(f, "Insufficient memory specified"),
            ECode::NoValidPages => write!(f, "No valid pages found during initialization"),
            ECode::PageSizeNotSupported => write!(f, "Unsupported page size"),
            ECode::ObjectSizeNotSupported => write!(f, "Unsupported object size"),
            ECode::StorageFull => write!(f, "Storage is full"),
            ECode::NotOpened => write!(f, "Module not successfully opened"),
            ECode::OpenedWithOtherParameters => {
                write!(f, "Module already opened with different parameters")
            }
            ECode::Parameter => write!(f, "Illegal parameter in operation"),
            ECode::KeyInvalid => write!(f, "Invalid key value"),
            ECode::KeyNotFound => write!(f, "Key not found"),
            ECode::ObjectIsNotData => write!(f, "Accessing data object as counter object"),
            ECode::ObjectIsNotACounter => write!(f, "Accessing counter object as data object"),
            ECode::EraseFailed => write!(f, "Failed to erase data"),
            ECode::WriteDataSize => write!(f, "Data object too large"),
            ECode::WriteFailed => write!(f, "Failed to write data"),
            ECode::ReadDataSize => write!(f, "Attempted to read data with incorrect length"),
            ECode::ReadFailed => write!(f, "Failed to read data"),
            ECode::InitWithFullNvm => write!(f, "Initialized with full memory"),
            ECode::ResizeParameter => write!(f, "Illegal resize parameter"),
            ECode::ResizeNotEnoughSpace => write!(f, "Not enough space to complete resize"),
            ECode::EraseCountError => write!(f, "Invalid erase counts"),
            ECode::AddressRange => write!(f, "Address and size out of range"),
            ECode::NvmAccess => write!(f, "Failed memory access"),
            ECode::Unknown => write!(f, "Unknown error"),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unknown protocol error")]
    UnknownProcotolError,
    #[error("Failed to deserialize")]
    DeserializationError(String),
    #[error("Failed to Serialize")]
    SerializationError(String),
    #[error("Received a response with an invalid transaction id: expected={0}, received={1}")]
    InvalidTransactionId(u8, u8),
    #[error("Bug: {0}")]
    Bug(String),
    #[error("Received a response with unexpected command id")]
    InvalidCommandId,
    #[error("Received a response with unexpected unique id: expected={0}, received={1}")]
    InvalidUniqueId(u32, u32),
    #[error("Received a response with invalid len: expected={0}, received={1}")]
    InvalidResponseLen(usize, u16),
}

#[derive(
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
    num_enum::TryFromPrimitive,
    PartialEq,
    Copy,
    Clone,
    Debug,
)]
#[repr(u8)]
enum HostCmd {
    CmdGetVersion = 0x00,
    CmdNoop = 0x03,
    CmdPropValueGet = 0x04,
    CmdWriteData = 0x06,
    CmdReadData = 0x08,
    CmdGetObjectInfo = 0x0A,
    CmdReadCounter = 0x0C,
    CmdWriteCounter = 0x0E,
    CmdIncrementCounter = 0x0F,
    CmdDeleteObject = 0x10,
    CmdEnumerateObjects = 0x11,
    CmdGetObjectCount = 0x13,
}

#[derive(
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
    num_enum::TryFromPrimitive,
    PartialEq,
    Copy,
    Clone,
    Debug,
)]
#[repr(u8)]
pub enum SecondaryCmd {
    CmdVersionIs = 0x01,
    CmdStatusIs = 0x02,
    CmdPropValueIs = 0x05,
    CmdReadDataIs = 0x09,
    CmdObjectInfoIs = 0x0B,
    CmdCounterIs = 0x0D,
    CmdEnumerateObjectsIs = 0x12,
    CmdObjectCountIs = 0x14,
    UnsupportedCmdIs = u8::MAX,
}

#[derive(num_enum::TryFromPrimitive, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum PropertyType {
    MaxObjectSize = 0x01,
    MaxWriteSize = 0x02,
    Unknown = 0xFF,
}

impl serde::Serialize for PropertyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PropertyValue::MaxObjectSize(_) => write!(f, "MaxObjectSize"),
            PropertyValue::MaxWriteSize(_) => write!(f, "MaxWriteSize"),
            PropertyValue::Unknown => write!(f, "Unknown"),
        }
    }
}
pub enum PropertyValue {
    MaxObjectSize(u16),
    MaxWriteSize(u16),
    Unknown,
}

#[derive(serde::Serialize, Copy)]
#[repr(C, packed)]
pub struct Header<T: Copy + Clone + std::fmt::Debug> {
    pub cmd: T,
    len: u16,
    unique_id: u32,
    transaction_id: TransactionId,
}

impl<T: Copy + Clone + std::fmt::Debug> Header<T> {
    fn new(cmd: T, len: u16, unique_id: u32, transaction_id: TransactionId) -> Self {
        Self {
            cmd,
            len,
            unique_id,
            transaction_id,
        }
    }

    pub fn validate(
        &self,
        expected_cmd: T,
        expected_len: usize,
        expected_unique_id: u32,
        expected_transaction_id: u8,
    ) -> Result<(), ProtocolError>
    where
        T: PartialEq,
    {
        let cmd = self.cmd; //reference to packed field is unaligned
        if cmd != expected_cmd {
            log::debug!(
                "Invalid command id, expected {:?}. Received {:?}",
                expected_cmd,
                cmd
            );
            return Err(ProtocolError::InvalidCommandId);
        }

        let len = self.len; //reference to packed field is unaligned
        if len != expected_len as u16 {
            log::error!(
                "Invalid response length, expected {:?}. Received {:?}",
                expected_len,
                len
            );
            return Err(ProtocolError::InvalidResponseLen(expected_len, len));
        }

        let unique_id = self.unique_id; //reference to packed field is unaligned
        if unique_id != expected_unique_id {
            log::debug!(
                "Invalid unique id, expected {:?}. Received {:?}",
                expected_unique_id,
                unique_id
            );
            return Err(ProtocolError::InvalidUniqueId(
                unique_id,
                expected_unique_id,
            ));
        }

        let transaction_id = self.transaction_id; //reference to packed field is unaligned
        if transaction_id.value != expected_transaction_id {
            log::debug!(
                "Invalid transaction id, expected {:?}. Received {:?}",
                expected_transaction_id,
                transaction_id.value
            );
            return Err(ProtocolError::InvalidTransactionId(
                expected_transaction_id,
                transaction_id.value,
            ));
        }

        Ok(())
    }
}

impl<T: Copy + std::fmt::Debug> Clone for Header<T> {
    fn clone(&self) -> Self {
        Self {
            cmd: self.cmd,
            len: self.len,
            unique_id: self.unique_id,
            transaction_id: self.transaction_id,
        }
    }
}

fn extract_and_validate_header(
    input: &[u8],
    expected_cmd: SecondaryCmd,
    expected_unique_id: u32,
    expected_transaction_id: u8,
) -> Result<(Header<SecondaryCmd>, &[u8]), ProtocolError> {
    let input_len = input.len();

    let (remaining, header) = deserialize_header(input)
        .map_err(|err| ProtocolError::DeserializationError(err.to_string()))?;

    let expected_len = input_len - std::mem::size_of::<Header<SecondaryCmd>>();

    header.validate(
        expected_cmd,
        expected_len,
        expected_unique_id,
        expected_transaction_id,
    )?;

    Ok((header, remaining))
}

fn parse_status_response(
    expected_transaction_id: u8,
    expected_unique_id: u32,
    input: &[u8],
) -> Result<StatusCode, ProtocolError> {
    let response = StatusIs::deserialize(input, expected_transaction_id, expected_unique_id)?;
    let status_code = response.status_code;
    log::debug!("Received status is {}", status_code);
    Ok(status_code)
}

pub trait Command {
    type Response;
    fn parse_response(&self, input: &[u8]) -> Result<Self::Response, ProtocolError>;
}

#[derive(serde::Serialize, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct TransactionId {
    value: u8,
}
impl TransactionId {
    fn new(transaction_id: &mut u8) -> Self {
        *transaction_id = transaction_id.wrapping_add(1);
        Self {
            value: *transaction_id,
        }
    }
}

pub trait Serializer: serde::Serialize {
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        log::debug!("Serializing");
        match bincode::serialize(&self) {
            Ok(bytestream) => Ok(bytestream),
            Err(err) => Err(ProtocolError::SerializationError(err.to_string())),
        }
    }
}

pub enum PropValueGetResponse {
    Value(PropertyValue),
    StatusCode(StatusCode),
}

#[derive(serde::Serialize)]
#[repr(C, packed)]
pub struct PropValueGet {
    header: Header<HostCmd>,
    property_type: PropertyType,
}
impl Command for PropValueGet {
    type Response = PropValueGetResponse;
    fn parse_response(&self, input: &[u8]) -> Result<PropValueGetResponse, ProtocolError> {
        // Check the very first byte to know which type of response we got
        let (_, cmd) = deserialize_cmd(input).map_err(|e| {
            ProtocolError::DeserializationError(format!("Failed to deserialize command: {:?}", e))
        })?;
        match cmd {
            SecondaryCmd::CmdStatusIs => {
                Ok(PropValueGetResponse::StatusCode(parse_status_response(
                    self.header.transaction_id.value,
                    self.header.unique_id,
                    input,
                )?))
            }
            SecondaryCmd::CmdPropValueIs => {
                log::debug!("Received property response");
                let response = PropValueIs::deserialize(
                    input,
                    self.header.transaction_id.value,
                    self.header.unique_id,
                )?;
                Ok(PropValueGetResponse::Value(response.property_value))
            }
            _ => {
                log::debug!("Invalid command id {:?}", cmd);
                Err(ProtocolError::InvalidCommandId)
            }
        }
    }
}

impl Serializer for PropValueGet {}
impl PropValueGet {
    pub fn new(unique_id: u32, transaction_id: &mut u8, property_type: PropertyType) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdPropValueGet,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            property_type,
        }
    }
}

#[derive(serde::Serialize)]
#[repr(C, packed)]
pub struct GetVersion {
    header: Header<HostCmd>,
}
impl Serializer for GetVersion {}
impl Command for GetVersion {
    type Response = VersionIs;
    fn parse_response(&self, input: &[u8]) -> Result<VersionIs, ProtocolError> {
        // Check the very first byte to know which type of response we got
        let (_, cmd) = deserialize_cmd(input).map_err(|e| {
            ProtocolError::DeserializationError(format!("Failed to deserialize command: {:?}", e))
        })?;
        match cmd {
            SecondaryCmd::CmdVersionIs => Ok(VersionIs::deserialize(
                input,
                self.header.transaction_id.value,
                self.header.unique_id,
            )?),
            _ => {
                log::debug!("Invalid command id {:?}", cmd);
                Err(ProtocolError::InvalidCommandId)
            }
        }
    }
}
impl GetVersion {
    pub fn new(unique_id: u32, transaction_id: &mut u8) -> Self {
        Self {
            header: Header::new(
                HostCmd::CmdGetVersion,
                0,
                unique_id,
                TransactionId::new(transaction_id),
            ),
        }
    }
}

#[repr(C, packed)]
pub struct PropValueIs {
    header: Header<SecondaryCmd>,
    property_type: PropertyType,
    property_value: PropertyValue,
}
impl PropValueIs {
    pub fn deserialize(
        input: &[u8],
        expected_transaction_id: u8,
        expected_unique_id: u32,
    ) -> Result<Self, ProtocolError> {
        let expected_cmd = SecondaryCmd::CmdPropValueIs;
        let (header, remaining) = extract_and_validate_header(
            input,
            expected_cmd,
            expected_unique_id,
            expected_transaction_id,
        )?;

        let result = || -> nom::IResult<&[u8], Self> {
            let (remaining, property_type) = deserialize_property_type(remaining)?;
            let (remaining, property_value) = deserialize_property_value(property_type, remaining)?;
            Ok((
                remaining,
                Self {
                    header,
                    property_type,
                    property_value,
                },
            ))
        };

        match result() {
            Ok(tuple) => Ok(tuple.1),
            Err(err) => Err(ProtocolError::DeserializationError(err.to_string())),
        }
    }
}

#[repr(C, packed)]
pub struct VersionIs {
    header: Header<SecondaryCmd>,
    pub major_version: u8,
    pub minor_version: u8,
    pub patch_version: u8,
}
impl VersionIs {
    pub fn deserialize(
        input: &[u8],
        expected_transaction_id: u8,
        expected_unique_id: u32,
    ) -> Result<Self, ProtocolError> {
        let expected_cmd = SecondaryCmd::CmdVersionIs;
        let (header, remaining) = extract_and_validate_header(
            input,
            expected_cmd,
            expected_unique_id,
            expected_transaction_id,
        )?;

        let result = || -> nom::IResult<&[u8], Self> {
            let (remaining, major_version) = nom::number::complete::u8(remaining)?;
            let (remaining, minor_version) = nom::number::complete::u8(remaining)?;
            let (remaining, patch_version) = nom::number::complete::u8(remaining)?;
            Ok((
                remaining,
                Self {
                    header,
                    major_version,
                    minor_version,
                    patch_version,
                },
            ))
        };

        match result() {
            Ok(tuple) => Ok(tuple.1),
            Err(err) => Err(ProtocolError::DeserializationError(err.to_string())),
        }
    }

    pub fn len(&self) -> u16 {
        std::mem::size_of::<VersionIs>() as u16
    }
}

#[repr(C, packed)]
pub struct CmdReadDataIsHeader {
    last_frag: bool,
}

#[repr(C, packed)]
pub struct CmdReadDataIs {
    header: Header<SecondaryCmd>,
    cmd_read_data_header: CmdReadDataIsHeader,
    data: Vec<u8>,
}
impl CmdReadDataIs {
    pub fn deserialize(
        input: &[u8],
        expected_transaction_id: u8,
        expected_unique_id: u32,
    ) -> Result<Self, ProtocolError> {
        let expected_cmd = SecondaryCmd::CmdReadDataIs;
        let (header, remaining) = extract_and_validate_header(
            input,
            expected_cmd,
            expected_unique_id,
            expected_transaction_id,
        )?;

        let result = || -> nom::IResult<&[u8], Self> {
            let (remaining, last_frag_u8) = nom::number::complete::u8(remaining)?;
            let last_frag = last_frag_u8 != 0;

            if remaining.len() == 0 {
                return Err(nom::Err::Incomplete(nom::Needed::Size(
                    NonZeroUsize::new(
                        std::mem::size_of::<CmdReadDataIsHeader>()
                            + std::mem::size_of::<Header<SecondaryCmd>>()
                            + 1,
                    )
                    .unwrap(),
                )));
            }

            let data = remaining.to_vec();

            Ok((
                remaining,
                Self {
                    header,
                    cmd_read_data_header: CmdReadDataIsHeader { last_frag },
                    data,
                },
            ))
        };

        match result() {
            Ok(tuple) => Ok(tuple.1),
            Err(err) => Err(ProtocolError::DeserializationError(err.to_string())),
        }
    }

    pub fn get_overhead() -> u16 {
        (std::mem::size_of::<CmdReadDataIsHeader>() + std::mem::size_of::<Header<SecondaryCmd>>())
            as u16
    }
}
pub enum CmdReadDataResponse {
    Data(Vec<u8>, bool),
    StatusCode(StatusCode),
}

#[repr(C, packed)]
#[derive(serde::Serialize)]
pub struct CmdReadData {
    header: Header<HostCmd>,
    object_key: u32,
    max_read_size: u16,
}
impl Command for CmdReadData {
    type Response = CmdReadDataResponse;
    fn parse_response(&self, input: &[u8]) -> Result<CmdReadDataResponse, ProtocolError> {
        // Check the very first byte to know which type of response we got
        let (_, cmd) = deserialize_cmd(input).map_err(|e| {
            ProtocolError::DeserializationError(format!("Failed to deserialize command: {:?}", e))
        })?;
        match cmd {
            SecondaryCmd::CmdStatusIs => {
                Ok(CmdReadDataResponse::StatusCode(parse_status_response(
                    self.header.transaction_id.value,
                    self.header.unique_id,
                    input,
                )?))
            }
            SecondaryCmd::CmdReadDataIs => {
                log::debug!("Received read response");
                let response = CmdReadDataIs::deserialize(
                    input,
                    self.header.transaction_id.value,
                    self.header.unique_id,
                )?;
                Ok(CmdReadDataResponse::Data(
                    response.data,
                    response.cmd_read_data_header.last_frag,
                ))
            }
            _ => {
                log::debug!("Invalid command id {:?}", cmd);
                Err(ProtocolError::InvalidCommandId)
            }
        }
    }
}
impl CmdReadData {
    pub fn new(
        unique_id: u32,
        transaction_id: &mut u8,
        object_key: u32,
        max_read_size: u16,
    ) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdReadData,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            object_key,
            max_read_size,
        }
    }
    pub fn serialize(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match bincode::serialize(&self) {
            Ok(bytestream) => Ok(bytestream),
            Err(err) => Err(ProtocolError::SerializationError(err.to_string())),
        }
    }
}

pub enum CmdEnumerateObjectsResponse {
    Data(Vec<u8>, bool),
    StatusCode(StatusCode),
}

#[repr(C, packed)]
#[derive(serde::Serialize)]
pub struct CmdEnumerateObjects {
    header: Header<HostCmd>,
    max_objects: u16,
}
impl Command for CmdEnumerateObjects {
    type Response = CmdEnumerateObjectsResponse;
    fn parse_response(&self, input: &[u8]) -> Result<CmdEnumerateObjectsResponse, ProtocolError> {
        // Check the very first byte to know which type of response we got
        let (_, cmd) = deserialize_cmd(input).map_err(|e| {
            ProtocolError::DeserializationError(format!("Failed to deserialize command: {:?}", e))
        })?;
        match cmd {
            SecondaryCmd::CmdStatusIs => Ok(CmdEnumerateObjectsResponse::StatusCode(
                parse_status_response(
                    self.header.transaction_id.value,
                    self.header.unique_id,
                    input,
                )?,
            )),
            SecondaryCmd::CmdEnumerateObjectsIs => {
                log::debug!("Received an object enumeration response");
                let response = CmdEnumerateObjectsIs::deserialize(
                    input,
                    self.header.transaction_id.value,
                    self.header.unique_id,
                )?;
                Ok(CmdEnumerateObjectsResponse::Data(
                    response.data,
                    response.last_frag,
                ))
            }
            _ => {
                log::debug!("Invalid command id {:?}", cmd);
                Err(ProtocolError::InvalidCommandId)
            }
        }
    }
}
impl CmdEnumerateObjects {
    pub fn new(unique_id: u32, transaction_id: &mut u8, max_objects: u16) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdEnumerateObjects,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            max_objects,
        }
    }
    pub fn serialize(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match bincode::serialize(&self) {
            Ok(bytestream) => Ok(bytestream),
            Err(err) => Err(ProtocolError::SerializationError(err.to_string())),
        }
    }
}

#[repr(C, packed)]
pub struct CmdEnumerateObjectsIs {
    header: Header<SecondaryCmd>,
    last_frag: bool,
    data: Vec<u8>,
}
impl CmdEnumerateObjectsIs {
    pub fn deserialize(
        input: &[u8],
        expected_transaction_id: u8,
        expected_unique_id: u32,
    ) -> Result<Self, ProtocolError> {
        let expected_cmd = SecondaryCmd::CmdEnumerateObjectsIs;
        let (header, remaining) = extract_and_validate_header(
            input,
            expected_cmd,
            expected_unique_id,
            expected_transaction_id,
        )?;

        let result = || -> nom::IResult<&[u8], Self> {
            let (remaining, last_frag_u8) = nom::number::complete::u8(remaining)?;
            let last_frag = last_frag_u8 != 0;

            if remaining.len() == 0 {
                return Err(nom::Err::Incomplete(nom::Needed::Size(
                    NonZeroUsize::new(
                        std::mem::size_of::<bool>()
                            + std::mem::size_of::<Header<SecondaryCmd>>()
                            + 1,
                    )
                    .unwrap(),
                )));
            }

            let data = remaining.to_vec();

            Ok((
                remaining,
                Self {
                    header,
                    last_frag,
                    data,
                },
            ))
        };

        match result() {
            Ok(tuple) => Ok(tuple.1),
            Err(err) => Err(ProtocolError::DeserializationError(err.to_string())),
        }
    }

    pub fn get_overhead() -> u16 {
        (std::mem::size_of::<CmdEnumerateObjectsIs>() + std::mem::size_of::<Header<SecondaryCmd>>())
            as u16
    }
}

#[repr(C, packed)]
pub struct ObjectInfoIs {
    header: Header<SecondaryCmd>,
    object_type: CpcNvm3ObjectType,
    object_size: u16,
}

impl ObjectInfoIs {
    pub fn deserialize(
        input: &[u8],
        expected_transaction_id: u8,
        expected_unique_id: u32,
    ) -> Result<Self, ProtocolError> {
        let expected_cmd = SecondaryCmd::CmdObjectInfoIs;
        let (header, remaining) = extract_and_validate_header(
            input,
            expected_cmd,
            expected_unique_id,
            expected_transaction_id,
        )?;

        let result = || -> nom::IResult<&[u8], Self> {
            let (remaining, object_type) = deserialize_object_type(remaining)?;
            let (remaining, object_size) = nom::number::complete::le_u16(remaining)?;
            Ok((
                remaining,
                Self {
                    header,
                    object_type,
                    object_size,
                },
            ))
        };

        match result() {
            Ok(tuple) => Ok(tuple.1),
            Err(err) => Err(ProtocolError::DeserializationError(err.to_string())),
        }
    }

    pub fn len(&self) -> u16 {
        std::mem::size_of::<ObjectInfoIs>() as u16
    }
}

pub enum CmdCounterValueResponse {
    Data(u32),
    StatusCode(StatusCode),
}

fn parse_response_counter_read_response(
    expected_unique_id: u32,
    expected_transaction_id: u8,
    input: &[u8],
) -> Result<CmdCounterValueResponse, ProtocolError> {
    // Check the very first byte to know which type of response we got
    let (_, cmd) = deserialize_cmd(input).map_err(|e| {
        ProtocolError::DeserializationError(format!("Failed to deserialize command: {:?}", e))
    })?;
    match cmd {
        SecondaryCmd::CmdStatusIs => Ok(CmdCounterValueResponse::StatusCode(
            parse_status_response(expected_transaction_id, expected_unique_id, input)?,
        )),
        SecondaryCmd::CmdCounterIs => {
            log::debug!("Received counter value");
            let response =
                CounterIs::deserialize(input, expected_transaction_id, expected_unique_id)?;
            Ok(CmdCounterValueResponse::Data(response.data))
        }
        _ => {
            log::debug!("Invalid command id {:?}", cmd);
            Err(ProtocolError::InvalidCommandId)
        }
    }
}

#[repr(C, packed)]
#[derive(serde::Serialize)]
pub struct CmdReadCounter {
    header: Header<HostCmd>,
    object_key: u32,
}
impl Serializer for CmdReadCounter {}
impl Command for CmdReadCounter {
    type Response = CmdCounterValueResponse;
    fn parse_response(&self, input: &[u8]) -> Result<CmdCounterValueResponse, ProtocolError> {
        Ok(parse_response_counter_read_response(
            self.header.unique_id,
            self.header.transaction_id.value,
            input,
        )?)
    }
}
impl CmdReadCounter {
    pub fn new(unique_id: u32, transaction_id: &mut u8, object_key: u32) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdReadCounter,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            object_key,
        }
    }
}

#[repr(C, packed)]
#[derive(serde::Serialize)]
pub struct CmdWriteCounter {
    header: Header<HostCmd>,
    object_key: u32,
    data: u32,
}

impl Serializer for CmdWriteCounter {}
impl Command for CmdWriteCounter {
    type Response = StatusCode;
    fn parse_response(&self, input: &[u8]) -> Result<StatusCode, ProtocolError> {
        parse_status_response(
            self.header.transaction_id.value,
            self.header.unique_id,
            input,
        )
    }
}
impl CmdWriteCounter {
    pub fn new(unique_id: u32, transaction_id: &mut u8, object_key: u32, data: u32) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdWriteCounter,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            object_key,
            data,
        }
    }
}

#[repr(C, packed)]
#[derive(serde::Serialize)]
pub struct CmdIncrementCounter {
    header: Header<HostCmd>,
    object_key: u32,
}
impl Serializer for CmdIncrementCounter {}
impl Command for CmdIncrementCounter {
    type Response = CmdCounterValueResponse;
    fn parse_response(&self, input: &[u8]) -> Result<CmdCounterValueResponse, ProtocolError> {
        Ok(parse_response_counter_read_response(
            self.header.unique_id,
            self.header.transaction_id.value,
            input,
        )?)
    }
}
impl CmdIncrementCounter {
    pub fn new(unique_id: u32, transaction_id: &mut u8, object_key: u32) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdIncrementCounter,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            object_key,
        }
    }
}

#[repr(C, packed)]
pub struct CounterIs {
    header: Header<SecondaryCmd>,
    data: u32,
}

impl CounterIs {
    pub fn deserialize(
        input: &[u8],
        expected_transaction_id: u8,
        expected_unique_id: u32,
    ) -> Result<Self, ProtocolError> {
        let expected_cmd = SecondaryCmd::CmdCounterIs;
        let (header, remaining) = extract_and_validate_header(
            input,
            expected_cmd,
            expected_unique_id,
            expected_transaction_id,
        )?;

        let result = || -> nom::IResult<&[u8], Self> {
            let (remaining, data_u32) = nom::number::complete::le_u32(remaining)?;
            Ok((
                remaining,
                Self {
                    header,
                    data: data_u32,
                },
            ))
        };

        match result() {
            Ok(tuple) => Ok(tuple.1),
            Err(err) => Err(ProtocolError::DeserializationError(err.to_string())),
        }
    }

    pub fn len(&self) -> u16 {
        std::mem::size_of::<CounterIs>() as u16
    }
}

pub enum CmdGetObjectInfoResponse {
    StatusCode(StatusCode),
    ObjectInfo {
        object_type: CpcNvm3ObjectType,
        object_size: u16,
    },
}

#[repr(C, packed)]
#[derive(serde::Serialize)]
pub struct CmdGetObjectInfo {
    header: Header<HostCmd>,
    object_key: u32,
}
impl Serializer for CmdGetObjectInfo {}
impl Command for CmdGetObjectInfo {
    type Response = CmdGetObjectInfoResponse;
    fn parse_response(&self, input: &[u8]) -> Result<CmdGetObjectInfoResponse, ProtocolError> {
        // Check the very first byte to know which type of response we got
        let (_, cmd) = deserialize_cmd(input).map_err(|e| {
            ProtocolError::DeserializationError(format!("Failed to deserialize command: {:?}", e))
        })?;
        match cmd {
            SecondaryCmd::CmdStatusIs => {
                Ok(CmdGetObjectInfoResponse::StatusCode(parse_status_response(
                    self.header.transaction_id.value,
                    self.header.unique_id,
                    input,
                )?))
            }
            SecondaryCmd::CmdObjectInfoIs => {
                log::debug!("Received counter value");
                let response = ObjectInfoIs::deserialize(
                    input,
                    self.header.transaction_id.value,
                    self.header.unique_id,
                )?;
                Ok(CmdGetObjectInfoResponse::ObjectInfo {
                    object_type: response.object_type,
                    object_size: response.object_size,
                })
            }
            _ => {
                log::debug!("Invalid command id {:?}", cmd);
                Err(ProtocolError::InvalidCommandId)
            }
        }
    }
}
impl CmdGetObjectInfo {
    pub fn new(unique_id: u32, transaction_id: &mut u8, object_key: u32) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdGetObjectInfo,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            object_key,
        }
    }
}

pub enum CmdGetObjectCountResponse {
    StatusCode(StatusCode),
    ObjectCount { object_count: u16 },
}
#[repr(C, packed)]
#[derive(serde::Serialize)]
pub struct CmdGetObjectCount {
    header: Header<HostCmd>,
}
impl Serializer for CmdGetObjectCount {}
impl Command for CmdGetObjectCount {
    type Response = CmdGetObjectCountResponse;
    fn parse_response(&self, input: &[u8]) -> Result<CmdGetObjectCountResponse, ProtocolError> {
        // Check the very first byte to know which type of response we got
        let (_, cmd) = deserialize_cmd(input).map_err(|e| {
            ProtocolError::DeserializationError(format!("Failed to deserialize command: {:?}", e))
        })?;
        match cmd {
            SecondaryCmd::CmdStatusIs => Ok(CmdGetObjectCountResponse::StatusCode(
                parse_status_response(
                    self.header.transaction_id.value,
                    self.header.unique_id,
                    input,
                )?,
            )),
            SecondaryCmd::CmdObjectCountIs => {
                log::debug!("Received counter value");
                let response = ObjectCountIs::deserialize(
                    input,
                    self.header.transaction_id.value,
                    self.header.unique_id,
                )?;
                Ok(CmdGetObjectCountResponse::ObjectCount {
                    object_count: response.object_count,
                })
            }
            _ => {
                log::debug!("Invalid command id {:?}", cmd);
                Err(ProtocolError::InvalidCommandId)
            }
        }
    }
}
impl CmdGetObjectCount {
    pub fn new(unique_id: u32, transaction_id: &mut u8) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdGetObjectCount,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
        }
    }
}

#[repr(C, packed)]
pub struct ObjectCountIs {
    header: Header<SecondaryCmd>,
    object_count: u16,
}

impl ObjectCountIs {
    pub fn deserialize(
        input: &[u8],
        expected_transaction_id: u8,
        expected_unique_id: u32,
    ) -> Result<Self, ProtocolError> {
        let expected_cmd = SecondaryCmd::CmdObjectCountIs;
        let (header, remaining) = extract_and_validate_header(
            input,
            expected_cmd,
            expected_unique_id,
            expected_transaction_id,
        )?;

        let result = || -> nom::IResult<&[u8], Self> {
            let (remaining, object_count) = nom::number::complete::le_u16(remaining)?;
            Ok((
                remaining,
                Self {
                    header,
                    object_count,
                },
            ))
        };

        match result() {
            Ok(tuple) => Ok(tuple.1),
            Err(err) => Err(ProtocolError::DeserializationError(err.to_string())),
        }
    }

    pub fn len(&self) -> u16 {
        std::mem::size_of::<ObjectCountIs>() as u16
    }
}

#[derive(serde::Serialize)]
pub struct CmdWriteData {
    header: Header<HostCmd>,
    object_key: u32,
    offset: u16,
    last_frag: u8,
    #[serde(skip_serializing)]
    data: Vec<u8>,
}
impl Command for CmdWriteData {
    type Response = StatusCode;
    fn parse_response(&self, input: &[u8]) -> Result<StatusCode, ProtocolError> {
        parse_status_response(
            self.header.transaction_id.value,
            self.header.unique_id,
            input,
        )
    }
}
impl CmdWriteData {
    pub fn base_size() -> u16 {
        let base_struct = Self {
            header: Header::new(HostCmd::CmdWriteData, 0, 0, TransactionId { value: 0 }),
            object_key: 0,
            offset: 0,
            last_frag: 0,
            data: vec![],
        };

        let serialized = bincode::serialize(&base_struct).unwrap();
        serialized.len() as u16
    }

    pub fn new(
        unique_id: u32,
        transaction_id: &mut u8,
        object_key: u32,
        offset: u16,
        last_frag: u8,
        data: Vec<u8>,
    ) -> Self {
        let len =
            Self::base_size() - std::mem::size_of::<Header<HostCmd>>() as u16 + data.len() as u16;
        Self {
            header: Header::new(
                HostCmd::CmdWriteData,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            object_key,
            offset,
            last_frag,
            data,
        }
    }

    pub fn get_overhead() -> u16 {
        (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16
    }

    pub fn serialize(&mut self) -> Result<Vec<u8>, ProtocolError> {
        let mut bytestream = match bincode::serialize(&self) {
            Ok(bytestream) => bytestream,
            Err(err) => return Err(ProtocolError::SerializationError(err.to_string())),
        };
        bytestream.append(&mut self.data);
        Ok(bytestream)
    }
}

pub struct StatusIs {
    pub status_code: StatusCode,
}

impl StatusIs {
    pub fn deserialize(
        input: &[u8],
        expected_transaction_id: u8,
        expected_unique_id: u32,
    ) -> Result<Self, ProtocolError> {
        let expected_cmd = SecondaryCmd::CmdStatusIs;
        let (_, remaining) = extract_and_validate_header(
            input,
            expected_cmd,
            expected_unique_id,
            expected_transaction_id,
        )?;

        let result = || -> nom::IResult<&[u8], Self> {
            let (remaining, response_type) = deserialize_response_type(remaining)?;
            let (remaining, status_code) = deserialize_status_code(response_type, remaining)?;
            Ok((remaining, Self { status_code }))
        };

        match result() {
            Ok(tuple) => Ok(tuple.1),
            Err(err) => Err(ProtocolError::DeserializationError(err.to_string())),
        }
    }
}

#[derive(serde::Serialize)]
#[repr(C, packed)]

pub struct CmdDeleteObject {
    header: Header<HostCmd>,
    object_key: u32,
}
impl Serializer for CmdDeleteObject {}
impl Command for CmdDeleteObject {
    type Response = StatusCode;
    fn parse_response(&self, input: &[u8]) -> Result<StatusCode, ProtocolError> {
        parse_status_response(
            self.header.transaction_id.value,
            self.header.unique_id,
            input,
        )
    }
}
impl CmdDeleteObject {
    pub fn new(unique_id: u32, transaction_id: &mut u8, object_key: u32) -> Self {
        let len = (std::mem::size_of::<Self>() - std::mem::size_of::<Header<HostCmd>>()) as u16;
        Self {
            header: Header::new(
                HostCmd::CmdDeleteObject,
                len,
                unique_id,
                TransactionId::new(transaction_id),
            ),
            object_key,
        }
    }
}

fn deserialize_status_code(
    response_type: StatusIsResponseType,
    input: &[u8],
) -> nom::IResult<&[u8], StatusCode> {
    let (remaining, value) = nom::number::complete::u32(nom::number::Endianness::Native)(input)?;

    match response_type {
        StatusIsResponseType::ResponseTypeSlStatus => {
            let st_status = SlStatus::try_from(value).unwrap_or(SlStatus::Unknown);
            log::debug!("Received a sl_status response {} {}", value, st_status);
            Ok((remaining, StatusCode::SlStatus(st_status)))
        }

        StatusIsResponseType::ResponseTypeEcode => {
            let e_code = ECode::try_from(value).unwrap_or(ECode::Unknown);
            log::debug!("Received an ecode response {} {}", value, e_code);
            Ok((remaining, StatusCode::ECode(e_code)))
        }
        _ => {
            log::debug!("Received an unknown response type {}", value);
            Ok((remaining, StatusCode::Unknown))
        }
    }
}

fn deserialize_response_type(input: &[u8]) -> nom::IResult<&[u8], StatusIsResponseType> {
    let (remaining, response_type) = nom::number::complete::u8(input)?;
    let response_type = StatusIsResponseType::try_from(response_type)
        .unwrap_or(StatusIsResponseType::ResponseTypeUnknown);
    Ok((remaining, response_type))
}
fn deserialize_property_type(input: &[u8]) -> nom::IResult<&[u8], PropertyType> {
    let (remaining, property_type) = nom::number::complete::u8(input)?;
    let property_type = PropertyType::try_from(property_type).unwrap_or(PropertyType::Unknown);
    Ok((remaining, property_type))
}

fn deserialize_property_value(
    property_type: PropertyType,
    input: &[u8],
) -> nom::IResult<&[u8], PropertyValue> {
    match property_type {
        PropertyType::MaxObjectSize => {
            let (remaining, value) = nom::number::complete::le_u16(input)?;
            Ok((remaining, PropertyValue::MaxObjectSize(value)))
        }
        PropertyType::MaxWriteSize => {
            let (remaining, value) = nom::number::complete::le_u16(input)?;
            Ok((remaining, PropertyValue::MaxWriteSize(value)))
        }
        _ => {
            log::error!("Unknown property type");
            Err(Err::Error(Error::new(input, ErrorKind::NoneOf)))
        }
    }
}

fn deserialize_object_type(input: &[u8]) -> nom::IResult<&[u8], CpcNvm3ObjectType> {
    let (remaining, object_type) = nom::number::complete::u8(input)?;
    let object_type = CpcNvm3ObjectType::from(object_type);
    Ok((remaining, object_type))
}

fn deserialize_cmd(input: &[u8]) -> nom::IResult<&[u8], SecondaryCmd> {
    let (remaining, cmd) = nom::number::complete::u8(input)?;
    let cmd = SecondaryCmd::try_from(cmd).unwrap_or(SecondaryCmd::UnsupportedCmdIs);
    Ok((remaining, cmd))
}

fn deserialize_header(input: &[u8]) -> nom::IResult<&[u8], Header<SecondaryCmd>> {
    let (remaining, cmd) = deserialize_cmd(input)?;
    let (remaining, len) = nom::number::complete::le_u16(remaining)?;
    let (remaining, unique_id) = nom::number::complete::le_u32(remaining)?;
    let (remaining, transaction_id) = nom::number::complete::u8(remaining)?;
    Ok((
        remaining,
        Header::new(
            cmd,
            len,
            unique_id,
            TransactionId {
                value: transaction_id,
            },
        ),
    ))
}
