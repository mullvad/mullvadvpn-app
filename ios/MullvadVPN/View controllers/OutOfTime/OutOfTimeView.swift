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

struct OutOfTimeView<ViewModel: OutOfTimeViewModelProtocol>: View {
    @State var viewModel: ViewModel

    var body: some View {
        VStack {
            ResizableImageView(image: Image.mullvadIconFail, dimension: .width(60))
                .padding(.top, 8)

            VStack(alignment: .leading) {
                Text("Out of time")
                    .foregroundStyle(Color.mullvadTextPrimary)
                    .font(.mullvadLarge)
                    .padding(.top, 16)

                Group {
                    if viewModel.isConnected {
                        Text(
                            "You have no more VPN time left on this account. To add more, you will need to disconnect and access the Internet with an unsecure connection."
                        )
                    } else {
                        Text(
                            "You have no more VPN time left on this account. Either buy credit on our website or make an in-app purchase via the Add time button below."
                        )
                    }
                }
                .foregroundStyle(Color.mullvadTextSecondary)
                .font(.mullvadSmall)
                .padding(.top, 8)
            }

            Spacer()

            buttonContainer
                .padding(.top, 16)
        }
        .padding(UIMetrics.contentInsets.toEdgeInsets)
        .scrollable(fill: .vertical)
        .scrollBounceBehavior(.basedOnSize)
        .background(Color.mullvadBackground.ignoresSafeArea())
        .onAppear {
            viewModel.startAccountPolling()
        }
        .onDisappear {
            viewModel.stopAccountPolling()
        }
    }

    private var buttonContainer: some View {
        VStack(spacing: 16) {
            MullvadButton(text: "Disconnect", style: .destructive, action: viewModel.disconnect)
                .disabled(!viewModel.disconnectButtonIsEnabled)
                .showIf(viewModel.isConnected)

            MullvadButton(text: "Add time", style: .success, action: { viewModel.showInAppPurchase?(.purchase) })
                .disabled(viewModel.isConnected)

            MullvadButton(
                text: "Restore purchases", style: .primary, action: { viewModel.showInAppPurchase?(.restorePurchase) }
            )
            .accessibilityIdentifier(AccessibilityIdentifier.purchaseButton)
        }
    }
}

#Preview("Connected") {
    OutOfTimeView(viewModel: OutOfTimeViewModelMock(isConnected: true, disconnectButtonIsEnabled: true))
}

#Preview("Disconnected") {
    OutOfTimeView(viewModel: OutOfTimeViewModelMock(isConnected: false, disconnectButtonIsEnabled: false))
}
