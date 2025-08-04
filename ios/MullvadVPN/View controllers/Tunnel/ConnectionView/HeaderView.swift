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

        @State var titleForCountryAndCity: LocalizedStringKey?
        @State var titleForServer: LocalizedStringKey?

        var body: some View {
            Button {
                withAnimation {
                    isExpanded.toggle()
                }
            } label: {
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 0) {
                        Text(viewModel.localizedTitleForSecureLabel)
                            .textCase(.uppercase)
                            .font(.title3.weight(.semibold))
                            .foregroundStyle(viewModel.textColorForSecureLabel.color)
                            .accessibilityIdentifier(viewModel.accessibilityIdForSecureLabel.asString)
                            .accessibilityLabel(viewModel.localizedAccessibilityLabelForSecureLabel)
                        if let titleForCountryAndCity {
                            Text(titleForCountryAndCity)
                                .font(.title3.weight(.semibold))
                                .foregroundStyle(UIColor.primaryTextColor.color)
                                .padding(.top, 4)
                        }
                        if let titleForServer {
                            Text(titleForServer)
                                .font(.body)
                                .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
                                .padding(.top, 2)
                                .accessibilityIdentifier(AccessibilityIdentifier.connectionPanelServerLabel.asString)
                                .multilineTextAlignment(.leading)
                        }
                    }

                    Group {
                        Spacer()
                        Button {
                            withAnimation {
                                isExpanded.toggle()
                            }
                        } label: {
                            Image(.iconChevronUp)
                                .renderingMode(.template)
                                .rotationEffect(isExpanded ? .degrees(-180) : .degrees(0))
                                .foregroundStyle(.white)
                                .accessibilityIdentifier(AccessibilityIdentifier.relayStatusCollapseButton.asString)
                        }
                    }
                    .showIf(viewModel.showsConnectionDetails)
                }
                .onAppear {
                    titleForServer = viewModel.titleForServer
                    titleForCountryAndCity = viewModel.titleForCountryAndCity
                }
                .onChange(of: viewModel.titleForCountryAndCity, perform: { newValue in
                    withAnimation {
                        titleForCountryAndCity = newValue
                    }
                })
                .onChange(of: viewModel.titleForServer, perform: { newValue in
                    withAnimation {
                        titleForServer = newValue
                    }
                })
            }
            .disabled(!viewModel.showsConnectionDetails)
        }
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true) { _, vm, isExpanded in
        ConnectionView.HeaderView(viewModel: vm, isExpanded: isExpanded)
    }
}
