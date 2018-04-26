#pragma once

#include "windns/wmi/eventsink.h"
#include "windns/wmi/iconnection.h"
#include "windns/configmanager.h"
#include <memory>

class NetConfigEventSink : public wmi::IEventSink
{
public:

	explicit NetConfigEventSink(std::shared_ptr<wmi::IConnection> connection, std::shared_ptr<ConfigManager> configManager);

	void update(CComPtr<IWbemClassObject> instance) override;

private:

	std::shared_ptr<wmi::IConnection> m_connection;
	std::shared_ptr<ConfigManager> m_configManager;
};
