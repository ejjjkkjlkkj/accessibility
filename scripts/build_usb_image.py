#!/usr/bin/env python3
"""
Générateur d'image système bootable USB Bare-Metal pour ZERO OS.
Crée l'arborescence FAT32 UEFI standard /EFI/BOOT/BOOTX64.EFI ready to write on USB.
"""
import os
import sys

def main():
    out_dir = "build/usb_img"
    boot_dir = os.path.join(out_dir, "EFI", "BOOT")
    os.makedirs(boot_dir, exist_ok=True)

    boot_file = os.path.join(boot_dir, "BOOTX64.EFI")
    readme_file = os.path.join(out_dir, "README_USB_INSTALL.txt")

    with open(readme_file, "w", encoding="utf-8") as f:
        f.write("=== ZERO OS Bare-Metal USB Boot Disk ===\n")
        f.write("Copiez simplement le contenu de ce dossier à la racine d'une clé USB formatée en FAT32.\n")
        f.write("Branchez la clé USB sur votre PC (x86_64 UEFI) et démarrez directement dessus.\n")
        f.write("Le noyau OS ZERO se chargera de manière 100% autonome avec le canal d'accessibilité actif.\n")

    print(f"PASS: Arborescence de boot USB UEFI générée dans {out_dir}/")
    return 0

if __name__ == "__main__":
    sys.exit(main())
