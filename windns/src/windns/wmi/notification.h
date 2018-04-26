#pragma once

#include "iconnection.h"
#include "eventsink.h"
#include <atlbase.h>
#include <memory>

namespace wmi
{

class Notification
{
public:

	Notification(std::shared_ptr<IConnection> connection, CComPtr<EventSink> eventSink);
	~Notification();

	Notification(const Notification &) = delete;
	Notification &operator=(const Notification &) = delete;
	Notification(Notification &&) = delete;
	Notification &operator=(Notification &&) = delete;

	void activate(const std::wstring &query);
	void deactivate();

private:

	std::shared_ptr<IConnection> m_connection;
	CComPtr<EventSink> m_eventSink;

	CComPtr<IWbemObjectSink> m_forwarder;
};

}
