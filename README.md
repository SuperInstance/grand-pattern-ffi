# grand-pattern-ffi

**C FFI bindings — use the Grand Pattern from C, C++, Python ctypes, Ruby FFI, Lua, or any language with C interop.**

This crate builds a C shared library (`libgrand_pattern.so` / `.dylib` / `.dll`) and static library (`libgrand_pattern.a`) that expose the full Grand Pattern graph API via a stable C ABI.

## Quick Start

```bash
# Build
cargo build --release

# Output (Linux):
#   target/release/libgrand_pattern.a      (static)
#   target/release/libgrand_pattern.so     (shared)
```

## C Header

Copy `include/grand_pattern_ffi.h` into your project.

```c
#include "grand_pattern_ffi.h"

int main() {
    GpGraph* g = gp_graph_new(120.0);
    gp_add_room(g, 1.0);
    gp_add_room(g, 2.0);
    gp_topology_chain(g);
    gp_tick(g);
    gp_diffuse(g, 0.5);
    printf("Total vibe: %f\n", gp_total_vibe(g));
    gp_graph_free(g);
    return 0;
}
```

```bash
gcc -o demo demo.c -L./target/release -lgrand_pattern -lm
```

## Python (ctypes)

```python
from ctypes import *

lib = CDLL("./libgrand_pattern.so")

lib.gp_graph_new.restype = c_void_p
lib.gp_graph_new.argtypes = [c_double]

g = lib.gp_graph_new(120.0)
lib.gp_add_room(g, 1.0)
lib.gp_add_room(g, 2.0)
print("Rooms:", lib.gp_room_count(g))
lib.gp_graph_free(g)
```

## API

| Function | Description |
|---|---|
| `gp_graph_new(bpm)` | Create a new graph |
| `gp_graph_free(graph)` | Free a graph (null-safe) |
| `gp_add_room(graph, vibe)` | Add a room, returns ID |
| `gp_remove_room(graph, id)` | Remove a room |
| `gp_room_vibe(graph, id)` | Get room vibe |
| `gp_room_surprise(graph, id)` | Get room surprise |
| `gp_set_room_vibe(graph, id, vibe)` | Set room vibe |
| `gp_add_edge(graph, from, to, weight)` | Add weighted edge |
| `gp_edge_count(graph)` | Count edges |
| `gp_tick(graph)` | Advance one tick |
| `gp_diffuse(graph, rate)` | Diffuse vibes through edges |
| `gp_learn(graph)` | Learn toward fleet mean |
| `gp_total_vibe(graph)` | Sum of all vibes |
| `gp_fleet_vibe(graph)` | Average vibe |
| `gp_fleet_surprise(graph)` | Average surprise |
| `gp_room_count(graph)` | Number of rooms |
| `gp_tick_count(graph)` | Ticks elapsed |
| `gp_verify_conservation(graph, tol)` | Check vibe conservation |
| `gp_topology_chain(graph)` | Chain topology |
| `gp_topology_ring(graph)` | Ring topology |
| `gp_topology_star(graph)` | Star topology |
| `gp_topology_mesh(graph)` | Full mesh topology |
| `gp_topology_small_world(graph, prob)` | Small-world topology |
| `gp_export_vibes(graph, buf, len)` | Export all vibes |
| `gp_export_surprises(graph, buf, len)` | Export all surprises |

## Safety

- All functions handle null pointers gracefully (return 0/null, no crash)
- Thread-safe: each graph is independent, create from any thread
- No dependencies — pure Rust, zero crates

## License

MIT
