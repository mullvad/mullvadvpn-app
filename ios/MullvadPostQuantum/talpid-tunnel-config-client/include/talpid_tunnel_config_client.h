#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Port used by the tunnel config service.
 */
#define CONFIG_SERVICE_PORT 1337

typedef struct PostQuantumCancelToken {
  void *context;
} PostQuantumCancelToken;

/**
 * Called by the Swift side to signal that the quantum-secure key exchange should be cancelled.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `PostQuantumCancelToken` created by the `PacketTunnelProvider`.
 */
void cancel_post_quantum_key_exchange(const struct PostQuantumCancelToken *sender);

/**
 * Called by the Swift side to signal that the Rust `PostQuantumCancelToken` can be safely dropped from memory.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `PostQuantumCancelToken` created by the `PacketTunnelProvider`.
 */
void drop_post_quantum_key_exchange_token(const struct PostQuantumCancelToken *sender);

/**
 * Called by Swift whenever data has been written to the in-tunnel TCP connection when exchanging
 * quantum-resistant pre shared keys.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `write_tx` created by the `IosTcpProvider`
 * Callback to call when the TCP connection has written data.
 */
void handle_sent(uintptr_t bytes_sent, const void *sender);

/**
 * Called by Swift whenever data has been read from the in-tunnel TCP connection when exchanging
 * quantum-resistant pre shared keys.
 *
 * # Safety
 * `sender` must be pointing to a valid instance of a `read_tx` created by the `IosTcpProvider`
 *
 * Callback to call when the TCP connection has received data.
 */
void handle_recv(const uint8_t *data, uintptr_t data_len, const void *sender);

/**
 * Entry point for exchanging post quantum keys on iOS.
 * The TCP connection must be created to go through the tunnel.
 * # Safety
 * `public_key` and `ephemeral_key` must be valid respective `PublicKey` and `PrivateKey` types.
 * They will not be valid after this function is called, and thus must be copied here.
 * `packet_tunnel` and `tcp_connection` must be valid pointers to a packet tunnel and a TCP connection
 * instances.
 * `cancel_token` should be owned by the caller of this function.
 */
int32_t negotiate_post_quantum_key(const uint8_t *public_key,
                                   const uint8_t *ephemeral_key,
                                   const void *packet_tunnel,
                                   const void *tcp_connection,
                                   struct PostQuantumCancelToken *cancel_token);

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
 * Called when the preshared post quantum key is ready.
 * `raw_preshared_key` might be NULL if the key negotiation failed.
 */
extern void swift_post_quantum_key_ready(const void *raw_packet_tunnel,
                                         const uint8_t *raw_preshared_key,
                                         const uint8_t *raw_ephemeral_private_key);
