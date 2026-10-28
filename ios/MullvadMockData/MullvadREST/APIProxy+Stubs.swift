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

public struct APIProxyStubError: Error {
    public init() {}
}

public struct APIProxyStub: APIQuerying {
    var getAddressListResult: Result<[AnyIPEndpoint], Error>
    var getRelaysResult: Result<REST.ServerRelaysCacheResponse?, Error>
    var sendProblemReportResult: Result<Void, Error>
    var initStorekitPaymentResult: Result<UUID, Error>
    var checkStorekitPaymentResult: Result<Void, Error>
    var checkApiAvailabilityResult: Result<Bool, Error>

    public init(
        getAddressListResult: Result<[AnyIPEndpoint], Error> = .failure(APIProxyStubError()),
        getRelaysResult: Result<REST.ServerRelaysCacheResponse?, Error> = .failure(APIProxyStubError()),
        sendProblemReportResult: Result<Void, Error> = .failure(APIProxyStubError()),
        initStorekitPaymentResult: Result<UUID, Error> = .failure(APIProxyStubError()),
        checkStorekitPaymentResult: Result<Void, Error> = .failure(APIProxyStubError()),
        checkApiAvailabilityResult: Result<Bool, Error> = .failure(APIProxyStubError())
    ) {
        self.getAddressListResult = getAddressListResult
        self.getRelaysResult = getRelaysResult
        self.sendProblemReportResult = sendProblemReportResult
        self.initStorekitPaymentResult = initStorekitPaymentResult
        self.checkStorekitPaymentResult = checkStorekitPaymentResult
        self.initStorekitPaymentResult = initStorekitPaymentResult
        self.checkApiAvailabilityResult = checkApiAvailabilityResult
    }

    public func getAddressList(
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<[AnyIPEndpoint]>
    ) -> Cancellable {
        completionHandler(getAddressListResult)
        return AnyCancellable()
    }

    public func getRelays(
        sigsum: (String, Int64)?,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<REST.ServerRelaysCacheResponse?>
    ) -> Cancellable {
        completionHandler(getRelaysResult)
        return AnyCancellable()
    }

    public func sendProblemReport(
        _ body: ProblemReportRequest,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<Void>
    ) -> Cancellable {
        completionHandler(sendProblemReportResult)
        return AnyCancellable()
    }

    public func initStoreKitPayment(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<UUID>
    ) -> any MullvadTypes.Cancellable {
        completionHandler(initStorekitPaymentResult)
        return AnyCancellable()
    }

    public func checkStoreKitPayment(
        transaction: StoreKitTransaction,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<Void>
    ) -> any MullvadTypes.Cancellable {
        completionHandler(checkStorekitPaymentResult)
        return AnyCancellable()
    }

    public func checkApiAvailability(
        retryStrategy: REST.RetryStrategy,
        accessMethod: PersistentAccessMethod,
        completion: @escaping ProxyCompletionHandler<Bool>
    ) -> any MullvadTypes.Cancellable {
        completion(checkApiAvailabilityResult)
        return AnyCancellable()
    }
}
