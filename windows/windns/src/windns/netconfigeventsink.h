#pragma once

#include "libcommon/wmi/ieventsink.h"
#include "libcommon/wmi/iconnection.h"
#include "configmanager.h"
#include "iclientsinkproxy.h"
#include <memory>

class NetConfigEventSink : public common::wmi::IModificationEventSink
{
public:

	NetConfigEventSink(std::shared_ptr<common::wmi::IConnection> connection,
		std::shared_ptr<ConfigManager> configManager, IClientSinkProxy *clientSinkProxy);

	void update(CComPtr<IWbemClassObject> previous, CComPtr<IWbemClassObject> target) override;

private:

	std::shared_ptr<common::wmi::IConnection> m_connection;
	std::shared_ptr<ConfigManager> m_configManager;

	IClientSinkProxy *m_clientSinkProxy;
};
