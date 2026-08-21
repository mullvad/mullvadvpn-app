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

extension ConnectionView {
    internal struct ButtonPanel: View {
        typealias Action = (ConnectionViewViewModel.TunnelAction) -> Void

        @ObservedObject var viewModel: ConnectionViewViewModel
        var action: Action?

        var body: some View {
            VStack(spacing: 16) {
                locationButton(with: action)
                    .disabled(viewModel.disableButtons)
                actionButton(with: action)
                    .disabled(viewModel.disableButtons)
            }
        }

        @ViewBuilder
        private func locationButton(with action: Action?) -> some View {
            let reloadButton: MullvadButton.Accessory? =
                switch viewModel.tunnelStatus.state {
                case .connecting, .connected, .reconnecting, .waitingForConnectivity, .negotiatingEphemeralPeer, .error:
                    .button(
                        .iconReload,
                        accessibilityId: .reconnectButton,
                        accessibilityLabel: LocalizedStringKey("Reconnect"),
                        accessibilityHint: LocalizedStringKey("Cycle through available servers"),
                        {
                            action?(.reconnect)
                        })
                case .disconnecting, .pendingReconnect, .disconnected:
                    nil
                }
            MullvadButton(
                text: viewModel.localizedTitleForSelectLocationButton,
                style: .primary,
                mainAccessibilityIdentifier: .selectLocationButton,
                trailingAccessory: reloadButton,
            ) { action?(.selectLocation) }
        }

        @ViewBuilder
        private func actionButton(with action: Action?) -> some View {
            switch viewModel.actionButton {
            case .connect:
                MullvadButton(
                    text: LocalizedStringKey("Connect"),
                    style: .success,
                    mainAccessibilityIdentifier: .connectButton,
                    action: { action?(.connect) }
                )
            case .disconnect:
                MullvadButton(
                    text: LocalizedStringKey("Disconnect"),
                    style: .destructive,
                    mainAccessibilityIdentifier: .disconnectButton,
                    action: { action?(.disconnect) }
                )
            case .cancel:
                MullvadButton(
                    text: LocalizedStringKey(
                        viewModel.tunnelStatus.state == .waitingForConnectivity(.noConnection)
                            ? "Disconnect"
                            : "Cancel"
                    ),
                    style: .destructive,
                    mainAccessibilityIdentifier: viewModel.tunnelStatus.state == .waitingForConnectivity(.noConnection)
                        ? .disconnectButton : .cancelButton,
                    action: { action?(.cancel) }
                )
            }
        }
    }
}

#Preview("connected") {
    ConnectionViewComponentPreview(showIndicators: true) { _, vm, _ in
        ConnectionView.ButtonPanel(viewModel: vm, action: nil)
    }
}

#Preview("disconnected") {
    ConnectionViewComponentPreview(showIndicators: true, isConnected: false) { _, vm, _ in
        ConnectionView.ButtonPanel(viewModel: vm, action: nil)
    }
}
