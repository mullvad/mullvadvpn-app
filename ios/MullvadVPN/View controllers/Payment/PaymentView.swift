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

struct PaymentView: View {
    let paymentViewModel: PaymentViewModelProtocol
    let onClose: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            ZStack {
                Text("Add time")
                    .font(.mullvadSmallSemiBold)
                HStack {
                    Button {
                        onClose()
                        paymentViewModel.resetState()
                    } label: {
                        ResizableImageView(image: .mullvadIconCross, dimension: .height(32))
                    }
                    .accessibilityIdentifier(.cancelButton)
                    Spacer()
                }
                .padding(.leading, 12)
            }
            .padding(.vertical, 8)

            VStack(spacing: 8) {
                ForEach(paymentViewModel.products, id: \.self) { product in
                    product.customLocalizedTitle.flatMap { productTitle in
                        MullvadButton(
                            text: LocalizedStringKey(productTitle),
                            style: .primary,
                            action: {
                                paymentViewModel.doPurchase(of: product)
                            }
                        )
                    }
                }
                MullvadButton(
                    text: "Restore purchases",
                    style: .secondary,
                    action: {
                        paymentViewModel.restorePurchases()
                    }
                )
                .accessibilityIdentifier(.restorePurchasesButton)
            }
            .overlay {
                if let notice = paymentViewModel.notice {
                    VStack(spacing: 16) {
                        Text(LocalizedStringKey(notice.message))
                            .fixedSize(horizontal: false, vertical: true)
                            .padding(.horizontal, 16)
                            .scrollable(fill: .vertical)
                            .scrollBounceBehavior(.basedOnSize)
                        MullvadButton(
                            text: "Got it!",
                            style: .secondary,
                            action: {
                                if case .timeAdded(_) = notice.outcome {
                                    onClose()
                                    paymentViewModel.resetState()
                                } else {
                                    paymentViewModel.notice = nil
                                }
                            }
                        )
                    }

                    .background(Color.mullvadBackground)
                } else if paymentViewModel.isProcessing {
                    ProgressView()
                        .progressViewStyle(MullvadProgressViewStyle())
                }
            }
            .disabled(paymentViewModel.isProcessing)
            .animation(.easeInOut(duration: 0.2), value: paymentViewModel.products)
            .animation(.easeInOut(duration: 0.2), value: paymentViewModel.notice)
            .animation(.easeInOut(duration: 0.2), value: paymentViewModel.isProcessing)
        }
        .padding(16)
        .background(Color.mullvadBackground)
        .presentationBackground(Color.mullvadBackground)
        .foregroundStyle(Color.mullvadTextPrimary)
        .cornerRadius(8)
    }
}
