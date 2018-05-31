#include "stdafx.h"
#include "notification.h"
#include "windns/comhelpers.h"

namespace wmi
{

Notification::Notification(std::shared_ptr<IConnection> connection, CComPtr<IEventDispatcher> dispatcher)
	: m_connection(connection)
	, m_dispatcher(dispatcher)
{
}

Notification::~Notification()
{
	//
	// TODO: Revise to avoid exceptions in dtor.
	//
	deactivate();
}

void Notification::activate(const std::wstring &query)
{
	CComPtr<IUnsecuredApartment> apartment;

	auto status = CoCreateInstance(CLSID_UnsecuredApartment, nullptr, CLSCTX_LOCAL_SERVER, IID_IUnsecuredApartment, (void**)&apartment);
	VALIDATE_COM(status, "Create unsecured COM apartment");

	CComPtr<IUnknown> unknownEventSink;

	status = m_dispatcher->QueryInterface(IID_IUnknown, (void**)&unknownEventSink);
	VALIDATE_COM(status, "Retrieve IUnkown interface for event sink");

	CComPtr<IUnknown> unknownForwarder;

	status = apartment->CreateObjectStub(unknownEventSink, &unknownForwarder);
	VALIDATE_COM(status, "Create forwarder for event sink");

	status = unknownForwarder->QueryInterface(IID_IWbemObjectSink, (void**)&m_forwarder);
	VALIDATE_COM(status, "Retrieve sink interface on event sink forwarder");

	status = m_connection->services()->ExecNotificationQueryAsync(_bstr_t("WQL"), _bstr_t(query.c_str()), 0, nullptr, m_forwarder);
	VALIDATE_COM(status, "Register notification query with WMI");
}

void Notification::deactivate()
{
	if (nullptr == m_forwarder)
	{
		return;
	}

	auto status = m_connection->services()->CancelAsyncCall(m_forwarder);
	VALIDATE_COM(status, "Cancel notification query");

	m_forwarder.Release();

	//
	// This is a hack-solution for a corner case issue.
	//
	// Since cancelling the notification registration does not wait for in-progress callbacks
	// to complete, we have to implement the corresponding logic ourselves.
	//
	// Using a Sleep() here is preferable to introducing a critical section in the callback.
	//
	while (m_dispatcher->processing())
	{
		Sleep(100);
	}
}

}
