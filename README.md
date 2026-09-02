# boolcodec

`boolcodec` provides a space-optimizd dynamic-length boolean sequence. It uses a proprietary encoding.

## Encoding
At the byte level

* First bit - FLAG, on/off

if ON then
* three bits - pattern of three booleans (1 => true 0 => false)
* four bits - repitation coeffiecent, i.e. how many times the pattern is repeated
if OFF then
* 7 bits - 7 bools, (1 => true 0 => false)


Therfore, a byte may express [7, 3 * (2^4-1)] boolean values.

## Example
```sh
cargo r --release --example availability  
```
```
availability samples: 48000
Vec<bool>:  24 inline + 65536 heap = 65560 bytes
BoolVec:    48 inline + 1024 heap = 1072 bytes
encoded payload: 1000 bytes
summary: 61.2x less total memory (98.4% smaller)
payload only: 48.0x smaller
```

## License

Copyright © 2026 Ron Friedhaber. Licensed under the
[GNU Affero General Public License v3.0](LICENSE).
