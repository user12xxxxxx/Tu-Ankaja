#ifndef ENTROPY_VAULT_H
#define ENTROPY_VAULT_H

#include <stdint.h>
#include <stddef.h>

void adc_init(void);
uint16_t adc_read_noise(void);

void sensors_init(void);
uint16_t sensors_read_jitter(void);

void entropy_init(uint32_t seed);
uint8_t entropy_next_byte(void);
void entropy_fill(uint8_t *buffer, size_t length);

void uart_init(uint32_t baud_rate);
void uart_write_hex(const uint8_t *buffer, size_t length);

#endif
