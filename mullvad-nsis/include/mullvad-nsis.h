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

/// Windows version details
struct WindowsVer {
  uint32_t major_version;
  uint32_t minor_version;
  uint32_t build_number;
};

extern "C" {

/// Creates a privileged directory at the specified Windows path.
///
/// # SAFETY
/// path needs to be a windows path encoded as a string of u16 that terminates in 0 (two
/// nul-bytes). The string is also not allowed to be greater than `MAX_PATH_SIZE`.
Status create_privileged_directory(const uint16_t *path);

/// Writes the system's app data path into `buffer` when `Status::Ok` is returned.
/// If `buffer` is `null`, or if the buffer is too small, `InsufficientBufferSize`
/// is returned, and the required buffer size (in chars) is returned in `buffer_size`.
/// On success, `buffer_size` is set to the length of the string, including
/// the final null terminator.
///
/// # SAFETY
/// if `buffer` is not null, it must point to a valid memory location that can hold
/// at least `buffer_size` number of `u16` values.
Status get_system_local_appdata(uint16_t *buffer, uintptr_t *buffer_size);

/// Writes the system's version data into `buffer` when `Status::Ok` is
/// returned. If `buffer` is `null`, or if the buffer is too small,
/// `InsufficientBufferSize` is returned, and the required buffer size (in
/// chars) is returned in `buffer_size`. On success, `buffer_size` is set to the
/// length of the string, including the final null terminator.
///
/// # Safety
/// If `buffer` is not null, it must point to a valid memory location that can hold
/// at least `*buffer_size` number of `u16` values. `buffer_size` must be a valid pointer.
Status get_system_version(uint16_t *buffer, uintptr_t *buffer_size);

/// Write OS version into `version_out` when `Status::Ok` is returned.
///
/// # Safety
/// `version_out` must point to a valid `WindowsVer`
Status get_system_version_struct(WindowsVer *version_out);

/// Identify processes that may be using files in the install path, and ask the user to close them.
///
/// # Safety
///
/// * `install_path` must be a null-terminated wide string (UTF-16).
Status find_in_use_processes(const uint16_t *install_path);

}  // extern "C"
