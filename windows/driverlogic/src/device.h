#pragma once

#include <windows.h>
#include <string>
#include <optional>
#include <setupapi.h>

struct EnumeratedDevice
{
	HDEVINFO deviceInfoSet;
	SP_DEVINFO_DATA deviceInfo;
};

//
// Generic functions
//

std::wstring
GetDeviceStringProperty
(
	HDEVINFO deviceInfoSet,
	const SP_DEVINFO_DATA &deviceInfo,
	const DEVPROPKEY *property
);

std::wstring
GetDeviceNetCfgInstanceId
(
	HDEVINFO deviceInfoSet,
	const SP_DEVINFO_DATA &deviceInfo
);

void
CreateDevice
(
	const GUID &classGuid,
	const std::wstring &deviceName,
	const std::wstring &deviceHardwareId
);

void
UninstallDevice
(
	const EnumeratedDevice &device
);

//
// Functions that are specific to our driver/implementation
//

HANDLE
OpenSplitTunnelDevice
(
);

void
CloseSplitTunnelDevice
(
	HANDLE device
);

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
);

void
SendIoControlReset
(
	HANDLE device
);
