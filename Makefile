TARGET := loongarch64-unknown-none
MODE := release# change from release
#文件模拟块设备
# FS_IMG := ./target/$(TARGET)/$(MODE)/fs.img #ext4
FS_IMG := ./img/ex4.img#ext4 由于makefile位置要改为在当前文件夹下
KERNEL_ELF := target/$(TARGET)/$(MODE)/kernel
TEST_DIR := ./test #ext4



KERNEL_BIN := $(KERNEL_ELF).bin
INFO := DEBUG
GUI ?= n
# BOARD
BOARD ?= qemu
VGA ?= -nographic

FEATURES := board_$(BOARD)

ifeq ($(GUI),y)
	FEATURES += gui
	VGA := -device VGA -serial stdio
endif

#qemu-system-loongarch64 \
		-m 1G \
		-smp 1 \
		-kernel target/loongarch64-unknown-none/release/kernel \
		-nographic \
		-drive file=./img/ex4.img,if=none,format=raw,id=x0 \
		-device ahci,id=ahci0 \
		-device ide-hd,drive=x0,bus=ahci0.0

fs-img:
	@rm -f $(FS_IMG)
	@cd ./user && make build TEST=$(TEST)
	@dd if=/dev/zero of=$(FS_IMG) bs=1M count=256
	@sudo mkfs.ext4 -F -d $(TEST_DIR) $(FS_IMG)


build: kernel fs-img

env:
	cargo install cargo-binutils

user_app:
	@make build -C user
	@cd easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../target/$(TARGET)/release/

kernel:
	echo Platform: $(BOARD)
	# cargo build -p kernel --target ${TARGET} --features "$(FEATURES)"
	cargo build --$(MODE) -p kernel --target ${TARGET} --features "$(FEATURES)"

run: run-inner

doc:
	@cargo doc --open --features"$(INFO)" --no-deps

run-inner: user_app build
ifeq ($(BOARD),qemu)
	qemu-system-loongarch64 \
		-m 1G \
		-smp 1 \
		-kernel $(KERNEL_ELF) \
		$(VGA) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
		-device ahci,id=ahci0 \
		-device ide-hd,drive=x0,bus=ahci0.0
endif


debug:build
	@tmux new-session -d \
		"qemu-system-loongarch64 -m 1G -smp 1 -kernel $(KERNEL_ELF) -vga none -nographic -drive file=$(FS_IMG),if=none,format=raw,id=x0 -device ahci,id=ahci0 -device ide-hd,drive=x0,bus=ahci0.0 -s -S" && \
		tmux split-window -h "loongarch64-unknown-linux-gnu-gdb -ex 'file $(KERNEL_ELF)'  -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdb-server: build
	qemu-system-loongarch64 \
		-m 1G -smp 1 \
		-kernel $(KERNEL_ELF) \
		-vga none -nographic \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
		-device ahci,id=ahci0 \
		-device ide-hd,drive=x0,bus=ahci0.0 \
		-s -S
gdb-client: build
	loongarch64-unknown-linux-gnu-gdb -ex 'file $(KERNEL_ELF)'  -ex 'target remote localhost:1234'

docs:
	@cargo doc --open --features "board_$(BOARD)" --features "$(INFO)" --no-deps


clean:
	@cargo clean
	@-rm -f ./efi-virtio.rom
	@-rm -f ./vgabios-stdvga.bin


.PHONY: build env kernel clean disasm disasm-vim run-inner docs gdb-server gdb-client gui