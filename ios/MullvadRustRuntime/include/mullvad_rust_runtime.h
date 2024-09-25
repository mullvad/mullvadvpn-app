// This file is generated automatically. To update it forcefully, run `cargo run -p mullvad-ios --target aarch64-apple-ios`.

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct EncryptedDnsProxyState EncryptedDnsProxyState;

typedef struct ProxyHandle {
  void *context;
  uint16_t port;
} ProxyHandle;

typedef struct EphemeralPeerCancelToken {
  void *context;
} EphemeralPeerCancelToken;

extern const uint16_t CONFIG_SERVICE_PORT;

/**
 * Initializes a valid pointer to an instance of `EncryptedDnsProxyState`.
 */
struct EncryptedDnsProxyState *encrypted_dns_proxy_init(void);

/**
 * This must be called only once to deallocate `EncryptedDnsProxyState`.
 *
 * # Safety
 * `ptr` must be a valid, exclusive pointer to `EncryptedDnsProxyState`, initialized
 * by `encrypted_dns_proxy_init`. This function is not thread safe, and should only be called
 * once.
 */
void encrypted_dns_proxy_free(struct EncryptedDnsProxyState *ptr);

/**
 * # Safety
 * encrypted_dns_proxy must be a valid, exclusive pointer to `EncryptedDnsProxyState`, initialized
 * by `encrypted_dns_proxy_init`. This function is not thread safe.
 * `proxy_handle` must be pointing to a valid memory region for the size of a `ProxyHandle`. This
 * function is not thread safe, but it can be called repeatedly. Each successful invocation should
 * clean up the resulting proxy via `[encrypted_dns_proxy_stop]`.
 *
 * `proxy_handle` will only contain valid values if the return value is zero. It is still valid to
 * deallocate the memory.
 */
int32_t encrypted_dns_proxy_start(struct EncryptedDnsProxyState *encrypted_dns_proxy,
                                  struct ProxyHandle *proxy_handle);

/**
 * # Safety
 * `proxy_config` must be a valid pointer to a `ProxyHandle` as initialized by
 * [`encrypted_dns_proxy_start`]. It should only ever be called once.
 */
int32_t encrypted_dns_proxy_stop(struct ProxyHandle *proxy_config);

/**
 * Called by the Swift side to signal that the ephemeral peer exchange should be cancelled.
 * After this call, the cancel token is no longer valid.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
 * `PacketTunnelProvider`.
 */
void cancel_ephemeral_peer_exchange(const struct EphemeralPeerCancelToken *sender);

/**
 * Called by the Swift side to signal that the Rust `EphemeralPeerCancelToken` can be safely dropped
 * from memory.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
 * `PacketTunnelProvider`.
 */
void drop_ephemeral_peer_exchange_token(const struct EphemeralPeerCancelToken *sender);

/**
 * Called by Swift whenever data has been written to the in-tunnel TCP connection when exchanging
 * quantum-resistant pre shared keys, or ephemeral peers.
 *
 * If `bytes_sent` is 0, this indicates that the connection was closed or that an error occurred.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `write_tx` created by the `IosTcpProvider`
 * Callback to call when the TCP connection has written data.
 */
void handle_sent(uintptr_t bytes_sent, const void *sender);

/**
 * Called by Swift whenever data has been read from the in-tunnel TCP connection when exchanging
 * quantum-resistant pre shared keys, or ephemeral peers.
 *
 * If `data` is null or empty, this indicates that the connection was closed or that an error
 * occurred. An empty buffer is sent to the underlying reader to signal EOF.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `read_tx` created by the `IosTcpProvider`
 *
 * Callback to call when the TCP connection has received data.
 */
void handle_recv(const uint8_t *data, uintptr_t data_len, const void *sender);

/**
 * Entry point for requesting ephemeral peers on iOS.
 * The TCP connection must be created to go through the tunnel.
 * # Safety
 * `public_key` and `ephemeral_key` must be valid respective `PublicKey` and `PrivateKey` types.
 * They will not be valid after this function is called, and thus must be copied here.
 * `packet_tunnel` and `tcp_connection` must be valid pointers to a packet tunnel and a TCP
 * connection instances.
 * `cancel_token` should be owned by the caller of this function.
 */
int32_t request_ephemeral_peer(const uint8_t *public_key,
                               const uint8_t *ephemeral_key,
                               const void *packet_tunnel,
                               const void *tcp_connection,
                               struct EphemeralPeerCancelToken *cancel_token,
                               uint64_t peer_exchange_timeout,
                               bool enable_post_quantum,
                               bool enable_daita);

/**
 * Called when there is data to send on the TCP connection.
 * The TCP connection must write data on the wire, then call the `handle_sent` function.
 */
extern void swift_nw_tcp_connection_send(const void *connection,
                                         const void *data,
                                         uintptr_t data_len,
                                         const void *sender);

/**
 * Called when there is data to read on the TCP connection.
 * The TCP connection must read data from the wire, then call the `handle_read` function.
 */
extern void swift_nw_tcp_connection_read(const void *connection, const void *sender);

/**
 * Called when the preshared post quantum key is ready,
 * or when a Daita peer has been successfully requested.
 * `raw_preshared_key` will be NULL if:
 * - The post quantum key negotiation failed
 * - A Daita peer has been requested without enabling post quantum keys.
 */
extern void swift_ephemeral_peer_ready(const void *raw_packet_tunnel,
                                       const uint8_t *raw_preshared_key,
                                       const uint8_t *raw_ephemeral_private_key);

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

int32_t start_tunnel_obfuscator_proxy(const uint8_t *peer_address,
                                      uintptr_t peer_address_len,
                                      uint16_t peer_port,
                                      struct ProxyHandle *proxy_handle);

int32_t stop_tunnel_obfuscator_proxy(struct ProxyHandle *proxy_handle);
