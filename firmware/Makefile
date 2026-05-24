CC ?= cc
CFLAGS ?= -std=c11 -Wall -Wextra -pedantic -Iinclude

SRC := src/main.c src/adc/adc.c src/sensors/sensors.c src/entropy/entropy.c src/uart/uart.c
BIN := build/entropy-vault-firmware

.PHONY: all run clean

all: $(BIN)

$(BIN): $(SRC)
	@mkdir -p build
	$(CC) $(CFLAGS) $(SRC) -o $(BIN)

run: $(BIN)
	./$(BIN)

clean:
	rm -rf build
