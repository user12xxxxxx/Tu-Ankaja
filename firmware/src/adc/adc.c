#include "entropy_vault.h"

#include <stdint.h>
#include <time.h>

static uint32_t adc_state = 0xA5A55A5A;

void adc_init(void) {
    adc_state ^= (uint32_t)time(NULL);
}

uint16_t adc_read_noise(void) {
    adc_state ^= adc_state << 13;
    adc_state ^= adc_state >> 17;
    adc_state ^= adc_state << 5;
    return (uint16_t)(adc_state & 0x03FFu);
}
