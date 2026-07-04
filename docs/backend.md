# Runtime (llama.cpp)

There is **no** backend abstraction layer.

`app/src/runtime/llama.rs` is the only module that knows llama.cpp flags.
`app/src/runtime/process.rs` spawns and stops `bin/llama-server.exe`.

OpenAI compatibility is provided by llama-server itself at `/v1`. WinServeAI ensures the process is up and ready (`GET /v1/models`).

Multi-backend support is explicitly out of scope until a real second backend exists.
