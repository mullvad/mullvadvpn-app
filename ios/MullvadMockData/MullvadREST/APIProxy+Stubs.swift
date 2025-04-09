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

    func createApplePayment(
        accountNumber: String,
        receiptString: Data
    ) -> any RESTRequestExecutor<REST.CreateApplePaymentResponse> {
        RESTRequestExecutorStub<REST.CreateApplePaymentResponse>(success: {
            .timeAdded(42, .distantFuture)
        })
    }

    func sendProblemReport(
        _ body: REST.ProblemReportRequest,
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

    func initStorekitPayment(
        accountNumber: String,
        retryStrategy: MullvadREST.REST.RetryStrategy,
        completionHandler: @escaping MullvadREST.ProxyCompletionHandler<String>
    ) -> any MullvadTypes.Cancellable {
        AnyCancellable()
    }

    func checkStorekitPayment(
        accountNumber: String,
        transaction: MullvadTypes.StorekitTransaction,
        retryStrategy: MullvadREST.REST.RetryStrategy,
        completionHandler: @escaping MullvadREST.ProxyCompletionHandler<Void>
    ) -> any MullvadTypes.Cancellable {
        AnyCancellable()
    }
}
