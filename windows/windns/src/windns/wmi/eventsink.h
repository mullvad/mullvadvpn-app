#pragma once

#include <memory>
#include <atlbase.h>
#include <wbemidl.h>

namespace wmi
{

struct IEventSink
{
	virtual ~IEventSink() = 0
	{
	}

	virtual void update(CComPtr<IWbemClassObject> instance) = 0;
};

class EventSink : public IWbemObjectSink
{
public:

	EventSink(std::shared_ptr<IEventSink> eventSink);
	~EventSink();

	bool processing() const;

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

private:

	LONG m_references;
	mutable LONG m_callbacks;
	std::shared_ptr<IEventSink> m_eventSink;
};

}
