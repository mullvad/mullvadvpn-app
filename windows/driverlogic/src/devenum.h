#pragma once

#include <windows.h>
#include <newdev.h>
#include <functional>
#include <memory>
#include "device.h"

class DeviceEnumerator
{
public:

	using Filter = std::function<bool(HDEVINFO, const SP_DEVINFO_DATA&)>;

	DeviceEnumerator(const GUID &deviceClass);

	static std::unique_ptr<DeviceEnumerator> Create(const GUID &deviceClass, Filter filter);

	~DeviceEnumerator();

	DeviceEnumerator(const DeviceEnumerator &) = delete;
	DeviceEnumerator(DeviceEnumerator &&) = delete;
	DeviceEnumerator &operator=(const DeviceEnumerator &) = delete;
	DeviceEnumerator &operator=(DeviceEnumerator &&) = delete;

	void setFilter(Filter filter)
	{
		m_filter = filter;
	}

	bool next(EnumeratedDevice &device);

private:

	HDEVINFO m_deviceInfoSet;

	int m_nextDeviceIndex;

	bool m_exhausted;

	Filter m_filter;
};
