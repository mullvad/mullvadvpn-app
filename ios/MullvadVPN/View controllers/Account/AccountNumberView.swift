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

struct AccountNumberView: View {
    let accountNumber: String

    @State private var isShowingAccountNumber = false
    @State private var hasJustCopiedAccountNumber = false

    var body: some View {
        VStack {
            MullvadListSectionHeader(title: "Account number")
            HStack(spacing: 24) {
                Text(
                    isShowingAccountNumber
                        ? accountNumber.formattedAccountNumber
                        : String(
                            repeating: "•",
                            count: accountNumber.count
                        ).formattedAccountNumber
                )
                .font(.mullvadSmall)
                Spacer()
                Button {
                    isShowingAccountNumber.toggle()
                } label: {
                    isShowingAccountNumber
                        ? Image.mullvadIconObscure
                        : Image.mullvadIconUnobscure
                }
                .animation(.default, value: isShowingAccountNumber)
                Button {
                    copyAccountNumberToClipboard()
                } label: {
                    hasJustCopiedAccountNumber ? Image.mullvadIconTick : Image.mullvadIconCopy
                }
                .foregroundStyle(
                    hasJustCopiedAccountNumber ? Color.mullvadSuccessColor : Color.mullvadTextPrimary
                )
                .animation(.default, value: hasJustCopiedAccountNumber)
            }
            .foregroundStyle(Color.mullvadTextPrimary)
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("Account number")
        .accessibilityRemoveTraits(.isButton)
        .apply {
            if isShowingAccountNumber {
                $0.accessibilityValue(accountNumber)
            } else {
                $0.accessibilityValue("Obscured")
            }
        }
        .accessibilityAction {
            isShowingAccountNumber.toggle()
        } label: {
            Text(isShowingAccountNumber ? "Hide account number" : "Show account number")
        }
        .accessibilityAction {
            copyAccountNumberToClipboard()
        } label: {
            Text("Copied Mullvad account number to pasteboard")
        }
    }

    private func copyAccountNumberToClipboard() {
        UIPasteboard.general.string = accountNumber
        hasJustCopiedAccountNumber = true
        Task {
            try? await Task.sleep(for: .seconds(2))
            hasJustCopiedAccountNumber = false
        }
    }
}

#Preview {
    AccountNumberView(accountNumber: "1234123412341234")
        .background(Color.mullvadBackground)
}
