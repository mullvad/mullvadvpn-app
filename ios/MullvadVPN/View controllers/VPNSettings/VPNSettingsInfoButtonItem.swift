//
//  VPNSettingsInfoButtonItem.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-10.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum VPNSettingsInfoButtonItem: CustomStringConvertible {
    case contentBlockers
    case blockMalware
    case wireGuardPorts(String)
    case wireGuardObfuscation
    case wireGuardObfuscationPort
    case quantumResistance
    case multihop
    case daita

    var description: String {
        switch self {
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
                    "VPN_SETTINGS_WIRE_GUARD_PORTS_GENERAL",
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
                "VPN_SETTINGS_WIRE_GUARD_OBFUSCATION_GENERAL",
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
                "VPN_SETTINGS_WIRE_GUARD_OBFUSCATION_PORT_GENERAL",
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
        case .daita:
            NSLocalizedString(
                "DAITA_INFORMATION_TEXT",
                tableName: "DAITA",
                value: """
                DAITA (Defence against AI-guided Traffic Analysis) hides patterns in your encrypted VPN traffic. \
                If anyone is monitoring your connection, this makes it significantly harder for them to identify \
                what websites you are visiting. It does this by carefully adding network noise and making all \
                network packets the same size.
                Attention: Since this increases your total network traffic, \
                be cautious if you have a limited data plan. \
                It can also negatively impact your network speed and battery usage.
                """,
                comment: ""
            )
        }
    }
}
