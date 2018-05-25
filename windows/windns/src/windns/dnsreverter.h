#pragma once

#include "wmi/iconnection.h"
#include "dnsconfig.h"
#include "itracesink.h"
#include <memory>

class DnsReverter
{
public:

	DnsReverter(std::shared_ptr<ITraceSink> traceSink = std::make_shared<NullTraceSink>());

	void revert(wmi::IConnection &connection, const DnsConfig &config);

private:

	std::shared_ptr<ITraceSink> m_traceSink;
};
