# group `cpc_nvm3` 

## Summary

 Members                        | Descriptions                                
--------------------------------|---------------------------------------------
`enum `[`CpcNvm3ErrorCodes`](#group__cpc__nvm3_1gaa8b94fbce332f8b18c85bad39059cb70)            | 
`enum `[`CpcNvm3LogLevel`](#group__cpc__nvm3_1gaf11f1025d0c7bf0dbaa5ac3142044c1b)            | 
`enum `[`CpcNvm3ObjectType`](#group__cpc__nvm3_1ga7d40fb3e035952a9e74d05cb9a83766d)            | 
`public int32_t `[`cpc_nvm3_init_logger`](#group__cpc__nvm3_1gadac9cdda212c6db67bd68dfa4a7d344f)`(const char * prefix,enum CpcNvm3LogLevel level,const char * file_path,bool append)`            | Initialize the logger for the CPC NVM3 library. The user must provide the desired log level and the output destination file.
`public int32_t `[`cpc_nvm3_init`](#group__cpc__nvm3_1ga8035245b32fe9abcb893edc87293c054)`(cpc_nvm3_handle_t * handle)`            | Initialize a new CPC NVM3 instance.
`public int32_t `[`cpc_nvm3_deinit`](#group__cpc__nvm3_1gaa5e0ddc9589e6beeac6fec23f74cc347)`(cpc_nvm3_handle_t cpc_nvm3_handle)`            | Deinitialize the specified CPC NVM3 instance, effectively releasing the associated resources.
`public int32_t `[`cpc_nvm3_open`](#group__cpc__nvm3_1ga89e856e29526124982f33e95c4bdb8e8)`(cpc_nvm3_handle_t cpc_nvm3_handle,const char * cpcd_instance_name,bool enable_cpc_traces)`            | Initialize the CPC NVM3 library. Upon success the user will get a handle that must be passed to subsequent calls.
`public int32_t `[`cpc_nvm3_close`](#group__cpc__nvm3_1ga427af8af88144198bc94236604656383)`(cpc_nvm3_handle_t cpc_nvm3_handle)`            | Close the CPC NVM3 library. Upon success the handle be considered invalid and cannot be used on subsequent calls to the library.
`public int32_t `[`cpc_nvm3_write_data`](#group__cpc__nvm3_1ga27f5e0730e56bca4fb536fc237eacff6)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,const uint8_t * data_ptr,uint16_t data_length)`            | Write data to the specified object in the CPC NVM3 library. The user must provide a valid handle obtained from the initialization process.
`public int32_t `[`cpc_nvm3_read_data`](#group__cpc__nvm3_1gab17ccd631347f670be97a0b9e282a4bb)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,void * buffer_ptr,uint16_t buffer_size,uint16_t * object_size)`            | Read data from the specified object in the CPC NVM3 library. The user must provide a valid handle obtained from the initialization process.
`public int32_t `[`cpc_nvm3_get_object_count`](#group__cpc__nvm3_1ga24bb76c4e36e558a7bedbe20a04f57a0)`(cpc_nvm3_handle_t cpc_nvm3_handle,uint16_t * object_count)`            | Retrieve the count of objects stored in the specified CPC NVM3 instance.
`public int32_t `[`cpc_nvm3_list_objects`](#group__cpc__nvm3_1gaf1c9e83cce1ba99b4ac1264d79683fba)`(cpc_nvm3_handle_t cpc_nvm3_handle,const cpc_nvm3_object_key_t * cpc_nvm3_object_keys_ptr,uint16_t max_key_count,uint16_t * object_count)`            | Get a list of objects available on the CPC NVM3 instance.
`public int32_t `[`cpc_nvm3_write_counter`](#group__cpc__nvm3_1gaed6657d3c24790ec07bc086697663823)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,uint32_t value)`            | Write a value to the specified counter.
`public int32_t `[`cpc_nvm3_read_counter`](#group__cpc__nvm3_1gac2d9387d1ad2589b2089879bc88544ec)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,uint32_t * value)`            | Read data from the specified counter.
`public int32_t `[`cpc_nvm3_increment_counter`](#group__cpc__nvm3_1gac5106197c83387ec138a9eea043b84fe)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,uint32_t * new_value)`            | Increment the specified counter.
`public int32_t `[`cpc_nvm3_get_maximum_write_size`](#group__cpc__nvm3_1gaee258c1f3a4e4c90525a27a29cb810f3)`(cpc_nvm3_handle_t cpc_nvm3_handle,uint16_t * max_write)`            | Retrieve the maximum allowable size for an object that can be written to the NVM3 instance on the remote device. The user must provide a valid handle obtained from the initialization process.
`public int32_t `[`cpc_nvm3_get_object_info`](#group__cpc__nvm3_1ga6a31a56bdbf1d85727a550a56206ae09)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,uint16_t * object_size,enum `[`CpcNvm3ObjectType`](api.md undefined#group__cpc__nvm3_1ga7d40fb3e035952a9e74d05cb9a83766d)` * object_type)`            | Query additional information about the NVM3 object.
`public int32_t `[`cpc_nvm3_delete_object`](#group__cpc__nvm3_1ga52fe95d27673ad54bf99fd7b23ab5bd5)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key)`            | Delete an NVM3 object.
`public int32_t `[`cpc_nvm3_set_cpc_timeout`](#group__cpc__nvm3_1gab702eb585c14b9d1fe6a3706e9803074)`(cpc_nvm3_handle_t cpc_nvm3_handle,int32_t seconds,int32_t microseconds)`            | Set the timeout on CPC operations. The timeout is the sum of the provided arguments.
`public int32_t `[`cpc_nvm3_get_cpc_timeout`](#group__cpc__nvm3_1ga6bf5a6103556725cdcbd7415582741cc)`(cpc_nvm3_handle_t cpc_nvm3_handle,int32_t * seconds,int32_t * microseconds)`            | Get the timeout on CPC operations.

## Members

#### `enum `[`CpcNvm3ErrorCodes`](#group__cpc__nvm3_1gaa8b94fbce332f8b18c85bad39059cb70) 

 Values                         | Descriptions                                
--------------------------------|---------------------------------------------
CPC_NVM3_FAILURE            | Generic CPC NVM3 API failure, refer to logs for more details
CPC_NVM3_NOT_INITIALIZED            | CPC NVM3 instance was not initialized
CPC_NVM3_NOT_OPEN            | An operation failed because the NVM3 instance was not opened
CPC_NVM3_NOT_CLOSED            | An operation failed because the NVM3 instance was not closed
CPC_NVM3_UNKNOWN_ERROR            | An unknown error occured, refer to logs for more details
CPC_NVM3_INVALID_ARG            | An invalid argument was provided
CPC_NVM3_INVALID_VERSION            | There is a version mismatch between the lib CPC NVM3 and the CPC NVM3 component on the remote device
CPC_NVM3_INVALID_OBJECT_KEY            | An invalid NVM3 object key was provided
CPC_NVM3_TRY_AGAIN            | The NVM3 instance is not ready yet, the API should be called again
CPC_NVM3_CPC_ENDPOINT_ERROR            | A CPC endpoint error occured, refer to logs for more details
CPC_NVM3_BUFFER_TOO_SMALL            | The read provided buffer is too small

#### `enum `[`CpcNvm3LogLevel`](#group__cpc__nvm3_1gaf11f1025d0c7bf0dbaa5ac3142044c1b) 

 Values                         | Descriptions                                
--------------------------------|---------------------------------------------
CPC_NVM3_LOG_OFF            | 
CPC_NVM3_LOG_ERROR            | 
CPC_NVM3_LOG_WARNING            | 
CPC_NVM3_LOG_INFO            | 
CPC_NVM3_LOG_DEBUG            | 
CPC_NVM3_LOG_TRACE            | 

#### `enum `[`CpcNvm3ObjectType`](#group__cpc__nvm3_1ga7d40fb3e035952a9e74d05cb9a83766d) 

 Values                         | Descriptions                                
--------------------------------|---------------------------------------------
CPC_NVM3_OBJECT_TYPE_COUNTER            | NVM3 entity is a counter
CPC_NVM3_OBJECT_TYPE_DATA            | NVM3 entity is a data object
CPC_NVM3_OBJECT_TYPE_UNKNOWN            | NVM3 entity is of an unknown type

#### `public int32_t `[`cpc_nvm3_init_logger`](#group__cpc__nvm3_1gadac9cdda212c6db67bd68dfa4a7d344f)`(const char * prefix,enum CpcNvm3LogLevel level,const char * file_path,bool append)` 

Initialize the logger for the CPC NVM3 library. The user must provide the desired log level and the output destination file.

#### Parameters
* `prefix` A prefix string to add to every logs. This can be used as an identifier when multiple processes log to a same file. If a prefix is not required, this argument can be NULL.

* `level` The desired log level. This is a value from the CpcNvm3LogLevel enumeration. 

* `file_path` The output destination file. If this parameter is NULL, the logs will be written to the standard output.

* `append` A boolean that indicates whether to append to the log file if provided or to overwrite the existing content (if false).

The logger can only be initialized once. Attempting to initialize the logger when it has already been initialized will be ignored.

#### `public int32_t `[`cpc_nvm3_init`](#group__cpc__nvm3_1ga8035245b32fe9abcb893edc87293c054)`(cpc_nvm3_handle_t * handle)` 

Initialize a new CPC NVM3 instance.

#### Parameters
* `handle` A pointer to where the CPC NVM3 Handle will be stored. This handle is a positive integer and it's unique for each instance. It's used to refer to the corresponding instance in subsequent function calls.

#### Returns
On success, the function returns 0. On error, the function returns a negative value, corresponding to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred.

#### `public int32_t `[`cpc_nvm3_deinit`](#group__cpc__nvm3_1gaa5e0ddc9589e6beeac6fec23f74cc347)`(cpc_nvm3_handle_t cpc_nvm3_handle)` 

Deinitialize the specified CPC NVM3 instance, effectively releasing the associated resources.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance to be deinitialized.

#### Returns
On success, the function returns 0. On error, the function returns a negative value, corresponding to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred.

Before calling this function, the user must ensure the CPC NVM3 instance is not connected to a CPC daemon. To do this, `cpc_nvm3_close` should be called first. If the instance is still open, the function will return an error.

#### `public int32_t `[`cpc_nvm3_open`](#group__cpc__nvm3_1ga89e856e29526124982f33e95c4bdb8e8)`(cpc_nvm3_handle_t cpc_nvm3_handle,const char * cpcd_instance_name,bool enable_cpc_traces)` 

Initialize the CPC NVM3 library. Upon success the user will get a handle that must be passed to subsequent calls.

#### Parameters
* `instance_name` The name of the daemon instance. It will be the value of the instance_name in the config file of the daemon. This value can be NULL, and so the default "cpcd_0" value will be used. If running a single instance, this can be left to NULL, but when running simultaneous instances, it will need to be supplied. 

* `instance_id` The id of the NVM3 instance. 

* `enable_tracing` Enable tracing

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN. 

Only one opened instance per process is allowed

#### `public int32_t `[`cpc_nvm3_close`](#group__cpc__nvm3_1ga427af8af88144198bc94236604656383)`(cpc_nvm3_handle_t cpc_nvm3_handle)` 

Close the CPC NVM3 library. Upon success the handle be considered invalid and cannot be used on subsequent calls to the library.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance.

#### Returns
On success, the function returns 0. On error, the function returns a negative value, corresponding to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred.

#### `public int32_t `[`cpc_nvm3_write_data`](#group__cpc__nvm3_1ga27f5e0730e56bca4fb536fc237eacff6)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,const uint8_t * data_ptr,uint16_t data_length)` 

Write data to the specified object in the CPC NVM3 library. The user must provide a valid handle obtained from the initialization process.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `cpc_nvm3_object_key` The key of the object to write data to. 

* `data_ptr` A pointer to the data buffer to be written. 

* `data_length` The length of the data to be written.

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.

The buffer is not copied. The user must ensure the data buffer is not modified during the write operation. 

This API will return CPC_NVM3_TRY_AGAIN if another process is writing to the same object.

#### `public int32_t `[`cpc_nvm3_read_data`](#group__cpc__nvm3_1gab17ccd631347f670be97a0b9e282a4bb)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,void * buffer_ptr,uint16_t buffer_size,uint16_t * object_size)` 

Read data from the specified object in the CPC NVM3 library. The user must provide a valid handle obtained from the initialization process.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `cpc_nvm3_object_key` The key of the object to read data from. 

* `buffer_ptr` A pointer to the buffer where the read data will be stored. 

* `buffer_size` The size of the provided buffer. 

* `object_size` A pointer to a variable where the actual size of the NVM3 object will be stored.

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.

The user must ensure the provided buffer is large enough to hold the read data.

#### `public int32_t `[`cpc_nvm3_get_object_count`](#group__cpc__nvm3_1ga24bb76c4e36e558a7bedbe20a04f57a0)`(cpc_nvm3_handle_t cpc_nvm3_handle,uint16_t * object_count)` 

Retrieve the count of objects stored in the specified CPC NVM3 instance.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `object_count` Pointer to a variable where the total count of stored objects will be written. The value at this pointer will be updated only if the function is successful.

#### Returns
On success, the function returns 0 and the object count is written to the variable pointed to by the `object_count` parameter. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred.

#### `public int32_t `[`cpc_nvm3_list_objects`](#group__cpc__nvm3_1gaf1c9e83cce1ba99b4ac1264d79683fba)`(cpc_nvm3_handle_t cpc_nvm3_handle,const cpc_nvm3_object_key_t * cpc_nvm3_object_keys_ptr,uint16_t max_key_count,uint16_t * object_count)` 

Get a list of objects available on the CPC NVM3 instance.

This function retrieves a list of keys for the objects stored in the NVM3 instance.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `cpc_nvm3_object_keys_ptr` Pointer to an array where the object keys will be stored. 

* `max_key_count` Maximum number of keys that can be stored in the array. 

* `object_count` Pointer to a variable where the actual count of keys will be stored.

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.

#### `public int32_t `[`cpc_nvm3_write_counter`](#group__cpc__nvm3_1gaed6657d3c24790ec07bc086697663823)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,uint32_t value)` 

Write a value to the specified counter.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `cpc_nvm3_object_key` The key of the counter. 

* `value` The value to write.

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.

#### `public int32_t `[`cpc_nvm3_read_counter`](#group__cpc__nvm3_1gac2d9387d1ad2589b2089879bc88544ec)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,uint32_t * value)` 

Read data from the specified counter.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `cpc_nvm3_object_key` The key of the counter object to read data from. 

* `value` A pointer to the variable where the counter data will be stored. This value is optional, when a NULL pointer is provided, it will be ignored.

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.

#### `public int32_t `[`cpc_nvm3_increment_counter`](#group__cpc__nvm3_1gac5106197c83387ec138a9eea043b84fe)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,uint32_t * new_value)` 

Increment the specified counter.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `cpc_nvm3_object_key` The key of the counter object to increment data from. 

* `new_value` A pointer to the variable where the counter new value will be stored.

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.

#### `public int32_t `[`cpc_nvm3_get_maximum_write_size`](#group__cpc__nvm3_1gaee258c1f3a4e4c90525a27a29cb810f3)`(cpc_nvm3_handle_t cpc_nvm3_handle,uint16_t * max_write)` 

Retrieve the maximum allowable size for an object that can be written to the NVM3 instance on the remote device. The user must provide a valid handle obtained from the initialization process.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance.

#### Returns
On success, the function returns 0. On error, the function returns a negative value, corresponding to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred.

Make sure to verify that the CPC NVM3 instance is opened and functional before calling this function, as it will fail otherwise.

#### `public int32_t `[`cpc_nvm3_get_object_info`](#group__cpc__nvm3_1ga6a31a56bdbf1d85727a550a56206ae09)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key,uint16_t * object_size,enum `[`CpcNvm3ObjectType`](api.md undefined#group__cpc__nvm3_1ga7d40fb3e035952a9e74d05cb9a83766d)` * object_type)` 

Query additional information about the NVM3 object.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `cpc_nvm3_object_key` The key of the NVM3 object to query information from 

* `object_size` A pointer to the variable where the object size will be stored. 

* `object_type` A pointer to the variable where the object type will be stored.

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.

#### `public int32_t `[`cpc_nvm3_delete_object`](#group__cpc__nvm3_1ga52fe95d27673ad54bf99fd7b23ab5bd5)`(cpc_nvm3_handle_t cpc_nvm3_handle,cpc_nvm3_object_key_t cpc_nvm3_object_key)` 

Delete an NVM3 object.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `cpc_nvm3_object_key` The key of the NVM3 object to query information from

#### Returns
On success, the function returns 0. On error, it returns a negative value. This negative number corresponds to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred. If the connection to the CPC endpoint is lost, the function will return CPC_NVM3_TRY_AGAIN.

#### `public int32_t `[`cpc_nvm3_set_cpc_timeout`](#group__cpc__nvm3_1gab702eb585c14b9d1fe6a3706e9803074)`(cpc_nvm3_handle_t cpc_nvm3_handle,int32_t seconds,int32_t microseconds)` 

Set the timeout on CPC operations. The timeout is the sum of the provided arguments.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `seconds` How many seconds to block. 

* `microseconds` How many microseconds to block.

#### Returns
On success, the function returns 0. On error, the function returns a negative value, corresponding to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred.

#### `public int32_t `[`cpc_nvm3_get_cpc_timeout`](#group__cpc__nvm3_1ga6bf5a6103556725cdcbd7415582741cc)`(cpc_nvm3_handle_t cpc_nvm3_handle,int32_t * seconds,int32_t * microseconds)` 

Get the timeout on CPC operations.

#### Parameters
* `cpc_nvm3_handle` The handle to the CPC NVM3 instance. 

* `seconds` How many seconds to block. 

* `microseconds` How many microseconds to block.

#### Returns
On success, the function returns 0. On error, the function returns a negative value, corresponding to a specific CpcNvm3ErrorCodes, indicating the type of error that occurred.

