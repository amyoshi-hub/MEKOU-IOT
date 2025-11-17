# OSAI-browser
- OSAI-browser is a new type of browser, designed with peer-to-peer (P2P) and AI culclation functionality at its core.
- apply to add tool as WORLD
## Key Features:

- P2P Functionality: OSAI-browser is built around a P2P network, allowing for decentralized content sharing and access.
- World Support: Additional content is managed as "Worlds."
- Easy World Import: Add new "Worlds" by simply dragging and dropping files into the "World Import" area.
- p2p file transfer (correspondence only "cargo tauri dev" maybe path issue)
- AI caluclation platform

## Technical Details

- Technology Stack: rust, tauri, (in p2o):pnet

## compile
in root:

```js
    cargo tauri build
```
## Current Status

P2P functionality is implemented.
file transfer is implemented.
    
# How to use
- 1,luanch server
select p2p -> "start server"
make share dir & cp files.json

- file transfer(only dev mode)
one by one device luanch OSAI-browser or

one host luanch (please use "1234" port)
```python3
    python3 -m htto.server 1234
```
    and then use [dummy_signal](https://github.com/amyoshi-hub/OSAI/tree/main/client/dummy_signal):
```c
    gcc start_ser.c -o test
    sudo "test your_ip" "port"
```
click display node -> loadFileList -> click filename

- AI culclation
this requre superuser
and only luanch server

## Important Notes
On Windows, a separate packet monitoring driver may be required.
Currently, P2P functionality requires administrator privileges.
## p2p function
 need sudo permission
## file drag install:
please include "index.html" in zip file

## process
- now divideing to library

## my crate
- vocaloid
- osai_core

## Suitable value list
now I set some suitable value

- build_protocol::data_vec
- AI::weight
- format_handler->vocaloid::input_Vec[0]
now it gonna flow vocaloid if input_vec[0] > 10, but We consider overroll; 2logical

## issue
- now sendText was success only extension 127~
- receive data chattering
