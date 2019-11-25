#pragma once

#include <string>
#include <optional>
#include <winsock2.h>
#include <windows.h>
#include <ws2def.h>
#include <ws2ipdef.h>
#include <iphlpapi.h>
//#include <netioapi.h>
//#include <functional>


namespace winnet::routing
{

using Network = IP_ADDRESS_PREFIX;
using NodeAddress = SOCKADDR_INET;

class Node
{
public:

	Node(const std::optional<std::wstring> &deviceName, const std::optional<NodeAddress> &gateway);

	const std::optional<std::wstring> &deviceName() const
	{
		return m_deviceName;
	}

	const std::optional<NodeAddress> &gateway() const
	{
		return m_gateway;
	}

	bool operator==(const Node &rhs) const;

private:

	std::optional<std::wstring> m_deviceName;
	std::optional<NodeAddress> m_gateway;
};

class Route
{
public:

	Route(const Network &network, const std::optional<Node> &node);

	const Network &network() const
	{
		return m_network;
	}

	const std::optional<Node> &node() const
	{
		return m_node;
	}

	bool operator==(const Route &rhs) const;

private:

	Network m_network;
	std::optional<Node> m_node;
};

struct InterfaceAndGateway
{
	NET_LUID iface;
	NodeAddress gateway;

	bool operator==(const InterfaceAndGateway &rhs);
	bool operator!=(const InterfaceAndGateway &rhs);
};

}
