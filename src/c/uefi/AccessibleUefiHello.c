#include <Uefi.h>
#include <Library/UefiLib.h>
#include <Library/UefiBootServicesTableLib.h>

/*
 * First UEFI application milestone: prove entry point and console output.
 * This is NOT a screen reader, does not produce audio, and is not an OS kernel.
 */
EFI_STATUS
EFIAPI
UefiMain (
  IN EFI_HANDLE        ImageHandle,
  IN EFI_SYSTEM_TABLE  *SystemTable
  )
{
  (VOID)ImageHandle;
  (VOID)SystemTable;

  Print(L"Accessibility UEFI prototype: console-only milestone.\r\n");
  Print(L"Audio output and accessible firmware navigation are not implemented.\r\n");
  return EFI_SUCCESS;
}
