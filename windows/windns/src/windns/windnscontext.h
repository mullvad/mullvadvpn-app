#pragma once

#include "windns.h"
#include "wmi/connection.h"
#include "wmi/notification.h"
#include "configmanager.h"
#include "clientsinkinfo.h"
#include <vector>
#include <string>
#include <memory>

class WinDnsContext
{
public:

	WinDnsContext();

	// TODO: Review.
	~WinDnsContext()
	{
		reset();
	}

	bool set(const std::vector<std::wstring> &servers, const ClientSinkInfo &sinkInfo);
	bool reset();

private:

	std::shared_ptr<wmi::Connection> m_connection;
	std::shared_ptr<ConfigManager> m_configManager;
	std::unique_ptr<wmi::Notification> m_notification;
	ClientSinkInfo m_sinkInfo;
};
