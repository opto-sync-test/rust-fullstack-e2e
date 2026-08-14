# Full-stack Rust Opto-Sync E2E

This repository exercises Opto-Sync on both sides of a Rust application:

- an `OptoSyncClient<InMemoryStore>` queues an offline mutation and renders its
  optimistic local view;
- an Axum server exposes the authoritative document over HTTP;
- the server invokes the statically linked `syncer.c` engine through the
  official Rust client dependency;
- the test acknowledges the queue only after the network merge succeeds.

The complete proof is one process but crosses a real TCP socket. Run it with:

```sh
git submodule update --init --recursive
cargo test --locked --all-targets
```
