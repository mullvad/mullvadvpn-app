#pragma once

#include "windns.h"
#include "libcommon/wmi/connection.h"
#include "libcommon/wmi/notification.h"
#include "configmanager.h"
#include "clientsinkinfo.h"
#include "iclientsinkproxy.h"
#include <vector>
#include <string>
#include <memory>

class WinDnsContext : public IClientSinkProxy
{
public:

	WinDnsContext();
	~WinDnsContext();

	void set(const std::vector<std::wstring> &servers, const ClientSinkInfo &sinkInfo);
	void reset();

	void IClientSinkProxy::error(const char *errorMessage, const char **details, uint32_t numDetails) override;
	void IClientSinkProxy::config(const void *configData, uint32_t dataLength) override;

private:

	std::shared_ptr<common::wmi::Connection> m_connection;
	std::shared_ptr<ConfigManager> m_configManager;
	std::unique_ptr<common::wmi::Notification> m_notification;
	ClientSinkInfo m_sinkInfo;
};
