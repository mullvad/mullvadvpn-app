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

    func initPayment(accountNumber: String) async -> Result<UUID, Error> {
        await withCheckedContinuation { continuation in
            _ = apiProxy.initStorekitPayment(
                accountNumber: accountNumber,
                retryStrategy: .noRetry,
                completionHandler: { result in
                    continuation.resume(returning: result)
                }
            )
        }
    }

    func checkPayment(accountNumber: String, jwsRepresentation: String) async -> Result<Void, Error> {
        await withCheckedContinuation { continuation in
            _ = apiProxy.checkStorekitPayment(
                accountNumber: accountNumber,
                transaction: StorekitTransaction(transaction: jwsRepresentation),
                retryStrategy: .noRetry,
                completionHandler: { result in
                    continuation.resume(returning: result)
                }
            )
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
}
