#include "stdafx.h"
#include "types.h"
#include "helpers.h"
#include <libcommon/string.h>

namespace winnet::routing
{

Node::Node(const std::optional<std::wstring> &deviceName, const std::optional<NodeAddress> &gateway)
	: m_deviceName(deviceName)
	, m_gateway(gateway)
{
	if (false == m_deviceName.has_value() && false == m_gateway.has_value())
	{
		throw std::runtime_error("Invalid node definition");
	}

	if (m_deviceName.has_value())
	{
		const auto trimmed = common::string::Trim<>(m_deviceName.value());

		if (trimmed.empty())
		{
			throw std::runtime_error("Invalid device name in node definition");
		}

		m_deviceName = std::move(trimmed);
	}
}

bool Node::operator==(const Node &rhs) const
{
	if (m_deviceName.has_value())
	{
		if (false == rhs.m_deviceName.has_value()
			|| 0 != _wcsicmp(m_deviceName.value().c_str(), rhs.deviceName().value().c_str()))
		{
			return false;
		}
	}

	if (m_gateway.has_value())
	{
		if (false == rhs.m_gateway.has_value()
			|| false == EqualAddress(m_gateway.value(), rhs.gateway().value()))
		{
			return false;
		}
	}

	return true;
}

Route::Route(const Network &network, const std::optional<Node> &node)
	: m_network(network)
	, m_node(node)
{
}

bool Route::operator==(const Route &rhs) const
{
	if (m_node.has_value())
	{
		return rhs.node().has_value()
			&& EqualAddress(m_network, rhs.network())
			&& m_node.value() == rhs.node().value();
	}

	return false == rhs.node().has_value()
		&& EqualAddress(m_network, rhs.network());
}

bool InterfaceAndGateway::operator==(const InterfaceAndGateway &rhs)
{
	return iface.Value == rhs.iface.Value
		&& EqualAddress(gateway, rhs.gateway);
}

bool InterfaceAndGateway::operator!=(const InterfaceAndGateway &rhs)
{
	return !(*this == rhs);
}

}
