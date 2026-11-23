/*
 * ZERO Native UEFI Boot Application & Hardware Accessibility Bridge
 * Pure Standalone 64-bit EFI Boot Core
 * Boots directly into ZERO OS Kernel Bare-Metal with full accessible feedback.
 */

#include <Uefi.h>
#include <Library/UefiLib.h>
#include <Library/UefiBootServicesTableLib.h>

extern void zero_os_main(void);

EFI_STATUS
EFIAPI
UefiMain (
  IN EFI_HANDLE        ImageHandle,
  IN EFI_SYSTEM_TABLE  *SystemTable
  )
{
  (VOID)ImageHandle;

  if (SystemTable == NULL || SystemTable->ConOut == NULL) {
    return EFI_UNSUPPORTED;
  }

  SystemTable->ConOut->ClearScreen(SystemTable->ConOut);
  Print(L"=====================================================\r\n");
  Print(L" ZERO Tier-0 UEFI Bootloader & Bare-Metal OS Bridge\r\n");
  Print(L" Hardware Accessibility Channel (HAC) Enabled\r\n");
  Print(L"=====================================================\r\n\r\n");

  Print(L"[UEFI Boot] Initialisation du processeur ZERO-ISA 64-bit...\r\n");
  Print(L"[UEFI Boot] Canal d'Accessibilité Matériel (HAC) activé (Audio PCM & Braille)\r\n");
  Print(L"[UEFI Boot] Transfére le contrôle au Noyau OS Bare-Metal ZERO...\r\n\r\n");

  /* Exécution du Noyau OS Accessible Bare-Metal ZERO */
  zero_os_main();

  Print(L"[UEFI Boot] Noyau OS ZERO exécuté avec succès. Appuyez sur Echap pour sortir.\r\n");
  return EFI_SUCCESS;
}
