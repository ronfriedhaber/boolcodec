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
