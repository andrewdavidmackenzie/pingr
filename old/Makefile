TARGET_HOST := andrew@pi
TARGET_ARCH := armv7-unknown-linux-gnueabihf
SOURCE_PATH := ./target/${TARGET_ARCH}/debug/pingr
SYSROOT := ./sysroot
PKG_CONFIG_PATH :=
PKG_CONFIG_LIBDIR := ${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig
PKG_CONFIG_SYSROOT_DIR := ${SYSROOT}
# armv7-unknown-linux-gnueabihf

.PHONY: all
all: target/debug/pingr
	sudo target/debug/pingr google.com

target/debug/pingr: build.rs Cargo.toml src
	cargo build

copy_from_pi:
	rsync -rzLR --safe-links \
          andrew@pi:/usr/lib/arm-linux-gnueabihf \
          andrew@pi:/usr/lib/gcc/arm-linux-gnueabihf \
          andrew@pi:/usr/include \
          andrew@pi:/lib/arm-linux-gnueabihf \
          sysroot/
# Should leave a copy of liboping.a here ./sysroot/usr/lib/arm-linux-gnueabihf/liboping.a

cross:
	cargo build --target=${TARGET_ARCH}