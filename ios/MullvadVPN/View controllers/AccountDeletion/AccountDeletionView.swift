//
//  AccountDeletionView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-08-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct AccountDeletionView: View {
    @ObservedObject var viewModel: AccountDeletionViewModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading) {
                Text(NSLocalizedString("Account deletion", comment: ""))
                    .font(.mullvadLarge)
                    .foregroundStyle(Color.white)
                    .padding(.init(top: 0, leading: 0, bottom: 8, trailing: 0))

                Text(viewModel.messageText)
                    .foregroundStyle(Color.white)
                    .padding(.bottom, 8)

                Text(NSLocalizedString(
                    """
                    This logs out all devices using this account and all \
                    VPN access will be denied even if there is time left on the account. \
                    Enter the last 4 digits of the account number and hit "Delete account" \
                    if you really want to delete the account:
                    """,
                    comment: ""
                ))
                .font(.mullvadSmallSemiBold)
                .foregroundStyle(Color.white)
                .padding(.init(top: 0, leading: 0, bottom: 8, trailing: 0))

                // accountTextField
                MullvadPrimaryTextField(
                    label: "Last 4 digits", placeholder: "XXXX", text: $viewModel.enteredAccountNumberSuffix
                )
                .padding(.init(top: 0, leading: 0, bottom: 4, trailing: 0))

                // Status information
                HStack {
                    if viewModel.isWorking {
                        ProgressView()
                            .progressViewStyle(MullvadProgressViewStyle(size: 20))
                        Spacer().frame(width: 10)
                    }

                    Text(viewModel.statusText)
                        .font(Font(UIFont.preferredFont(forTextStyle: .body)))
                        .foregroundStyle(Color.white)
                }

                Spacer()

                MainButton(text: "Delete Account", style: .danger) {
                    viewModel.deleteButtonTapped()
                }
                .disabled(!viewModel.canDelete)

                MainButton(text: "Cancel", style: .default) {
                    viewModel.cancelButtonTapped()
                }
            }
        }
        .padding(16)
        .background(Color.mullvadBackground)
    }
}

#Preview {
    AccountDeletionView(viewModel: AccountDeletionViewModel(accountNumber: "1234 5678 9012 3456"))
}
