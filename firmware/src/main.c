#include "entropy_vault.h"

#include <stdint.h>
#include <stdio.h>

int main(void) {
    uint8_t sample[32];

    adc_init();
    sensors_init();
    uart_init(115200);
    entropy_init((uint32_t)adc_read_noise() ^ ((uint32_t)sensors_read_jitter() << 16));

    entropy_fill(sample, sizeof(sample));

    printf("Entropy Vault firmware simulator\n");
    printf("sample=");
    uart_write_hex(sample, sizeof(sample));
    printf("\n");

    return 0;
}
