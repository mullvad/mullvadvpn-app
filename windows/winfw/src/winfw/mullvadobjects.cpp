#include "stdafx.h"
#include "mullvadobjects.h"
#include "mullvadguids.h"

MullvadObjects::MullvadObjects(const WinFwSublayerGuids &guids)
	: m_guids(guids)
{
}

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

std::unique_ptr<wfp::SublayerBuilder> MullvadObjects::sublayerBaseline() const
{
	auto builder = std::make_unique<wfp::SublayerBuilder>();

	(*builder)
		.name(L"Mullvad VPN baseline")
		.description(L"Filters that enforce a good baseline")
		.key(m_guids.baseline)
		.provider(MullvadGuids::Provider())
		.weight(MAXUINT16);

	return builder;
}

std::unique_ptr<wfp::SublayerBuilder> MullvadObjects::sublayerDns() const
{
	auto builder = std::make_unique<wfp::SublayerBuilder>();

	(*builder)
		.name(L"Mullvad VPN DNS")
		.description(L"Filters that restrict DNS traffic")
		.key(m_guids.dns)
		.provider(MullvadGuids::Provider())
		.weight(MAXUINT16 - 1);

	return builder;
}

//static
std::unique_ptr<wfp::ProviderBuilder> MullvadObjects::ProviderPersistent()
{
	auto builder = std::make_unique<wfp::ProviderBuilder>();

	(*builder)
		.name(L"Mullvad VPN persistent")
		.description(L"Mullvad VPN firewall integration")
		.persistent()
		.key(MullvadGuids::ProviderPersistent());

	return builder;
}

std::unique_ptr<wfp::SublayerBuilder> MullvadObjects::sublayerPersistent() const
{
	auto builder = std::make_unique<wfp::SublayerBuilder>();

	(*builder)
		.name(L"Mullvad VPN persistent")
		.description(L"Filters that restrict traffic before WinFw is initialized")
		.key(m_guids.persistent)
		.provider(MullvadGuids::ProviderPersistent())
		.persistent()
		.weight(MAXUINT16);

	return builder;
}
