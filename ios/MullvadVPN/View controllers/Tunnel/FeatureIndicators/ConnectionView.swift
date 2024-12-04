//
//  ConnectionView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import Network
import SwiftUI

typealias ButtonAction = (ConnectionViewViewModel.TunnelControlAction) -> Void

struct ConnectionView: View {
    @StateObject var viewModel: ConnectionViewViewModel
    @StateObject var indicatorsViewModel: FeatureIndicatorsViewModel
    @State var expandConnectionDetails = false

    var action: ButtonAction?
    var onContentUpdate: (() -> Void)?
    var onChevronToggle: (() -> Void)?

    var body: some View {
        VStack(spacing: 22) {
            if viewModel.showsActivityIndicator {
                CustomProgressView(style: .large)
            }

            ZStack {
                BlurView(style: .dark)

                VStack(alignment: .leading, spacing: 16) {
                    ConnectionPanel(viewModel: viewModel)

                    if !indicatorsViewModel.chips.isEmpty {
                        FeatureIndicatorsView(viewModel: indicatorsViewModel)
                    }
                    ConnectionPanel(viewModel: viewModel, onChevronToggle: {
                        expandConnectionDetails.toggle()
                    }, isExpanded: $expandConnectionDetails)
                    Divider()
                        .background(UIColor.secondaryTextColor.color)
                    FeatureIndicatorsScrollContainerView(
                        isExpanded: $expandConnectionDetails,
                        content: { Text("Hello") }
                    )
                    .frame(maxWidth: .infinity)
                    .border(.white)

                    ButtonPanel(viewModel: viewModel, action: action)
                }
                .padding(16)
            }
            .cornerRadius(12)
            .padding(16)
        }
        .padding(.bottom, 8) // Adding some spacing so to not overlap with the map legal link.
        .onReceive(
            indicatorsViewModel.$isExpanded
                .combineLatest(
                    viewModel.$tunnelState,
                    viewModel.$showsActivityIndicator
                )
        ) { _ in
            onContentUpdate?()
        }
    }
}

#Preview {
    ConnectionView(
        viewModel: ConnectionViewViewModel(tunnelState: .disconnected),
        indicatorsViewModel: FeatureIndicatorsViewModel(tunnelSettings: LatestTunnelSettings(), ipOverrides: [])
    ) { action in
        print(action)
    let selectedRelays = SelectedRelays(
        entry: nil,
        exit: SelectedRelay(
            endpoint: MullvadEndpoint(
                ipv4Relay: IPv4Endpoint(ip: .loopback, port: 42),
                ipv4Gateway: IPv4Address.loopback,
                ipv6Gateway: IPv6Address.loopback,
                publicKey: Data()
            ),
            hostname: "se-got-wg-001",
            location: Location(
                country: "Sweden",
                countryCode: "se",
                city: "Gothenburg",
                cityCode: "got",
                latitude: 42,
                longitude: 42
            )
        ),
        retryAttempt: 0
    )
    let connectedState = TunnelState.connected(selectedRelays, isPostQuantum: true, isDaita: true)

    return ZStack {
        VStack {
            HeaderBarSwiftUIHostedView()
                .frame(maxHeight: 100)
            ConnectionView(
                viewModel: ConnectionViewViewModel(tunnelState: connectedState),
                action: { action in print(action) },
                onContentUpdate: { print("On content Update") },
                onChevronToggle: { print("Chevron toggle") }
            )
        }
    }
    .background(UIColor.secondaryColor.color)
}

private struct ConnectionPanel: View {
    @StateObject var viewModel: ConnectionViewViewModel
    var onChevronToggle: (() -> Void)?
    var isExpanded: Binding<Bool>

    var body: some View {
        HStack(alignment: .top) {
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
            if case .connected = viewModel.tunnelState {
                if let onChevronToggle {
                    Spacer()
                    Button(action: onChevronToggle) {
                        Image(.iconChevron)
                            .renderingMode(.template)
                            .rotationEffect(isExpanded.wrappedValue ? .degrees(-90) : .degrees(90))
                            .frame(width: 44, height: 44, alignment: .topTrailing)
                            .foregroundStyle(.white)
                            .transaction { transaction in
                                transaction.animation = nil
                            }
                    }
                }
            }
        }
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
