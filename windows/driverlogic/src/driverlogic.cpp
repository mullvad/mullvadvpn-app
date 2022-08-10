#include "stdafx.h"
#include "error.h"
#include "device.h"
#include "service.h"
#include "log.h"
#include "version.h"
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

constexpr wchar_t SPLIT_TUNNEL_HARDWARE_ID[] = L"Root\\mullvad-split-tunnel";

DEFINE_GUID(WFP_CALLOUTS_CLASS_ID,
	0x57465043, 0x616C, 0x6C6F, 0x75, 0x74, 0x5F, 0x63, 0x6C, 0x61, 0x73, 0x73);

constexpr wchar_t SPLIT_TUNNEL_DEVICE_NAME[] = L"Mullvad Split Tunnel Device";

enum ReturnCode
{
	GENERAL_SUCCESS = 0,
	GENERAL_ERROR = 1,
	ST_DRIVER_NONE_INSTALLED = 2,
	ST_DRIVER_SAME_VERSION_INSTALLED = 3,
	ST_DRIVER_OLDER_VERSION_INSTALLED = 4,
	ST_DRIVER_NEWER_VERSION_INSTALLED = 5
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
		auto candidateDeviceName = GetDeviceStringProperty(deviceInfoSet, deviceInfo, &DEVPKEY_NAME);

		return 0 == candidateDeviceName.compare(SPLIT_TUNNEL_DEVICE_NAME);
	});
}

//
// CommandSplitTunnelEvaluate()
//
// Search for existing device.
// Evaluate if provided inf can/should be installed.
//
ReturnCode CommandSplitTunnelEvaluate(const std::vector<std::wstring> &args)
{
	ArgumentContext argsContext(args);

	argsContext.ensureExactArgumentCount(1);

	const auto infPath = argsContext.next();

	//
	// Find first matching device
	//

	auto enumerator = CreateSplitTunnelDeviceEnumerator();

	EnumeratedDevice device;

	if (!enumerator->next(device))
	{
		return ReturnCode::ST_DRIVER_NONE_INSTALLED;
	}

	//
	// Retrieve driver versions
	//

	auto existingVersion = GetDriverVersion(device);
	auto proposedVersion = InfGetDriverVersion(infPath);

	//
	// Compare driver versions
	//

	switch (EvaluateDriverUpgrade(existingVersion, proposedVersion))
	{
		case DRIVER_UPGRADE_STATUS::WOULD_UPGRADE:
			return ReturnCode::ST_DRIVER_OLDER_VERSION_INSTALLED;
		case DRIVER_UPGRADE_STATUS::WOULD_DOWNGRADE:
			return ReturnCode::ST_DRIVER_NEWER_VERSION_INSTALLED;
		case DRIVER_UPGRADE_STATUS::WOULD_INSTALL_SAME_VERSION:
			return ReturnCode::ST_DRIVER_SAME_VERSION_INSTALLED;
		default:
			Log(L"Unexpected return value from EvaluateDriverUpgrade()");
	}

	return ReturnCode::GENERAL_ERROR;
}

ReturnCode CommandSplitTunnelNewInstall(const std::vector<std::wstring> &args)
{
	ArgumentContext argsContext(args);

	argsContext.ensureExactArgumentCount(1);

	const auto infPath = argsContext.next();

	CreateDevice(WFP_CALLOUTS_CLASS_ID, SPLIT_TUNNEL_DEVICE_NAME, SPLIT_TUNNEL_HARDWARE_ID);

	InstallDriverForDevice(SPLIT_TUNNEL_HARDWARE_ID, infPath);

	return ReturnCode::GENERAL_SUCCESS;
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

	//
	// Find first matching device
	//

	auto enumerator = CreateSplitTunnelDeviceEnumerator();

	EnumeratedDevice device;

	if (!enumerator->next(device))
	{
		Log(L"Could not find split tunnel device");

		return ReturnCode::GENERAL_SUCCESS;
	}

	ResetDriverState();

	UninstallDevice(device);

	PokeService(L"mullvad-split-tunnel", true, true);

	return ReturnCode::GENERAL_SUCCESS;
}

//
// CommandSplitTunnelForceInstall()
//
// There's an existing device that needs to be stopped and removed.
// After this, create a new device and associate the specified inf.
//
ReturnCode CommandSplitTunnelForceInstall(const std::vector<std::wstring> &args)
{
	auto status = CommandSplitTunnelRemove({});

	if (ReturnCode::GENERAL_SUCCESS != status)
	{
		return status;
	}

	return CommandSplitTunnelNewInstall(args);
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

		auto candidateAdapterGuid = GetDeviceNetCfgInstanceId(deviceInfoSet, deviceInfo);

		return 0 == _wcsicmp(candidateAdapterGuid.c_str(), WintunMullvadAdapter);
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
		{ L"st-evaluate", CommandSplitTunnelEvaluate },
		{ L"st-new-install", CommandSplitTunnelNewInstall },
		{ L"st-force-install", CommandSplitTunnelForceInstall },
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
