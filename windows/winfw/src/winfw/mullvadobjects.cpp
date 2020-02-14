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
std::unique_ptr<wfp::SublayerBuilder> MullvadObjects::SublayerBaseline()
{
	auto builder = std::make_unique<wfp::SublayerBuilder>();

	(*builder)
		.name(L"Mullvad VPN baseline")
		.description(L"Filters that enforce a good baseline")
		.key(MullvadGuids::SublayerBaseline())
		.provider(MullvadGuids::Provider())
		.weight(MAXUINT16);

	return builder;
}

//static
std::unique_ptr<wfp::SublayerBuilder> MullvadObjects::SublayerNonTunnelDns()
{
	auto builder = std::make_unique<wfp::SublayerBuilder>();

	(*builder)
		.name(L"Mullvad VPN non-tunnel DNS")
		.description(L"Filters that restrict DNS traffic outside tunnel")
		.key(MullvadGuids::SublayerNonTunnelDns())
		.provider(MullvadGuids::Provider())
		.weight(MAXUINT16 - 1);

	return builder;
}

//static
std::unique_ptr<wfp::SublayerBuilder> MullvadObjects::SublayerTunnelDns()
{
	auto builder = std::make_unique<wfp::SublayerBuilder>();

	(*builder)
		.name(L"Mullvad VPN tunnel DNS")
		.description(L"Filters that restrict DNS traffic inside tunnel")
		.key(MullvadGuids::SublayerTunnelDns())
		.provider(MullvadGuids::Provider())
		.weight(MAXUINT16 - 1);

	return builder;
}
