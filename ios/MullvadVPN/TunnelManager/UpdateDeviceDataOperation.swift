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
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations

class UpdateDeviceDataOperation: ResultOperation<StoredDeviceData>, @unchecked Sendable {
    private let devicesProxy: DeviceHandling
    private let deviceState: @Sendable () -> DeviceState
    private let onUpdateAccount: @Sendable (DeviceState) -> Void

    private var task: Cancellable?

    init(
        dispatchQueue: DispatchQueue,
        devicesProxy: DeviceHandling,
        deviceState: @escaping @Sendable () -> DeviceState,
        onUpdateAccount: @escaping @Sendable (DeviceState) -> Void
    ) {
        self.devicesProxy = devicesProxy
        self.deviceState = deviceState
        self.onUpdateAccount = onUpdateAccount
        super.init(dispatchQueue: dispatchQueue, completionQueue: nil, completionHandler: nil)
    }

    override func main() {
        guard case let .loggedIn(accountData, deviceData) = deviceState() else {
            finish(result: .failure(InvalidDeviceStateError()))
            return
        }

        task = devicesProxy.getDevice(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            retryStrategy: .default,
            completion: { [weak self] result in
                self?.dispatchQueue.async { [weak self] in
                    self?.didReceiveDeviceResponse(result: result)
                }
            }
        )
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }

    private func didReceiveDeviceResponse(result: Result<Device, Error>) {
        let result = result.tryMap { device -> StoredDeviceData in
            switch deviceState() {
            case .loggedIn(let storedAccount, var storedDevice):
                storedDevice.update(from: device)
                let newDeviceState = DeviceState.loggedIn(storedAccount, storedDevice)
                onUpdateAccount(newDeviceState)

                return storedDevice

            default:
                throw InvalidDeviceStateError()
            }
        }
        finish(result: result)
    }
}
