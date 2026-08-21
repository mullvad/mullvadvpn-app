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

extension View {
    @ViewBuilder func `if`<Content: View>(
        _ conditional: Bool,
        @ViewBuilder _ content: (Self) -> Content
    ) -> some View {
        if conditional {
            content(self)
        } else {
            self
        }
    }

    @ViewBuilder func ifLet<Content: View, T>(
        _ conditional: T?,
        @ViewBuilder _ content: (Self, _ value: T) -> Content
    ) -> some View {
        if let value = conditional {
            content(self, value)
        } else {
            self
        }
    }

    @ViewBuilder func showIf(_ conditional: Bool) -> some View {
        if conditional {
            self
        } else {
            EmptyView()
        }
    }
}
