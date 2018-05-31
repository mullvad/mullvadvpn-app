#pragma once

#include "ieventsink.h"
#include <memory>
#include <atlbase.h>
#include <wbemidl.h>

namespace wmi
{

struct IEventDispatcher : public IWbemObjectSink
{
	virtual ~IEventDispatcher() = 0
	{
	}

	virtual bool processing() const = 0;
};

//
// EventDispatcherBase
//
// Base class for event dispatcher implementations.
//
// The base class has all the logic and COM management but defers actual dispatching
// to the derived class.
//
// From the perspective of COM, this is an event sink, but from the perspective of
// WINDNS code it's a dispatcher that dispatches to the actual sink.
//
class EventDispatcherBase : public IEventDispatcher
{
public:

	EventDispatcherBase();

	virtual ~EventDispatcherBase() = 0
	{
	}

	bool processing() const override;

	ULONG STDMETHODCALLTYPE AddRef() override;
	ULONG STDMETHODCALLTYPE Release() override;
	HRESULT STDMETHODCALLTYPE QueryInterface(REFIID riid, void **ppv) override;

	HRESULT STDMETHODCALLTYPE Indicate
	(
		LONG numObjects,
		IWbemClassObject __RPC_FAR *__RPC_FAR *objects
	)
	override;

	HRESULT STDMETHODCALLTYPE SetStatus
	(
		LONG flags,
		HRESULT result,
		BSTR param,
		IWbemClassObject __RPC_FAR *object
	)
	override;

protected:

	virtual void dispatch(CComPtr<IWbemClassObject> eventRecord) = 0;

private:

	LONG m_references;
	mutable LONG m_callbacks;
};

class EventDispatcher : public EventDispatcherBase
{
public:

	EventDispatcher(std::shared_ptr<IEventSink> eventSink);

protected:

	void dispatch(CComPtr<IWbemClassObject> eventRecord) override;

private:

	std::shared_ptr<IEventSink> m_eventSink;
};

class ModificationEventDispatcher : public EventDispatcherBase
{
public:

	ModificationEventDispatcher(std::shared_ptr<IModificationEventSink> eventSink);

protected:

	void dispatch(CComPtr<IWbemClassObject> eventRecord) override;

private:

	std::shared_ptr<IModificationEventSink> m_eventSink;
};

}
