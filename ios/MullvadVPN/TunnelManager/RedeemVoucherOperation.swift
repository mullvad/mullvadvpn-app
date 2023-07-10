//
//  RedeemVoucherOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 29/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations

class RedeemVoucherOperation: ResultOperation<REST.SubmitVoucherResponse> {
    private let logger = Logger(label: "RedeemVoucherOperation")
    private let interactor: TunnelInteractor

    private let voucherCode: String
    private let apiProxy: REST.APIProxy
    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        voucherCode: String,
        apiProxy: REST.APIProxy
    ) {
        self.interactor = interactor
        self.voucherCode = voucherCode
        self.apiProxy = apiProxy

        super.init(dispatchQueue: dispatchQueue)
    }

    override func main() {
        guard case let .loggedIn(accountData, _) = interactor.deviceState else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }
        task = apiProxy.submitVoucher(
            voucherCode: voucherCode,
            accountNumber: accountData.number,
            retryStrategy: .default
        ) { result in
            self.dispatchQueue.async {
                self.didReceiveVoucherResponse(result: result)
            }
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveVoucherResponse(result: Result<REST.SubmitVoucherResponse, Error>) {
        let result = result.inspectError { error in
            guard !error.isOperationCancellationError else { return }

            self.logger.error(
                error: error,
                message: "Failed to redeem voucher."
            )
        }.tryMap { voucherResponse in
            switch interactor.deviceState {
            case .loggedIn(var storedAccountData, let storedDeviceData):
                storedAccountData.expiry = voucherResponse.newExpiry

                // flip the value to `false` when adding credit is successful
                storedAccountData.isNew = false

                let newDeviceState = DeviceState.loggedIn(storedAccountData, storedDeviceData)

                interactor.setDeviceState(newDeviceState, persist: true)

                return voucherResponse

            default:
                throw InvalidDeviceStateError()
            }
        }

        finish(result: result)
    }
}
