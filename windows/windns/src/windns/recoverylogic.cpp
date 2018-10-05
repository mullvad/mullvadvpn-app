#include "stdafx.h"
#include "recoverylogic.h"
#include "netsh.h"
#include "confineoperation.h"
#include <libcommon/trace/xtrace.h>
#include <stdexcept>

//static
void RecoveryLogic::RestoreInterfaces(const RecoveryFormatter::Unpacked &data,
	ILogSink *logSink, uint32_t timeout)
{
	if (nullptr == logSink)
	{
		throw std::runtime_error("Invalid logger sink");
	}

	auto forwardError = [logSink](const char *msg, const char **details, uint32_t numDetails)
	{
		logSink->error(msg, details, numDetails);
	};

	bool success = true;

	for (const auto &snap : data.v4Snaps)
	{
		const auto status = ConfineOperation("Reset interface DNS settings", forwardError, [&snap, &timeout]()
		{
			if (snap.internalInterface())
			{
				//
				// This is an interface used for internal communication.
				// We haven't changed any settings on it and therefore should not restore it.
				//
				return;
			}

			XTRACE(L"Resetting name server configuration for interface ", snap.interfaceGuid());

			uint32_t interfaceIndex = 0;

			try
			{
				interfaceIndex = NetSh::ConvertInterfaceGuidToIndex(snap.interfaceGuid());
			}
			catch (...)
			{
				//
				// The interface cannot be linked to a virtual or physical adapter.
				// It's either floating or has been removed.
				//

				XTRACE(L"Ignoring floating/invalid interface ", snap.interfaceGuid());
				return;
			}

			if (snap.nameServers().empty())
			{
				NetSh::Instance().SetIpv4DhcpDns(interfaceIndex, timeout);
			}
			else
			{
				NetSh::Instance().SetIpv4StaticDns(interfaceIndex, snap.nameServers(), timeout);
			}
		});

		if (false == status)
		{
			success = false;
		}
	}

	for (const auto &snap : data.v6Snaps)
	{
		const auto status = ConfineOperation("Reset interface DNS settings", forwardError, [&snap, &timeout]()
		{
			if (snap.internalInterface())
			{
				//
				// This is an interface used for internal communication.
				// We haven't changed any settings on it and therefore should not restore it.
				//
				return;
			}

			XTRACE(L"Resetting name server configuration for interface ", snap.interfaceGuid());

			uint32_t interfaceIndex = 0;

			try
			{
				interfaceIndex = NetSh::ConvertInterfaceGuidToIndex(snap.interfaceGuid());
			}
			catch (...)
			{
				//
				// The interface cannot be linked to a virtual or physical adapter.
				// It's either floating or has been removed.
				//

				XTRACE(L"Ignoring floating/invalid interface ", snap.interfaceGuid());
				return;
			}

			if (snap.nameServers().empty())
			{
				NetSh::Instance().SetIpv6DhcpDns(interfaceIndex, timeout);
			}
			else
			{
				NetSh::Instance().SetIpv6StaticDns(interfaceIndex, snap.nameServers(), timeout);
			}
		});

		if (false == status)
		{
			success = false;
		}
	}

	if (false == success)
	{
		throw std::runtime_error("Could not reset DNS settings for one of more interfaces");
	}
}
