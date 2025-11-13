//
//  APIProxy+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import WireGuardKitTypes

struct APIProxyStub: APIQuerying {
    func getAddressList(
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<[AnyIPEndpoint]>
    ) -> Cancellable {
        AnyCancellable()
    }

    func getRelays(
        etag: String?,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
    ) -> Cancellable {
        AnyCancellable()
    }

    func sendProblemReport(
        _ body: ProblemReportRequest,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<Void>
    ) -> Cancellable {
        AnyCancellable()
    }

    func submitVoucher(
        voucherCode: String,
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<REST.SubmitVoucherResponse>
    ) -> Cancellable {
        AnyCancellable()
    }

    func legacyStoreKitPayment(
        accountNumber: String,
        request: LegacyStoreKitRequest,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<REST.CreateApplePaymentResponse>
    ) -> any Cancellable {
        AnyCancellable()
    }

    func initStoreKitPayment(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<UUID>
    ) -> any MullvadTypes.Cancellable {
        AnyCancellable()
    }

    func checkStoreKitPayment(
        transaction: StoreKitTransaction,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<Void>
    ) -> any MullvadTypes.Cancellable {
        AnyCancellable()
    }

    func checkApiAvailability(
        retryStrategy: REST.RetryStrategy,
        accessMethod: PersistentAccessMethod,
        completion: @escaping ProxyCompletionHandler<Bool>
    ) -> any MullvadTypes.Cancellable {
        AnyCancellable()
    }
}
