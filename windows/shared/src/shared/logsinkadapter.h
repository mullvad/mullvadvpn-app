#include "logsink.h"
#include <libcommon/logging/logsink.h>

namespace shared
{

//
// Adapt common::logging::LogSink C++ world to
// MullvadLogSink C world.
//
class LogSinkAdapter : public common::logging::LogSink
{
public:

	LogSinkAdapter(MullvadLogSink target, void *context)
		: LogSink(MakeAdapter(target, context))
	{
	}

private:

	static common::logging::LogTarget MakeAdapter(MullvadLogSink target, void *context)
	{
		return [target, context](common::logging::Severity s, const char *msg)
		{
			if (nullptr == target)
			{
				return;
			}

			const MULLVAD_LOG_SINK_SEVERITY severity = [s]()
			{
				switch (s)
				{
					case common::logging::Severity::Warning:
						return MULLVAD_LOG_SINK_SEVERITY_WARNING;
					case common::logging::Severity::Info:
						return MULLVAD_LOG_SINK_SEVERITY_INFO;
					case common::logging::Severity::Trace:
						return MULLVAD_LOG_SINK_SEVERITY_TRACE;
					case common::logging::Severity::Error:
					default:
						return MULLVAD_LOG_SINK_SEVERITY_ERROR;
				}
			}();

			target(severity, msg, context);
		};
	}
};

}
