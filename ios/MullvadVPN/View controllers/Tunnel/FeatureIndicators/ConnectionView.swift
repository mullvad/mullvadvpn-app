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

    @State private(set) var isExpanded: Bool = false
    @State private var scrollViewHeight: CGFloat = 0

    var action: ButtonAction?
    var onContentUpdate: (() -> Void)?

    var body: some View {
        Spacer()
        VStack(spacing: 22) {
            if viewModel.showsActivityIndicator {
                CustomProgressView(style: .large)
            }

            ZStack {
                BlurView(style: .dark)

                VStack(alignment: .leading, spacing: 16) {
                    ConnectionInfo(viewModel: viewModel, isExpanded: $isExpanded)

                    if isExpanded {
                        Divider()
                            .background(UIColor.secondaryTextColor.color)
                    }

                    // This geometry reader is somewhat of a workaround. It's "smart" in that it takes up as much
                    // space as it can and thereby helps the view to understand the maximum allowed height when
                    // placed in a UIKit context. If ConnectionView would ever be placed as a subview of SwiftUI
                    // parent, this reader could probably be removed.
                    if viewModel.isConnected {
                        GeometryReader { _ in
                            ScrollView {
                                VStack(spacing: 16) {
                                    if !indicatorsViewModel.chips.isEmpty {
                                        FeatureIndicatorsView(
                                            viewModel: indicatorsViewModel,
                                            isExpanded: $isExpanded
                                        )
                                    }

                                    if isExpanded {
                                        ConnectionDetails(viewModel: viewModel)
                                    }
                                }
                                .sizeOfView { scrollViewHeight = $0.height }
                            }
                        }
                        .frame(maxHeight: scrollViewHeight)
                    }

                    ButtonPanel(viewModel: viewModel, action: action)
                }
                .padding(16)
            }
            .cornerRadius(12)
            .padding(16)
        }
        .padding(.bottom, 8) // Adding some spacing so as not to overlap with the map legal link.
        .onChange(of: isExpanded) { _ in
            onContentUpdate?()
        }
        .onReceive(viewModel.combinedState) { (tunnelStatus, _) in
            onContentUpdate?()

            if tunnelStatus.state == .disconnected {
                isExpanded = false
            }
        }
    }
}

#Preview("ConnectionView (Normal)") {
    ConnectionViewPreview(configuration: .normal).make()
}

#Preview("ConnectionView (Normal, no indicators)") {
    ConnectionViewPreview(configuration: .normalNoIndicators).make()
}

#Preview("ConnectionView (Expanded)") {
    ConnectionViewPreview(configuration: .expanded).make()
}

#Preview("ConnectionView (Expanded, no indicators)") {
    ConnectionViewPreview(configuration: .expandedNoIndicators).make()
}

private struct ConnectionInfo: View {
    @StateObject var viewModel: ConnectionViewViewModel
    @Binding var isExpanded: Bool

    var body: some View {
        HStack(alignment: .top) {
            VStack(alignment: .leading, spacing: 0) {
                Text(viewModel.localizedTitleForSecureLabel)
                    .textCase(.uppercase)
                    .font(.title3.weight(.semibold))
                    .foregroundStyle(viewModel.textColorForSecureLabel.color)

                if let countryAndCity = viewModel.titleForCountryAndCity, let server = viewModel.titleForServer {
                    Text(countryAndCity)
                        .font(.title3.weight(.semibold))
                        .foregroundStyle(UIColor.primaryTextColor.color)
                        .padding(.top, 4)
                    Text(server)
                        .font(.body)
                        .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                }
            }
            .accessibilityLabel(viewModel.localizedAccessibilityLabel)

            if viewModel.isConnected {
                Spacer()
                Button(
                    action: { isExpanded.toggle() },
                    label: {
                        Image(.iconChevron)
                            .renderingMode(.template)
                            .rotationEffect(isExpanded ? .degrees(-90) : .degrees(90))
                            .frame(width: 44, height: 44, alignment: .topTrailing)
                            .foregroundStyle(.white)
                            .transaction { transaction in
                                transaction.animation = nil
                            }
                    }
                )
            }
        }
    }
}

private struct ConnectionDetails: View {
    @StateObject var viewModel: ConnectionViewViewModel
    @State private var columnWidth: CGFloat = 0

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(LocalizedStringKey("Connection details"))
                    .font(.footnote.weight(.semibold))
                    .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                Spacer()
            }

            VStack(alignment: .leading, spacing: 0) {
                if let inAddress = viewModel.inAddress {
                    connectionDetailRow(title: LocalizedStringKey("In"), value: inAddress)
                }
                if let outAddressIpv4 = viewModel.outAddressIpv4 {
                    connectionDetailRow(title: LocalizedStringKey("Out IPv4"), value: outAddressIpv4)
                }
                if let outAddressIpv6 = viewModel.outAddressIpv6 {
                    connectionDetailRow(title: LocalizedStringKey("Out IPv6"), value: outAddressIpv6)
                }
            }
        }
    }

    @ViewBuilder
    private func connectionDetailRow(title: LocalizedStringKey, value: LocalizedStringKey) -> some View {
        HStack(alignment: .top, spacing: 8) {
            Text(title)
                .font(.subheadline)
                .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                .frame(minWidth: columnWidth, alignment: .leading)
                .sizeOfView { columnWidth = max(columnWidth, $0.width) }
            Text(value)
                .font(.subheadline)
                .foregroundStyle(UIColor.primaryTextColor.color)
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
        switch viewModel.tunnelStatus.state {
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
                    viewModel.tunnelStatus.state == .waitingForConnectivity(.noConnection)
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
