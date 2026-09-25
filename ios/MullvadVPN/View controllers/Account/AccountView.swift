// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import StoreKit
import SwiftUI

struct AccountView: View {
    enum SheetType: Identifiable {
        var id: Self { self }
        case accountDeletion
        case deviceManagement
    }

    var viewModel: AccountViewModelProtocol
    var deviceManaging: DeviceManaging

    @State private var presentedSheet: SheetType? = nil
    @State private var alert: MullvadAlert? = nil
    @State private var isAccountDeleted = false
    @State private var isLoading = false

    #if DEBUG
        @State private var isShowingRefundSheet = false
        @State private var latestTransaction: StoreKit.Transaction?
    #endif

    var body: some View {
        VStack(spacing: 32) {
            deviceNameView()
            AccountNumberView(accountNumber: viewModel.accountNumber)
            paidUntilView()
            Spacer()
            MullvadButton(
                text: "Log out",
                style: .destructive,
                mainAccessibilityIdentifier: .logoutButton
            ) {
                Task {
                    isLoading = true
                    await viewModel.logout()
                    try? await Task.sleep(for: .seconds(1))
                    isLoading = false
                    viewModel.onFinish?(.userLoggedOut)
                }
            }
        }
        .disabled(isLoading)
        .foregroundStyle(Color.mullvadTextPrimary)
        .scrollable(fill: [.horizontal, .vertical])
        .scrollBounceBehavior(.basedOnSize)
        .padding()
        .navigationTitle("Account")
        .navigationBarTitleDisplayMode(.large)
        .accessibilityIdentifier(.accountView)
        .toolbar {
            MullvadDoneToolbarItem {
                viewModel.onFinish?(.none)
            }
            ToolbarItem(placement: .topBarLeading) {
                Menu {
                    #if DEBUG
                        Button("Refund last purchase") {
                            Task {
                                let latestTransaction = await Transaction.latest(
                                    for: StoreSubscription.thirtyDays.rawValue)
                                guard case .verified(let latestTransaction) = latestTransaction else {
                                    print("Could not find transaction")
                                    self.latestTransaction = nil
                                    return
                                }
                                self.latestTransaction = latestTransaction
                                isShowingRefundSheet.toggle()
                            }
                        }
                        Button("Finish unfinished purchase") {
                            Task {
                                await viewModel.finishUnfinishedPurchase()
                            }
                        }
                        Button("Factory reset", role: .destructive) {
                            Task {
                                await viewModel.factoryReset()
                            }
                        }
                    #endif
                    #if NEVER_IN_PRODUCTION
                        Button(String("Invalidate Wireguard key")) {
                            viewModel.invalidateWireguardKey()
                        }
                        Button(String("Use GotaTun")) {
                            viewModel.toggleGotaTun()
                        }
                    #endif
                    Button(role: .destructive) {
                        presentedSheet = .accountDeletion
                    } label: {
                        HStack {
                            Text("Delete account")
                            Image.mullvadIconDelete
                                .renderingMode(.template)
                        }
                    }
                    .accessibilityIdentifier(.deleteButton)
                } label: {
                    Image(systemName: "ellipsis")
                        .foregroundStyle(Color.mullvadTextPrimary)
                }
                .accessibilityIdentifier(.accountToolbarMenuButton)
            }
            .hideLiquidGlassEffect()
        }
        .sheet(item: $presentedSheet) {
            if isAccountDeleted {
                viewModel.onFinish?(.userLoggedOut)
            }
        } content: { sheetType in
            switch sheetType {
            case .accountDeletion:
                accountDeletionView()
            case .deviceManagement:
                deviceManagementView()
            }
        }
        .mullvadAlert(item: $alert)
        .background(Color.mullvadBackground)
        .mullvadLoadingSpinner(isPresented: isLoading)
        #if DEBUG
            .refundRequestSheet(
                for: latestTransaction?.id ?? 0,
                isPresented: $isShowingRefundSheet
            ) { result in
                switch result {
                case .success(let status):
                    switch status {
                    case .success:
                        print("Refund request submitted.")
                    case .userCancelled:
                        break
                    @unknown default:
                        break
                    }
                case .failure(let error):
                    print("Couldn't submit refund request: \(error.localizedDescription)")
                }
            }
        #endif
    }

    private func deviceNameView() -> some View {
        VStack {
            MullvadListSectionHeader(title: "Device name")
            HStack {
                Text(viewModel.deviceName)
                Spacer()
                Button("Manage devices") {
                    presentedSheet = .deviceManagement
                }
                .font(.mullvadSmallSemiBold)
                .underline()
                .accessibilityIdentifier(.deviceManagementButton)
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityRemoveTraits(.isHeader)
    }

    private func paidUntilView() -> some View {
        VStack {
            MullvadListSectionHeader(title: "Paid until")
            VStack(spacing: 24) {
                HStack(spacing: 8) {
                    viewModel.isOutOfTime
                        ? Text("OUT OF TIME")
                            .foregroundStyle(Color.mullvadDangerColor)
                            .accessibilityIdentifier(.accountPagePaidUntilLabel)
                        : Text(viewModel.paidUntilString)
                            .accessibilityIdentifier(.accountPagePaidUntilLabel)
                    Spacer()
                    Button("Add time") {
                        viewModel.addTime()
                    }
                    .font(.mullvadSmallSemiBold)
                    .underline()
                    .accessibilityIdentifier(.purchaseButton)
                }
                HStack {
                    Button("Restore purchases") {
                        viewModel.restorePurchase()
                    }
                    .font(.mullvadSmallSemiBold)
                    .underline()
                    .accessibilityIdentifier(.restorePurchasesButton)
                    Button {
                        alert = .init(
                            type: .none,
                            title: "If you haven’t received additional VPN time after purchasing",
                            messages: [
                                """
                                You can use the "restore purchases" function to check for any in-app payments \
                                made via Apple services. If there is a payment that has not been credited, it will \
                                add the time to the currently logged in Mullvad account.
                                """
                            ],
                            actions: [
                                .init(
                                    type: .primary, title: "Got it!",
                                    handler: {
                                        alert = nil
                                    })
                            ]
                        )
                    } label: {
                        Image.mullvadIconInfo
                    }
                    Spacer()
                }
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityRemoveTraits(.isHeader)
    }

    private func accountDeletionView() -> some View {
        AccountDeletionView(
            viewModel: AccountDeletionViewModel(
                accountNumber: viewModel.accountNumber,
                onDeleteAccount: { try await viewModel.deleteAccount() },
                onConclusion: { success in
                    isAccountDeleted = success
                    presentedSheet = nil
                }
            )
        )
    }

    private func deviceManagementView() -> some View {
        NavigationStack {
            DeviceManagementView(
                deviceManaging: deviceManaging,
                style: .deviceManagement
            ) { a, error in

            }
            .toolbar {
                MullvadDoneToolbarItem {
                    presentedSheet = nil
                }
            }
        }
    }
}

#Preview {
    @Previewable @State var viewModel = AccountViewModelMock()
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                AccountView(
                    viewModel: viewModel,
                    deviceManaging: MockDeviceManaging()
                )
                .background(Color.mullvadBackground)
            }
        }
}
