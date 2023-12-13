# CPC NVM3 Library
The Co-Processor Communication (CPC) NVM3 is a software component designed to enable a
CPC Primary/Host device to utilize a Secondary NVM3 for storing persistent data.

## Features
- Read and write data from an NVM3 object or counter.
- Increment NVM3 counter
- Query information about an NVM3 object
- Logging to file capability

## Dependency 
## Installation
A CMake script with an associated pkg-config entry is provided.

To compile:
```
mkdir build
cmake ../
make
```

To install:
```
make install
```

To clean the project workspace:
```
make cleanup
```

## Usage
This library is designed to interact with a SiliconLabs microcontroller 
that utilizes the CPC NVM3 component.

To initiate interaction with the microcontroller, use `sl_cpc_nvm3_init` which 
initializes a CPC NVM3 instance. Following the initialization, `sl_cpc_nvm3_open`
is used to establish a connection with a running 
[CPC daemon](https://github.com/SiliconLabs/cpc-daemon).

Once the connection is active, users can manipulate data or counter objects 
through read or write operations.



## Logging 
To enable logging, use the `sl_cpc_nvm3_init_logger` function. 
This function accepts two arguments: the destination for the log output 
(such as a file or the standard output), and a log level of type `CpcNvm3LogLevel`.