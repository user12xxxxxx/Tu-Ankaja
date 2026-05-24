#include "entropy_vault.h"

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

void uart_init(uint32_t baud_rate) {
    (void)baud_rate;
}

void uart_write_hex(const uint8_t *buffer, size_t length) {
    for (size_t i = 0; i < length; i++) {
        printf("%02x", buffer[i]);
    }
}
