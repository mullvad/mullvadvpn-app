#pragma once
#include <cstdint>

//
// WFPCTL public API
//

#ifdef WFPCTL_EXPORTS
#define WFPCTL_LINKAGE __declspec(dllexport)
#else
#define WFPCTL_LINKAGE __declspec(dllimport)
#endif

#define WFPCTL_API __stdcall

///////////////////////////////////////////////////////////////////////////////
// Structures
///////////////////////////////////////////////////////////////////////////////

#pragma pack(push, 1)

typedef struct tag_WfpctlSettings
{
	// Permit outbound DHCP requests and inbound DHCP responses on all interfaces.
	bool permitDhcp;

	// Permit all traffic to and from private address ranges.
	bool permitLan;
}
WfpctlSettings;

enum WfpctlProtocol : uint8_t
{
	Tcp = 0,
	Udp = 1
};

typedef struct tag_WfpctlRelay
{
	const wchar_t *ip;
	uint16_t port;
	WfpctlProtocol protocol;
}
WfpctlRelay;

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
typedef void (WFPCTL_API *WfpctlErrorSink)(const char *errorMessage, void *context);

extern "C"
WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_Initialize(
	uint32_t timeout,
	WfpctlErrorSink errorSink,
	void *errorContext
);

//
// Deinitialize:
//
// Call this function once before unloading WFPCTL or exiting the process.
//
extern "C"
WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_Deinitialize();

//
// ApplyPolicyConnecting:
//
// Apply restrictions in the firewall that blocks all traffic, except:
// - What is specified by settings
// - Communication with the relay server
//
extern "C"
WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_ApplyPolicyConnecting(
	const WfpctlSettings &settings,
	const WfpctlRelay &relay
);

//
// ApplyPolicyConnected:
//
// Apply restrictions in the firewall that blocks all traffic, except:
// - What is specified by settings
// - Communication with the relay server
// - Non-DNS traffic inside the VPN tunnel
// - DNS requests inside the VPN tunnel, to the specified DNS server
//
// Parameters:
//
// tunnelInterfaceAlias:
//   Friendly name of VPN tunnel interface
// primaryDns:
//   String encoded IP address of DNS to use inside tunnel
//
extern "C"
WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_ApplyPolicyConnected(
	const WfpctlSettings &settings,
	const WfpctlRelay &relay,
	const wchar_t *tunnelInterfaceAlias,
	const wchar_t *primaryDns
);

//
// Reset:
//
// Clear the policy in effect, if any.
//
extern "C"
WFPCTL_LINKAGE
bool
WFPCTL_API
Wfpctl_Reset();
