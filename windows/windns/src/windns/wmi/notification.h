#pragma once

#include "iconnection.h"
#include "eventdispatcher.h"
#include <atlbase.h>
#include <memory>

namespace wmi
{

class Notification
{
public:

	Notification(std::shared_ptr<IConnection> connection, CComPtr<IEventDispatcher> dispatcher);
	~Notification();

	Notification(const Notification &) = delete;
	Notification &operator=(const Notification &) = delete;
	Notification(Notification &&) = delete;
	Notification &operator=(Notification &&) = delete;

	void activate(const std::wstring &query);
	void deactivate();

private:

	std::shared_ptr<IConnection> m_connection;
	CComPtr<IEventDispatcher> m_dispatcher;

	CComPtr<IWbemObjectSink> m_forwarder;
};

}
