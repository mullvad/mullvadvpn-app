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

struct MethodTestingStatusView: View {
    let status: ApiAccessMethodTestState
    let onStop: () -> Void

    @ScaledMetric(wrappedValue: 24, relativeTo: .body)
    var iconWidth: CGFloat

    var body: some View {
        switch status {
        case .reachable:
            statusView(
                text: "API reachable", filledColor: .mullvadSuccessColor,
                accessibilityIdentifier: AccessibilityIdentifier.addAccessMethodTestStatusReachableLabel)
        case .unreachable:
            statusView(
                text: "API unreachable", filledColor: .mullvadDangerColor,
                accessibilityIdentifier: AccessibilityIdentifier.addAccessMethodTestStatusUnreachableLabel)
        case .testing:
            VStack {
                HStack {
                    ProgressView()
                        .progressViewStyle(MullvadProgressViewStyle(size: 24.0))
                        .frame(width: iconWidth)
                    labelView(text: "Testing...", accessibilityIdentifier: .accessMethodTestButton)
                }

                MullvadButton(
                    text: "Cancel", style: .primary,
                    action: {
                        onStop()
                    })

            }
        default:
            EmptyView()
        }
    }

    @ViewBuilder
    private func statusView(
        text: LocalizedStringKey, filledColor: Color, accessibilityIdentifier: AccessibilityIdentifier
    ) -> some View {
        HStack {
            Circle()
                .fill(filledColor)
                .frame(width: iconWidth)
            labelView(text: text, accessibilityIdentifier: accessibilityIdentifier)
        }
    }

    @ViewBuilder
    private func labelView(text: LocalizedStringKey, accessibilityIdentifier: AccessibilityIdentifier) -> some View {
        Text(text)
            .font(.mullvadSmall)
            .foregroundStyle(Color.mullvadTextPrimary)
            .accessibilityIdentifier(accessibilityIdentifier.asString)
    }
}
