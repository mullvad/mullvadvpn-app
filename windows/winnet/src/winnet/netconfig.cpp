#include "stdafx.h"
#include "netconfig.h"
#include <stdexcept>
#include <sstream>
#include <windows.h>
#include <netcfgx.h>
#include <devguid.h>
#include <libcommon/error.h>
#include <libcommon/string.h>
#include <libcommon/memory.h>
#include <libshared/network/interfaceutils.h>


namespace
{

const wchar_t NETCFG_LOCK_CLIENT_NAME[] = L"MULLVAD";
constexpr uint16_t NETCFG_LOCK_TIMEOUT = 5000; // milliseconds
const wchar_t NETCFG_IPV6_COMPONENT_NAME[] = L"MS_TCPIP6";

void SetIpv6BindingForBindName(INetCfg *netCfg, const std::wstring &bindName, bool enable)
{
	INetCfgComponent *transactionComponent = nullptr;
	HRESULT result = netCfg->FindComponent(NETCFG_IPV6_COMPONENT_NAME, &transactionComponent);

	if (S_OK != result)
	{
		THROW_ERROR("Failed to obtain transaction component");
	}

	common::memory::ScopeDestructor scopeDest;
	scopeDest += [&transactionComponent]() {
		transactionComponent->Release();
	};

	INetCfgComponentBindings *bindings = nullptr;
	result = transactionComponent->QueryInterface(
		IID_INetCfgComponentBindings,
		reinterpret_cast<void**>(&bindings)
	);

	if (S_OK != result)
	{
		std::wstringstream ss;
		ss << L"Failed to obtain component bindings for ";
		ss << NETCFG_IPV6_COMPONENT_NAME;
		THROW_ERROR(common::string::ToAnsi(ss.str()).c_str());
	}

	IEnumNetCfgBindingPath *pathsEnum = NULL;
	result = bindings->EnumBindingPaths(EBP_BELOW, &pathsEnum);

	bindings->Release();
	bindings = nullptr;

	if (S_OK != result)
	{
		THROW_ERROR("Failed to acquire binding path enumerator");
	}

	scopeDest += [&pathsEnum]() {
		pathsEnum->Release();
	};

	INetCfgBindingPath *bindingPath = NULL;

	result = pathsEnum->Next(1, &bindingPath, nullptr);

	for (; S_OK == result; result = pathsEnum->Next(1, &bindingPath, nullptr))
	{
		IEnumNetCfgBindingInterface *enumInterface = nullptr;
		HRESULT enumResult = bindingPath->EnumBindingInterfaces(&enumInterface);

		if (S_OK != enumResult)
		{
			THROW_ERROR("Failed to acquire binding path interfaces");
		}

		common::memory::ScopeDestructor enumDestructor;
		enumDestructor += [&enumInterface]() {
			enumInterface->Release();
		};

		INetCfgBindingInterface *iface = nullptr;

		while (S_OK == enumInterface->Next(1, &iface, nullptr))
		{
			INetCfgComponent *cfgComponent = nullptr;

			if (S_OK != iface->GetLowerComponent(&cfgComponent))
			{
				THROW_ERROR("Failed to acquire binding interface component");
			}

			iface->Release();

			wchar_t *componentBindName = 0;

			if (S_OK != cfgComponent->GetBindName(&componentBindName))
			{
				THROW_ERROR("Failed to acquire bind name");
			}

			cfgComponent->Release();

			if (0 == _wcsicmp(bindName.c_str(), componentBindName))
			{
				CoTaskMemFree(componentBindName);

				//
				// Apply the changes and exit the function
				//

				result = bindingPath->Enable(enable);
				if (S_OK != result)
				{
					THROW_ERROR("Failed to set IPv6 status");
				}
				netCfg->Apply();

				bindingPath->Release();
				bindingPath = nullptr;

				return;
			}

			CoTaskMemFree(componentBindName);
		}

		bindingPath->Release();
		bindingPath = nullptr;
	}
}

} // anonymous namespace


void EnableIpv6ForAdapter(const std::wstring &adapterGuid)
{
	//
	// Initialize COM
	// TODO: Should we do this when winnet is initialized?
	//

	HRESULT result = CoInitialize(nullptr);

	if (S_OK != result)
	{
		std::stringstream ss;
		ss << "Failed to initialize COM: " << result;
		THROW_ERROR(ss.str().c_str());
	}

	common::memory::ScopeDestructor scopeDest;
	scopeDest += []() {
		CoUninitialize();
	};

	//
	// Initialize INetCfg
	//

	INetCfg *netCfg = nullptr;
	result = CoCreateInstance(
		CLSID_CNetCfg,
		nullptr,
		CLSCTX_INPROC_SERVER,
		IID_INetCfg,
		reinterpret_cast<void **>(&netCfg)
	);

	if (S_OK != result)
	{
		std::stringstream ss;
		ss << "Failed to initialize COM: " << result;
		THROW_ERROR(ss.str().c_str());

	}

	scopeDest += [&netCfg]() { netCfg->Release(); };

	INetCfgLock *netCfgLock = nullptr;
	result = netCfg->QueryInterface(IID_INetCfgLock, reinterpret_cast<void **>(&netCfgLock));

	if (S_OK != result)
	{
		std::stringstream ss;
		ss << "Failed to obtain INetCfg lock interface: " << result;
		THROW_ERROR(ss.str().c_str());
	}

	scopeDest += [&netCfgLock]() {
		netCfgLock->Release();
	};

	wchar_t *blockingApplication = nullptr;
	scopeDest += [&blockingApplication]() {
		CoTaskMemFree(blockingApplication);
	};

	// NOTE: This should be done before initializing INetCfg
	result = netCfgLock->AcquireWriteLock(
		NETCFG_LOCK_TIMEOUT,
		NETCFG_LOCK_CLIENT_NAME,
		&blockingApplication
	);

	if (S_OK != result)
	{
		std::wstringstream ss;
		ss << L"Failed to acquire write lock";
		if (nullptr != blockingApplication)
		{
			ss << L" due to application: " << blockingApplication;
		}
		ss << ". (" << result << ")";

		THROW_ERROR(common::string::ToAnsi(ss.str()).c_str());
	}

	scopeDest += [&netCfgLock]() {
		netCfgLock->ReleaseWriteLock();
	};

	result = netCfg->Initialize(nullptr);

	if (S_OK != result)
	{
		std::stringstream ss;
		ss << "Failed to initialize INetCfg: " << result;
		THROW_ERROR(ss.str().c_str());
	}

	scopeDest += [&netCfg]() { netCfg->Uninitialize(); };

	SetIpv6BindingForBindName(netCfg, adapterGuid, true);
}
