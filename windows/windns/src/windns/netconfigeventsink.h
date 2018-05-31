#pragma once

#include "windns/wmi/ieventsink.h"
#include "windns/wmi/iconnection.h"
#include "windns/configmanager.h"
#include <memory>

class NetConfigEventSink : public wmi::IModificationEventSink
{
public:

	NetConfigEventSink(std::shared_ptr<wmi::IConnection> connection, std::shared_ptr<ConfigManager> configManager);

	void update(CComPtr<IWbemClassObject> previous, CComPtr<IWbemClassObject> target) override;

private:

	std::shared_ptr<wmi::IConnection> m_connection;
	std::shared_ptr<ConfigManager> m_configManager;
};
