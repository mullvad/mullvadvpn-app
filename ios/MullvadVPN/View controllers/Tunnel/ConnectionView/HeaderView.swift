//
//  HeaderView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension ConnectionView {
    internal struct HeaderView: View {
        @ObservedObject var viewModel: ConnectionViewViewModel
        @Binding var isExpanded: Bool

        var body: some View {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 0) {
                    Text(viewModel.localizedTitleForSecureLabel)
                        .textCase(.uppercase)
                        .font(.title3.weight(.semibold))
                        .foregroundStyle(viewModel.textColorForSecureLabel.color)
                        .accessibilityIdentifier(viewModel.accessibilityIdForSecureLabel.asString)
                        .accessibilityLabel(viewModel.localizedAccessibilityLabelForSecureLabel)

                    if let countryAndCity = viewModel.titleForCountryAndCity {
                        Text(countryAndCity)
                            .font(.title3.weight(.semibold))
                            .foregroundStyle(UIColor.primaryTextColor.color)
                            .padding(.top, 4)
                    }

                    if let server = viewModel.titleForServer {
                        Text(server)
                            .font(.body)
                            .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                            .padding(.top, 2)
                            .accessibilityIdentifier(AccessibilityIdentifier.connectionPanelServerLabel.asString)
                    }
                }

                Group {
                    Spacer()
                    Image(.iconChevronUp)
                        .renderingMode(.template)
                        .rotationEffect(isExpanded ? .degrees(180) : .degrees(0))
                        .foregroundStyle(.white)
                        .accessibilityIdentifier(AccessibilityIdentifier.relayStatusCollapseButton.asString)
                }
                .showIf(viewModel.showsConnectionDetails)
            }
            .contentShape(Rectangle())
            .onTapGesture {
                isExpanded.toggle()
            }
        }
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true, isExpanded: true) { _, vm, isExpanded in
        ConnectionView.HeaderView(viewModel: vm, isExpanded: isExpanded)
    }
}
