#pragma once

#include <atlbase.h>
#include <wbemidl.h>

namespace wmi
{

//
// IEventSink, use with:
//
// __InstanceCreationEvent
// __InstanceDeletionEvent
//
struct IEventSink
{
	virtual ~IEventSink() = 0
	{
	}

	virtual void update(CComPtr<IWbemClassObject> instance) = 0;
};

//
// IModificationEventSink, use with:
//
// __InstanceModificationEvent
//
struct IModificationEventSink
{
	virtual ~IModificationEventSink() = 0
	{
	}

	virtual void update(CComPtr<IWbemClassObject> previous, CComPtr<IWbemClassObject> target) = 0;
};

}
