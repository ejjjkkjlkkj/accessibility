#include <Uefi.h>
#include <Library/UefiLib.h>
#include <Library/UefiBootServicesTableLib.h>

/*
 * UEFI keyboard-navigation milestone.
 * This provides a small menu through the firmware text console.
 * It is NOT a complete screen reader: no TTS/audio, HII/IFR parser,
 * arbitrary firmware-form navigation, braille, or standalone OS kernel.
 */

STATIC
VOID
RenderMenu (
  IN UINTN Selected
  )
{
  Print(L"\r\nAccessibility UEFI prototype - clavier\r\n");
  Print(L"Utilisez Fleche haut/bas, Entree, Echap.\r\n\r\n");
  Print(L"%s Lire les commandes\r\n", Selected == 0 ? L">" : L" ");
  Print(L"%s A propos\r\n", Selected == 1 ? L">" : L" ");
  Print(L"%s Quitter\r\n", Selected == 2 ? L">" : L" ");
  Print(L"\r\nAudio/TTS : non implemente.\r\n");
}

STATIC
VOID
WaitForAnyKey (
  VOID
  )
{
  EFI_INPUT_KEY Key;
  UINTN EventIndex;

  if (!EFI_ERROR(gBS->WaitForEvent(1, &gST->ConIn->WaitForKey, &EventIndex))) {
    gST->ConIn->ReadKeyStroke(gST->ConIn, &Key);
  }
}

EFI_STATUS
EFIAPI
UefiMain (
  IN EFI_HANDLE        ImageHandle,
  IN EFI_SYSTEM_TABLE  *SystemTable
  )
{
  EFI_STATUS Status;
  EFI_INPUT_KEY Key;
  UINTN Selected;
  UINTN EventIndex;

  (VOID)ImageHandle;
  (VOID)SystemTable;
  Selected = 0;

  if (gST == NULL || gST->ConIn == NULL || gST->ConOut == NULL || gBS == NULL) {
    return EFI_UNSUPPORTED;
  }

  for (;;) {
    gST->ConOut->ClearScreen(gST->ConOut);
    RenderMenu(Selected);

    Status = gBS->WaitForEvent(1, &gST->ConIn->WaitForKey, &EventIndex);
    if (EFI_ERROR(Status)) {
      Print(L"\r\nErreur attente clavier: %r\r\n", Status);
      return Status;
    }

    Status = gST->ConIn->ReadKeyStroke(gST->ConIn, &Key);
    if (EFI_ERROR(Status)) {
      continue;
    }

    if (Key.ScanCode == SCAN_UP) {
      Selected = (Selected + 2) % 3;
    } else if (Key.ScanCode == SCAN_DOWN) {
      Selected = (Selected + 1) % 3;
    } else if (Key.UnicodeChar == CHAR_ESC) {
      return EFI_SUCCESS;
    } else if (Key.UnicodeChar == CHAR_CARRIAGE_RETURN) {
      if (Selected == 0) {
        gST->ConOut->ClearScreen(gST->ConOut);
        Print(L"Commandes clavier disponibles :\r\n");
        Print(L"Fleche haut/bas : changer l'element selectionne.\r\n");
        Print(L"Entree : activer l'element. Echap : quitter.\r\n");
        Print(L"Appuyez sur une touche pour revenir au menu.\r\n");
        WaitForAnyKey();
      } else if (Selected == 1) {
        gST->ConOut->ClearScreen(gST->ConOut);
        Print(L"Prototype de navigation clavier UEFI.\r\n");
        Print(L"Ce prototype n'est pas un lecteur d'ecran complet.\r\n");
        Print(L"Pas de voix, audio, HII/IFR, braille ni noyau autonome.\r\n");
        Print(L"Appuyez sur une touche pour revenir au menu.\r\n");
        WaitForAnyKey();
      } else {
        return EFI_SUCCESS;
      }
    }
  }
}