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

struct ServerDetailsSection: View {
    let itemFactory = SegmentedListItemFactory()
    let viewModel: any ConfigureApiAccessMethodViewModelProtocol

    private var serverBinding: Binding<String> {
        Binding(
            get: { viewModel.server },
            set: { viewModel.server = $0 }
        )
    }

    private var portBinding: Binding<String> {
        Binding(
            get: { viewModel.port },
            set: { viewModel.port = $0 }
        )
    }

    private var typeBinding: Binding<AccessMethodKind> {
        Binding(
            get: { viewModel.type },
            set: { viewModel.type = $0 }
        )
    }

    var body: some View {
        MullvadListSectionHeader(title: "Server details")
            .padding(.vertical, 12.0)
        serverField
        portField
    }

    private var serverField: some View {
        SegmentedListItem(
            isLastInList: false,
            accessibilityIdentifier: .addAccessMethodServerCell,
            leading: {
                itemFactory.leading(for: .generic(title: "Server"))
            },
            trailing: {
                ConfigurableTextField(
                    placeholder: LocalizedStringKey("Required"),
                    text: serverBinding,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(backgroundColor: .clear)
                )
            }
        )
        .padding(.bottom, 1.0)
        .environment(\.isNestedInSegmentedListItem, false)
    }

    private var portField: some View {
        SegmentedListItem(
            isLastInList: false,
            accessibilityIdentifier: .addAccessMethodPortCell,
            leading: {
                itemFactory.leading(for: .generic(title: "Port"))
            },
            trailing: {
                ConfigurableTextField(
                    placeholder: LocalizedStringKey("Required"),
                    text: portBinding,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(backgroundColor: .clear)
                )
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
    }

    //    private var
}
