// This file is generated automatically. To update it forcefully, run `cargo run -p mullvad-ios --target aarch64-apple-ios`.

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * WireGuard overhead. Size of UDP header, plus header and footer of a WireGuard data packet.
 */
#define WIREGUARD_OVERHEAD (8 + 32)

typedef struct ExchangeCancelToken ExchangeCancelToken;

typedef struct LogRedactor LogRedactor;

typedef struct SwiftData {
  void *ptr;
} SwiftData;

typedef struct SwiftServerMock {
  const void *server_ptr;
  const void *mock_ptr;
  uint16_t port;
} SwiftServerMock;

typedef struct DaitaParameters {
  uint8_t *machines;
  double max_padding_frac;
  double max_blocking_frac;
} DaitaParameters;

typedef struct WgTcpConnectionFunctions {
  int32_t (*open_fn)(int32_t tunnel_handle, const char *address, uint64_t timeout);
  int32_t (*close_fn)(int32_t tunnel_handle, int32_t socket_handle);
  int32_t (*recv_fn)(int32_t tunnel_handle, int32_t socket_handle, uint8_t *data, int32_t len);
  int32_t (*send_fn)(int32_t tunnel_handle, int32_t socket_handle, const uint8_t *data, int32_t len);
} WgTcpConnectionFunctions;

typedef struct EphemeralPeerParameters {
  uint64_t peer_exchange_timeout;
  bool enable_post_quantum;
  bool enable_daita;
  struct WgTcpConnectionFunctions funcs;
} EphemeralPeerParameters;

/**
 * Callback function type for logging.
 * - `context`: Opaque pointer to a Swift Logger instance, passed back on each invocation.
 * - `level`: The log level (1=Error, 2=Warn, 3=Info, 4=Debug, 5=Trace)
 * - `target`: Null-terminated UTF-8 string containing the module/target name
 * - `message`: Null-terminated UTF-8 string containing the log message
 *
 * # Thread safety
 * This callback may be invoked concurrently from any thread.
 */
typedef void (*LogCallback)(void *context, uint8_t level, const char *target, const char *message);

typedef struct ProxyHandle {
  void *context;
  uint16_t port;
} ProxyHandle;

extern const uint16_t CONFIG_SERVICE_PORT;

extern void swift_store_address_cache(const uint8_t *data, uint64_t data_size);

extern struct SwiftData swift_read_address_cache(void);

char *get_shadowsocks_chipers(void);

/**
 * Deallocates a CString returned by the Mullvad API client.
 *
 * # Safety
 *
 * `cstr_ptr` must be a pointer to a string allocated by another `mullvad_api` function.
 */
void mullvad_api_cstring_drop(char *cstr_ptr);

/**
 * # Safety
 *
 * `method` must be a pointer to a null terminated string representing the http method.
 *
 * `path` must be a pointer to a null terminated string representing the url path.
 *
 * `response_code` must be a usize representing the http response code.
 *
 * `response_body` must be a pointer to a null terminated string representing the body.
 *
 * This function is safe.
 */
struct SwiftServerMock mullvad_api_mock_get(const char *path,
                                            uintptr_t response_code,
                                            const uint8_t *response_body);

/**
 * # Safety
 *
 * `path` must be a pointer to a null terminated string representing the url path.
 *
 * `response_code` must be a usize representing the http response code.
 *
 * `match_body` must be a pointer to a null terminated json string representing the body the server expects.
 *
 * This function is safe.
 */
struct SwiftServerMock mullvad_api_mock_post(const char *path,
                                             uintptr_t response_code,
                                             const char *match_body);

/**
 * Called by the Swift side to signal that the Rust `SwiftServerMock` can be safely
 * dropped from memory.
 *
 * # Safety
 *
 * `mock_ptr` must be pointing to a valid instance of `SwiftServerMock`. This function
 * is not safe to call multiple times with the same `SwiftServerMock`.
 */
void mullvad_api_mock_drop(struct SwiftServerMock mock_ptr);

extern uint8_t *swift_data_get_ptr(const struct SwiftData *data);

extern uintptr_t swift_data_get_len(const struct SwiftData *data);

extern void swift_data_drop(struct SwiftData *data);

/**
 * To be called when ephemeral peer exchange has finished. All parameters except
 * `raw_packet_tunnel` are optional.
 *
 * # Safety:
 * If the key exchange failed, all pointers except `raw_packet_tunnel` must be null. If the
 * key exchange was successful, `raw_ephemeral_private_key` must be a valid pointer to 32
 * bytes for the lifetime of this call. If PQ was enabled, `raw_preshared_key` must be a valid
 * pointer to 32 bytes for the lifetime of this call. If DAITA was requested, the
 * `daita_prameters` must point to a valid instance of `DaitaParameters`.
 */
extern void swift_ephemeral_peer_ready(const void *raw_packet_tunnel,
                                       const uint8_t *raw_preshared_key,
                                       const uint8_t *raw_ephemeral_private_key,
                                       const struct DaitaParameters *daita_parameters);

/**
 * Called by the Swift side to signal that the ephemeral peer exchange should be cancelled.
 * After this call, the cancel token is no longer valid.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
 * `PacketTunnelProvider`.
 */
void cancel_ephemeral_peer_exchange(struct ExchangeCancelToken *sender);

/**
 * Called by the Swift side to signal that the Rust `EphemeralPeerCancelToken` can be safely
 * dropped from memory.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `EphemeralPeerCancelToken` created by the
 * `PacketTunnelProvider`.
 */
void drop_ephemeral_peer_exchange_token(struct ExchangeCancelToken *sender);

/**
 * Entry point for requesting ephemeral peers on iOS.
 * The TCP connection must be created to go through the tunnel.
 * # Safety
 * `public_key` and `ephemeral_key` must be valid respective `PublicKey` and `PrivateKey` types,
 * specifically, they must be valid pointers to 32 bytes. They will not be valid after this
 * function is called, and thus must be copied here. `packet_tunnel` must be valid pointers to a
 * packet tunnel, the packet tunnel pointer must outlive the ephemeral peer exchange.
 * `cancel_token` should be owned by the caller of this function.
 */
struct ExchangeCancelToken *request_ephemeral_peer(const uint8_t *public_key,
                                                   const uint8_t *ephemeral_key,
                                                   const void *packet_tunnel,
                                                   int32_t tunnel_handle,
                                                   struct EphemeralPeerParameters peer_parameters);

/**
 * Create a new log redactor with the given container paths.
 *
 * # Safety
 * - `paths` must be a valid pointer to an array of `paths_count` pointers to null-terminated
 *   UTF-8 strings, or null if `paths_count` is 0.
 * - The returned pointer must be freed by calling `log_redactor_free`.
 */
struct LogRedactor *create_log_redactor(const char *const *paths, uintptr_t paths_count);

/**
 * Redact sensitive information from a string using the given redactor.
 *
 * # Safety
 * - `redactor` must be a valid pointer returned by `create_log_redactor`.
 * - `input` must be a valid pointer to a null-terminated UTF-8 string.
 * - The returned pointer must be freed by calling `log_redactor_free_string`.
 */
char *log_redactor_redact(const struct LogRedactor *redactor, const char *input);

/**
 * Free a string returned by `log_redactor_redact`.
 *
 * # Safety
 * - `ptr` must be a pointer returned by `log_redactor_redact`, or null.
 * - `ptr` must not have been freed before.
 */
void log_redactor_free_string(char *ptr);

/**
 * Add a custom string to the log redactor.
 *
 * # Safety
 * - `redactor` must be a valid pointer returned by `create_log_redactor`.
 * - `input` must be a valid pointer to a null-terminated UTF-8 string.
 */
void log_redactor_add_custom_string(struct LogRedactor *redactor, const char *input);

/**
 * Free a log redactor created by `create_log_redactor`.
 *
 * # Safety
 * - `redactor` must be a pointer returned by `create_log_redactor`, or null.
 * - `redactor` must not have been freed before.
 * - `redactor` must not be used after this call.
 */
void log_redactor_free(struct LogRedactor *redactor);

/**
 * Initialize the Rust logger with a Swift callback and context.
 *
 * The `context` pointer is passed back to `callback` on each log event, allowing
 * the Swift side to recover a Logger instance without relying on global state.
 *
 * # Safety
 * - `callback` must be a valid function pointer that remains valid for the lifetime of the program.
 * - `context` must be a valid pointer that remains valid for the lifetime of the program.
 * - This function is safe to call multiple times, but only the first call will have an effect.
 */
void init_rust_logging(LogCallback callback, void *context);

/**
 * Start a udp2tcp obfuscator proxy.
 *
 * # SAFETY
 * `peer_address` must be a valid pointer to `peer_address_len` bytes, these bytes will be
 * interpreted  as an IP address. `proxy_handle` must be a valid pointer for a `ProxyHandle`
 * struct. This function will initialize `proxy_handle` to contain a valid `ProxyHandle` instance.
 */
int32_t start_udp2tcp_obfuscator_proxy(const uint8_t *peer_address,
                                       uintptr_t peer_address_len,
                                       uint16_t peer_port,
                                       struct ProxyHandle *proxy_handle);

int32_t start_shadowsocks_obfuscator_proxy(const uint8_t *peer_address,
                                           uintptr_t peer_address_len,
                                           uint16_t peer_port,
                                           struct ProxyHandle *proxy_handle);

int32_t start_quic_obfuscator_proxy(const uint8_t *peer_address,
                                    uintptr_t peer_address_len,
                                    uint16_t peer_port,
                                    const char *hostname,
                                    const char *token,
                                    struct ProxyHandle *proxy_handle);

int32_t start_lwo_obfuscator_proxy(const uint8_t *peer_address,
                                   uintptr_t peer_address_len,
                                   uint16_t peer_port,
                                   const uint8_t *client_public_key,
                                   const uint8_t *server_public_key,
                                   struct ProxyHandle *proxy_handle);

int32_t stop_tunnel_obfuscator_proxy(struct ProxyHandle *proxy_handle);

/**
 * Generate a new random WireGuard private key, writing 32 bytes to `key_out`.
 * This function is safe to call concurrently with different pointers. Not safe to call
 * concurrently with the same pointers.
 *
 * # Safety
 * `key_out` must be a valid pointer to a 32-byte buffer.
 */
void mullvad_generate_private_key(uint8_t *key_out);

/**
 * Derive a WireGuard public key from a private key.
 * This function is safe to call concurrently if different parameters are used.
 *
 * # Safety
 * `private_key` must be a valid pointer to 32 bytes.
 * `public_key_out` must be a valid pointer to a 32-byte buffer.
 */
void mullvad_derive_public_key(const uint8_t *private_key, uint8_t *public_key_out);
