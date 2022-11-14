# 1. fetch and build limine
git clone https://github.com/limine-bootloader/limine.git --branch=v3.0-branch-binary --depth=1
make -C limine

# 2. build the iso file
rm -rf iso_root
mkdir -p iso_root
cp target/x86_64-unknown-none/debug/viperstem limine.cfg limine/limine.sys limine/limine-cd.bin limine/limine-cd-efi.bin iso_root/
xorriso -as mkisofs -b limine-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot limine-cd-efi.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    iso_root -o viperstem.iso
limine/limine-deploy viperstem.iso
rm -rf iso_root
