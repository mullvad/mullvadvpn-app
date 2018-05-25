#pragma once

#include "libcommon/error.h"
#include <string>
#include <vector>
#include <stdexcept>
#include <winerror.h>
#include <atlbase.h>
#include <comutil.h>
#include <atlsafe.h>
#include <wbemidl.h>

#define VALIDATE_COM(status, operation)\
if(FAILED(status))\
{\
	::common::error::Throw(operation, status);\
}

_variant_t ComGetProperty(CComPtr<IWbemClassObject> obj, const std::wstring &name);
_variant_t ComGetPropertyAlways(CComPtr<IWbemClassObject> obj, const std::wstring &name);
std::wstring ComConvertString(BSTR src);
std::vector<std::wstring> ComConvertStringArray(SAFEARRAY *src);
CComSafeArray<BSTR> ComConvertIntoStringArray(const std::vector<std::wstring> &src);

// NOTE: This consumes the source variable
_variant_t ComPackageStringArray(CComSafeArray<BSTR> &src);
