#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class Status {
  Ok,
  InvalidArguments,
  InsufficientBufferSize,
  OsError,
  Panic,
};

extern "C" {

/// Writes the system's app data path into `buffer` when `Status::Ok` is returned.
/// If `buffer` is `null`, or if the buffer is too small, `InsufficientBufferSize`
/// is returned, and the required buffer size (in chars) is returned in `buffer_size`.
/// On success, `buffer_size` is set to the length of the string, including
/// the final null terminator.
Status get_system_local_appdata(uint16_t *buffer, uintptr_t *buffer_size);

Status create_privileged_directory(const uint16_t* path);

} // extern "C"
