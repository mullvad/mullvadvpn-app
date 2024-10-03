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
            Not all our servers are DAITA-enabled. In order to use the internet, you might have to \
            select a new location after enabling.
            """
        case .daitaSettingIncompatibleWithMultihop:
            """
            Not all our servers are DAITA-enabled. In order to use the internet, you might have to \
            select a new entry location after enabling.
            """
        }
    }
}
