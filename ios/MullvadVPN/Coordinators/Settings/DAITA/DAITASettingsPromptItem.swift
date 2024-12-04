//
//  DAITASettingsPromptItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-09-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
enum DAITASettingsPromptItem: CustomStringConvertible {
    case daitaSettingIncompatibleWithSinglehop(Setting)
    case daitaSettingIncompatibleWithMultihop(Setting)

    enum Setting {
        case daita
        case directOnly
    }

    var title: String {
        switch self {
        case let .daitaSettingIncompatibleWithSinglehop(setting), let .daitaSettingIncompatibleWithMultihop(setting):
            switch setting {
            case .daita:
                "DAITA"
            case .directOnly:
                "direct only"
            }
        }
    }

    var description: String {
        switch self {
        case .daitaSettingIncompatibleWithSinglehop:
            """
            DAITA isn't available at the currently selected location. After enabling, please go to \
            the "Select location" view and select a location that supports DAITA.
            """
        case .daitaSettingIncompatibleWithMultihop:
            """
            DAITA isn't available on the current entry server. After enabling, please go to the \
            "Select location" view and select an entry location that supports DAITA.
            """
        }
    }
}
