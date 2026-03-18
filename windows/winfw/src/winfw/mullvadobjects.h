#pragma once

#include "winfw.h"
#include "libwfp/providerbuilder.h"
#include "libwfp/sublayerbuilder.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/filterbuilder.h"
#include <memory>

class MullvadObjects
{
public:

	MullvadObjects(const WinFwSublayerGuids &guids);

	static std::unique_ptr<wfp::ProviderBuilder> Provider();
	std::unique_ptr<wfp::SublayerBuilder> sublayerBaseline() const;
	std::unique_ptr<wfp::SublayerBuilder> sublayerDns() const;

	static std::unique_ptr<wfp::ProviderBuilder> ProviderPersistent();
	std::unique_ptr<wfp::SublayerBuilder> sublayerPersistent() const;

private:

	WinFwSublayerGuids m_guids;
};
