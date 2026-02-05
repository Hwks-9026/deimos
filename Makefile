.PHONY: bootimage emulate

PROJECT_DIR = deimos
ABS_PROJECT_DIR = $(shell pwd)/$(PROJECT_DIR)

CARGO_BOOT_IMAGE_MAKER = cargo bootimage --release

BOOT_IMAGE_PATH = target/deimos_target/release/
BOOT_IMAGE_NAME = bootimage-deimos.bin

QEMU = qemu-system-x86_64
QEMU_FLAGS = -m 2048 -enable-kvm -serial stdio -vga virtio -net nic,model=e1000 -net user
QEMU_IMAGE_FLAG = -drive format=raw,file=$(BOOT_IMAGE_PATH)$(BOOT_IMAGE_NAME)

bootimage:
	@cd $(PROJECT_DIR) && CARGO_MANIFEST_DIR=$(ABS_PROJECT_DIR) $(CARGO_BOOT_IMAGE_MAKER)

emulate: bootimage 
	@cd $(ABS_PROJECT_DIR) && $(QEMU) $(QEMU_FLAGS) $(QEMU_IMAGE_FLAG)



