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

struct ActionBox: View {
    struct Action {
        let onAction: () -> Void
        let label: LocalizedStringKey
    }

    struct AdditionalInfo {
        var warningTitle: LocalizedStringKey
        var warningMessage: LocalizedStringKey
    }

    @Binding var isChecked: Bool
    @State private var isExpanded = false
    let toggleTitle: LocalizedStringKey
    var additionalInfo: AdditionalInfo?
    var action: Action?

    var body: some View {
        VStack(alignment: .leading) {
            Button {
                withAnimation {
                    isChecked.toggle()
                }
            } label: {
                HStack {
                    Toggle("", isOn: $isChecked.animation())
                        .toggleStyle(CheckboxToggleStyle(accessibilityId: .checkbox))
                    Text(toggleTitle)
                        .font(.mullvadTiny)
                        .foregroundStyle(Color.mullvadTextPrimary)
                        .multilineTextAlignment(.leading)
                    Spacer()
                }
            }
            if let additionalInfo, isChecked {
                VStack(alignment: .leading) {
                    Button {
                        withAnimation {
                            isExpanded.toggle()
                        }
                    } label: {
                        HStack {
                            Image.mullvadIconWarning
                            Text(additionalInfo.warningTitle)
                                .font(Font.mullvadTinySemiBold)
                                .foregroundStyle(Color.mullvadTextPrimary)
                            Spacer()
                            Button {
                                withAnimation {
                                    isExpanded.toggle()
                                }
                            } label: {
                                Image.mullvadIconChevron
                                    .rotationEffect(
                                        .degrees(
                                            isExpanded ? -90 : 90
                                        )
                                    )
                            }
                        }
                    }
                    if isExpanded {
                        Text(additionalInfo.warningMessage)
                            .font(Font.mullvadTiny)
                            .foregroundStyle(Color.mullvadTextPrimary)
                        if let action {
                            MullvadButton(
                                text: action.label,
                                style: .primary,
                                action: action.onAction
                            )
                        }
                    }
                }
                .padding(8)
                .background(Color.mullvadDarkBackground)
                .cornerRadius(UIMetrics.ActionBox.cornerRadius)
            }
        }
        .geometryGroup()
        .padding(8)
        .background(Color.mullvadBackground)
        .cornerRadius(UIMetrics.ActionBox.cornerRadius)
        .overlay(
            RoundedRectangle(cornerRadius: UIMetrics.ActionBox.cornerRadius)
                .stroke(Color.MullvadActionBox.border, lineWidth: 1)
        )
    }
}

#Preview {
    @Previewable @State var isChecked: Bool = false
    VStack {
        Spacer()
        ActionBox(
            isChecked: $isChecked,
            toggleTitle: "By checking this box I agree to the risks involved with proceeding with this action.",
            additionalInfo: .init(
                warningTitle: "This impacts your anonymity",
                warningMessage:
                    """
                    The Internet traffic in the excluded applications will not go through
                    the VPN. Your own IP address will be exposed. 
                    When you exclude some apps, other apps may be unintentionally excluded too.
                    """),
            action: .init(onAction: {}, label: "Action")
        )
        ActionBox(
            isChecked: $isChecked,
            toggleTitle: "By checking this box I agree to the risks involved with proceeding with this action."
        )
        Spacer()
    }
    .padding()
    .background(Color.mullvadBackground)
}
