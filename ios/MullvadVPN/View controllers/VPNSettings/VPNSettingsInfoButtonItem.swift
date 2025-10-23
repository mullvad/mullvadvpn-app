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
            [
                NSLocalizedString(
                    "This feature allows access to other devices on the local network, such as for sharing, printing, streaming, etc.",
                    comment: ""
                ),
                NSLocalizedString(
                    "Attention: toggling “Local network sharing” requires restarting the VPN connection.",
                    comment: ""
                ),
            ].joinedParagraphs(lineBreaks: 1)
        case .contentBlockers:
            [
                NSLocalizedString(
                    "When this feature is enabled it stops the device from contacting certain "
                        + "domains or websites known for distributing ads, malware, trackers and more.",
                    comment: ""
                ),
                NSLocalizedString(
                    "This might cause issues on certain websites, services, and apps.",
                    comment: ""
                ),
                String(
                    format: NSLocalizedString(
                        "Attention: this setting cannot be used in combination with **%@**",
                        comment: ""
                    ),
                    NSLocalizedString("Use custom DNS server", comment: "")
                ),
            ]
            .joinedParagraphs(lineBreaks: 1)
        case .blockMalware:
            NSLocalizedString(
                "Warning: The malware blocker is not an anti-virus and should not be treated as such,"
                    + " this is just an extra layer of protection.",
                comment: ""
            )
        case let .wireGuardPorts(portsString):
            [
                NSLocalizedString(
                    "The automatic setting will randomly choose from the valid port ranges shown below.",
                    comment: ""
                ),
                String(
                    format: NSLocalizedString(
                        "The custom port can be any value inside the valid ranges: %@.",
                        comment: ""
                    ),
                    portsString
                ),
            ].joinedParagraphs(lineBreaks: 1)
        case .wireGuardObfuscation:
            NSLocalizedString(
                "Obfuscation hides the WireGuard traffic inside another protocol. "
                    + "It can be used to help circumvent censorship and other types of "
                    + "filtering, where a plain WireGuard connection would be blocked.",
                comment: ""
            )
        case .wireGuardObfuscationPort:
            NSLocalizedString(
                "Which TCP port the UDP-over-TCP obfuscation protocol should connect to on the VPN server.",
                comment: ""
            )
        case .quantumResistance:
            [
                NSLocalizedString(
                    "This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.",
                    comment: ""
                ),
                NSLocalizedString(
                    "It does this by performing an extra key exchange using a quantum safe algorithm and mixing "
                        + "the result into WireGuard’s regular encryption. This extra step uses approximately 500 kiB "
                        + "of traffic every time a new tunnel is established.",
                    comment: ""
                ),
            ].joinedParagraphs(lineBreaks: 1)

        case .multihop:
            NSLocalizedString(
                "Multihop routes your traffic into one WireGuard server and out another, "
                    + "making it harder to trace. This results in increased latency but "
                    + "increases anonymity online.",
                comment: ""
            )
        }
    }
}
