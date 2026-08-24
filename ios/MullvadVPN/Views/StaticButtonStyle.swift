// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

struct StaticButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
    }
}

struct InactiveButtonStyle: ButtonStyle {
    let isInactive: Bool = true

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .opacity(isInactive ? 0.4 : (configuration.isPressed ? 0.7 : 1.0))
    }
}
