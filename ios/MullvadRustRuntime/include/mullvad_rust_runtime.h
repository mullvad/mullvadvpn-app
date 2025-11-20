// This file is generated automatically. To update it forcefully, run `cargo run -p mullvad-ios --target aarch64-apple-ios`.

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Used by Swift to instruct which access method kind it is trying to convert
 */
enum SwiftAccessMethodKind {
  KindDirect = 0,
  KindBridge,
  KindEncryptedDnsProxy,
  KindShadowsocks,
  KindSocks5Local,
};
typedef uint8_t SwiftAccessMethodKind;

typedef struct ApiContext ApiContext;

typedef struct ExchangeCancelToken ExchangeCancelToken;

typedef struct Map Map;

typedef struct RequestCancelHandle RequestCancelHandle;

typedef struct RetryStrategy RetryStrategy;

typedef struct SwiftAccessMethodSettingsContext SwiftAccessMethodSettingsContext;

typedef struct SwiftApiContext {
  const struct ApiContext *_0;
} SwiftApiContext;

typedef struct SwiftAccessMethodSettingsWrapper {
  struct SwiftAccessMethodSettingsContext *_0;
} SwiftAccessMethodSettingsWrapper;

typedef struct SwiftShadowsocksLoaderWrapperContext {
  const void *shadowsocks_loader;
} SwiftShadowsocksLoaderWrapperContext;

typedef struct SwiftShadowsocksLoaderWrapper {
  struct SwiftShadowsocksLoaderWrapperContext _0;
} SwiftShadowsocksLoaderWrapper;

typedef struct SwiftAddressCacheProviderContext {
  const void *address_cache;
} SwiftAddressCacheProviderContext;

typedef struct SwiftAddressCacheWrapper {
  struct SwiftAddressCacheProviderContext _0;
} SwiftAddressCacheWrapper;

typedef struct SwiftCancelHandle {
  struct RequestCancelHandle *ptr;
} SwiftCancelHandle;

typedef struct SwiftRetryStrategy {
  struct RetryStrategy *_0;
} SwiftRetryStrategy;

/**
 * A struct used to deallocate a pointer to a C String later than when the pointer's control is relinquished from Swift.
 * Use the `deallocate_ptr` function on `ptr` to call the custom deallocator provided by Swift.
 */
typedef struct LateStringDeallocator {
  const char *ptr;
  void (*deallocate_ptr)(const char*);
} LateStringDeallocator;

typedef struct SwiftMullvadApiResponse {
  uint8_t *body;
  uintptr_t body_size;
  char *etag;
  uint16_t status_code;
  char *error_description;
  char *server_response_code;
  bool success;
} SwiftMullvadApiResponse;

typedef struct CompletionCookie {
  void *inner;
} CompletionCookie;

typedef struct SwiftServerMock {
  const void *server_ptr;
  const void *mock_ptr;
  uint16_t port;
} SwiftServerMock;

typedef struct ProblemReportMetadata {
  struct Map *inner;
} ProblemReportMetadata;

typedef struct SwiftProblemReportRequest {
  const char *address;
  const char *message;
  const char *log;
  struct ProblemReportMetadata metadata;
} SwiftProblemReportRequest;

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

typedef struct ProxyHandle {
  void *context;
  uint16_t port;
} ProxyHandle;

extern const uint16_t CONFIG_SERVICE_PORT;

/**
 * Called by Swift to set the available access methods
 */
void mullvad_api_update_access_methods(struct SwiftApiContext api_context,
                                       struct SwiftAccessMethodSettingsWrapper settings_wrapper);

/**
 * Called by Swift to update the currently used access methods
 *
 * # SAFETY
 * `access_method_id` must point to a null terminated string in a UUID format
 *
 */
void mullvad_api_use_access_method(struct SwiftApiContext api_context,
                                   const char *access_method_id);

/**
 * # Safety
 *
 * `host` must be a pointer to a null terminated string representing a hostname for Mullvad API host.
 * This hostname will be used for TLS validation but not used for domain name resolution.
 *
 * `address` must be a pointer to a null terminated string representing a socket address through which
 * the Mullvad API can be reached directly.
 *
 * If a context cannot be constructed this function will panic since the call site would not be able
 * to proceed in a meaningful way anyway.
 *
 * This function is safe.
 */
struct SwiftApiContext mullvad_api_init_new_tls_disabled(const char *host,
                                                         const char *address,
                                                         const char *domain,
                                                         struct SwiftShadowsocksLoaderWrapper bridge_provider,
                                                         struct SwiftAccessMethodSettingsWrapper settings_provider,
                                                         struct SwiftAddressCacheWrapper address_cache,
                                                         void (*access_method_change_callback)(const void*,
                                                                                               const uint8_t*),
                                                         const void *access_method_change_context);

/**
 * # Safety
 *
 * `host` must be a pointer to a null terminated string representing a hostname for Mullvad API host.
 * This hostname will be used for TLS validation but not used for domain name resolution.
 *
 * `address` must be a pointer to a null terminated string representing a socket address through which
 * the Mullvad API can be reached directly.
 *
 * address_method_change_callback is a function with the C calling convention which will be called
 * whenever the access method changes with a user-specified opaque pointer and a pointer to the bytes
 * of the access method's UUID. Note that this callback must remain valid for the lifetime of the
 * program.
 *
 * access_method_change_context is the pointer passed verbatim to the callback. It is not dereferenced
 * by the Rust code, but remains opaque.
 *
 * If a context cannot be constructed this function will panic since the call site would not be able
 * to proceed in a meaningful way anyway.
 *
 * This function is safe.
 */
struct SwiftApiContext mullvad_api_init_new(const char *host,
                                            const char *address,
                                            const char *domain,
                                            struct SwiftShadowsocksLoaderWrapper bridge_provider,
                                            struct SwiftAccessMethodSettingsWrapper settings_provider,
                                            struct SwiftAddressCacheWrapper address_cache,
                                            void (*access_method_change_callback)(const void*,
                                                                                  const uint8_t*),
                                            const void *access_method_change_context);

/**
 * # Safety
 *
 * `host` must be a pointer to a null terminated string representing a hostname for Mullvad API host.
 * This hostname will be used for TLS validation but not used for domain name resolution.
 *
 * `address` must be a pointer to a null terminated string representing a socket address through which
 * the Mullvad API can be reached directly.
 *
 * If a context cannot be constructed this function will panic since the call site would not be able
 * to proceed in a meaningful way anyway.
 *
 * This function is safe.
 */
struct SwiftApiContext mullvad_api_init_inner(const char *host,
                                              const char *address,
                                              const char *domain,
                                              bool disable_tls,
                                              struct SwiftShadowsocksLoaderWrapper bridge_provider,
                                              struct SwiftAccessMethodSettingsWrapper settings_provider,
                                              struct SwiftAddressCacheWrapper address_cache,
                                              void (*access_method_change_callback)(const void*,
                                                                                    const uint8_t*),
                                              const void *access_method_change_context);

/**
 * Converts parameters into a `Box<AccessMethodSetting>` raw representation that
 * can be passed across the FFI boundary
 *
 * # SAFETY:
 * `unique_identifier` and `name` must point to valid memory regions and contain NULL terminators.
 * They are only valid for the duration of this call.
 *
 * `proxy_configuration` can be NULL, or must be a pointer gotten through
 * either the `convert_shadowsocks` or `convert_socks5` methods.
 */
void *convert_builtin_access_method_setting(const char *unique_identifier,
                                            const char *name,
                                            bool is_enabled,
                                            SwiftAccessMethodKind method_kind,
                                            const void *proxy_configuration);

/**
 * Creates a wrapper around a `Settings` object that can be safely sent across the FFI boundary.
 *
 * # SAFETY
 * `direct_method_raw`, `bridges_method_raw` and `encrypted_dns_method_raw` must be raw pointers
 * resulting from a call to `convert_builtin_access_method_setting`
 * `custom_methods_raw` is an array of pointers to instances of `AccessMethodSetting`
 */
struct SwiftAccessMethodSettingsWrapper init_access_method_settings_wrapper(const void *direct_method_raw,
                                                                            const void *bridges_method_raw,
                                                                            const void *encrypted_dns_method_raw,
                                                                            const void *custom_methods_raw,
                                                                            uintptr_t custom_method_count);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `account_number` must be a pointer to a null terminated string.
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_get_account(struct SwiftApiContext api_context,
                                                 void *completion_cookie,
                                                 struct SwiftRetryStrategy retry_strategy,
                                                 const char *account_number);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_create_account(struct SwiftApiContext api_context,
                                                    void *completion_cookie,
                                                    struct SwiftRetryStrategy retry_strategy);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `account_number` must be a pointer to a null terminated string.
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_delete_account(struct SwiftApiContext api_context,
                                                    void *completion_cookie,
                                                    struct SwiftRetryStrategy retry_strategy,
                                                    const char *account_number);

/**
 * Return the latest available endpoint, or a default one if none are cached
 *
 * # SAFETY
 * `rawAddressCacheProvider` **must** be provided by a call to `init_swift_address_cache_wrapper`
 * It is okay to persist it, and use it accross multiple threads.
 */
extern struct LateStringDeallocator swift_get_cached_endpoint(const void *rawAddressCacheProvider);

/**
 * Called by the Swift side in order to provide an object to rust that provides API addresses in a UTF-8 string form
 *
 * # SAFETY
 * `address_cache` **must be** pointing to a valid instance of a `DefaultAddressCacheProvider`
 * That instance's lifetime has to be equivalent to a `'static` lifetime in Rust
 * This function does not take ownership of `address_cache`
 */
struct SwiftAddressCacheWrapper init_swift_address_cache_wrapper(const void *address_cache);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_get_addresses(struct SwiftApiContext api_context,
                                                   void *completion_cookie,
                                                   struct SwiftRetryStrategy retry_strategy);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_api_addrs_available(struct SwiftApiContext api_context,
                                                         void *completion_cookie,
                                                         struct SwiftRetryStrategy retry_strategy,
                                                         const void *access_method_setting);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `etag` must be a pointer to a null terminated string.
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_get_relays(struct SwiftApiContext api_context,
                                                void *completion_cookie,
                                                struct SwiftRetryStrategy retry_strategy,
                                                const char *etag);

/**
 * Called by the Swift side to signal that a Mullvad API call should be cancelled.
 * After this call, the cancel token is no longer valid.
 *
 * # Safety
 *
 * `handle_ptr` must be pointing to a valid instance of `SwiftCancelHandle`.
 */
void mullvad_api_cancel_task(struct SwiftCancelHandle *handle_ptr);

/**
 * Called by the Swift side to signal that the Rust `SwiftCancelHandle` can be safely
 * dropped from memory.
 *
 * # Safety
 *
 * `handle_ptr` must be pointing to a valid instance of `SwiftCancelHandle`.
 */
void mullvad_api_cancel_task_drop(struct SwiftCancelHandle *handle_ptr);

/**
 * Maps to `mullvadApiCompletionFinish` on Swift side to facilitate callback based completion flow when doing
 * network calls through Mullvad API on Rust side.
 *
 * # Safety
 *
 * `response` must be pointing to a valid instance of `SwiftMullvadApiResponse`.
 *
 * `completion_cookie` must be pointing to a valid instance of `CompletionCookie`. `CompletionCookie` is safe
 * because the pointer in `MullvadApiCompletion` is valid for the lifetime of the process where this type is
 * intended to be used.
 */
extern void mullvad_api_completion_finish(struct SwiftMullvadApiResponse response,
                                          struct CompletionCookie completion_cookie);

/**
 * Get device info via the Mullvad API client.
 *
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_ios_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_ios_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * the `account_number` must be a pointer to a null terminated string.
 * the `identifier` must be a pointer to a null terminated string.
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_get_device(struct SwiftApiContext api_context,
                                                void *completion_cookie,
                                                struct SwiftRetryStrategy retry_strategy,
                                                const char *account_number,
                                                const char *identifier);

/**
 * Get devices info via the Mullvad API client.
 *
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * the `account_number` must be a pointer to a null terminated string.
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_get_devices(struct SwiftApiContext api_context,
                                                 void *completion_cookie,
                                                 struct SwiftRetryStrategy retry_strategy,
                                                 const char *account_number);

/**
 * create device via the Mullvad API client.
 *
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * the `account_number` must be a pointer to a null terminated string.
 * the `identifier` must be a pointer to a null terminated string.
 * the `public_key` pointer must be a valid pointer to 32 unsigned bytes.
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_create_device(struct SwiftApiContext api_context,
                                                   void *completion_cookie,
                                                   struct SwiftRetryStrategy retry_strategy,
                                                   const char *account_number,
                                                   const uint8_t *public_key);

/**
 * delete device via the Mullvad API client.
 *
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * the `account_number` must be a pointer to a null terminated string.
 * the `identifier` must be a pointer to a null terminated string.
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_delete_device(struct SwiftApiContext api_context,
                                                   void *completion_cookie,
                                                   struct SwiftRetryStrategy retry_strategy,
                                                   const char *account_number,
                                                   const char *identifier);

/**
 * rotate device key via the Mullvad API client.
 *
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * the `account_number` must be a pointer to a null terminated string.
 * the `identifier` must be a pointer to a null terminated string.
 * the `public_key` pointer must be a valid pointer to 32 unsigned bytes.
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_rotate_device_key(struct SwiftApiContext api_context,
                                                       void *completion_cookie,
                                                       struct SwiftRetryStrategy retry_strategy,
                                                       const char *account_number,
                                                       const char *identifier,
                                                       const uint8_t *public_key);

/**
 * Converts parameters into a boxed `Shadowsocks` configuration that is safe
 * to send across the FFI boundary
 *
 * # SAFETY
 * `address` must be a pointer to at least `address_len` bytes.
 * `c_password` and `c_cipher` must be pointers to null terminated strings
 */
const void *new_shadowsocks_access_method_setting(const uint8_t *address,
                                                  uintptr_t address_len,
                                                  uint16_t port,
                                                  const char *c_password,
                                                  const char *c_cipher);

/**
 * Converts parameters into a boxed `Socks5Remote` configuration that is safe
 *
 * to send across the FFI boundary
 *
 * # SAFETY
 * `address` must be a pointer to at least `address_len` bytes.
 * `c_username` and `c_password` must be pointers to null terminated strings, or null
 */
const void *new_socks5_access_method_setting(const uint8_t *address,
                                             uintptr_t address_len,
                                             uint16_t port,
                                             const char *c_username,
                                             const char *c_password);

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

/**
 * Send a problem report via the Mullvad API client.
 *
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * the string properties of `SwiftProblemReportRequest` must be pointers to a null terminated strings.
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_send_problem_report(struct SwiftApiContext api_context,
                                                         void *completion_cookie,
                                                         struct SwiftRetryStrategy retry_strategy,
                                                         struct SwiftProblemReportRequest request);

struct ProblemReportMetadata swift_problem_report_metadata_new(void);

/**
 * Add key and value pair to the `ProblemReportMetadata`
 *
 * # Safety
 *
 * `map.inner` must be non-null and point to a valid
 * - `key` must be a null-terminated UTF-8 string, containing LF-separated machines.
 * - `value` must be a valid pointer to some valid and aligned pointer-sized memory.
 */
bool swift_problem_report_metadata_add(struct ProblemReportMetadata map,
                                       const char *key,
                                       const char *value);

void swift_problem_report_metadata_free(struct ProblemReportMetadata map);

/**
 * Called by the Swift side to signal that the Rust `SwiftMullvadApiResponse` can be safely
 * dropped from memory.
 *
 * # Safety
 *
 * `response` must be pointing to a valid instance of `SwiftMullvadApiResponse`. This function
 * is not safe to call multiple times with the same `SwiftMullvadApiResponse`.
 */
void mullvad_response_drop(struct SwiftMullvadApiResponse response);

/**
 * Creates a retry strategy that never retries after failure.
 * The result needs to be consumed.
 */
struct SwiftRetryStrategy mullvad_api_retry_strategy_never(void);

/**
 * Creates a retry strategy that retries `max_retries` times with a constant delay of `delay_sec`.
 * The result needs to be consumed.
 */
struct SwiftRetryStrategy mullvad_api_retry_strategy_constant(uintptr_t max_retries,
                                                              uint64_t delay_sec);

/**
 * Creates a retry strategy that retries `max_retries` times with a exponantially increating delay.
 * The delay will never exceed `max_delay_sec`
 * The result needs to be consumed.
 */
struct SwiftRetryStrategy mullvad_api_retry_strategy_exponential(uintptr_t max_retries,
                                                                 uint64_t initial_sec,
                                                                 uint32_t factor,
                                                                 uint64_t max_delay_sec);

/**
 * Creates a `Shadowsocks` configuration.
 *
 * # SAFETY
 * `rawBridgeProvider` **must** be provided by a call to `init_swift_shadowsocks_loader_wrapper`
 * It is okay to persist it, and use it across multiple threads.
 */
extern const void *swift_get_shadowsocks_bridges(const void *rawBridgeProvider);

/**
 * Called by the Swift side in order to provide an object to rust that can create
 * Shadowsocks configurations
 *
 * # SAFETY
 * `shadowsocks_loader` **must be** pointing to a valid instance of a `SwiftShadowsocksBridgeProvider`
 * That instance's lifetime has to be equivalent to a `'static` lifetime in Rust
 * This function does not take ownership of `shadowsocks_loader`
 */
struct SwiftShadowsocksLoaderWrapper init_swift_shadowsocks_loader_wrapper(const void *shadowsocks_loader);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * `account_number` must be a pointer to a null terminated string.
 *
 * `body` must be a pointer to a contiguous memory segment
 *
 * `body_size` must be the size of the body
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_legacy_storekit_payment(struct SwiftApiContext api_context,
                                                             void *completion_cookie,
                                                             struct SwiftRetryStrategy retry_strategy,
                                                             const char *account_number,
                                                             const uint8_t *body,
                                                             uintptr_t body_size);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `account_number` must be a pointer to a null terminated string.
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_init_storekit_payment(struct SwiftApiContext api_context,
                                                           void *completion_cookie,
                                                           struct SwiftRetryStrategy retry_strategy,
                                                           const char *account_number);

/**
 * # Safety
 *
 * `api_context` must be pointing to a valid instance of `SwiftApiContext`. A `SwiftApiContext` is created
 * by calling `mullvad_api_init_new`.
 *
 * This function takes ownership of `completion_cookie`, which must be pointing to a valid instance of Swift
 * object `MullvadApiCompletion`. The pointer will be freed by calling `mullvad_api_completion_finish`
 * when completion finishes (in completion.finish).
 *
 * `retry_strategy` must have been created by a call to either of the following functions
 * `mullvad_api_retry_strategy_never`, `mullvad_api_retry_strategy_constant` or `mullvad_api_retry_strategy_exponential`
 *
 * `account_number` must be a pointer to a null terminated string.
 *
 * `body` must be a pointer to a contiguous memory segment
 *
 * `body_size` must be the size of the body
 *
 * This function is not safe to call multiple times with the same `CompletionCookie`.
 */
struct SwiftCancelHandle mullvad_ios_check_storekit_payment(struct SwiftApiContext api_context,
                                                            void *completion_cookie,
                                                            struct SwiftRetryStrategy retry_strategy,
                                                            const char *account_number,
                                                            const uint8_t *body,
                                                            uintptr_t body_size);

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

int32_t stop_tunnel_obfuscator_proxy(struct ProxyHandle *proxy_handle);
