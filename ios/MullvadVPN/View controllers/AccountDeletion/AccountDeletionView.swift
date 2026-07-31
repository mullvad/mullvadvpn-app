//
//  AccountDeletionView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct AccountDeletionView: View {
    @ObservedObject var viewModel: AccountDeletionViewModel

    @ScaledMetric var spinnerSize = 20.0
    @ScaledMetric var spinnerStatusGap = 10.0
    @State private var borderStyle: BorderStyle = .normal
    @State private var message: Message? = .init(text: "", appearance: .info)

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
                .padding(.bottom, 4)

                // Status information
                HStack {
                    if viewModel.isWorking {
                        ProgressView()
                            .progressViewStyle(MullvadProgressViewStyle(size: spinnerSize))
                        Spacer().frame(width: spinnerStatusGap)
                    }
                    if let statusText = viewModel.statusText {
                        Text(statusText)
                            .font(.mullvadSmall)
                            .foregroundStyle(Color.white)
                    }
                }

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
