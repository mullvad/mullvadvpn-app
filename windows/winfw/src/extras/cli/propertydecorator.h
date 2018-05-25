#pragma once

#include "ipropertydecorator.h"
#include "libwfp/filterengine.h"
#include <memory>

class PropertyDecorator : public IPropertyDecorator
{
public:

	PropertyDecorator(std::shared_ptr<wfp::FilterEngine> engine);

	//
	// The format of the returned string:
	// [name, first 50 chars of description]
	//

	std::wstring FilterDecoration(UINT64 id) override;
	std::wstring LayerDecoration(UINT16 id) override;
	std::wstring LayerDecoration(const GUID &key) override;
	std::wstring ProviderDecoration(const GUID &key) override;
	std::wstring SublayerDecoration(const GUID &key) override;

private:

	std::shared_ptr<wfp::FilterEngine> m_engine;
};
