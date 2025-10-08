//
//  StorePaymentManagerInteractor.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import StoreKit

final class StorePaymentManagerInteractor {
    private let tunnelManager: TunnelManager
    private(set) var apiProxy: APIQuerying
    private(set) var accountProxy: RESTAccountHandling

    var accountNumber: String? {
        tunnelManager.deviceState.accountData?.number
    }

    init(tunnelManager: TunnelManager, apiProxy: APIQuerying, accountProxy: RESTAccountHandling) {
        self.tunnelManager = tunnelManager
        self.apiProxy = apiProxy
        self.accountProxy = accountProxy
    }

    // MARK: Tunnel manager

    func updateAccountData(for account: Account) {
        guard case .loggedIn(var storedAccountData, let deviceData) = tunnelManager.deviceState else {
            return
        }

        storedAccountData.expiry = account.expiry
        let newDeviceState = DeviceState.loggedIn(storedAccountData, deviceData)

        tunnelManager.setDeviceState(newDeviceState, persist: true)
    }

    // MARK: API proxy

    func initPayment() async -> Result<UUID, Error> {
        guard let accountNumber = accountNumber else {
            return .failure(NSError(domain: "User is not logged in", code: 0))
        }

        return await withCheckedContinuation { continuation in
            _ = apiProxy.initStorekitPayment(
                accountNumber: accountNumber,
                retryStrategy: .noRetry,
            ) { result in
                continuation.resume(returning: result)
            }
        }
    }

    func checkPayment(jwsRepresentation: String) async -> Result<Void, Error> {
        await withCheckedContinuation { continuation in
            _ = apiProxy.checkStorekitPayment(
                transaction: StorekitTransaction(transaction: jwsRepresentation),
                retryStrategy: .noRetry,
            ) { result in
                continuation.resume(returning: result)
            }
        }
    }

    func legacySendReceipt() async -> Result<Void, Error> {
        guard let accountNumber = accountNumber else {
            return .failure(NSError(domain: "User is not logged in", code: 0))
        }

        let receiptData: Data
        do {
            receiptData = try readReceiptFromDisk()
        } catch {
            return .failure(error)
        }

        return await withCheckedContinuation { continuation in
            _ = apiProxy.legacyStorekitPayment(
                accountNumber: accountNumber,
                request: LegacyStorekitRequest(receiptString: receiptData),
                retryStrategy: .default,
            ) { result in
                switch result {
                case .success:
                    continuation.resume(returning: .success(()))
                case let .failure(error):
                    continuation.resume(returning: .failure(error))
                }
            }
        }
    }

    // MARK: Account proxy

    func getAccountData(accountNumber: String) async -> Result<Account, Error> {
        await withCheckedContinuation { continuation in
            _ = self.accountProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .default
            ) { result in
                continuation.resume(returning: result)
            }
        }
    }

    // MARK: Private functions

    private func readReceiptFromDisk() throws -> Data {
        guard let appStoreReceiptURL = Bundle.main.appStoreReceiptURL else {
            throw StoreReceiptNotFound()
        }

        do {
            return try Data(contentsOf: appStoreReceiptURL)
        } catch let error as CocoaError
            where error.code == .fileReadNoSuchFile || error.code == .fileNoSuchFile
        {
            throw StoreReceiptNotFound()
        } catch {
            throw error
        }
    }
}
