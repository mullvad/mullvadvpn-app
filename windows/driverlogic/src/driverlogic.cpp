#include "stdafx.h"
#include "error.h"
#include "device.h"
#include "service.h"
#include "log.h"
#include "wintun.h"
#include "wireguard.h"
#include "devenum.h"
#include <string>
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <libcommon/string.h>
#include <initguid.h>
#include <devpkey.h>
#include <devguid.h>
#include <io.h>
#include <fcntl.h>

namespace
{

DEFINE_GUID(WFP_CALLOUTS_CLASS_ID,
	0x57465043, 0x616C, 0x6C6F, 0x75, 0x74, 0x5F, 0x63, 0x6C, 0x61, 0x73, 0x73);

constexpr wchar_t SPLIT_TUNNEL_DEVICE_NAME[] = L"Mullvad Split Tunnel Device";

enum ReturnCode
{
	GENERAL_SUCCESS = 0,
	GENERAL_ERROR = 1
};

class ArgumentContext
{
public:

	ArgumentContext(const std::vector<std::wstring> &args)
		: m_args(args)
		, m_remaining(m_args.size())
	{
	}

	size_t total() const
	{
		return m_args.size();
	}

	void ensureExactArgumentCount(size_t count) const
	{
		if (m_args.size() != count)
		{
			throw std::runtime_error("Invalid number of arguments");
		}
	}

	const std::wstring &next()
	{
		if (0 == m_remaining)
		{
			throw std::runtime_error("Argument missing");
		}

		const auto &str = m_args.at(m_args.size() - m_remaining);

		--m_remaining;

		return str;
	}

private:

	const std::vector<std::wstring> &m_args;
	size_t m_remaining;
};

void ResetDriverState()
{
	auto deviceHandle = OpenSplitTunnelDevice();

	common::memory::ScopeDestructor dtor;

	dtor += [deviceHandle]()
	{
		CloseSplitTunnelDevice(deviceHandle);
	};

	SendIoControlReset(deviceHandle);
}

std::unique_ptr<DeviceEnumerator> CreateSplitTunnelDeviceEnumerator()
{
	return DeviceEnumerator::Create(WFP_CALLOUTS_CLASS_ID, [](HDEVINFO deviceInfoSet, const SP_DEVINFO_DATA &deviceInfo)
	{
		try
		{
			auto candidateDeviceName = GetDeviceStringProperty(deviceInfoSet, deviceInfo, &DEVPKEY_NAME);

			return 0 == candidateDeviceName.compare(SPLIT_TUNNEL_DEVICE_NAME);
		}
		catch (const common::error::WindowsException &e)
		{
			if (ERROR_NOT_FOUND == e.errorCode())
			{
				// DEVPKEY_NAME is not guaranteed to be set.
				// If it isn't, just assume it's not a matching device.

				return false;
			}

			throw;
		}
		catch (...)
		{
			throw;
		}
	});
}

//
// CommandSplitTunnelRemove()
//
// Reset driver
// Uninstall device
// Stop service
// Delete service
//
ReturnCode CommandSplitTunnelRemove(const std::vector<std::wstring> &args)
{
	ArgumentContext argsContext(args);

	argsContext.ensureExactArgumentCount(0);

	if (ServiceIsRunning(L"mullvad-split-tunnel"))
	{
		ResetDriverState();
	}

	//
	// Uninstall device, if it exists
	//

	auto enumerator = CreateSplitTunnelDeviceEnumerator();

	EnumeratedDevice device;

	if (enumerator->next(device))
	{
		UninstallDevice(device);
	}

	PokeService(L"mullvad-split-tunnel", true, true);

	return ReturnCode::GENERAL_SUCCESS;
}

ReturnCode CommandWintunDeleteDriver(const std::vector<std::wstring> &args)
{
	ArgumentContext argsContext(args);

	argsContext.ensureExactArgumentCount(0);

	WintunDll wintun;

	if (FALSE == wintun.deleteDriver())
	{
		// NOTE: This is expected if there are other adapters in use.
		throw std::runtime_error("Failed to delete wintun driver");
	}

	std::wstringstream ss;

	ss << L"Deleted Wintun driver";

	Log(ss.str());

	return ReturnCode::GENERAL_SUCCESS;
}

ReturnCode CommandWintunDeleteAbandonedDevice(const std::vector<std::wstring> &args)
{
	ArgumentContext argsContext(args);

	argsContext.ensureExactArgumentCount(0);

	auto enumerator = DeviceEnumerator::Create(GUID_DEVCLASS_NET, [](HDEVINFO deviceInfoSet, const SP_DEVINFO_DATA &deviceInfo)
	{
		static wchar_t WintunMullvadAdapter[] = L"{AFE43773-E1F8-4EBB-8536-576AB86AFE9A}";

		try
		{
			auto candidateAdapterGuid = GetDeviceNetCfgInstanceId(deviceInfoSet, deviceInfo);

			return 0 == _wcsicmp(candidateAdapterGuid.c_str(), WintunMullvadAdapter);
		}
		catch (...)
		{
			// Skip adapters for which we cannot obtain NetCfgInstanceId.
			return false;
		}
	});

	EnumeratedDevice device;

	if (enumerator->next(device))
	{
		UninstallDevice(device);
	}

	return GENERAL_SUCCESS;
}

ReturnCode CommandWireGuardNtCleanup(const std::vector<std::wstring> &args)
{
	ArgumentContext argsContext(args);

	argsContext.ensureExactArgumentCount(0);

	WireGuardNtDll wgNt;

	if (FALSE == wgNt.deleteDriver())
	{
		throw std::runtime_error("Failed to delete WireGuardNT driver");
	}

	Log(L"Successfully deleted WireGuardNT driver");

	return ReturnCode::GENERAL_SUCCESS;
}

} // anonymous namespace

int wmain(int argc, const wchar_t *argv[])
{
	if (-1 == _setmode(_fileno(stdout), _O_U16TEXT)
		|| -1 == _setmode(_fileno(stderr), _O_U16TEXT))
	{
		Log(L"Failed to set translation mode");
	}

	if (argc < 2)
	{
		Log(L"Command not specified");

		return ReturnCode::GENERAL_ERROR;
	}

	//
	// Re-package command arguments
	//

	const std::wstring command = argv[1];

	std::vector<std::wstring> arguments;

	for (size_t argumentIndex = 2; argumentIndex < argc; ++argumentIndex)
	{
		arguments.emplace_back(argv[argumentIndex]);
	}

	//
	// Declare all handlers
	//

	struct CommandHandler
	{
		std::wstring commandName;
		std::function<ReturnCode(const std::vector<std::wstring> &)> handler;
	};

	std::vector<CommandHandler> handlers =
	{
		{ L"st-remove", CommandSplitTunnelRemove },
		{ L"wintun-delete-driver", CommandWintunDeleteDriver },
		{ L"wintun-delete-abandoned-device", CommandWintunDeleteAbandonedDevice },
		{ L"wg-nt-cleanup", CommandWireGuardNtCleanup }
	};

	//
	// Find and invoke matching handler
	//

	for (const auto &candidate : handlers)
	{
		if (0 != _wcsicmp(command.c_str(), candidate.commandName.c_str()))
		{
			continue;
		}

		try
		{
			return candidate.handler(arguments);
		}
		catch (const common::error::WindowsException &e)
		{
			Log(common::string::ToWide(e.what()));
			return e.errorCode();
		}
		catch (const std::exception &e)
		{
			Log(common::string::ToWide(e.what()));
			return GENERAL_ERROR;
		}
		catch (...)
		{
			Log(L"Unknown exception was raised/thrown");
			return GENERAL_ERROR;
		}
	}

	//
	// Could not find matching handler
	//

	Log(L"Could not find handler for specified command");
	return GENERAL_ERROR;
}
