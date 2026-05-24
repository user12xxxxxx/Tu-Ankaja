#include "entropy_vault.h"

#include <stdint.h>
#include <time.h>

void sensors_init(void) {
}

uint16_t sensors_read_jitter(void) {
    struct timespec ts;
    timespec_get(&ts, TIME_UTC);
    return (uint16_t)((uint64_t)ts.tv_nsec ^ ((uint64_t)ts.tv_sec << 7));
}
