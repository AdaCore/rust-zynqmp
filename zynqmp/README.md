# zynqmp

Support for the AMD Zynq UltraScale+ MPSoC.

## Examples

The command `cargo run --example NAME -- -gdb tcp::3333 -S` executes an example using QEMU with remote debugging enabled.
QEMU will freeze the machine at startup and wait for a GDB connection.
After starting the application connect to QEMU in a GDB shell with `target remote :3333`.
