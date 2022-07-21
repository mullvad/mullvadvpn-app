#include "stdafx.h"
#include "service.h"
#include "log.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>

#undef min
#undef max
#include <chrono>

template<typename TTime = std::chrono::milliseconds>
class TimeBox
{
	// `steady_clock` wraps around every ~292 years.
	using Clock = std::chrono::steady_clock;
	using ClockTimePoint = std::chrono::time_point<Clock>;

public:

	TimeBox(typename TTime::rep maxWaitTime)
		: m_startTime(Clock::now())
		, m_maxWaitTime(TTime(maxWaitTime))
	{
	}

	bool expired() const
	{
		const auto now = Clock::now();

		const auto elapsed =
		(
			(now < m_startTime)
			? (ClockTimePoint::max() - m_startTime) + (now - ClockTimePoint::min())
			: now - m_startTime
		);

		return std::chrono::duration_cast<TTime>(elapsed) > m_maxWaitTime;
	}

private:

	ClockTimePoint m_startTime;
	TTime m_maxWaitTime;
};

SERVICE_STATUS_PROCESS GetServiceProcessStatus(SC_HANDLE service)
{
	SERVICE_STATUS_PROCESS ssp;

	DWORD bytesNeeded;

	auto status = QueryServiceStatusEx
	(
		service,
		SC_STATUS_PROCESS_INFO,
		reinterpret_cast<BYTE*>(&ssp),
		sizeof(ssp),
		&bytesNeeded
	);

	if (status != 0)
	{
		return ssp;
	}

	THROW_WINDOWS_ERROR(GetLastError(), "QueryServiceStatusEx");
}

void WaitUntilServiceStopped(SC_HANDLE service, DWORD maxWaitMs)
{
	TimeBox timer(maxWaitMs);

	for (;;)
	{
		try
		{
			const auto status = GetServiceProcessStatus(service);

			if (status.dwCurrentState == SERVICE_STOPPED)
			{
				return;
			}
		}
		catch (...)
		{
		}

		if (timer.expired())
		{
			THROW_ERROR("Failed when waiting for service to stop");
		}

		Sleep(100);
	}
}

bool ServiceIsRunning(const std::wstring &serviceName)
{
	const auto serviceManager = OpenSCManagerW(nullptr, SERVICES_ACTIVE_DATABASE, SC_MANAGER_ALL_ACCESS);

	if (serviceManager == NULL)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "OpenSCManagerW");
	}

	common::memory::ScopeDestructor dtor;

	dtor += [serviceManager]()
	{
		CloseServiceHandle(serviceManager);
	};

	const auto service = OpenServiceW(serviceManager, serviceName.c_str(), SERVICE_ALL_ACCESS);

	if (service == NULL)
	{
		const auto error = GetLastError();

		if (error != ERROR_SERVICE_DOES_NOT_EXIST)
		{
			THROW_WINDOWS_ERROR(error, "OpenServiceW");
		}

		return false;
	}

	dtor += [service]()
	{
		CloseServiceHandle(service);
	};

	return GetServiceProcessStatus(service).dwCurrentState == SERVICE_RUNNING;
}

void PokeService(const std::wstring &serviceName, bool stopService, bool deleteService)
{
	auto serviceManager = OpenSCManagerW(nullptr, SERVICES_ACTIVE_DATABASE, SC_MANAGER_ALL_ACCESS);

	if (serviceManager == NULL)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "OpenSCManagerW");
	}

	common::memory::ScopeDestructor dtor;

	dtor += [serviceManager]()
	{
		CloseServiceHandle(serviceManager);
	};

	auto service = OpenServiceW(serviceManager, serviceName.c_str(), SERVICE_ALL_ACCESS);

	if (service == NULL)
	{
		const auto error = GetLastError();

		if (error != ERROR_SERVICE_DOES_NOT_EXIST)
		{
			THROW_WINDOWS_ERROR(error, "OpenServiceW");
		}

		// If the service does not exist, we're done.

		return;
	}

	dtor += [service]()
	{
		CloseServiceHandle(service);
	};

	if (stopService)
	{
		Log(L"Stopping service");

		SERVICE_STATUS ss;

		ControlService(service, SERVICE_CONTROL_STOP, &ss);

		WaitUntilServiceStopped(service, 1000 * 5);

		Log(L"Successfully stopped service");
	}

	if (deleteService)
	{
		Log(L"Deleting service");

		auto status = DeleteService(service);

		if (status == 0)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "DeleteService");
		}

		Log(L"Successfully deleted service");
	}
}
