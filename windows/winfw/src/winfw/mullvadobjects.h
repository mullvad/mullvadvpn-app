#pragma once

#include "libwfp/providerbuilder.h"
#include "libwfp/sublayerbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/filterbuilder.h"
#include <memory>

class MullvadObjects
{
public:

	MullvadObjects() = delete;

	static std::unique_ptr<wfp::ProviderBuilder> Provider();
	static std::unique_ptr<wfp::SublayerBuilder> SublayerBaseline();
	static std::unique_ptr<wfp::SublayerBuilder> SublayerDns();

	static std::unique_ptr<wfp::ProviderBuilder> ProviderPersistent();
	static std::unique_ptr<wfp::SublayerBuilder> SublayerPersistent();
};
