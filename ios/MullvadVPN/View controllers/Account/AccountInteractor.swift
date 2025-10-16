//
//  AccountInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import StoreKit

final class AccountInteractor: Sendable {
    let tunnelManager: TunnelManager
    let accountsProxy: RESTAccountHandling
    let apiProxy: APIQuerying
    let deviceProxy: DeviceHandling

    nonisolated(unsafe) var didReceiveTunnelState: (() -> Void)?
    nonisolated(unsafe) var didReceiveDeviceState: (@Sendable (DeviceState) -> Void)?

    nonisolated(unsafe) private var tunnelObserver: TunnelObserver?

    init(
        tunnelManager: TunnelManager,
        accountsProxy: RESTAccountHandling,
        apiProxy: APIQuerying,
        deviceProxy: DeviceHandling
    ) {
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy
        self.apiProxy = apiProxy
        self.deviceProxy = deviceProxy

        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, _ in
                    self?.didReceiveTunnelState?()
                },
                didUpdateDeviceState: { [weak self] _, deviceState, _ in
                    self?.didReceiveDeviceState?(deviceState)
                }
            )

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }

    var tunnelState: TunnelState {
        tunnelManager.tunnelStatus.state
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    func logout() async {
        await tunnelManager.unsetAccount()
    }

    // This function is for testing only
    func getPaymentToken(for accountNumber: String) async -> Result<UUID, Error> {
        await withCheckedContinuation { continuation in
            _ =
                apiProxy
                .initStorekitPayment(
                    accountNumber: accountNumber,
                    retryStrategy: .noRetry,
                    completionHandler: { result in
                        continuation.resume(returning: result)
                    }
                )
        }
    }

    // This function is for testing only
    func sendStoreKitReceipt(
        _ transaction: VerificationResult<Transaction>,
        for accountNumber: String
    ) async -> Result<Void, Error> {
        await withCheckedContinuation { c in
            _ =
                apiProxy
                .checkStorekitPayment(
                    accountNumber: accountNumber,
                    transaction: StorekitTransaction(transaction: transaction.jwsRepresentation),
                    retryStrategy: .noRetry,
                    completionHandler: { result in
                        c.resume(returning: result.map( {_ in } ))
                    }
                )
        }
    }
}
