#include "stdafx.h"
#include "logsinkadapter.h"

namespace shared::logging
{

LogSinkAdapter::LogSinkAdapter(MullvadLogSink target, void *context)
	: LogSink(MakeAdapter(target, context))
{
}

//static
common::logging::LogTarget LogSinkAdapter::MakeAdapter(MullvadLogSink target, void *context)
{
	return [target, context](common::logging::LogLevel level, const char* msg)
	{
		if (nullptr == target)
		{
			return;
		}

		//
		// TODO: Replace manual mapping with ValueMapper once the updated
		// ValueMapper reaches libcommon.
		//

		const MULLVAD_LOG_LEVEL translatedLevel = [level]()
		{
			switch (level)
			{
				case common::logging::LogLevel::Warning:
					return MULLVAD_LOG_LEVEL_WARNING;
				case common::logging::LogLevel::Info:
					return MULLVAD_LOG_LEVEL_INFO;
				case common::logging::LogLevel::Trace:
					return MULLVAD_LOG_LEVEL_TRACE;
				case common::logging::LogLevel::Debug:
					return MULLVAD_LOG_LEVEL_DEBUG;
				case common::logging::LogLevel::Error:
				default:
					return MULLVAD_LOG_LEVEL_ERROR;
			}
		}();

		target(translatedLevel, msg, context);
	};
}

}
