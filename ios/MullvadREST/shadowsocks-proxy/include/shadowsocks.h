#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ProxyHandle {
  void *context;
  uint16_t port;
} ProxyHandle;

int32_t start_shadowsocks_proxy(const uint8_t *addr,
                                uintptr_t addr_len,
                                uint16_t port,
                                const uint8_t *password,
                                uintptr_t password_len,
                                const uint8_t *cipher,
                                uintptr_t cipher_len,
                                struct ProxyHandle *proxy_config);

int32_t stop_shadowsocks_proxy(struct ProxyHandle *proxy_config);
