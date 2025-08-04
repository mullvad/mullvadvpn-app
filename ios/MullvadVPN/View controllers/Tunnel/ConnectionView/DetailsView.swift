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
                    Text(NSLocalizedString(
                        "CONNECTION_DETAILS_TITLE",
                        tableName: "Main",
                        value: "Connection details",
                        comment: ""
                    ))
                    .font(.footnote.weight(.semibold))
                    .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                    Spacer()
                }

                VStack(alignment: .leading, spacing: 0) {
                    if let inAddress = viewModel.inAddress {
                        connectionDetailRow(
                            title: NSLocalizedString("IN_LABEL", tableName: "Main", value: "In", comment: ""),
                            value: inAddress,
                            accessibilityId: .connectionPanelInAddressRow
                        )
                    }
                    if viewModel.tunnelIsConnected {
                        if let outAddressIpv4 = viewModel.outAddressIpv4 {
                            connectionDetailRow(
                                title: NSLocalizedString(
                                    "OUT_IPV4_LABEL",
                                    tableName: "Main",
                                    value: "Out IPv4",
                                    comment: ""
                                ),
                                value: outAddressIpv4,
                                accessibilityId: .connectionPanelOutAddressRow
                            )
                        }
                        if let outAddressIpv6 = viewModel.outAddressIpv6 {
                            connectionDetailRow(
                                title: NSLocalizedString(
                                    "OUT_IPV6_LABEL",
                                    tableName: "Main",
                                    value: "Out IPv6",
                                    comment: ""
                                ),
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
            title: String,
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
