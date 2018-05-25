#include "stdafx.h"
#include "eventsink.h"
#include "windns/comhelpers.h"

namespace wmi
{

EventSink::EventSink(std::shared_ptr<IEventSink> eventSink)
	: m_references(0)
	, m_callbacks(0)
	, m_eventSink(eventSink)
{
}

EventSink::~EventSink()
{
}

bool EventSink::processing() const
{
	//
	// Cancelling the notification registration WILL NOT wait for the completion of
	// callbacks currently in progress :-(
	//
	// (Observed on Win10)
	//

	return 0 != InterlockedAdd(&m_callbacks, 0);
}

ULONG STDMETHODCALLTYPE EventSink::AddRef()
{
	return InterlockedIncrement(&m_references);
}

ULONG STDMETHODCALLTYPE EventSink::Release()
{
	auto refs = InterlockedDecrement(&m_references);

	if (refs == 0)
	{
		delete this;
	}

	return refs;
}

HRESULT STDMETHODCALLTYPE EventSink::QueryInterface(REFIID riid, void **ppv)
{
	if (IID_IUnknown == riid || IID_IWbemObjectSink == riid)
	{
		*ppv = (IWbemObjectSink *)this;
		AddRef();

		return WBEM_S_NO_ERROR;
	}

	return E_NOINTERFACE;
}

HRESULT STDMETHODCALLTYPE EventSink::Indicate
(
	LONG numObjects,
	IWbemClassObject __RPC_FAR *__RPC_FAR *objects
)
{
	InterlockedIncrement(&m_callbacks);

	for (LONG i = 0; i < numObjects; ++i)
	{
		CComPtr<IWbemClassObject> eventRecord(objects[i]);

		auto rawTarget = ComGetPropertyAlways(eventRecord, L"TargetInstance");

		CComQIPtr<IWbemClassObject> target(V_UNKNOWN(&rawTarget));

		m_eventSink->update(target);
	}

	InterlockedDecrement(&m_callbacks);

	return WBEM_S_NO_ERROR;
}

HRESULT STDMETHODCALLTYPE EventSink::SetStatus
(
	LONG, HRESULT, BSTR, IWbemClassObject __RPC_FAR *
)
{
	return WBEM_S_NO_ERROR;
}

}
