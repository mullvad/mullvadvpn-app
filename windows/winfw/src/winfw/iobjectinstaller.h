#pragma once

#include "libwfp/iconditionbuilder.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/providerbuilder.h"
#include "libwfp/sublayerbuilder.h"

struct IObjectInstaller
{
	virtual ~IObjectInstaller() = 0
	{
	}

	virtual bool addProvider(wfp::ProviderBuilder &providerBuilder) = 0;
	virtual bool addSublayer(wfp::SublayerBuilder &sublayerBuilder) = 0;
	virtual bool addFilter(wfp::FilterBuilder &filterBuilder, const wfp::IConditionBuilder &conditionBuilder) = 0;
};
