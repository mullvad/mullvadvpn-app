#pragma once
#include <cstdint>

//
// WINFW public API
//

#ifdef WINFW_EXPORTS
#define WINFW_LINKAGE __declspec(dllexport)
#else
#define WINFW_LINKAGE __declspec(dllimport)
#endif

#define WINFW_API __stdcall

///////////////////////////////////////////////////////////////////////////////
// Structures
///////////////////////////////////////////////////////////////////////////////

#pragma pack(push, 1)

typedef struct tag_WinFwSettings
{
	// Permit outbound DHCP requests and inbound DHCP responses on all interfaces.
	bool permitDhcp;

	// Permit all traffic to and from private address ranges.
	bool permitLan;
}
WinFwSettings;

enum WinFwProtocol : uint8_t
{
	Tcp = 0,
	Udp = 1
};

typedef struct tag_WinFwRelay
{
	const wchar_t *ip;
	uint16_t port;
	WinFwProtocol protocol;
}
WinFwRelay;

#pragma pack(pop)

///////////////////////////////////////////////////////////////////////////////
// Functions
///////////////////////////////////////////////////////////////////////////////

//
// Initialize:
//
// Call this function once at startup, to register the provider etc.
//
// The timeout, in seconds, specifies how long to wait for the
// transaction lock to become available. Specify 0 to use a default timeout
// determined by Windows.
//
// Optionally provide a callback if you are interested in logging exceptions.
//
typedef void (WINFW_API *WinFwErrorSink)(const char *errorMessage, void *context);

extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_Initialize(
	uint32_t timeout,
	WinFwErrorSink errorSink,
	void *errorContext
);

//
// WinFw_InitializeBlocked
//
// Same as `WinFw_Initialize` with the addition that the blocked policy is
// immediately applied, within the same initialization transaction.
//
// This function is preferred rather than first initializing and then applying
// the blocked policy. Using two separate operations leaves a tiny window
// for traffic to leak out.
//
extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_InitializeBlocked(
	uint32_t timeout,
	const WinFwSettings &settings,
	WinFwErrorSink errorSink,
	void *errorContext
);

//
// Deinitialize:
//
// Call this function once before unloading WINFW or exiting the process.
//
extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_Deinitialize();

//
// PingableHosts:
//
// Specifies a set of IP addresses that should be reachable by ICMP when the connecting
// policy is effective.
//
// The interface alias is optional and can be used to restrict the traffic such
// that it is only allowed on that specific interface.
//
typedef struct tag_PingableHosts
{
	const wchar_t *tunnelInterfaceAlias;
	const wchar_t **hosts;
	size_t numHosts;
}
PingableHosts;

//
// ApplyPolicyConnecting:
//
// Apply restrictions in the firewall that block all traffic, except:
// - What is specified by settings
// - Communication with the relay server
// - ICMP (for ping) to/from tunnel gateway
//
extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_ApplyPolicyConnecting(
	const WinFwSettings &settings,
	const WinFwRelay &relay,
	const PingableHosts *pingableHosts
);

//
// ApplyPolicyConnected:
//
// Apply restrictions in the firewall that block all traffic, except:
// - What is specified by settings
// - Communication with the relay server
// - Non-DNS traffic inside the VPN tunnel
// - DNS requests inside the VPN tunnel, to the specified DNS server
//
// Parameters:
//
// tunnelInterfaceAlias:
//   Friendly name of VPN tunnel interface
// v4DnsHost/v6DnsHost:
//   String encoded IP address of DNS to use inside tunnel
//
extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_ApplyPolicyConnected(
	const WinFwSettings &settings,
	const WinFwRelay &relay,
	const wchar_t *tunnelInterfaceAlias,
	const wchar_t *v4DnsHost,
	const wchar_t *v6DnsHost
);

//
// ApplyPolicyBlocked:
//
// Apply restrictions in the firewall that block all traffic, except:
// - What is specified by settings
//
extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_ApplyPolicyBlocked(
	const WinFwSettings &settings
);

//
// Reset:
//
// Clear the policy in effect, if any.
//
extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_Reset();
