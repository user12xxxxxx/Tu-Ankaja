#include "entropy_vault.h"

#include <stddef.h>
#include <stdint.h>

static uint32_t entropy_state = 0x6D2B79F5;

void entropy_init(uint32_t seed) {
    entropy_state ^= seed ? seed : 0xC001D00Du;
}

uint8_t entropy_next_byte(void) {
    uint32_t mixed = entropy_state ^ adc_read_noise() ^ ((uint32_t)sensors_read_jitter() << 11);
    mixed += 0x9E3779B9u;
    mixed ^= mixed >> 16;
    mixed *= 0x85EBCA6Bu;
    mixed ^= mixed >> 13;
    mixed *= 0xC2B2AE35u;
    mixed ^= mixed >> 16;
    entropy_state = mixed;
    return (uint8_t)(mixed & 0xFFu);
}

void entropy_fill(uint8_t *buffer, size_t length) {
    for (size_t i = 0; i < length; i++) {
        buffer[i] = entropy_next_byte();
    }
}
