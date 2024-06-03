#include <stdint.h>
#include <stdbool.h>

/// Activate DAITA for the specified tunnel.
bool wgActivateDaita(int8_t* machines, int32_t tunnelHandle, uint32_t eventsCapacity, uint32_t actionsCapacity);
char* wgGetConfig(int32_t tunnelHandle);
int32_t wgSetConfig(int32_t tunnelHandle, char* cSettings);
void wgFreePtr(void*);
