#include "stdafx.h"
#include <libcommon/valuemapper.h>
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

		const std::optional<MULLVAD_LOG_LEVEL> translatedLevel = common::ValueMapper::TryMap<>(level, {
			std::make_pair(common::logging::LogLevel::Warning, MULLVAD_LOG_LEVEL_WARNING),
			std::make_pair(common::logging::LogLevel::Info, MULLVAD_LOG_LEVEL_INFO),
			std::make_pair(common::logging::LogLevel::Trace, MULLVAD_LOG_LEVEL_TRACE),
			std::make_pair(common::logging::LogLevel::Debug, MULLVAD_LOG_LEVEL_DEBUG),
			std::make_pair(common::logging::LogLevel::Error, MULLVAD_LOG_LEVEL_ERROR),
		});

		target(translatedLevel.value_or(MULLVAD_LOG_LEVEL_ERROR), msg, context);
	};
}

}
