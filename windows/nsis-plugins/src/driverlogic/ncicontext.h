#pragma once

#include <windows.h>
#include <string>

//
// Interface for nci.dll.
//

class NciContext
{
	HMODULE dllHandle;

	using nciGetConnectionNameFunc = DWORD(__stdcall*)(const GUID*, wchar_t*, DWORD, DWORD*);
	using nciSetConnectionNameFunc = DWORD(__stdcall*)(const GUID*, const wchar_t*);

	nciGetConnectionNameFunc m_nciGetConnectionName;
	nciSetConnectionNameFunc m_nciSetConnectionName;

	NciContext(NciContext&) = delete;
	NciContext& operator=(NciContext&) = delete;

public:

	NciContext();
	~NciContext();

	NciContext(NciContext&&) = default;
	NciContext& operator=(NciContext&&) = default;

	std::wstring getConnectionName(const GUID& guid);
	void setConnectionName(const GUID& guid, const wchar_t* newName);
};
