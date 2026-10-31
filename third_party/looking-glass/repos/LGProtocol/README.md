# LGProtocol

LGProtocol is a standalone, header-only distribution of the Looking Glass
wire protocol declarations. Its install surface contains seven portable
protocol headers.

Consumers include protocol declarations from the exported include root, for
example:

```c
#include <LGProtocol/KVMFR.h>
```

CMake packages expose the `LGProtocol::LGProtocol` interface target. The target
requires C11 or C++11 and adds only the installed include root:

```cmake
find_package(LGProtocol REQUIRED)
target_link_libraries(my_target PRIVATE LGProtocol::LGProtocol)
```

Configure and run the native validation suite with:

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
ctest --test-dir build --output-on-failure
```

## ABI constraints

The common wire format uses default x86-64 Linux/Windows packing, native
little-endian values, 32-bit C enums and `int`, eight-bit bytes, and 32-bit
IEEE-754 `float`. Short-enum compilation is incompatible.
`KVMFRFrameBuffer` requires the four-byte, four-byte-aligned, lock-free
`KVMFRFrameBufferWritePointer` atomic type.
The umbrella retains source-visible trailing arrays, so C and C++ compilers
must accept the documented zero-length/unsized-array extensions.

## License

Except where otherwise stated, every file in this repository is publicly
licensed under the GNU General Public License, version 2 or (at your option)
any later version (`GPL-2.0-or-later`). See [LICENSE](LICENSE).

Alternative license terms are available only under a separate written
agreement with the copyright holder. Such an agreement is independent of, and
does not restrict, the rights granted under the public GPL license.
