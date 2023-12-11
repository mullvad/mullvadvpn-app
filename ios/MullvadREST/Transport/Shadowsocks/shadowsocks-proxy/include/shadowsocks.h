#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ProxyHandle {
  void *context;
  uint16_t port;
} ProxyHandle;

/**
 * # Safety
 * `addr`, `password`, `cipher` must be valid for the lifetime of this function call and they must
 * be backed by the amount of bytes as stored in the respective `*_len` parameters.
 *
 * `proxy_config` must be pointing to a valid memory region for the size of a `ProxyHandle`
 * instance.
 */
int32_t start_shadowsocks_proxy(const uint8_t *forward_address,
                                uintptr_t forward_address_len,
                                uint16_t forward_port,
                                const uint8_t *addr,
                                uintptr_t addr_len,
                                uint16_t port,
                                const uint8_t *password,
                                uintptr_t password_len,
                                const uint8_t *cipher,
                                uintptr_t cipher_len,
                                struct ProxyHandle *proxy_config);

/**
 * # Safety
 * `proxy_config` must be pointing to a valid instance of a `ProxyInstance`, as instantiated by
 * `start_shadowsocks_proxy`.
 */
int32_t stop_shadowsocks_proxy(struct ProxyHandle *proxy_config);
