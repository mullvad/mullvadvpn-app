//
//  RedeemVoucherOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 29/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import Operations

class RedeemVoucherOperation: ResultOperation<REST.SubmitVoucherResponse, Error> {
    private let interactor: TunnelInteractor
    private let apiProxy: REST.APIProxy
    private let voucherCode: String

    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        apiProxy: REST.APIProxy,
        voucherCode: String
    ) {
        self.interactor = interactor
        self.apiProxy = apiProxy
        self.voucherCode = voucherCode

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard case let .loggedIn(accountData, _) = interactor.deviceState else {
            finish(completion: .failure(InvalidDeviceStateError()))
            return
        }

        task = apiProxy.submitVoucher(
            voucherCode: voucherCode,
            accountNumber: accountData.number,
            retryStrategy: .default,
            completionHandler: { [weak self] completion in
                self?.dispatchQueue.async {
                    self?.didReceiveVoucherResponse(
                        completion: completion
                    )
                }
            }
        )
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveVoucherResponse(
        completion: OperationCompletion<REST.SubmitVoucherResponse, REST.Error>
    ) {
        let mappedCompletion = completion
            .tryMap { response -> REST.SubmitVoucherResponse in
                switch interactor.deviceState {
                case .loggedIn(var accountData, let deviceData):
                    accountData.expiry = response.newExpiry
                    interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)

                    return response

                default:
                    throw InvalidDeviceStateError()
                }
            }

        finish(completion: mappedCompletion)
    }
}
