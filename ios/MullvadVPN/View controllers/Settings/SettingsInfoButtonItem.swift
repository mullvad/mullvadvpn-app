//
//  SettingsInfoButtonItem.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-10-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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
                what websites you are visiting. It does this by carefully adding network noise and making all \
                network packets the same size.
                Attention: Since this increases your total network traffic, \
                be cautious if you have a limited data plan. \
                It can also negatively impact your network speed and battery usage.
                """,
                comment: ""
            )
        case .daitaDirectOnly:
            NSLocalizedString(
                "DAITA_INFORMATION_TEXT",
                tableName: "DAITA",
                value: """
                Todo
                """,
                comment: ""
            )
        }
    }
}
