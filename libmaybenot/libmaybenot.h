#include <stdint.h>
#include <stdbool.h>

typedef enum : uint32_t {
	OK = 0,
	MACHINE_STRING_NOT_UTF8 = 1,
	INVALID_MACHINE_STRING = 2,
	START_FRAMEWORK = 3,
	UNKNOWN_MACHINE = 4,
} MaybenotError;

typedef struct {} Maybenot;

typedef struct {
	/// number of whole seconds
	uint64_t secs;
	/// a nanosecond fraction of a second
	uint32_t nanos;
} MaybenotDuration;

// TODO:
// typedef enum : uint32_t {
// } MaybenotEventTag;

typedef struct {
	uint32_t eventType;

	// The ID of the machine that triggered the event, if any.
	uint32_t machine;

	// The number of bytes that was sent or received.
	uint16_t xmitBytes;
} MaybenotEvent;

typedef struct {
	uint16_t byteCount;
	bool replace;
} MaybenotPadding;

typedef enum : uint32_t {
	CANCEL = 0,
	INJECT_PADDING = 1,
	BLOCK_OUTGOING = 2,
} MaybenotActionTag;

typedef struct {
	uint64_t machine;
	MaybenotDuration timeout;
} ActionCancel;

typedef struct {
	uint64_t machine;
	MaybenotDuration timeout;
	bool replace;
	bool bypass;
	uint16_t size;
} ActionInjectPadding;

typedef struct {
	uint64_t machine;
	MaybenotDuration timeout;
	bool replace;
	bool bypass;
	MaybenotDuration duration;
} ActionBlockOutgoing;

typedef union {
	ActionCancel cancel;
	ActionInjectPadding inject_padding;
	ActionBlockOutgoing block_outgoing;
} MaybenotActionValue;

typedef struct {
	MaybenotActionTag actionTag;
	MaybenotActionValue action;
} MaybenotAction;

/// A function that is called by `maybenot_on_event` once for every generated `MaybenotAction`.
// TODO: consider passing a &mut [MaybenotAction] to maybenot_on_event instead of using a callback
typedef void (*onActionCallback)(void*, MaybenotAction);

/// Start a new Maybenot instance.
MaybenotError maybenot_start(
	/// A string containing newline-separated machines
	char* const machines,
	double max_padding_bytes,
	double max_blocking_bytes,
	uint16_t mtu,
	onActionCallback on_action,
	Maybenot** out
);

/// Stop a running Maybenot instance.
///
/// This will free the Maybenot pointer.
void maybenot_stop(Maybenot* self);

/// Feed an event to the Maybenot instance.
///
/// This may generate MaybenotActions that will be sent to the callback provided to maybenotInit.
/// `user_data` will be passed to the callback as-is. It will not be read or modified.
MaybenotError maybenot_on_event(Maybenot* self, void* user_data, MaybenotEvent event);
