// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Logging
import MullvadTypes
import StoreKit
import SwiftUI

struct AccountView: View {
    enum SheetType: Identifiable {
        var id: Self { self }
        case accountDeletion
        case deviceManagement
        case payment
    }

    var accountViewModel: AccountViewModelProtocol
    var paymentViewModel: PaymentViewModelProtocol
    var deviceManaging: DeviceManaging

    @State private var presentedSheet: SheetType? = nil
    @State private var alert: MullvadAlert? = nil
    @State private var isAccountDeleted = false
    @State private var isLoading = false
    @State var height: CGFloat = 300
    @State var selectedDetent: PresentationDetent = .height(300)
    @State var detents: Set<PresentationDetent> = [.height(300)]

    #if NEVER_IN_PRODUCTION
        let logger = Logger(label: "AccountView")
        @State private var isShowingRefundSheet = false
        @State private var latestTransaction: StoreKit.Transaction?
    #endif

    var body: some View {
        VStack(spacing: 32) {
            deviceNameView()
            AccountNumberView(accountNumber: accountViewModel.accountNumber)
            paidUntilView()
            Spacer()
            MullvadButton(
                text: "Log out",
                style: .destructive,
                mainAccessibilityIdentifier: .logoutButton
            ) {
                Task {
                    isLoading = true
                    await accountViewModel.logout()
                    try? await Task.sleep(for: .seconds(1))
                    isLoading = false
                    accountViewModel.onFinish?(.userLoggedOut)
                }
            }
        }
        .disabled(isLoading)
        .foregroundStyle(Color.mullvadTextPrimary)
        .scrollable(fill: [.horizontal, .vertical])
        .scrollBounceBehavior(.basedOnSize)
        .padding()
        .preferredColorScheme(.dark)
        .navigationTitle("Account")
        .accessibilityIdentifier(.accountView)
//        .onChange(
//            of: paymentViewModel.notice,
//            { (oldValue, notice) in
//                if oldValue != notice, let notice {
//                    isLoading = false
//                    alert = .init(
//                        type: notice.type,
//                        title: LocalizedStringKey(notice.title),
//                        messages: [LocalizedStringKey(notice.message)],
//                        actions: [
//                            .init(type: .primary, title: "Got it!") {
//                                alert = nil
//                                paymentViewModel.notice = nil
//                            }
//                        ]
//                    )
//                }
//            }
//        )
        .toolbar {
            MullvadDoneToolbarItem {
                accountViewModel.onFinish?(.none)
            }
            ToolbarItem(placement: .topBarLeading) {
                Menu {
                    #if NEVER_IN_PRODUCTION
                        debugButtons()
                    #endif
                    Button(role: .destructive) {
                        presentedSheet = .accountDeletion
                    } label: {
                        HStack {
                            Text("Delete account")
                            Image.mullvadIconDelete
                                .renderingMode(.template)
                                .foregroundStyle(Color.mullvadDangerColor)
                        }
                    }
                    .accessibilityIdentifier(.deleteButton)
                } label: {
                    Image(systemName: "ellipsis.circle.fill")
                        .foregroundStyle(Color.mullvadTextPrimary)
                }
                .accessibilityIdentifier(.accountToolbarMenuButton)
            }
            .hideLiquidGlassEffect()
        }
        .sheet(item: $presentedSheet) {
            if isAccountDeleted {
                accountViewModel.onFinish?(.userLoggedOut)
            }
        } content: { sheetType in
            switch sheetType {
            case .accountDeletion:
                accountDeletionView()
            case .deviceManagement:
                deviceManagementView()
            case .payment:
                paymentView()
            }
        }
        .mullvadAlert(item: $alert)
        .background(Color.mullvadBackground)
        .mullvadLoadingSpinner(isPresented: isLoading)
        #if NEVER_IN_PRODUCTION
            .refundRequestSheet(
                for: latestTransaction?.id ?? 0,
                isPresented: $isShowingRefundSheet
            ) { result in
                switch result {
                case .success(let status):
                    switch status {
                    case .success:
                        logger.debug("Refund request submitted.")
                    case .userCancelled:
                        break
                    @unknown default:
                        break
                    }
                case .failure(let error):
                    logger.debug("Couldn't submit refund request: \(error.localizedDescription)")
                }
            }
        #endif
    }

    private func deviceNameView() -> some View {
        VStack {
            MullvadListSectionHeader(title: "Device name")
            HStack {
                Text(accountViewModel.deviceName)
                    .accessibilityIdentifier(.accountPageDeviceNameLabel)
                Spacer()
                Button {
                    presentedSheet = .deviceManagement
                } label: {
                    Text("Manage devices")
                        .underline()
                        .multilineTextAlignment(.leading)
                }
                .font(.mullvadSmallSemiBold)
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
                    accountViewModel.isOutOfTime
                        ? Text("OUT OF TIME")
                            .foregroundStyle(Color.mullvadDangerColor)
                            .accessibilityIdentifier(.accountPagePaidUntilLabel)
                        : Text(accountViewModel.paidUntilString)
                            .accessibilityIdentifier(.accountPagePaidUntilLabel)
                    Spacer()
                    Button {
//                        isLoading = true
//                        Task {
                            paymentViewModel.initPurchase()
                            presentedSheet = .payment
//                            isLoading = false
//                        }
                    } label: {
                        Text("Add time")
                            .underline()
                            .multilineTextAlignment(.leading)
                    }
                    .font(.mullvadSmallSemiBold)
                    .accessibilityIdentifier(.purchaseButton)
                }
                HStack {
                    Button {
                        isLoading = true
                        paymentViewModel.restorePurchases()
                    } label: {
                        Text("Restore purchases")
                            .underline()
                            .multilineTextAlignment(.leading)
                    }
                    .font(.mullvadSmallSemiBold)
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
                        ResizableImageView(
                            image:
                                Image.mullvadIconInfo,
                            dimension: .height(18)
                        )
                    }
                    Spacer()
                }
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityRemoveTraits(.isHeader)
    }

    private func accountDeletionView() -> some View {
        NavigationStack {
            AccountDeletionView(
                viewModel: AccountDeletionViewModel(
                    accountNumber: accountViewModel.accountNumber,
                    onDeleteAccount: { try await accountViewModel.deleteAccount() },
                    onConclusion: { success in
                        isAccountDeleted = success
                        presentedSheet = nil
                    }
                )
            )
        }
    }

    private func deviceManagementView() -> some View {
        NavigationStack {
            DeviceManagementView(
                deviceManaging: deviceManaging,
                style: .deviceManagement
            ) { a, error in
                // Do nothing.
            }
            .toolbar {
                MullvadDoneToolbarItem {
                    presentedSheet = nil
                }
            }
        }
    }

    private func paymentView() -> some View {
        VStack(spacing: 16) {
            ZStack {
                Text("Add time")
                    .font(.mullvadSmallSemiBold)
                HStack {
                    Button {
                        presentedSheet = nil
                    } label: {
                        ResizableImageView(image: .mullvadIconCross, dimension: .width(32))
                    }
                    .accessibilityIdentifier(.cancelButton)
                    Spacer()
                }
                .padding(.leading, 12)
            }
            .padding(.vertical, 8)

            Group {
                if let notice = paymentViewModel.notice {
                    Text(LocalizedStringKey(notice.message))
                    MullvadButton(
                        text: "Got it!",
                        style: .secondary,
                        action: {
                            paymentViewModel.notice = nil
                        }
                    )
//                    alert = .init(
//                        type: notice.type,
//                        title: LocalizedStringKey(notice.title),
//                        messages: [LocalizedStringKey(notice.message)],
//                        actions: [
//                            .init(type: .primary, title: "Got it!") {
////                                alert = nil
//                                paymentViewModel.notice = nil
//                            }
//                        ]
//                    )
                } else if paymentViewModel.isProcessing {
                    ProgressView()
                        .progressViewStyle(MullvadProgressViewStyle())
                } else {
                    Group {
                        ForEach(paymentViewModel.products, id: \.self) { product in
                            product.customLocalizedTitle.flatMap { productTitle in
                                MullvadButton(
                                    text: LocalizedStringKey(productTitle),
                                    style: .primary,
                                    action: {
                                        //                                    presentedSheet = nil
                                        paymentViewModel.doPurchase(of: product)
                                    }
                                )
                            }
                        }
                        MullvadButton(
                            text: "Restore purchases",
                            style: .secondary,
                            action: {
                                presentedSheet = nil
                                isLoading = true
                                paymentViewModel.restorePurchases()
                            }
                        )
                        .accessibilityIdentifier(.restorePurchasesButton)
                    }
                    .disabled(paymentViewModel.isProcessing)
                }
            }
            .transition(.opacity)
            .animation(.easeInOut(duration: 0.25), value: paymentViewModel.isProcessing)

        }
        .padding(16)
        .background(Color.mullvadBackground)
        .presentationBackground(Color.mullvadBackground)
        .foregroundStyle(Color.mullvadTextPrimary)
        .cornerRadius(8)
        .sizeOfView {
            height = $0.height
        }
        .onChange(of: height) {
            detents.insert(.height(height))
            selectedDetent = .height(height)
            Task {
                try? await Task.sleep(for: .seconds(0.1))
                detents.removeFirst()
            }
        }
        .presentationDetents(Set(detents), selection: $selectedDetent)
    }

    #if NEVER_IN_PRODUCTION
        @ViewBuilder
        func debugButtons() -> some View {
            Button("Refund last purchase".verbatim) {
                Task {
                    let latestTransaction = await Transaction.latest(
                        for: StoreSubscription.thirtyDays.rawValue)
                    guard case .verified(let latestTransaction) = latestTransaction else {
                        logger.debug("Could not find transaction")
                        self.latestTransaction = nil
                        return
                    }
                    self.latestTransaction = latestTransaction
                    isShowingRefundSheet.toggle()
                }
            }
            Button("Finish unfinished purchase".verbatim) {
                paymentViewModel.finishUnfinishedPurchases()
            }
            Button("Factory reset".verbatim, role: .destructive) {
                Task {
                    await accountViewModel.factoryReset()
                }
            }
            Button("Invalidate Wireguard key".verbatim) {
                accountViewModel.invalidateWireguardKey()
            }
            Button(getTunnelString()) {
                accountViewModel.toggleWireGuard()
            }
        }

        func getTunnelString() -> String {
            PacketTunnelDebugSettings.useWireGuard ? "Use GotaTun" : "Use WireGuard"
        }
    #endif
}

#Preview {
    @Previewable @State var accountViewModel = AccountViewModelMock()
    @Previewable @State var paymentViewModel = PaymentViewModelMock()
    @Previewable @Environment(\.dynamicTypeSize) var size
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationStack {
                AccountView(
                    accountViewModel: accountViewModel,
                    paymentViewModel: paymentViewModel,
                    deviceManaging: MockDeviceManaging()
                )
                .background(Color.mullvadBackground)
            }
            .dynamicTypeSize(size)
        }
}
