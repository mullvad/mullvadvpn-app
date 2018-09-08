#pragma once

#include <cstdint>

struct IClientSinkProxy
{
	virtual ~IClientSinkProxy() = 0
	{
	}

	virtual void error(const char *errorMessage, const char **details, uint32_t numDetails) = 0;
	virtual void config(const void *configData, uint32_t dataLength) = 0;
};
