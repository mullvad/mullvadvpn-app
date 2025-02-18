//
//  DetailsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension ConnectionView {
    internal struct DetailsView: View {
        @ObservedObject var viewModel: ConnectionViewViewModel
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
                        connectionDetailRow(
                            title: LocalizedStringKey("In"),
                            value: inAddress,
                            accessibilityId: .connectionPanelInAddressRow
                        )
                    }
                    if viewModel.tunnelIsConnected {
                        if let outAddressIpv4 = viewModel.outAddressIpv4 {
                            connectionDetailRow(
                                title: LocalizedStringKey("Out IPv4"),
                                value: outAddressIpv4,
                                accessibilityId: .connectionPanelOutAddressRow
                            )
                        }
                        if let outAddressIpv6 = viewModel.outAddressIpv6 {
                            connectionDetailRow(
                                title: LocalizedStringKey("Out IPv6"),
                                value: outAddressIpv6,
                                accessibilityId: .connectionPanelOutIpv6AddressRow
                            )
                        }
                    }
                }
            }
            .animation(.default, value: viewModel.inAddress)
            .animation(.default, value: viewModel.tunnelIsConnected)
        }

        @ViewBuilder
        private func connectionDetailRow(
            title: LocalizedStringKey,
            value: String,
            accessibilityId: AccessibilityIdentifier
        ) -> some View {
            HStack(alignment: .top, spacing: 8) {
                Text(title)
                    .font(.subheadline)
                    .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                    .frame(minWidth: columnWidth, alignment: .leading)
                    .sizeOfView { columnWidth = max(columnWidth, $0.width) }
                Text(value)
                    .font(.subheadline)
                    .foregroundStyle(UIColor.primaryTextColor.color)
                    .accessibilityIdentifier(accessibilityId.asString)
            }
        }
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true) { _, vm, _ in
        ConnectionView.DetailsView(viewModel: vm)
    }
}
