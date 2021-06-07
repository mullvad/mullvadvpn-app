#include "stdafx.h"
#include <winioctl.h>
#include <newdev.h>
#include <initguid.h>
#include <devpkey.h>
#include <devguid.h>
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <libcommon/registry/registry.h>
#include "log.h"
#include "device.h"
#include "error.h"
#include "devenum.h"
#include <vector>
#include <sstream>
#include <functional>

namespace
{

//
// Identifiers defined by split tunneling driver.
//

constexpr wchar_t DeviceSymbolicName[] = L"\\\\.\\MULLVADSPLITTUNNEL";

#define ST_DEVICE_TYPE 0x8000

#define IOCTL_ST_GET_STATE \
	CTL_CODE(ST_DEVICE_TYPE, 9, METHOD_BUFFERED, FILE_ANY_ACCESS)

#define IOCTL_ST_RESET \
	CTL_CODE(ST_DEVICE_TYPE, 11, METHOD_NEITHER, FILE_ANY_ACCESS)

constexpr SIZE_T ST_DRIVER_STATE_STARTED = 1;

//
// Onwards.
//

void
ThrowUpdateException
(
	DWORD lastError,
	const char *operation
)
{
	if (ERROR_DEVICE_INSTALLER_NOT_READY == lastError)
	{
		bool deviceInstallDisabled = false;

		try
		{
			const auto key = common::registry::Registry::OpenKey
			(
				HKEY_LOCAL_MACHINE,
				L"SYSTEM\\CurrentControlSet\\Services\\DeviceInstall\\Parameters"
			);

			deviceInstallDisabled = (0 != key->readUint32(L"DeviceInstallDisabled"));
		}
		catch (...)
		{
		}

		if (deviceInstallDisabled)
		{
			throw common::error::WindowsException
			(
				"Device installs must be enabled to continue. "
				"Enable them in the Local Group Policy editor, or "
				"update the registry value DeviceInstallDisabled in "
				"[HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services\\DeviceInstall\\Parameters]",
				lastError
			);
		}
	}

	THROW_SETUPAPI_ERROR(lastError, operation);
}

} // anonymous namespace

std::wstring
GetDeviceStringProperty
(
	HDEVINFO deviceInfoSet,
	const SP_DEVINFO_DATA &deviceInfo,
	const DEVPROPKEY *property
)
{
	//
	// Obtain required buffer size
	//

	DWORD requiredSize = 0;
	DEVPROPTYPE type;

	const auto sizeStatus = SetupDiGetDevicePropertyW
	(
		deviceInfoSet,
		const_cast<PSP_DEVINFO_DATA>(&deviceInfo),
		property,
		&type,
		nullptr,
		0,
		&requiredSize,
		0
	);

	if (FALSE == sizeStatus)
	{
		const auto lastError = GetLastError();

		if (ERROR_INSUFFICIENT_BUFFER != lastError)
		{
			THROW_SETUPAPI_ERROR(lastError, "SetupDiGetDevicePropertyW");
		}
	}

	std::vector<wchar_t> buffer(requiredSize / sizeof(wchar_t));

	//
	// Read property
	//

	const auto status = SetupDiGetDevicePropertyW
	(
		deviceInfoSet,
		const_cast<PSP_DEVINFO_DATA>(&deviceInfo),
		property,
		&type,
		reinterpret_cast<PBYTE>(&buffer[0]),
		requiredSize,
		nullptr,
		0
	);

	if (FALSE == status)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "Failed to read device property");
	}

	return buffer.data();
}

std::wstring
GetDeviceNetCfgInstanceId
(
	HDEVINFO deviceInfoSet,
	const SP_DEVINFO_DATA &deviceInfo
)
{
	auto registryKey = SetupDiOpenDevRegKey
	(
		deviceInfoSet,
		const_cast<SP_DEVINFO_DATA *>(&deviceInfo),
		DICS_FLAG_GLOBAL,
		0,
		DIREG_DRV,
		KEY_READ
	);

	if (registryKey == INVALID_HANDLE_VALUE)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiOpenDevRegKey");
	}

	std::vector<wchar_t> buffer(128, L'\0');

	DWORD bufferByteLength = static_cast<DWORD>(buffer.size()) * sizeof(wchar_t);

	const auto status = RegGetValueW
	(
		registryKey,
		nullptr,
		L"NetCfgInstanceId",
		RRF_RT_REG_SZ,
		nullptr,
		&buffer[0],
		&bufferByteLength
	);

	RegCloseKey(registryKey);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "RegGetValueW");
	}

	//
	// RegGetValueW ensures the string is null-terminated.
	//

	return std::wstring(&buffer[0]);
}

void
CreateDevice
(
	const GUID &classGuid,
	const std::wstring &deviceName,
	const std::wstring &deviceHardwareId
)
{
	Log(L"Attempting to create device");

	const auto deviceInfoSet = SetupDiCreateDeviceInfoList(&classGuid, 0);

	if (INVALID_HANDLE_VALUE == deviceInfoSet)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiCreateDeviceInfoList");
	}

	common::memory::ScopeDestructor scopeDestructor;

	scopeDestructor += [&deviceInfoSet]()
	{
		SetupDiDestroyDeviceInfoList(deviceInfoSet);
	};

	SP_DEVINFO_DATA devInfoData {0};
	devInfoData.cbSize = sizeof(SP_DEVINFO_DATA);

	auto status = SetupDiCreateDeviceInfoW
	(
		deviceInfoSet,
		deviceName.c_str(),
		&classGuid,
		nullptr,
		0,
		DICD_GENERATE_ID,
		&devInfoData
	);

	if (FALSE == status)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiCreateDeviceInfoW");
	}

	status = SetupDiSetDeviceRegistryPropertyW
	(
		deviceInfoSet,
		&devInfoData,
		SPDRP_HARDWAREID,
		reinterpret_cast<const BYTE *>(deviceHardwareId.c_str()),
		static_cast<DWORD>(deviceHardwareId.size() * sizeof(wchar_t))
	);

	if (FALSE == status)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiSetDeviceRegistryPropertyW");
	}

	//
	// Create a devnode in the PnP HW tree
	//
	status = SetupDiCallClassInstaller
	(
		DIF_REGISTERDEVICE,
		deviceInfoSet,
		&devInfoData
	);

	if (FALSE == status)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiCallClassInstaller");
	}

	Log(L"Created new device successfully");
}

void
InstallDriverForDevice
(
	const std::wstring &deviceHardwareId,
	const std::wstring &infPath
)
{
	Log(L"Attempting to install new driver");

	DWORD installFlags = 0;
	BOOL rebootRequired = FALSE;

	for (;;)
	{
		auto result = UpdateDriverForPlugAndPlayDevicesW
		(
			nullptr,
			deviceHardwareId.c_str(),
			infPath.c_str(),
			installFlags,
			&rebootRequired
		);

		if (FALSE != result)
		{
			break;
		}

		const auto lastError = GetLastError();

		if (ERROR_NO_MORE_ITEMS == lastError
			&& (0 == (installFlags & INSTALLFLAG_FORCE)))
		{
			Log(L"Driver installation/update failed. Attempting forced install.");
			installFlags |= INSTALLFLAG_FORCE;

			continue;
		}

		ThrowUpdateException(lastError, "UpdateDriverForPlugAndPlayDevicesW");
	}

	//
	// Driver successfully installed or updated
	//

	std::wstringstream ss;

	ss << L"Device driver update complete. Reboot required: "
		<< rebootRequired;

	Log(ss.str());
}

void
UninstallDevice
(
	const EnumeratedDevice &device
)
{
	Log(L"Uninstalling device");

	BOOL needReboot;

	auto status = DiUninstallDevice
	(
		nullptr,
		device.deviceInfoSet,
		const_cast<PSP_DEVINFO_DATA>(&device.deviceInfo),
		0,
		&needReboot
	);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "DiUninstallDevice");
	}

	std::wstringstream ss;

	ss << L"Successfully uninstalled device. Reboot required: "
		<< needReboot;

	Log(ss.str());
}

HANDLE
OpenSplitTunnelDevice
(
)
{
	auto handle = CreateFileW(DeviceSymbolicName, GENERIC_READ | GENERIC_WRITE,
		0, nullptr, OPEN_EXISTING, FILE_FLAG_OVERLAPPED, nullptr);

	if (handle == INVALID_HANDLE_VALUE)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Open split tunnel device");
	}

	return handle;
}

void
CloseSplitTunnelDevice
(
	HANDLE device
)
{
	CloseHandle(device);
}

void
SendIoControl
(
	HANDLE device,
	DWORD code,
	void *inBuffer,
	DWORD inBufferSize,
	void *outBuffer,
	DWORD outBufferSize,
	DWORD *bytesReturned
)
{
	OVERLAPPED o = { 0 };

	//
	// Event should not be created on-the-fly.
	//
	// Create an event for each thread that needs to send a request
	// and keep the event around.
	//
	o.hEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);

	auto status = DeviceIoControl(device, code,
		inBuffer, inBufferSize, outBuffer, outBufferSize, bytesReturned, &o);

	if (FALSE != status)
	{
		CloseHandle(o.hEvent);

		return;
	}

	if (ERROR_IO_PENDING != GetLastError())
	{
		const auto err = GetLastError();

		CloseHandle(o.hEvent);

		THROW_WINDOWS_ERROR(err, "DeviceIoControl");
	}

	DWORD tempBytesReturned = 0;

	status = GetOverlappedResult(device, &o, &tempBytesReturned, TRUE);

	CloseHandle(o.hEvent);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "GetOverlappedResult");
	}

	*bytesReturned = tempBytesReturned;
}

void
SendIoControlReset
(
	HANDLE device
)
{
	DWORD dummy;

	SendIoControl(device, (DWORD)IOCTL_ST_RESET, nullptr, 0, nullptr, 0, &dummy);

	DWORD bytesReturned;

	SIZE_T currentState;

	SendIoControl(device, (DWORD)IOCTL_ST_GET_STATE, nullptr, 0, &currentState, sizeof(currentState), &bytesReturned);

	if (bytesReturned != sizeof(currentState))
	{
		throw std::runtime_error("Failed to send reset request to driver");
	}

	//
	// If successful, state is ST_DRIVER_STATE_STARTED
	//
	// Otherwise, state is probably ST_DRIVER_STATE_ZOMBIE
	//

	if (currentState != ST_DRIVER_STATE_STARTED)
	{
		throw std::runtime_error("Failed to reset driver state");
	}
}
