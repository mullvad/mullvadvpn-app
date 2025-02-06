//
//  VPNSettingsInfoButtonItem.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum VPNSettingsInfoButtonItem: CustomStringConvertible {
    case localNetworkSharing
    case contentBlockers
    case blockMalware
    case wireGuardPorts(String)
    case wireGuardObfuscation
    case wireGuardObfuscationPort
    case quantumResistance
    case multihop

    var description: String {
        switch self {
        case .localNetworkSharing:
            NSLocalizedString(
                "VPN_SETTINGS_LOCAL_NETWORK_SHARING",
                tableName: "LocalNetworkSharing",
                value: """
                This feature allows access to other devices on the local network, such as for sharing, printing, streaming, etc.

                It does this by allowing network communication outside the tunnel to local multicast and broadcast ranges as well as to and from these private IP ranges:
                10.0.0.0/8
                172.16.0.0/12
                192.168.0.0/16
                169.254.0.0/16
                fe80::/10
                fc00::/7

                Attention: toggling “Local network sharing” requires restarting the VPN connection.
                """,
                comment: ""
            )
        case .contentBlockers:
            NSLocalizedString(
                "VPN_SETTINGS_CONTENT_BLOCKERS_GENERAL",
                tableName: "ContentBlockers",
                value: """
                When this feature is enabled it stops the device from contacting certain \
                domains or websites known for distributing ads, malware, trackers and more. \

                This might cause issues on certain websites, services, and apps.
                Attention: this setting cannot be used in combination with **Use custom DNS server**.
                """,
                comment: ""
            )
        case .blockMalware:
            NSLocalizedString(
                "VPN_SETTINGS_CONTENT_BLOCKERS_MALWARE",
                tableName: "ContentBlockers",
                value: """
                Warning: The malware blocker is not an anti-virus and should not \
                be treated as such, this is just an extra layer of protection.
                """,
                comment: ""
            )
        case let .wireGuardPorts(portsString):
            String(
                format: NSLocalizedString(
                    "VPN_SETTINGS_WIREGUARD_PORTS_GENERAL",
                    tableName: "WireGuardPorts",
                    value: """
                    The automatic setting will randomly choose from the valid port ranges shown below.
                    The custom port can be any value inside the valid ranges:
                    %@
                    """,
                    comment: ""
                ),
                portsString
            )
        case .wireGuardObfuscation:
            NSLocalizedString(
                "VPN_SETTINGS_WIREGUARD_OBFUSCATION_GENERAL",
                tableName: "WireGuardObfuscation",
                value: """
                Obfuscation hides the WireGuard traffic inside another protocol. \
                It can be used to help circumvent censorship and other types of filtering, \
                where a plain WireGuard connect would be blocked.
                """,
                comment: ""
            )
        case .wireGuardObfuscationPort:
            NSLocalizedString(
                "VPN_SETTINGS_WIREGUARD_OBFUSCATION_PORT_GENERAL",
                tableName: "WireGuardObfuscation",
                value: "Which TCP port the UDP-over-TCP obfuscation protocol should connect to on the VPN server.",
                comment: ""
            )
        case .quantumResistance:
            NSLocalizedString(
                "VPN_SETTINGS_QUANTUM_RESISTANCE_GENERAL",
                tableName: "QuantumResistance",
                value: """
                This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.
                It does this by performing an extra key exchange using a quantum safe algorithm and mixing \
                the result into WireGuard’s regular encryption.
                This extra step uses approximately 500 kiB of traffic every time a new tunnel is established.
                """,
                comment: ""
            )
        case .multihop:
            NSLocalizedString(
                "MULTIHOP_INFORMATION_TEXT",
                tableName: "Multihop",
                value: """
                Multihop routes your traffic into one WireGuard server and out another, making it harder to trace.
                This results in increased latency but increases anonymity online.
                """,
                comment: ""
            )
        }
    }
}
