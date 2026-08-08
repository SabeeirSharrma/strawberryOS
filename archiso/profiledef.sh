#!/usr/bin/env bash
# shellcheck disable=SC2034
# Strawberry OS — archiso profile definition
# Based on the official releng profile, stripped to headless mining baseline.

iso_name="strawberry-os"
iso_label="STRAWBERRY_$(date +%Y%m)"
iso_publisher="Strawberry OS Contributors"
iso_application="Strawberry OS - Headless Mining ISO"
iso_version="$(date +%Y.%m.%d)"
install_dir="arch"
buildmodes=('iso')
bootmodes=('bios.syslinux'
           'uefi.systemd-boot')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '19')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/usr/local/bin/strawberry-cli"]="0:0:755"
)
