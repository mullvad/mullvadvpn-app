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

    nonisolated(unsafe) var didReceiveDeviceState: (@Sendable (DeviceState) -> Void)?

    nonisolated(unsafe) private var tunnelObserver: TunnelObserver?

    init(
        tunnelManager: TunnelManager,
        accountsProxy: RESTAccountHandling,
        apiProxy: APIQuerying
    ) {
        self.tunnelManager = tunnelManager
        self.accountsProxy = accountsProxy
        self.apiProxy = apiProxy

        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, _ in
                self?.didReceiveDeviceState?(deviceState)
            })

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    func logout() async {
        await tunnelManager.unsetAccount()
    }

    // This function is for testing only
    func getPaymentToken(for accountNumber: String) async -> Result<String, Error> {
        await withCheckedContinuation { continuation in
            let proxy = REST.MullvadAPIProxy(
                transportProvider: APITransportProvider(
                    requestFactory: .init(
                        apiContext: REST.apiContext,
                        encoder: REST.Coding.makeJSONEncoder()
                    )
                ),
                dispatchQueue: .main,
                responseDecoder: REST.Coding.makeJSONDecoder()
            )
            _ = proxy
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
            let proxy = REST.MullvadAPIProxy(
                transportProvider: APITransportProvider(
                    requestFactory: .init(
                        apiContext: REST.apiContext,
                        encoder: REST.Coding.makeJSONEncoder()
                    )
                ),
                dispatchQueue: .main,
                responseDecoder: REST.Coding.makeJSONDecoder()
            )
            _ = proxy
                .checkStorekitPayment(
                    accountNumber: accountNumber,
                    transaction: StorekitTransaction(transaction: transaction.jwsRepresentation),
                    retryStrategy: .noRetry,
                    completionHandler: { result in
                        c.resume(returning: result)
                    }
                )
        }
    }
}
