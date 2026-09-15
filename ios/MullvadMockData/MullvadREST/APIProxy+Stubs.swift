// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadREST
import MullvadRustRuntime
import MullvadTypes

struct APIProxyStubError: Error {}

struct APIProxyStub: APIQuerying {
    var getAddressListResult: Result<[AnyIPEndpoint], Error> = .failure(APIProxyStubError())
    var getRelaysResult: Result<REST.ServerRelaysCacheResponse, Error> = .failure(APIProxyStubError())
    var sendProblemReportResult: Result<Void, Error> = .failure(APIProxyStubError())
    var initStorekitPaymentResult: Result<UUID, Error> = .failure(APIProxyStubError())
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

    func sendProblemReport(
        _ body: ProblemReportRequest,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<Void>
    ) -> Cancellable {
        completionHandler(sendProblemReportResult)
        return AnyCancellable()
    }

    func initStoreKitPayment(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<UUID>
    ) -> any MullvadTypes.Cancellable {
        completionHandler(initStorekitPaymentResult)
        return AnyCancellable()
    }

    func checkStoreKitPayment(
        transaction: StoreKitTransaction,
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
