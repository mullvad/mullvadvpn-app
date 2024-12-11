//
//  ConnectionView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

typealias ButtonAction = (ConnectionViewViewModel.TunnelControlAction) -> Void

struct ConnectionView: View {
    @StateObject var viewModel: ConnectionViewViewModel

    var action: ButtonAction?
    var onContentUpdate: (() -> Void)?

    var body: some View {
        VStack(spacing: 22) {
            if viewModel.showsActivityIndicator {
                CustomProgressView(style: .large)
            }

            ZStack {
                BlurView(style: .dark)

                VStack(alignment: .leading, spacing: 16) {
                    ConnectionPanel(viewModel: viewModel)
                    FeaturesIndicatoresView(viewModel: FeaturesIndicatoresMockViewModel())
                    ButtonPanel(viewModel: viewModel, action: action)
                }
                .padding(16)
            }
            .cornerRadius(12)
            .padding(16)
        }
        .onReceive(viewModel.$tunnelState, perform: { _ in
            onContentUpdate?()
        })
        .onReceive(viewModel.$showsActivityIndicator, perform: { _ in
            onContentUpdate?()
        })
    }
}

#Preview {
    ConnectionView(viewModel: ConnectionViewViewModel(tunnelState: .disconnected)) { action in
        print(action)
    }
    .background(UIColor.secondaryColor.color)
}

private struct ConnectionPanel: View {
    @StateObject var viewModel: ConnectionViewViewModel

    var body: some View {
        VStack(alignment: .leading) {
            Text(viewModel.localizedTitleForSecureLabel)
                .textCase(.uppercase)
                .font(.title3.weight(.semibold))
                .foregroundStyle(viewModel.textColorForSecureLabel.color)
                .padding(.bottom, 4)

            if let countryAndCity = viewModel.titleForCountryAndCity, let server = viewModel.titleForServer {
                Text(countryAndCity)
                    .font(.title3.weight(.semibold))
                    .foregroundStyle(UIColor.primaryTextColor.color)
                Text(server)
                    .font(.body)
                    .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
            }
        }
        .accessibilityLabel(viewModel.localizedAccessibilityLabel)
    }
}

private struct ButtonPanel: View {
    @StateObject var viewModel: ConnectionViewViewModel
    var action: ButtonAction?

    var body: some View {
        VStack(spacing: 16) {
            locationButton(with: action)
            actionButton(with: action)
        }
    }

    @ViewBuilder
    private func locationButton(with action: ButtonAction?) -> some View {
        switch viewModel.tunnelState {
        case .connecting, .connected, .reconnecting, .waitingForConnectivity, .negotiatingEphemeralPeer, .error:
            SplitMainButton(
                text: viewModel.localizedTitleForSelectLocationButton,
                image: .iconReload,
                style: .default,
                disabled: viewModel.disableButtons,
                primaryAction: { action?(.selectLocation) },
                secondaryAction: { action?(.reconnect) }
            )
        case .disconnecting, .pendingReconnect, .disconnected:
            MainButton(
                text: viewModel.localizedTitleForSelectLocationButton,
                style: .default,
                disabled: viewModel.disableButtons,
                action: { action?(.selectLocation) }
            )
        }
    }

    @ViewBuilder
    private func actionButton(with action: ButtonAction?) -> some View {
        switch viewModel.actionButton {
        case .connect:
            MainButton(
                text: LocalizedStringKey("Connect"),
                style: .success,
                disabled: viewModel.disableButtons,
                action: { action?(.connect) }
            )
        case .disconnect:
            MainButton(
                text: LocalizedStringKey("Disconnect"),
                style: .danger,
                disabled: viewModel.disableButtons,
                action: { action?(.disconnect) }
            )
        case .cancel:
            MainButton(
                text: LocalizedStringKey(
                    viewModel.tunnelState == .waitingForConnectivity(.noConnection)
                        ? "Disconnect"
                        : "Cancel"
                ),
                style: .danger,
                disabled: viewModel.disableButtons,
                action: { action?(.cancel) }
            )
        }
    }
}
