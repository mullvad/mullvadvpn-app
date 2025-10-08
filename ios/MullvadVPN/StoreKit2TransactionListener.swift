//
//  StoreKit2TransactionListener.swift
//  MullvadVPN
//
//  Created by pronebird on 08/10/2025.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import StoreKit

/// Listener for StoreKit 2 transactions that uploads them to the API and updates account data.
///
/// This class listens for transaction updates from StoreKit 2 and handles successful transactions
/// by uploading them to the backend and refreshing the account data.
final class StoreKit2TransactionListener: @unchecked Sendable {
    private let logger = Logger(label: "StoreKit2TransactionListener")
    private let apiProxy: APIQuerying
    private let accountsProxy: RESTAccountHandling
    private var updateListenerTask: Task<Void, Never>?

    /// Delegate to provide account number for transactions.
    weak var delegate: StoreKit2TransactionListenerDelegate?

    init(apiProxy: APIQuerying, accountsProxy: RESTAccountHandling) {
        self.apiProxy = apiProxy
        self.accountsProxy = accountsProxy
    }

    /// Start listening for transaction updates.
    func start() {
        logger.debug("Starting StoreKit 2 transaction listener.")

        updateListenerTask = Task.detached { [weak self] in
            guard let self else { return }

            for await verificationResult in Transaction.updates {
                await self.handleTransactionUpdate(verificationResult)
            }
        }
    }

    /// Stop listening for transaction updates.
    func stop() {
        logger.debug("Stopping StoreKit 2 transaction listener.")
        updateListenerTask?.cancel()
        updateListenerTask = nil
    }

    deinit {
        updateListenerTask?.cancel()
    }

    // MARK: - Private methods

    private func handleTransactionUpdate(_ verificationResult: VerificationResult<Transaction>) async {
        guard let transaction = try? verificationResult.payloadValue else {
            logger.error("Failed to verify transaction.")
            return
        }

        // Only process purchased transactions
        guard transaction.productType == .autoRenewable else {
            logger.debug("Ignoring non-subscription transaction: \(transaction.id)")
            return
        }

        logger.info("Received transaction update for product: \(transaction.productID)")

        // Get account number from delegate
        guard let accountNumber = await getAccountNumber() else {
            logger.warning("No account number available for transaction.")
            return
        }

        // Upload transaction to API
        await uploadTransaction(verificationResult, accountNumber: accountNumber)
    }

    private func getAccountNumber() async -> String? {
        await MainActor.run {
            self.delegate?.fetchAccountNumber()
        }
    }

    private func uploadTransaction(_ transaction: VerificationResult<Transaction>, accountNumber: String) async {
        // Get the transaction JWT
        let jwsRepresentation = transaction.jwsRepresentation
        let storekitTransaction = StorekitTransaction(transaction: jwsRepresentation)

        // Upload to API
        let result = await withCheckedContinuation { continuation in
            _ = self.apiProxy.checkStorekitPayment(
                accountNumber: accountNumber,
                transaction: storekitTransaction,
                retryStrategy: .default
            ) { result in
                continuation.resume(returning: result)
            }
        }

        switch result {
        case .success:
            guard let payment = try? transaction.payloadValue else {
                logger.error("Transaction did not contain a payment")
                return
            }

            // Finish the transaction
            await payment.finish()

            // Update account data
            await updateAccountData(accountNumber: accountNumber)

        case let .failure(error):
            if !error.isOperationCancellationError {
                logger.error(error: error, message: "Failed to upload transaction")
            }
        }
    }

    private func updateAccountData(accountNumber: String) async {
        logger.debug("Updating account data after successful transaction.")

        let result = await withCheckedContinuation { continuation in
            _ = self.accountsProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .default
            ) { result in
                continuation.resume(returning: result)
            }
        }

        switch result {
        case let .success(accountData):
            logger.info("Successfully updated account data. New expiry: \(accountData.expiry.logFormatted)")

            // Notify delegate about successful account update
            await MainActor.run {
                self.delegate?.update(accountData: accountData)
            }

        case let .failure(error):
            if !error.isOperationCancellationError {
                logger.error(error: error, message: "Failed to update account data.")
            }
        }
    }
}

/// Delegate protocol for StoreKit2TransactionListener.
protocol StoreKit2TransactionListenerDelegate: AnyObject {
    /// Called when the listener needs the current account number.
    func fetchAccountNumber() -> String?

    /// Called when account data has been successfully updated.
    func update(accountData: Account)
}
