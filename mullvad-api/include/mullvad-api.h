#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum MullvadApiErrorKind {
  NoError = 0,
  StringParsing = -1,
  SocketAddressParsing = -2,
  AsyncRuntimeInitialization = -3,
  BadResponse = -4,
} MullvadApiErrorKind;

typedef struct DeviceIterator DeviceIterator;

/**
 * A Mullvad API client that can be used via a C FFI.
 */
typedef struct FfiClient FfiClient;

/**
 * MullvadApiErrorKind contains a description and an error kind. If the error kind is
 * `MullvadApiErrorKind` is NoError, the pointer will be nil.
 */
typedef struct MullvadApiError {
  char *description;
  enum MullvadApiErrorKind kind;
} MullvadApiError;

typedef struct MullvadApiClient {
  const struct FfiClient *ptr;
} MullvadApiClient;

typedef struct MullvadApiDeviceIterator {
  struct DeviceIterator *ptr;
} MullvadApiDeviceIterator;

typedef struct MullvadApiDevice {
  const char *name_ptr;
  uint8_t id[16];
} MullvadApiDevice;

/**
 * Initializes a Mullvad API client.
 *
 * #Arguments
 * * `client_ptr`: Must be a pointer to that is valid for the length of a `MullvadApiClient`
 * struct.
 *
 * * `api_address`: pointer to nul-terminated UTF-8 string containing a socket address
 *   representation
 * ("143.32.4.32:9090"), the port is mandatory.
 *
 * * `hostname`: pointer to a null-terminated UTF-8 string representing the hostname that will be
 * used for TLS validation.
 */
struct MullvadApiError mullvad_api_client_initialize(struct MullvadApiClient *client_ptr,
                                                     const char *api_address_ptr,
                                                     const char *hostname);

/**
 * Removes all devices from a given account
 *
 * #Arguments
 * * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
 *
 * * `account_str_ptr`: pointer to nul-terminated UTF-8 string containing the account number of the
 * account that will have all of it's devices removed.
 */
struct MullvadApiError mullvad_api_remove_all_devices(struct MullvadApiClient client_ptr,
                                                      const char *account_ptr);

/**
 * Removes all devices from a given account
 *
 * #Arguments
 * * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
 *
 * * `account_str_ptr`: pointer to nul-terminated UTF-8 string containing the account number of the
 * account that will have all of it's devices removed.
 *
 * * `expiry_unix_timestamp`: a pointer to a signed 64 bit integer. If this function returns no
 * error, the expiry timestamp will be written to this pointer.
 */
struct MullvadApiError mullvad_api_get_expiry(struct MullvadApiClient client_ptr,
                                              const char *account_str_ptr,
                                              int64_t *expiry_unix_timestamp);

/**
 * Gets a list of all devices associated with the specified account from the API.
 *
 * #Arguments
 * * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
 *
 * * `account_str_ptr`: pointer to nul-terminated UTF-8 string containing the account number of the
 * account that will have all of it's devices removed.
 *
 * * `device_iter_ptr`: a pointer to a `device::MullvadApiDeviceIterator`. If this function
 * doesn't return an error, the pointer will be initialized with a valid instance of
 * `device::MullvadApiDeviceIterator`, which can be used to iterate through the devices.
 */
struct MullvadApiError mullvad_api_list_devices(struct MullvadApiClient client_ptr,
                                                const char *account_str_ptr,
                                                struct MullvadApiDeviceIterator *device_iter_ptr);

/**
 * Adds a device to the specified account with the specified public key. Note that the device
 * name, associated addresess and UUID are not returned.
 *
 * #Arguments
 * * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
 *
 * * `account_str_ptr`: pointer to nul-terminated UTF-8 string containing the account number of the
 * account that will have a device added to ita device added to it.
 *
 * * `public_key_ptr`: a pointer to 32 bytes of a WireGuard public key that will be uploaded.
 *
 * * `new_device_ptr`: a pointer to enough memory to allocate a `MullvadApiDevice`. If this
 * function doesn't return an error, it will be initialized.
 */
struct MullvadApiError mullvad_api_add_device(struct MullvadApiClient client_ptr,
                                              const char *account_str_ptr,
                                              const uint8_t *public_key_ptr,
                                              struct MullvadApiDevice *new_device_ptr);

/**
 * Creates a new account.
 *
 * #Arguments
 * * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
 *
 * * `account_str_ptr`: If a new account is created successfully, a pointer to an allocated C
 *   string containing the new
 * account number will be written to this pointer. It must be freed via
 * `mullvad_api_cstring_drop`.
 */
struct MullvadApiError mullvad_api_create_account(struct MullvadApiClient client_ptr,
                                                  const char **account_str_ptr);

/**
 * Deletes the specified account.
 *
 * #Arguments
 * * `client_ptr`: Must be a valid, initialized instance of `MullvadApiClient`
 *
 * * `account_str_ptr`: A null-terminated string representing the account to be deleted.
 */
struct MullvadApiError mullvad_api_delete_account(struct MullvadApiClient client_ptr,
                                                  const char *account_str_ptr);

void mullvad_api_client_drop(struct MullvadApiClient client);

/**
 * Deallocates a CString returned by the Mullvad API client.
 */
void mullvad_api_cstring_drop(char *cstr_ptr);

bool mullvad_api_device_iter_next(struct MullvadApiDeviceIterator iter,
                                  struct MullvadApiDevice *device_ptr);

void mullvad_api_device_iter_drop(struct MullvadApiDeviceIterator iter);

void mullvad_api_device_drop(struct MullvadApiDevice device);

void mullvad_api_error_drop(struct MullvadApiError error);
