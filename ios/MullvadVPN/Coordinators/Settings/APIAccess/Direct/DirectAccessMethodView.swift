// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes
import SwiftUI

struct DirectAccessMethodView<ViewModel: DirectAccessMethodViewModelProtocol>: View {
    @State var viewModel: ViewModel
    @State private var showDetails = false
    private let itemFactory = SegmentedListItemFactory()

    init(viewModel: ViewModel) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                InlineLinkText(
                    text: "The app communicates with a Mullvad API server directly.",
                    linkTitle: "About \("Direct") method..."
                ) {
                    showDetails = true
                }
                .padding(.bottom, 16.0)
                .background(Color.mullvadBackground)
                .sheet(isPresented: $showDetails) {
                    Text("Direct Method")
                        .navigationTitle("Direct Method")
                }

                SegmentedListItem(
                    isLastInList: true,
                    userInteraction: .enabledWithoutHighlight,
                    accessibilityIdentifier: .accessMethodEnableSwitch,
                    leading: {
                        itemFactory.leading(for: .generic(title: NSLocalizedString("Enable", comment: "")))
                    },
                    trailing: {
                        itemFactory.trailing(
                            for: .toggle(
                                isOn: $viewModel.isEnabled,
                                isDisabled: false
                            )
                        )
                    }
                )
                .padding(.bottom, 24.0)

                MullvadButton(
                    text: "Test method", style: .primary,
                    action: {

                    })
            }
        }
        .scrollBounceBehavior(.automatic)
        .padding(.horizontal, 16)
        .padding(.bottom, 16)
        .background(Color.mullvadBackground)
    }
}

#Preview {
    DirectAccessMethodView(
        viewModel: DirectAccessMethodViewModel(
            accessMethod: PersistentAccessMethod(
                id: AccessMethodRepository.directId,
                name: "Direct",
                isEnabled: true,
                proxyConfiguration: .direct
            ),
            onChange: { newValue in
                print(newValue)
            }))
}
