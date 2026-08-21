// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
                    .showIf(viewModel.showsConnectionDetails)
            }
            .accessibilityElement(children: .contain)
            .contentShape(Rectangle())
            .onTapGesture {
                guard viewModel.showsConnectionDetails else { return }
                withAnimation {
                    isExpanded.toggle()
                }
            }
            .accessibilityIdentifier(
                AccessibilityIdentifier.relayStatusCollapseButton.asString
            )
        }
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true) { _, vm, isExpanded in
        ConnectionView.HeaderView(viewModel: vm, isExpanded: isExpanded)
    }
}
