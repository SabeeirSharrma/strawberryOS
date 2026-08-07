#!/usr/bin/env bash
# run-vm.sh — Boot Strawberry OS ISO in a QEMU/KVM virtual machine.
#
# Usage: ./scripts/run-vm.sh [--install]
#   --install  Boot into the installer (Calamares) instead of live env

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
ISO_PATH=$(ls "$PROJECT_DIR"/archiso/out/*.iso 2>/dev/null | head -1)
VM_DIR="$PROJECT_DIR/vm"
DISK_PATH="$VM_DIR/strawberry.qcow2"
MEMORY="4096"
CPUS="4"
EFI_VARS="$VM_DIR/OVMF_VARS.fd"

# Find ISO
if [ -z "$ISO_PATH" ]; then
    echo "ERROR: No ISO found in archiso/out/"
    echo "Run: sudo ./scripts/build-iso.sh"
    exit 1
fi

echo "=== Strawberry OS VM Launcher ==="
echo "ISO:   $ISO_PATH"
echo "Disk:  $DISK_PATH"
echo "RAM:   ${MEMORY}MB"
echo "CPUs:  $CPUS"
echo ""

# Create VM directory
mkdir -p "$VM_DIR"

# Create disk image if it doesn't exist (20GB qcow2)
if [ ! -f "$DISK_PATH" ]; then
    echo "Creating 20GB disk image..."
    qemu-img create -f qcow2 "$DISK_PATH" 20G
fi

# Copy OVMF vars for UEFI if not present
if [ ! -f "$EFI_VARS" ]; then
    if [ -f /usr/share/edk2/x64/OVMF_VARS.4m.fd ]; then
        cp /usr/share/edk2/x64/OVMF_VARS.4m.fd "$EFI_VARS"
    elif [ -f /usr/share/edk2/x64/OVMF_VARS.fd ]; then
        cp /usr/share/edk2/x64/OVMF_VARS.fd "$EFI_VARS"
    else
        echo "WARNING: OVMF_VARS.fd not found, falling back to BIOS boot"
        EFI_VARS=""
    fi
fi

# Build QEMU command
QEMU_ARGS=(
    -machine q35,accel=kvm
    -cpu host
    -m "$MEMORY"
    -smp "$CPUS"
    -drive "if=pflash,format=raw,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd"
    -cdrom "$ISO_PATH"
    -drive "file=$DISK_PATH,format=qcow2,if=virtio"
    -net nic,model=virtio
    -net user,hostfwd=tcp::2222-:22
    -vga virtio
    -display gtk
    -usb -device usb-tablet
)

# Add EFI vars if available
if [ -n "$EFI_VARS" ]; then
    QEMU_ARGS+=(-drive "if=pflash,format=raw,file=$EFI_VARS")
fi

echo "Launching QEMU..."
echo "  SSH forwarding: localhost:2222 -> VM:22"
echo "  Press Ctrl+Alt+G to release mouse"
echo ""

exec qemu-system-x86_64 "${QEMU_ARGS[@]}" "$@"
