//
//  SettingsPromptAlertItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-09-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
enum DAITASettingsPromptItem: CustomStringConvertible {
    case daitaSettingIncompatibleWithSinglehop
    case daitaSettingIncompatibleWithMultihop

    var description: String {
        switch self {
        case .daitaSettingIncompatibleWithSinglehop:
            """
            DAITA isn’t available on the current server. After enabling, please go to the Switch \
            location view and select a location that supports DAITA.
            Attention: Since this increases your total network traffic, be cautious if you have a \
            limited data plan. It can also negatively impact your network speed and battery usage.
            """
        case .daitaSettingIncompatibleWithMultihop:
            """
            DAITA isn’t available on the current entry server. After enabling, please go to the Switch \
            location view and select an entry location that supports DAITA.
            Attention: Since this increases your total network traffic, be cautious if you have a \
            limited data plan. It can also negatively impact your network speed and battery usage.
            """
        }
    }
}
