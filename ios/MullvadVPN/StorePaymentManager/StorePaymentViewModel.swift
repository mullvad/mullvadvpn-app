// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import StoreKit

@MainActor
protocol PaymentViewModelProtocol: AnyObject {
    var products: [Product] { get }
    var isProcessing: Bool { get set }
    var notice: PaymentNotice? { get set }

    func initPurchase()
    func doPurchase(of: Product)
    func restorePurchases()
    func finishUnfinishedPurchases()
}

struct PaymentNotice: Equatable {
    var title: String
    var message: String
    var type: MullvadAlert.AlertType

    init(error: StorePaymentError, context: StorePaymentOutcome.Context) {
        self.title = context.errorTitle
        self.message = error.description
        type = .error
    }

    init(outcome: StorePaymentOutcome, context: StorePaymentOutcome.Context) {
        self.title = context.alertTitle
        self.message = outcome.alertMessage(for: context)
        type = .info
    }
}

@Observable
final class PaymentViewModel: PaymentViewModelProtocol, Sendable {
    private let logger = Logger(label: "PaymentViewModel")
    private let paymentManager: StorePaymentManager

    var products: [Product] = []
    var isProcessing: Bool = false
    var notice: PaymentNotice?

    init(paymentManager: StorePaymentManager) {
        self.paymentManager = paymentManager
    }

    func initPurchase() {
        isProcessing = true

        Task {
            defer { isProcessing = false }

            do {
                let outcome = try await paymentManager.processOutstandingTransactions()
                notice = PaymentNotice(outcome: outcome, context: .restorationBeforePurchase)
                if case .timeAdded = outcome {
                }
            } catch {
                notice = PaymentNotice(error: .restorationError, context: .restorationBeforePurchase)
                return
            }

            do {
                products = try await paymentManager.products()
            } catch {
                notice = PaymentNotice(error: .purchaseError(.productUnavailable), context: .purchase)
            }
        }
    }

    func doPurchase(of product: Product) {
        isProcessing = true

        Task {
            defer { isProcessing = false }

            let event = await paymentManager.purchase(product: product)

            switch event {
            case .successfulPayment(let storePaymentOutcome):
                notice = PaymentNotice(outcome: storePaymentOutcome, context: .purchase)
            case .failed(let storePaymentError):
                notice = PaymentNotice(error: storePaymentError, context: .purchase)
            case .userCancelled, .pending:
                break
            }
        }
    }

    func restorePurchases() {
        isProcessing = true

        Task {
            defer { isProcessing = false }

            do {
                let outcome = try await paymentManager.processOutstandingTransactions()
                notice = PaymentNotice(outcome: outcome, context: .restoration)
            } catch {
                notice = PaymentNotice(error: .restorationError, context: .restoration)
            }
        }
    }

    func finishUnfinishedPurchases() {
        Task {
            await StorePaymentManager.cleanupUnfinishedTransactions()
        }
    }
}

@Observable
class PaymentViewModelMock: PaymentViewModelProtocol {
    let products: [Product] = []
    var isProcessing: Bool = false
    var notice: PaymentNotice?

    func initPurchase() {}

    func doPurchase(of: Product) {}

    func restorePurchases() {}

    func finishUnfinishedPurchases() {}
}
