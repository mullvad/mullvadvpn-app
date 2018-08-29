#pragma once

#include <string>
#include <vector>

//
// Import-only header
//
// Note on interfaces: While it's safer to use plain types for arguments, this is OK
// since the plugins are all built at the same time, and have the same interpretation of used types.
//

void __declspec(dllimport) __stdcall PluginLog
(
	const std::wstring &message
);

void __declspec(dllimport) __stdcall PluginLogWithDetails
(
	const std::wstring &message,
	const std::vector<std::wstring> &details
);
