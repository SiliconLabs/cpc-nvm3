use std::cell::RefCell;
use std::collections::VecDeque;

pub use libcpc::cpc_endpoint_id;
pub use libcpc::cpc_endpoint_read_flags_t_enum;
pub use libcpc::cpc_endpoint_write_flags_t_enum;
pub use libcpc::cpc_timeval_t;
pub use libcpc::sl_cpc_service_endpoint_id_t_enum;
pub use libcpc::Error;

use crate::nvm3::CPC_NVM3_MAJOR_VERSION;
use crate::nvm3::CPC_NVM3_MINOR_VERSION;
use crate::nvm3::CPC_NVM3_PATCH_VERSION;

const CPC_NVM3_MAX_WRITE_CAPABILITY: usize = 256;

#[allow(non_camel_case_types)] // This will be used in a generated a C header file
#[derive(Debug, Copy, Clone)]
pub struct cpc_handle;

#[allow(non_camel_case_types)] // This will be used in a generated a C header file
#[derive(Debug)]
pub struct cpc_endpoint {
    // A queue of byte vectors for testing. `RefCell` enables interior mutability,
    // allowing us to modify the queue with an immutable reference to the `CpcNvm3Instance`
    test_data_fifo_rx: RefCell<VecDeque<Vec<u8>>>,
    _test_data_fifo_tx: RefCell<VecDeque<Vec<u8>>>,
}

impl cpc_handle {
    pub fn open_endpoint(
        &self,
        _id: cpc_endpoint_id,
        _tx_window_size: u8,
    ) -> Result<cpc_endpoint, Error> {
        let mut endpoint = cpc_endpoint {
            test_data_fifo_rx: RefCell::new(VecDeque::new()),
            _test_data_fifo_tx: RefCell::new(VecDeque::new()),
        };

        let version_response = vec![
            0x01, // cmd
            0x03, // len
            0x00, // len
            0x00, // unique_id
            0x00, // unique_id
            0x00, // unique_id
            0x00, // unique_id
            0x01, // seq
            CPC_NVM3_MAJOR_VERSION,
            CPC_NVM3_MINOR_VERSION,
            CPC_NVM3_PATCH_VERSION,
        ];
        // We always query the version as soon as we open the endpoint
        // so it makes sense to prepare it this response right away.
        endpoint.push_rx(version_response);

        let maximum_write_response = vec![
            0x05, // cmd
            0x03, // len
            0x00, // len
            0x00, // unique_id
            0x00, // unique_id
            0x00, // unique_id
            0x00, // unique_id
            0x02, // seq
            0x02, // prop
            0xFF, // data
            0x00, // data
        ];
        // We always query the maximum write property as soon as we open the endpoint
        // so it makes sense to prepare it this response right away.
        endpoint.push_rx(maximum_write_response);

        Ok(endpoint)
    }
}

impl cpc_endpoint {
    pub fn push_rx(&mut self, rx_data: Vec<u8>) {
        self.test_data_fifo_rx.borrow_mut().push_back(rx_data);
    }

    pub fn close(&self) -> Result<(), Error> {
        Ok(())
    }

    pub fn write(
        &self,
        _data: &Vec<u8>,
        _flags: &[cpc_endpoint_write_flags_t_enum],
    ) -> Result<(), Error> {
        Ok(())
    }

    pub fn read(&self, _flags: &[cpc_endpoint_read_flags_t_enum]) -> Result<Vec<u8>, Error> {
        let mut test_data_fifo: std::cell::RefMut<VecDeque<Vec<u8>>> =
            self.test_data_fifo_rx.borrow_mut();
        let test_data = match test_data_fifo.pop_front() {
            Some(test_data) => test_data,
            None => return Err(Error::Errno(std::io::Error::from_raw_os_error(-1))),
        };
        log::debug!("Read {:?}", test_data);
        Ok(test_data)
    }

    pub fn get_read_timeout(&self) -> Result<cpc_timeval_t, Error> {
        let timeval = cpc_timeval_t {
            seconds: 0,
            microseconds: 0,
        };
        return Ok(timeval);
    }

    pub fn set_read_timeout(&self, _timeval: cpc_timeval_t) -> Result<(), Error> {
        Ok(())
    }

    pub fn get_max_write_size(&self) -> Result<usize, Error> {
        Ok(CPC_NVM3_MAX_WRITE_CAPABILITY)
    }
}

pub fn init(
    _instance_name: &str,
    _enable_tracing: bool,
    _reset_callback: std::option::Option<unsafe extern "C" fn()>,
) -> Result<cpc_handle, Error> {
    let handle = cpc_handle {};
    Ok(handle)
}
