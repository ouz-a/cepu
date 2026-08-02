# Cepu

![Cepu booting Linux to a shell](demo.gif)

Cepu (short for cerebral processing unit) is an AArch64 system emulator.
It boots Linux 7.1 to a BusyBox shell in about 3 seconds on my MacBook using a device tree and
initramfs. It includes a custom accelerator device (CepuCel) and an out-of-tree
Rust kernel driver that lets the emulator run simple CNN models.

## Technical Details

- **Architecture**: Single-core AArch64 system emulator targeting an ARMv9-A subset, EL0/EL1 only, little-endian
- **CPU**: 200+ instruction encodings covering integer, SIMD/FP, load/store, branch, and system operations. 4-level MMU with 4KB pages.
- **Devices**: GICv2 interrupt controller, PL011 UART, ARMv8 generic timer (24 MHz)
- **Accelerator**: CepuCel, an MMIO device with ring-buffer command queue, runs matrix multiplication on a dedicated thread
- **Driver**: Out-of-tree Linux kernel module written in Rust

## Running

See [docs/running.md](docs/running.md) for how to build the kernel, initramfs, and device tree, and boot the emulator.

## Limitations

- Single core only, no SMP
- No EL2 (hypervisor) or EL3 (secure monitor)
- No GPU or display, headless with serial console
- CepuCel accelerator supports F32 matrix multiplication only
- Not cycle-accurate

I built Cepu to teach myself how CPUs work. You can read about the process at [oguza.com/bits-all-the-way-down](https://oguza.com/bits-all-the-way-down).

Licensed under the MIT license. See [LICENSE](LICENSE).
