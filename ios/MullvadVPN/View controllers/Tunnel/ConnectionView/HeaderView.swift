//
//  HeaderView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension ConnectionView {
    internal struct HeaderView: View {
        @ObservedObject var viewModel: ConnectionViewViewModel
        @Binding var isExpanded: Bool

        var body: some View {
            HStack(alignment: .top) {
                Text(viewModel.localizedTitleForSecureLabel)
                    .font(.title3.weight(.semibold))
                    .foregroundStyle(viewModel.textColorForSecureLabel.color)
                    .accessibilityIdentifier(viewModel.accessibilityIdForSecureLabel.asString)
                    .accessibilityLabel(viewModel.localizedAccessibilityLabelForSecureLabel)
                    .accessibilityRemoveTraits(.isButton)

                Spacer()

                Image(.iconChevronUp)
                    .renderingMode(.template)
                    .rotationEffect(isExpanded ? .degrees(-180) : .degrees(0))
                    .foregroundStyle(.white)
                    .accessibilityRemoveTraits(.isImage)
                    .accessibilityLabel(
                        isExpanded
                            ? LocalizedStringKey("Collapse connection details")
                            : LocalizedStringKey("Expand connection details")
                    )
            }
            .showIf(viewModel.showsConnectionDetails)
            .accessibilityIdentifier(AccessibilityIdentifier.relayStatusCollapseButton.asString)
            .overlay {
                Color.clear
                    .contentShape(Rectangle())
                    .onTapGesture {
                        withAnimation {
                            isExpanded.toggle()
                        }
                    }
                    .accessibilityHidden(true)  // does not interfere with VoiceOver
            }
        }
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true) { _, vm, isExpanded in
        ConnectionView.HeaderView(viewModel: vm, isExpanded: isExpanded)
    }
}
