# Running Cepu

To boot the emulator you need three files in the repo root:

- `Image`, uncompressed Linux arm64 kernel
- `cepu.dtb`, compiled device tree
- `initramfs.cpio`, root filesystem (uncompressed cpio)

You also need Rust nightly (`edition = "2024"`) and `dtc` (device tree compiler).

## Kernel

Build a Linux arm64 kernel however you prefer. I use 7.1-rc2 from [Andreas Hindborg's rnull tree](https://git.kernel.org/pub/scm/linux/kernel/git/a.hindborg/linux.git) (branch `rnull-v7.1-rc2`, which carries the Rust block-layer bindings the CepuCel work builds on) with Clang/LLD in Docker, but any arm64 cross-compile setup works. Note that Rust support on 7.1 needs rustc >= 1.85.0 and bindgen >= 0.71.1. Start from `defconfig` and tweak:

```
# the emulator only does 4KB pages, 48-bit VA/PA, no LPA2
ARM64_4K_PAGES=y
ARM64_VA_BITS=48
ARM64_PA_BITS=48
ARM64_LPA2=n

# PL011 UART (our only I/O device for console)
SERIAL_AMBA_PL011=y
SERIAL_AMBA_PL011_CONSOLE=y

# no KASLR, no relocation, emulator loads the kernel at a fixed address
RANDOMIZE_BASE=n
RELOCATABLE=n

# no EFI, bare-metal boot
EFI=n
EFI_STUB=n

# not implemented
ARM64_SW_TTBR0_PAN=n
```

If you want to load the CepuCel driver, also enable `RUST=y`, `MODULES=y`, `MODULE_UNLOAD=y`.

Kernel command line (either bake it in with `CMDLINE_FORCE` or let the device tree handle it):

```
console=ttyAMA0,115200 earlycon=pl011,mmio,0x90000000 nokaslr
```

Copy `arch/arm64/boot/Image` to the repo root.

## Initramfs

The emulator expects a raw (not gzipped) cpio archive. You can build the rootfs however you want (Alpine mini rootfs, Buildroot, from scratch) as long as it has:

- An aarch64 BusyBox (static or dynamic with musl). If dynamic, include `lib/ld-musl-aarch64.so.1`.
- `/dev/console` (char device, major 5 minor 1). Kernel needs this before init runs.
- An `/init` script at the root.

BusyBox applets I have symlinked in `/bin`:

```
sh ash ls cat echo mount umount mkdir rmdir cp mv rm
ln chmod chown pwd ps kill sleep dmesg grep sed awk
vi less head tail wc sort uniq tr cut printf test
true false yes expr env id whoami uname hostname
df du free top reboot poweroff halt init
```

My `/init`:

```sh
#!/bin/sh
mount -t proc none /proc
mount -t sysfs none /sys
mount -t devtmpfs none /dev
exec /bin/sh
```

Directory layout:

```
initramfs/
├── bin/          # busybox + symlinks
├── dev/console   # mknod console c 5 1
├── etc/
├── lib/          # ld-musl-aarch64.so.1 if dynamic
├── proc/
├── sbin/
├── sys/
├── tmp/
└── init
```

Pack it:

```bash
cd initramfs && find . | cpio -o -H newc > ../initramfs.cpio
```

## Device tree

`cepu.dts` is already in the repo. Compile it:

```bash
dtc -I dts -O dtb -o cepu.dtb cepu.dts
```

## Run

```bash
cargo run --release
```
