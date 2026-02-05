.PHONY: bootimage emulate

PROJECT_DIR = deimos
ABS_PROJECT_DIR = $(shell pwd)/$(PROJECT_DIR)

CARGO_BOOT_IMAGE_MAKER = cargo bootimage --release

CARGO_RUN = cargo run --release
CARGO_TEST = cargo test
CARGO_TEST_RELEASE = cargo test --release

BOOT_IMAGE_PATH = target/deimos_target/release/
BOOT_IMAGE_NAME = bootimage-deimos.bin

QEMU = qemu-system-x86_64
QEMU_FLAGS = -m 2048 -enable-kvm -serial stdio -vga virtio -net nic,model=e1000 -net user
QEMU_IMAGE_FLAG = -drive format=raw,file=$(BOOT_IMAGE_PATH)$(BOOT_IMAGE_NAME)

bootimage:
	@cd $(PROJECT_DIR) && CARGO_MANIFEST_DIR=$(ABS_PROJECT_DIR) $(CARGO_BOOT_IMAGE_MAKER)

run:
	@cd $(PROJECT_DIR) && CARGO_MANIFEST_DIR=(ABS_PROJECT_DIR) $(CARGO_RUN)

test:
	@cd $(PROJECT_DIR) && CARGO_MANIFEST_DIR=(ABS_PROJECT_DIR) $(CARGO_TEST)

test_release:
	@cd $(PROJECT_DIR) && CARGO_MANIFEST_DIR=(ABS_PROJECT_DIR) $(CARGO_TEST_RELEASE)

# You can also execute 'cargo run' inside the project directory
emulate: bootimage 
	@cd $(ABS_PROJECT_DIR) && $(QEMU) $(QEMU_FLAGS) $(QEMU_IMAGE_FLAG)


