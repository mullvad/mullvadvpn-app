#pragma once

#include "interfaceconfig.h"
#include "itracesink.h"
#include <memory>

class DnsReverter
{
public:

	DnsReverter(std::shared_ptr<ITraceSink> traceSink = std::make_shared<NullTraceSink>());

	void revert(const InterfaceConfig &config);

private:

	std::shared_ptr<ITraceSink> m_traceSink;
};
