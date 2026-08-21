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

struct AccountDeletionView: View {
    @ObservedObject var viewModel: AccountDeletionViewModel
    @State private var borderStyle: BorderStyle = .normal
    @State private var message: MessageView.Message? = nil
    var body: some View {
        ScrollView {
            VStack(alignment: .leading) {
                Text("Account deletion")
                    .font(.mullvadLarge)
                    .foregroundStyle(Color.white)
                    .padding(.bottom, 8)

                Text(viewModel.messageText)
                    .foregroundStyle(Color.white)
                    .padding(.bottom, 8)

                Text(
                    """
                    This logs out all devices using this account and all \
                    VPN access will be denied even if there is time left on the account. \
                    Enter the last 4 digits of the account number and hit "Delete account" \
                    if you really want to delete the account:
                    """
                )
                .font(.mullvadSmallSemiBold)
                .foregroundStyle(Color.white)
                .padding(.bottom, 8)

                // accountTextField
                ConfigurableTextField(
                    title: "Last 4 digits",
                    placeholder: "XXXX",
                    text: $viewModel.enteredAccountNumberSuffix,
                    message: $message,
                    accessibilityIdentifier: .deleteAccountTextField,
                    borderStyle: $borderStyle,
                    configuration: TextFieldNamespace.Configuration(
                        formatter: GroupedTextFormatter(
                            configuration: .init(
                                allowedInput: .numeric,
                                groupSeparator: "-",
                                groupSize: 4, maxGroups: 1)),
                        keyboardType: .numberPad)
                )
                .padding(.bottom, 4.0)

                Spacer()

                MullvadButton(text: "Delete account", style: .destructive) {
                    viewModel.deleteButtonTapped()
                }
                .accessibilityIdentifier(.deleteButton)
                .disabled(!viewModel.canDelete)

                MullvadButton(text: "Cancel", style: .primary) {
                    viewModel.cancelButtonTapped()
                }
            }
        }
        .padding(16)
        .background(Color.mullvadBackground)
        .onReceive(viewModel.$state) { state in
            switch state {
            case .failure(let error):
                borderStyle = .error
                message = .init(text: error.localizedDescription, appearance: .error)
            case .working:
                message = .init(text: "Deleting account...", appearance: .loading)
                borderStyle = .normal
            default:
                borderStyle = .normal
                message = nil
            }
        }
    }
}

#Preview {
    AccountDeletionView(viewModel: AccountDeletionViewModel(mockAccountNumber: "1234567890123456"))
}
