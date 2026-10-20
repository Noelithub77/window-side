Captured the installed `ws` help and read-only host status/config commands; sanitized `/home/noel` to `/home/you`.
Guest transcripts are derived from the Rust serializers, tests, and guest scripts because the VM stayed off.
Refresh raw reads with `ws status`, `ws config path`, `ws config get vm.memory_mib`, `ws config get vm.vcpus`, `ws config validate`, and `ws --help`; keep raw output in `/tmp/ws-landing/cli/` (mode 700).
Update derived entries from `src/guest/`, `src/vm/`, and tests, then sanitize paths and identities before writing `cli.json`. Never start the VM just to refresh a transcript.
