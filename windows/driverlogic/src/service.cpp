#include "stdafx.h"
#include "service.h"
#include "log.h"
#include <libcommon/error.h>
#include <libcommon/memory.h>

void WaitUntilServiceStopped(SC_HANDLE service, DWORD maxWaitMs)
{
	const auto endTime = GetTickCount() + maxWaitMs;

	for (;;)
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

		if (status != 0
			&& ssp.dwCurrentState == SERVICE_STOPPED)
		{
			return;
		}

		if (GetTickCount() > endTime)
		{
			throw std::runtime_error("Failed when waiting for service to stop");
		}

		Sleep(100);
	}
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
		THROW_WINDOWS_ERROR(GetLastError(), "OpenServiceW");
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
