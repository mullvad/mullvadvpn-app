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
    @FocusState private var isTextFieldFocused: Bool

    @Environment(\.dismiss) private var dismiss
    var body: some View {
        VStack(alignment: .leading) {
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
            let placeholder = "XXXX"
            ConfigurableTextField(
                title: "Last 4 digits",
                placeholder: LocalizedStringKey(placeholder),
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
            .focused($isTextFieldFocused)
            .padding(.bottom, 4.0)

            Spacer()

            MullvadButton(text: "Delete account", style: .destructive) {
                Task {
                    if await viewModel.deleteButtonTapped() {
                        dismiss()
                    }
                }
            }
            .accessibilityIdentifier(.deleteButton)
            .disabled(!viewModel.canDelete)
        }
        .padding()
        .scrollable(fill: [.horizontal, .vertical])
        .scrollBounceBehavior(.basedOnSize)
        .contentMargins(.bottom, 16, for: .scrollContent)
        .navigationTitle("Delete account")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") {
                    viewModel.cancelButtonTapped()
                }
                .foregroundStyle(Color.mullvadTextPrimary)
            }
            .hideLiquidGlassEffect()
        }
        .onTapGesture {
            isTextFieldFocused = false
        }
        .accessibilityIdentifier(.deleteAccountView)
        .background(Color.mullvadBackground.ignoresSafeArea())
        .onReceive(viewModel.$state) { state in
            switch state {
            case .failure(let error):
                borderStyle = .error
                message = .init(text: LocalizedStringKey(error.localizedDescription), appearance: .error)
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
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                AccountDeletionView(
                    viewModel: AccountDeletionViewModel(
                        accountNumber: "1234567890123456",
                        onDeleteAccount: {}
                    )
                )
            }
        }
}
