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

struct APIProxyStubError: Error {}

struct APIProxyStub: APIQuerying {
    var getAddressListResult: Result<[AnyIPEndpoint], Error> = .failure(APIProxyStubError())
    var getRelaysResult: Result<REST.ServerRelaysCacheResponse, Error> = .failure(APIProxyStubError())
    var sendProblemReportResult: Result<Void, Error> = .failure(APIProxyStubError())
    var submitVoucherResult: Result<REST.SubmitVoucherResponse, Error> = .failure(APIProxyStubError())
    var legacyStorekitPaymentResult: Result<REST.CreateApplePaymentResponse, Error> = .failure(APIProxyStubError())
    var initStorekitPaymentResult: Result<String, Error> = .failure(APIProxyStubError())
    var checkStorekitPaymentResult: Result<Void, Error> = .failure(APIProxyStubError())
    var checkApiAvailabilityResult: Result<Bool, Error> = .failure(APIProxyStubError())

    func getAddressList(
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<[AnyIPEndpoint]>
    ) -> Cancellable {
        completionHandler(getAddressListResult)
        return AnyCancellable()
    }

    func getRelays(
        etag: String?,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
    ) -> Cancellable {
        completionHandler(getRelaysResult)
        return AnyCancellable()
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
        _ body: ProblemReportRequest,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<Void>
    ) -> Cancellable {
        completionHandler(sendProblemReportResult)
        return AnyCancellable()
    }

    func submitVoucher(
        voucherCode: String,
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<REST.SubmitVoucherResponse>
    ) -> Cancellable {
        completionHandler(submitVoucherResult)
        return AnyCancellable()
    }

    func legacyStorekitPayment(
        accountNumber: String,
        request: LegacyStorekitRequest,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<REST.CreateApplePaymentResponse>
    ) -> any Cancellable {
        completionHandler(legacyStorekitPaymentResult)
        return AnyCancellable()
    }

    func initStorekitPayment(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<String>
    ) -> any MullvadTypes.Cancellable {
        completionHandler(initStorekitPaymentResult)
        return AnyCancellable()
    }

    func checkStorekitPayment(
        accountNumber: String,
        transaction: StorekitTransaction,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<Void>
    ) -> any MullvadTypes.Cancellable {
        completionHandler(checkStorekitPaymentResult)
        return AnyCancellable()
    }

    func checkApiAvailability(
        retryStrategy: REST.RetryStrategy,
        accessMethod: PersistentAccessMethod,
        completion: @escaping ProxyCompletionHandler<Bool>
    ) -> any MullvadTypes.Cancellable {
        completion(checkApiAvailabilityResult)
        return AnyCancellable()
    }
}
