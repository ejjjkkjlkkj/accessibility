/** Read-only UEFI PCI inventory via EFI_PCI_IO_PROTOCOL.
 *
 * This application reads identifiers only. It does not write PCI configuration,
 * map ECAM/MMIO, access I/O ports, perform DMA, change firmware variables, or
 * request a reboot. The firmware protocol is the only PCI access path used here.
 */
#include <Uefi.h>
#include <Library/UefiApplicationEntryPoint.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/UefiLib.h>
#include <Library/MemoryAllocationLib.h>
#include <Protocol/PciIo.h>

EFI_STATUS
EFIAPI
UefiMain (
  IN EFI_HANDLE        ImageHandle,
  IN EFI_SYSTEM_TABLE  *SystemTable
  )
{
  EFI_STATUS  Status;
  EFI_HANDLE  *Handles;
  UINTN       HandleCount;
  UINTN       Index;
  UINTN       Found;
  UINTN       ReadErrors;

  (VOID)ImageHandle;
  (VOID)SystemTable;
  Handles = NULL;
  HandleCount = 0;
  Found = 0;
  ReadErrors = 0;

  Print (L"Accessibility Hardware Bridge - read-only UEFI PCI inventory\r\n");
  Print (L"Evidence: firmware-reported; not a cryptographic attestation.\r\n");

  Status = gBS->LocateHandleBuffer (
                  ByProtocol,
                  &gEfiPciIoProtocolGuid,
                  NULL,
                  &HandleCount,
                  &Handles
                  );
  if (EFI_ERROR (Status)) {
    Print (L"FAIL: LocateHandleBuffer(PciIo): %r\r\n", Status);
    return Status;
  }

  for (Index = 0; Index < HandleCount; ++Index) {
    EFI_PCI_IO_PROTOCOL  *PciIo;
    UINTN                Segment;
    UINTN                Bus;
    UINTN                Device;
    UINTN                Function;
    UINT16               VendorId;
    UINT16               DeviceId;
    UINT8                ClassBytes[3];

    PciIo = NULL;
    Status = gBS->HandleProtocol (
                    Handles[Index],
                    &gEfiPciIoProtocolGuid,
                    (VOID **)&PciIo
                    );
    if (EFI_ERROR (Status) || (PciIo == NULL)) {
      ++ReadErrors;
      continue;
    }

    Status = PciIo->GetLocation (
                      PciIo,
                      &Segment,
                      &Bus,
                      &Device,
                      &Function
                      );
    if (EFI_ERROR (Status)) {
      ++ReadErrors;
      continue;
    }

    VendorId = 0xFFFF;
    DeviceId = 0xFFFF;
    ClassBytes[0] = 0;
    ClassBytes[1] = 0;
    ClassBytes[2] = 0;

    Status = PciIo->Pci.Read (
                         PciIo,
                         EfiPciIoWidthUint16,
                         0x00,
                         1,
                         &VendorId
                         );
    if (EFI_ERROR (Status)) {
      ++ReadErrors;
      continue;
    }
    if (VendorId == 0xFFFF) {
      continue;
    }

    Status = PciIo->Pci.Read (
                         PciIo,
                         EfiPciIoWidthUint16,
                         0x02,
                         1,
                         &DeviceId
                         );
    if (EFI_ERROR (Status)) {
      ++ReadErrors;
      continue;
    }

    Status = PciIo->Pci.Read (
                         PciIo,
                         EfiPciIoWidthUint8,
                         0x09,
                         sizeof (ClassBytes),
                         ClassBytes
                         );
    if (EFI_ERROR (Status)) {
      ++ReadErrors;
      continue;
    }

    Print (
      L"PCI %04x:%02x:%02x.%x VID=%04x DID=%04x CLASS=%02x%02x%02x\r\n",
      Segment,
      Bus,
      Device,
      Function,
      VendorId,
      DeviceId,
      ClassBytes[2],
      ClassBytes[1],
      ClassBytes[0]
      );
    ++Found;
  }

  if (Handles != NULL) {
    FreePool (Handles);
  }

  Print (L"Summary: %u PCI functions reported; %u read errors.\r\n", Found, ReadErrors);
  Print (L"No PCI writes, MMIO mappings, DMA, firmware-variable writes, or reboot requested.\r\n");
  return (ReadErrors == 0) ? EFI_SUCCESS : EFI_WARN_UNKNOWN_GLYPH;
}
