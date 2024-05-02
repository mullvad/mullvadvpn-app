#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <stdbool.h>

typedef struct Event {
 uint8_t peer[32];
 uint32_t eventType;
 uint16_t xmitBytes;
} Event;

typedef struct Padding {
	uint16_t byteCount;
    bool replace;
} Padding;

typedef struct Action {
 uint8_t peer[32];
 uint32_t actionType;
 Padding padding;
} Action;