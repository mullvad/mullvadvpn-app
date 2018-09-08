#include "stdafx.h"
#include "windns.h"
#include "windnscontext.h"
#include "clientsinkinfo.h"
#include "interfaceconfig.h"
#include "netconfighelpers.h"
#include "confineoperation.h"
#include "libcommon/serialization/deserializer.h"
#include <vector>
#include <string>

namespace
{

WinDnsErrorSink g_ErrorSink = nullptr;
void *g_ErrorContext = nullptr;

WinDnsContext *g_Context = nullptr;

std::vector<std::wstring> MakeStringArray(const wchar_t **strings, uint32_t numStrings)
{
	std::vector<std::wstring> v;

	while (numStrings--)
	{
		v.emplace_back(*strings++);
	}

	return v;
}

void ForwardError(const char *errorMessage, const char **details, uint32_t numDetails)
{
	if (nullptr != g_ErrorSink)
	{
		g_ErrorSink(errorMessage, details, numDetails, g_ErrorContext);
	}
}

} // anonymous namespace

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Initialize(
	WinDnsErrorSink errorSink,
	void *errorContext
)
{
	if (nullptr != g_Context)
	{
		return false;
	}

	g_ErrorSink = errorSink;
	g_ErrorContext = errorContext;

	return ConfineOperation("Initialize", ForwardError, []()
	{
		g_Context = new WinDnsContext;
	});
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Deinitialize(
)
{
	if (nullptr == g_Context)
	{
		return true;
	}

	delete g_Context;
	g_Context = nullptr;

	return true;
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Set(
	const wchar_t **servers,
	uint32_t numServers,
	WinDnsConfigSink configSink,
	void *configContext
)
{
	if (nullptr == g_Context
		|| 0 == numServers
		|| nullptr == configSink)
	{
		return false;
	}

	return ConfineOperation("Enforce DNS settings", ForwardError, [&]()
	{
		ClientSinkInfo sinkInfo;

		sinkInfo.errorSinkInfo = ErrorSinkInfo{ g_ErrorSink, g_ErrorContext };
		sinkInfo.configSinkInfo = ConfigSinkInfo{ configSink, configContext };

		g_Context->set(MakeStringArray(servers, numServers), sinkInfo);
	});
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Reset(
)
{
	if (nullptr == g_Context)
	{
		return true;
	}

	return ConfineOperation("Reset DNS settings", ForwardError, []()
	{
		g_Context->reset();
	});
}

WINDNS_LINKAGE
bool
WINDNS_API
WinDns_Recover(
	const void *configData,
	uint32_t dataLength
)
{
	std::vector<InterfaceConfig> configs;

	const auto status = ConfineOperation("Deserialize recovery data", ForwardError, [&]()
	{
		common::serialization::Deserializer d(reinterpret_cast<const uint8_t *>(configData), dataLength);

		auto numConfigs = d.decode<uint32_t>();

		if (numConfigs > 50)
		{
			throw std::runtime_error("Too many configuration entries");
		}

		configs.reserve(numConfigs);

		for (; numConfigs != 0; --numConfigs)
		{
			configs.emplace_back(InterfaceConfig(d));
		}
	});

	if (false == status)
	{
		return false;
	}

	//
	// Try to restore each config and update 'success' if any update fails.
	//

	bool success = true;

	for (const auto &config : configs)
	{
		const auto adapterStatus = ConfineOperation("Restore adapter DNS settings", ForwardError, [&config]()
		{
			nchelpers::RevertDnsServers(config);
		});

		if (false == adapterStatus)
		{
			success = false;
		}
	}

	return success;
}
