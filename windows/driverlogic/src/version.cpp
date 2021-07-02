#include "stdafx.h"
#include "version.h"
#include "device.h"
#include <setupapi.h>
#include <initguid.h>
#include <devpkey.h>
#include <libcommon/string.h>
#include <libcommon/memory.h>
#include <stdexcept>

DRIVER_UPGRADE_STATUS
EvaluateDriverUpgrade
(
	const std::wstring &existingVersion,
	const std::wstring &proposedVersion
)
{
	//
	// "x.y.z.a"
	//

	using namespace common::string;

	auto et = Tokenize(existingVersion, L".");
	auto pt = Tokenize(proposedVersion, L".");

	auto items = min(et.size(), pt.size());

	for (auto index = 0; index < items; ++index)
	{
		auto ev = wcstoul(et[index].c_str(), nullptr, 10);
		auto pv = wcstoul(pt[index].c_str(), nullptr, 10);

		if (pv > ev)
		{
			return DRIVER_UPGRADE_STATUS::WOULD_UPGRADE;
		}

		if (ev > pv)
		{
			return DRIVER_UPGRADE_STATUS::WOULD_DOWNGRADE;
		}
	}

	if (pt.size() > et.size())
	{
		return DRIVER_UPGRADE_STATUS::WOULD_UPGRADE;
	}

	if (et.size() > pt.size())
	{
		return DRIVER_UPGRADE_STATUS::WOULD_DOWNGRADE;
	}

	return DRIVER_UPGRADE_STATUS::WOULD_INSTALL_SAME_VERSION;
}

std::wstring
InfGetDriverVersion
(
	const std::wstring &filePath
)
{
	auto infHandle = SetupOpenInfFileW(filePath.c_str(), nullptr, INF_STYLE_WIN4, nullptr);

	if (infHandle == INVALID_HANDLE_VALUE)
	{
		throw std::runtime_error("SetupOpenInfFileW()");
	}

	common::memory::ScopeDestructor dtor;

	dtor += [infHandle]()
	{
		SetupCloseInfFile(infHandle);
	};

	INFCONTEXT infContext { 0 };

	auto status = SetupFindFirstLineW(infHandle, L"Version", L"DriverVer", &infContext);

	if (status == FALSE)
	{
		throw std::runtime_error("SetupFindFirstLineW()");
	}

	DWORD requiredSize;

	//
	// This is a multi-value key.
	// 0 = key, 1 = driver date
	//
	const DWORD VersionFieldIndex = 2;

	status = SetupGetStringFieldW(&infContext, VersionFieldIndex, nullptr, 0, &requiredSize);

	if (status == FALSE || requiredSize < 2)
	{
		throw std::runtime_error("SetupGetStringFieldW()");
	}

	std::vector<wchar_t> buffer(requiredSize);

	status = SetupGetStringFieldW(&infContext, VersionFieldIndex,
		&buffer[0], static_cast<DWORD>(buffer.size()), nullptr);

	if (status == FALSE)
	{
		throw std::runtime_error("SetupGetStringFieldW()");
	}

	// Remove null terminator.
	buffer.resize(requiredSize - 1);

	return buffer.data();
}

std::wstring
GetDriverVersion
(
	const EnumeratedDevice &device
)
{
	return GetDeviceStringProperty(device.deviceInfoSet, device.deviceInfo, &DEVPKEY_Device_DriverVersion);
}
