//
//  SettingsInfoButtonItem.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum SettingsInfoButtonItem: CustomStringConvertible {
    case daita
    case daitaDirectOnly

    var description: String {
        switch self {
        case .daita:
            NSLocalizedString(
                "DAITA_INFORMATION_TEXT",
                tableName: "DAITA",
                value: """
                DAITA (Defence against AI-guided Traffic Analysis) hides patterns in your encrypted VPN traffic. \
                If anyone is monitoring your connection, this makes it significantly harder for them to identify \
                what websites you are visiting.
                It does this by carefully adding network noise and making all network packets the same size.
                Not all our servers are DAITA-enabled. Therefore, we use multihop automatically to enable DAITA \
                with any server.
                Attention: Be cautious if you have a limited data plan as this feature will increase your network \
                traffic.
                """,
                comment: ""
            )
        case .daitaDirectOnly:
            NSLocalizedString(
                "DIRECT_ONLY_INFORMATION_TEXT",
                tableName: "DAITA",
                value: """
                By enabling "Direct only" you will have to manually select a server that is DAITA-enabled. \
                This can cause you to end up in a blocked state until you have selected a compatible server \
                in the "Select location" view.
                """,
                comment: ""
            )
        }
    }
}
