#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ProxyHandle {
  void *context;
  uint16_t port;
} ProxyHandle;

int32_t start_tunnel_obfuscator_proxy(const uint8_t *peer_address,
                                      uintptr_t peer_address_len,
                                      uint16_t peer_port,
                                      struct ProxyHandle *proxy_handle);

int32_t stop_tunnel_obfuscator_proxy(struct ProxyHandle *proxy_handle);
