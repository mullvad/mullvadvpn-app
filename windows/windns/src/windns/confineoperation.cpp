#include "stdafx.h"
#include "confineoperation.h"
#include "netsh.h"

bool ConfineOperation
(
	const char *literalOperation,
	std::function<void(const char *, const char **, uint32_t)> errorCallback,
	std::function<void()> operation
)
{
	try
	{
		operation();
		return true;
	}
	catch (NetShError &err)
	{
		auto raw = CreateRawStringArray(err.details());

		const char **details = reinterpret_cast<const char **>(&raw[0]);
		uint32_t numDetails = static_cast<uint32_t>(err.details().size());

		if (0 == numDetails)
		{
			details = nullptr;
		}

		const auto what = std::string(literalOperation).append(": ").append(err.what());

		errorCallback(what.c_str(), details, numDetails);

		return false;
	}
	catch (std::exception &err)
	{
		const auto what = std::string(literalOperation).append(": ").append(err.what());

		errorCallback(what.c_str(), nullptr, 0);

		return false;
	}
	catch (...)
	{
		const auto what = std::string(literalOperation).append(": Unspecified failure");

		errorCallback(what.c_str(), nullptr, 0);

		return false;
	}
}

bool ConfineOperation
(
	const char *literalOperation,
	ILogSink *logSink,
	std::function<void()> operation
)
{
	auto ForwardError = [logSink](const char *error, const char **details, uint32_t numDetails)
	{
		if (nullptr != logSink)
		{
			logSink->error(error, details, numDetails);
		}
	};

	return ConfineOperation(literalOperation, ForwardError, operation);
}

std::vector<uint8_t> CreateRawStringArray(const std::vector<std::string> &arr)
{
	//
	// Return a buffer containing a nullptr if there are no items in the array.
	// This enables clients of this function to address the pointer table.
	//

	if (arr.empty())
	{
		return std::vector<uint8_t>(sizeof(char *), 0);
	}

	//
	// Determine total size needed.
	//

	size_t bufferSize = 0;

	for (const auto &str : arr)
	{
		bufferSize += sizeof(char *);
		bufferSize += (str.size() + 1);
	}

	//
	// Copy strings and populate pointer table.
	//

	std::vector<uint8_t> buffer(bufferSize, 0);

	char **pointerTable = reinterpret_cast<char**>(&buffer[0]);
	char *data = reinterpret_cast<char*>(&buffer[0] + (sizeof(char*) * arr.size()));

	for (const auto &str : arr)
	{
		const auto fullStringSize = str.size() + 1;

		*pointerTable = data;
		memcpy(data, str.c_str(), fullStringSize);

		++pointerTable;
		data += fullStringSize;
	}

	return buffer;
}
