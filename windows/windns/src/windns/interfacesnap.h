#pragma once

#include "types.h"
#include <libcommon/serialization/serializer.h>
#include <libcommon/serialization/deserializer.h>
#include <string>
#include <vector>

class InterfaceSnap
{
public:

	explicit InterfaceSnap(Protocol protocol, const std::wstring &interfaceGuid);

	explicit InterfaceSnap(common::serialization::Deserializer &deserializer);
	void serialize(common::serialization::Serializer &serializer) const;

	bool needsOverriding(const std::vector<std::wstring> &enforcedServers) const;

	const std::wstring &interfaceGuid() const;

	const std::vector<std::wstring> &nameServers() const;

	bool internalInterface() const;

private:

	Protocol m_protocol;
	std::wstring m_interfaceGuid;

	bool m_configuredForDhcp;

	std::vector<std::wstring> m_staticNameServers;
	std::vector<std::wstring> m_dhcpNameServers;
};
