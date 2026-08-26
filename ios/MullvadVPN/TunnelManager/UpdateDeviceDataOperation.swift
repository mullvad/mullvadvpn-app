//// This Source Code Form is subject to the terms of the GPLv3 License.
//// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
////
//// This file incorporates work covered by the following copyright and
//// permission notice:
////
////   Copyright (c) Mullvad VPN AB. All rights reserved.
////
//// SPDX-License-Identifier: GPL-3.0-only
//
//import Foundation
//import MullvadLogging
//import MullvadREST
//import MullvadSettings
//import MullvadTypes
//import Operations
//
////class UpdateDeviceDataOperation: ResultOperation<StoredDeviceData>, @unchecked Sendable {
////    private let interactor: TunnelInteractor
////    private let devicesProxy: DeviceHandling
////
////    private var task: Cancellable?
////
////    init(
////        dispatchQueue: DispatchQueue,
////        interactor: TunnelInteractor,
////        devicesProxy: DeviceHandling
////    ) {
////        self.interactor = interactor
////        self.devicesProxy = devicesProxy
////
////        super.init(dispatchQueue: dispatchQueue)
////    }
////
////    override func main() {
//        guard case let .loggedIn(accountData, deviceData) = interactor.deviceState else {
//            finish(result: .failure(InvalidDeviceStateError()))
//            return
//        }
////
////        task = devicesProxy.getDevice(
////            accountNumber: accountData.number,
////            identifier: deviceData.identifier,
////            retryStrategy: .default,
////            completion: { [weak self] result in
////                self?.dispatchQueue.async { [weak self] in
////                    self?.didReceiveDeviceResponse(result: result)
////                }
////            }
////        )
////    }
////
////    override func operationDidCancel() {
////        task?.cancel()
////        task = nil
////    }
////
////    private func didReceiveDeviceResponse(result: Result<Device, Error>) {
////        let result = result.tryMap { device -> StoredDeviceData in
////            switch interactor.deviceState {
////            case .loggedIn(let storedAccount, var storedDevice):
////                storedDevice.update(from: device)
////                let newDeviceState = DeviceState.loggedIn(storedAccount, storedDevice)
////                interactor.setDeviceState(newDeviceState, persist: true)
////
////                return storedDevice
////
////            default:
////                throw InvalidDeviceStateError()
////            }
////        }
////
////        if let error = result.error {
////            interactor.handleRestError(error)
////        }
////
////        finish(result: result)
////    }
////}
