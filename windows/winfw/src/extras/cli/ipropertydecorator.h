#pragma once

//
// These methods should be used during property extraction in order to translate
// identifiers (references) into something more meaningful.
//
// Ideally we would nest a PropertyList inside a PropertyList, but that's a rather
// large update since each element in the PropertyList would need to be wrapped.
// If such an update is made one day, be sure to also add a "group" element so
// we can have better structuring within a PropertyList
//

#include <windows.h>
#include <string>

struct IPropertyDecorator
{
	//
	// These methods should return a short string that adds
	// value for human operators/analysts.
	//

	virtual std::wstring FilterDecoration(UINT64 id) = 0;
	virtual std::wstring LayerDecoration(UINT16 id) = 0;
	virtual std::wstring LayerDecoration(const GUID &key) = 0;
	virtual std::wstring ProviderDecoration(const GUID &key) = 0;
	virtual std::wstring SublayerDecoration(const GUID &key) = 0;

	virtual ~IPropertyDecorator() = 0
	{
	}
};
