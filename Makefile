# LibreDash Makefile for Raspberry Pi Bare Metal

# Toolchain
TOOLCHAIN ?= arm-none-eabi-
CC = $(TOOLCHAIN)gcc
LD = $(TOOLCHAIN)ld
OBJCOPY = $(TOOLCHAIN)objcopy

# Target architecture (Raspberry Pi 3/4)
ARCH = armv8-a
CPU = cortex-a53

# Directories
SRC_DIR = src
BUILD_DIR = build
INCLUDE_DIR = include

# Compiler flags
CFLAGS = -march=$(ARCH) -mcpu=$(CPU) -nostdlib -nostartfiles -ffreestanding
CFLAGS += -O2 -Wall -Wextra -I$(INCLUDE_DIR)

# Linker flags
LDFLAGS = -T linker.ld -nostdlib

# Source files
C_SOURCES = $(wildcard $(SRC_DIR)/*.c)
ASM_SOURCES = $(wildcard $(SRC_DIR)/*.S)

# Object files
C_OBJECTS = $(patsubst $(SRC_DIR)/%.c,$(BUILD_DIR)/%.o,$(C_SOURCES))
ASM_OBJECTS = $(patsubst $(SRC_DIR)/%.S,$(BUILD_DIR)/%.o,$(ASM_SOURCES))
OBJECTS = $(ASM_OBJECTS) $(C_OBJECTS)

# Output
TARGET = kernel8.img
ELF = $(BUILD_DIR)/kernel8.elf

.PHONY: all clean

all: $(BUILD_DIR) $(TARGET)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(TARGET): $(ELF)
	$(OBJCOPY) -O binary $(ELF) $(TARGET)
	@echo "Built $(TARGET) successfully"

$(ELF): $(OBJECTS)
	$(LD) $(LDFLAGS) -o $(ELF) $(OBJECTS)

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.S
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -rf $(BUILD_DIR) $(TARGET)

.PHONY: help
help:
	@echo "LibreDash Build System"
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all     - Build the kernel image (default)"
	@echo "  clean   - Remove build artifacts"
	@echo "  help    - Show this help message"
