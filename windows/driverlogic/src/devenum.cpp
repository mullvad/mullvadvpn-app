#include "stdafx.h"
#include "devenum.h"
#include "error.h"

DeviceEnumerator::DeviceEnumerator(const GUID &deviceClass)
{
	m_deviceInfoSet = SetupDiGetClassDevsW
	(
		&deviceClass,
		nullptr,
		nullptr,
		DIGCF_PRESENT
	);

	if (INVALID_HANDLE_VALUE == m_deviceInfoSet)
	{
		THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiGetClassDevsW");
	}

	m_nextDeviceIndex = 0;
	m_exhausted = false;
}

//static
std::unique_ptr<DeviceEnumerator> DeviceEnumerator::Create(const GUID& deviceClass, Filter filter)
{
	auto enumerator = std::make_unique<DeviceEnumerator>(deviceClass);

	enumerator->setFilter(filter);

	return enumerator;
}

DeviceEnumerator::~DeviceEnumerator()
{
	SetupDiDestroyDeviceInfoList(m_deviceInfoSet);
}

bool DeviceEnumerator::next(EnumeratedDevice &device)
{
	if (m_exhausted)
	{
		return false;
	}

	SP_DEVINFO_DATA deviceInfo { 0 };
	deviceInfo.cbSize = sizeof(deviceInfo);

	for (;;)
	{
		if (FALSE == SetupDiEnumDeviceInfo(m_deviceInfoSet, m_nextDeviceIndex, &deviceInfo))
		{
			if (GetLastError() != ERROR_NO_MORE_ITEMS)
			{
				THROW_SETUPAPI_ERROR(GetLastError(), "SetupDiEnumDeviceInfo");
			}

			m_exhausted = true;

			return false;
		}

		++m_nextDeviceIndex;

		if (!m_filter || m_filter(m_deviceInfoSet, deviceInfo))
		{
			break;
		}
	}

	device.deviceInfoSet = m_deviceInfoSet;
	device.deviceInfo = deviceInfo;

	return true;
}
