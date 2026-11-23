/*
 * Intel High Definition Audio (HDA) Bare-Metal Driver Stub & Specification Alignment
 * Document Reference: Intel HD Audio Controller Specification Rev 1.0a
 */

#include <stdint.h>

#define HDA_GCAP    0x00  /* Global Capabilities (16-bit) */
#define HDA_GCTL    0x08  /* Global Control (32-bit) */
#define HDA_WAKEEN  0x0C  /* Wake Enable (16-bit) */
#define HDA_STATESTS 0x0E /* State Status (16-bit) */
#define HDA_CORBLBASE 0x40 /* CORB Lower Base Address (32-bit) */
#define HDA_CORBUBASE 0x44 /* CORB Upper Base Address (32-bit) */
#define HDA_RIRBLBASE 0x50 /* RIRB Lower Base Address (32-bit) */
#define HDA_RIRBUBASE 0x54 /* RIRB Upper Base Address (32-bit) */

typedef struct {
    uintptr_t mmio_base;
    uint8_t   reset_status;
} IntelHdaController;

void intel_hda_init(IntelHdaController *hda, uintptr_t mmio_base) {
    hda->mmio_base = mmio_base;
    hda->reset_status = 1;
}

uint16_t intel_hda_read_capabilities(IntelHdaController *hda) {
    if (hda->mmio_base == 0) return 0;
    volatile uint16_t *gcap = (volatile uint16_t *)(hda->mmio_base + HDA_GCAP);
    return *gcap;
}
