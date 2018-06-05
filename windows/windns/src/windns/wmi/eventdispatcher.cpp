#include "stdafx.h"
#include "eventdispatcher.h"
#include "windns/comhelpers.h"

namespace wmi
{

EventDispatcherBase::EventDispatcherBase()
	: m_references(0)
	, m_callbacks(0)
{
}

bool EventDispatcherBase::processing() const
{
	//
	// Cancelling the notification registration WILL NOT wait for the completion of
	// callbacks currently in progress :-(
	//
	// (Observed on Win10)
	//

	return 0 != InterlockedAdd(&m_callbacks, 0);
}

ULONG STDMETHODCALLTYPE EventDispatcherBase::AddRef()
{
	return InterlockedIncrement(&m_references);
}

ULONG STDMETHODCALLTYPE EventDispatcherBase::Release()
{
	auto refs = InterlockedDecrement(&m_references);

	if (refs == 0)
	{
		delete this;
	}

	return refs;
}

HRESULT STDMETHODCALLTYPE EventDispatcherBase::QueryInterface(REFIID riid, void **ppv)
{
	if (IID_IUnknown == riid || IID_IWbemObjectSink == riid)
	{
		*ppv = (IWbemObjectSink *)this;
		AddRef();

		return WBEM_S_NO_ERROR;
	}

	return E_NOINTERFACE;
}

HRESULT STDMETHODCALLTYPE EventDispatcherBase::Indicate
(
	LONG numObjects,
	IWbemClassObject __RPC_FAR *__RPC_FAR *objects
)
{
	InterlockedIncrement(&m_callbacks);

	try
	{
		for (LONG i = 0; i < numObjects; ++i)
		{
			CComPtr<IWbemClassObject> eventRecord(objects[i]);

			dispatch(eventRecord);
		}
	}
	catch (...)
	{
		//
		// There is nowhere to forward this error :-(
		//
	}

	InterlockedDecrement(&m_callbacks);

	return WBEM_S_NO_ERROR;
}

HRESULT STDMETHODCALLTYPE EventDispatcherBase::SetStatus
(
	LONG, HRESULT, BSTR, IWbemClassObject __RPC_FAR *
)
{
	return WBEM_S_NO_ERROR;
}

EventDispatcher::EventDispatcher(std::shared_ptr<IEventSink> eventSink)
	: m_eventSink(eventSink)
{
}

void EventDispatcher::dispatch(CComPtr<IWbemClassObject> eventRecord)
{
	auto rawTarget = ComGetPropertyAlways(eventRecord, L"TargetInstance");

	CComQIPtr<IWbemClassObject> target(V_UNKNOWN(&rawTarget));

	m_eventSink->update(target);
}

ModificationEventDispatcher::ModificationEventDispatcher(std::shared_ptr<IModificationEventSink> eventSink)
	: m_eventSink(eventSink)
{
}

void ModificationEventDispatcher::dispatch(CComPtr<IWbemClassObject> eventRecord)
{
	auto rawPrevious = ComGetPropertyAlways(eventRecord, L"PreviousInstance");
	auto rawTarget = ComGetPropertyAlways(eventRecord, L"TargetInstance");

	CComQIPtr<IWbemClassObject> previous(V_UNKNOWN(&rawPrevious));
	CComQIPtr<IWbemClassObject> target(V_UNKNOWN(&rawTarget));

	m_eventSink->update(previous, target);
}

}
