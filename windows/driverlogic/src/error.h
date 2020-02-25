#pragma once

#include <cstdint>


bool IsSetupApiError(uint32_t code);
const char *FormatSetupApiError(uint32_t code);

[[noreturn]] void ThrowSetupApiError(const char *operation, uint32_t code, const char *file, size_t line);

#define THROW_SETUPAPI_ERROR(errorCode, operation)\
{\
	ThrowSetupApiError(operation, errorCode, __FILE__, __LINE__);\
}
