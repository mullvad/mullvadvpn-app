//
//  ButtonPanel.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension ConnectionView {
    internal struct ButtonPanel: View {
        typealias Action = (ConnectionViewViewModel.TunnelAction) -> Void

        @StateObject var viewModel: ConnectionViewViewModel
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
            switch viewModel.tunnelStatus.state {
            case .connecting, .connected, .reconnecting, .waitingForConnectivity, .negotiatingEphemeralPeer, .error:
                SplitMainButton(
                    text: viewModel.localizedTitleForSelectLocationButton,
                    image: .iconReload,
                    style: .default,
                    accessibilityId: .selectLocationButton,
                    primaryAction: { action?(.selectLocation) },
                    secondaryAction: { action?(.reconnect) }
                )
            case .disconnecting, .pendingReconnect, .disconnected:
                MainButton(
                    text: viewModel.localizedTitleForSelectLocationButton,
                    style: .default,
                    action: { action?(.selectLocation) }
                )
                .accessibilityIdentifier(AccessibilityIdentifier.selectLocationButton.asString)
            }
        }

        @ViewBuilder
        private func actionButton(with action: Action?) -> some View {
            switch viewModel.actionButton {
            case .connect:
                MainButton(
                    text: LocalizedStringKey("Connect"),
                    style: .success,
                    action: { action?(.connect) }
                )
                .accessibilityIdentifier(AccessibilityIdentifier.connectButton.asString)
            case .disconnect:
                MainButton(
                    text: LocalizedStringKey("Disconnect"),
                    style: .danger,
                    action: { action?(.disconnect) }
                )
                .accessibilityIdentifier(AccessibilityIdentifier.disconnectButton.asString)
            case .cancel:
                MainButton(
                    text: LocalizedStringKey(
                        viewModel.tunnelStatus.state == .waitingForConnectivity(.noConnection)
                            ? "Disconnect"
                            : "Cancel"
                    ),
                    style: .danger,
                    action: { action?(.cancel) }
                )
                .accessibilityIdentifier(
                    viewModel.tunnelStatus.state == .waitingForConnectivity(.noConnection)
                        ? AccessibilityIdentifier.disconnectButton.asString
                        : AccessibilityIdentifier.cancelButton.asString
                )
            }
        }
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true, isExpanded: true) { _, vm, _ in
        ConnectionView.ButtonPanel(viewModel: vm, action: nil)
    }
}
