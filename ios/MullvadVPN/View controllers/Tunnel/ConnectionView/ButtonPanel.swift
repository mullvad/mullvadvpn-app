//
//  ButtonPanel.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
                trailingAccessory: reloadButton,
            ) { action?(.selectLocation) }
            .accessibilityIdentifier(AccessibilityIdentifier.selectLocationButton.asString)
        }

        @ViewBuilder
        private func actionButton(with action: Action?) -> some View {
            switch viewModel.actionButton {
            case .connect:
                MullvadButton(
                    text: LocalizedStringKey("Connect"),
                    style: .success,
                    action: { action?(.connect) }
                )
                .accessibilityIdentifier(AccessibilityIdentifier.connectButton.asString)
            case .disconnect:
                MullvadButton(
                    text: LocalizedStringKey("Disconnect"),
                    style: .destructive,
                    action: { action?(.disconnect) }
                )
                .accessibilityIdentifier(AccessibilityIdentifier.disconnectButton.asString)
            case .cancel:
                MullvadButton(
                    text: LocalizedStringKey(
                        viewModel.tunnelStatus.state == .waitingForConnectivity(.noConnection)
                            ? "Disconnect"
                            : "Cancel"
                    ),
                    style: .destructive,
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
