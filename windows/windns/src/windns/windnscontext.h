#pragma once

#include "windns.h"
#include "libcommon/wmi/connection.h"
#include "libcommon/wmi/notification.h"
#include "configmanager.h"
#include "clientsinkinfo.h"
#include <vector>
#include <string>
#include <memory>

class WinDnsContext
{
public:

	WinDnsContext();
	~WinDnsContext();

	void set(const std::vector<std::wstring> &servers, const ClientSinkInfo &sinkInfo);
	void reset();

private:

	std::shared_ptr<common::wmi::Connection> m_connection;
	std::shared_ptr<ConfigManager> m_configManager;
	std::unique_ptr<common::wmi::Notification> m_notification;
	ClientSinkInfo m_sinkInfo;
};
