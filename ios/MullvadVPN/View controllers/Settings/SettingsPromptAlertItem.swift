// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
