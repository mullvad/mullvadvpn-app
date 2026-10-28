// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import SwiftUI

@MainActor
private final class DestinationModal<Value>: ObservableObject {
    let value: Value

    init(value: Value) {
        self.value = value
    }
}

struct RetainedDestination<Value, Content: View>: View {
    @StateObject private var viewModel: DestinationModal<Value>
    private let content: (Value) -> Content

    init(
        create: @escaping () -> Value,
        @ViewBuilder content: @escaping (Value) -> Content
    ) {
        _viewModel = StateObject(wrappedValue: DestinationModal(value: create()))
        self.content = content

    }
    var body: some View {
        content(viewModel.value)
    }
}
