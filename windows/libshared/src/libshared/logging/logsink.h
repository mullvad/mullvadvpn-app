#pragma once

//
// This file is shared between DLL modules to help define their public interface.
// It should always be C-compatible.
//

enum MULLVAD_LOG_LEVEL
{
	MULLVAD_LOG_LEVEL_ERROR = 1,
	MULLVAD_LOG_LEVEL_WARNING,
	MULLVAD_LOG_LEVEL_INFO,
	MULLVAD_LOG_LEVEL_DEBUG,
	MULLVAD_LOG_LEVEL_TRACE
};

//
// The log sink is registered with a DLL during e.g. initialization.
// It may later be activated as a direct or indirect result of calling into the DLL.
//
// The parameters are:
//
// `MULLVAD_LOG_LEVEL` - Severity of the message.
// `const char *` - The message itself.
// `void *` - The sink context that was registered along with the sink.
//
typedef void (__stdcall *MullvadLogSink)(MULLVAD_LOG_LEVEL, const char *, void *);
