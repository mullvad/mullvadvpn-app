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
import MullvadTypes

struct APIProxyStubError: Error {}

struct APIProxyStub: APIQuerying {
    var getAddressListResult: Result<[AnyIPEndpoint], Error> = .failure(APIProxyStubError())
    var getRelaysResult: Result<REST.ServerRelaysCacheResponse, Error> = .failure(APIProxyStubError())
    var sendProblemReportResult: Result<Void, Error> = .failure(APIProxyStubError())
    var initStorekitPaymentResult: Result<UUID, Error> = .failure(APIProxyStubError())
    var checkStorekitPaymentResult: Result<Void, Error> = .failure(APIProxyStubError())
    var checkApiAvailabilityResult: Result<Bool, Error> = .failure(APIProxyStubError())
    var submitVoucherResult: Result<REST.SubmitVoucherResponse, Error> = .failure(APIProxyStubError())

    func getAddressList(retryStrategy: REST.RetryStrategy) async -> Result<[AnyIPEndpoint], any Error> {
        getAddressListResult
    }

    func getRelays(etag: String?, retryStrategy: REST.RetryStrategy) async -> Result<
        REST.ServerRelaysCacheResponse, any Error
    > {
        getRelaysResult
    }

    func sendProblemReport(_ body: ProblemReportRequest, retryStrategy: REST.RetryStrategy) async -> Result<
        Void, any Error
    > {
        sendProblemReportResult
    }

    func initStoreKitPayment(accountNumber: String, retryStrategy: REST.RetryStrategy) async -> Result<UUID, any Error>
    {
        initStorekitPaymentResult
    }

    func checkStoreKitPayment(transaction: StoreKitTransaction, retryStrategy: REST.RetryStrategy) async -> Result<
        Void, any Error
    > {
        checkStorekitPaymentResult
    }

    func checkApiAvailability(retryStrategy: REST.RetryStrategy, accessMethod: PersistentAccessMethod) async -> Result<
        Bool, any Error
    > {
        checkApiAvailabilityResult
    }

    func getRelays(
        etag: String?,
        retryStrategy: REST.RetryStrategy,
        completionHandler: @escaping ProxyCompletionHandler<REST.ServerRelaysCacheResponse>
    ) -> Cancellable {
        completionHandler(getRelaysResult)
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
}
