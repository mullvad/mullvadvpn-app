#pragma once

#include "logsink.h"
#include <iostream>

namespace shared
{

void __stdcall StdoutLogger(MULLVAD_LOG_LEVEL level, const char *msg, void *)
{
	switch (level)
	{
		case MULLVAD_LOG_LEVEL_WARNING:
			std::cout << "Warning: ";
			break;
		case MULLVAD_LOG_LEVEL_INFO:
			std::cout << "Info: ";
			break;
		case MULLVAD_LOG_LEVEL_DEBUG:
			std::cout << "Debug: ";
			break;
		case MULLVAD_LOG_LEVEL_TRACE:
			std::cout << "Trace: ";
			break;
		case MULLVAD_LOG_LEVEL_ERROR:
		default:
			std::cout << "Error: ";
			break;
	}

	std::cout << msg << std::endl;
}

}
