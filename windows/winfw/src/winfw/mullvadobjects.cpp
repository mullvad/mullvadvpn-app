#include "stdafx.h"
#include "mullvadobjects.h"
#include "mullvadguids.h"

//static
std::unique_ptr<wfp::ProviderBuilder> MullvadObjects::Provider()
{
	auto builder = std::make_unique<wfp::ProviderBuilder>();

	(*builder)
		.name(L"Mullvad VPN")
		.description(L"Mullvad VPN firewall integration")
		.key(MullvadGuids::Provider());

	return builder;
}

//static
std::unique_ptr<wfp::SublayerBuilder> MullvadObjects::SublayerWhitelist()
{
	auto builder = std::make_unique<wfp::SublayerBuilder>();

	(*builder)
		.name(L"Mullvad VPN whitelist")
		.description(L"Filters that permit traffic")
		.key(MullvadGuids::SublayerWhitelist())
		.provider(MullvadGuids::Provider())
		.weight(MAXUINT16);

	return builder;
}

//static
std::unique_ptr<wfp::SublayerBuilder> MullvadObjects::SublayerBlacklist()
{
	auto builder = std::make_unique<wfp::SublayerBuilder>();

	(*builder)
		.name(L"Mullvad VPN blacklist")
		.description(L"Filters that block traffic")
		.key(MullvadGuids::SublayerBlacklist())
		.provider(MullvadGuids::Provider())
		.weight(MAXUINT16 - 1);

	return builder;
}
