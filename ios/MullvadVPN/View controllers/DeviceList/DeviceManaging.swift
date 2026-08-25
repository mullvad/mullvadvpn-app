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

protocol DeviceManaging: Sendable {
    var currentDeviceId: String? { get }
    func getDevices() async -> Result<[Device], Error>
    func deleteDevice(_ identifier: String) async -> Result<Bool, Error>
}

class DeviceManagementInteractor: DeviceManaging, @unchecked Sendable {
    private let devicesProxy: DeviceHandling
    private let accountNumber: String
    let currentDeviceId: String?

    init(accountNumber: String, currentDeviceId: String? = nil, devicesProxy: DeviceHandling) {
        self.accountNumber = accountNumber
        self.devicesProxy = devicesProxy
        self.currentDeviceId = currentDeviceId
    }

    func getDevices() async -> Result<[Device], any Error> {
        await devicesProxy.getDevices(accountNumber: accountNumber, retryStrategy: .default)
    }

    func deleteDevice(_ identifier: String) async -> Result<Bool, any Error> {
        await devicesProxy.deleteDevice(accountNumber: accountNumber, identifier: identifier, retryStrategy: .default)
    }
}

final class MockDeviceManaging: DeviceManaging {

    let currentDeviceId: String? = "123"

    let getDevicesCompletionHandler: (@Sendable () -> Result<[Device], any Error>)?

    static private let mockDevices = [
        Device(
            id: "123",
            name: "Blind Mole",
            pubkey: WireGuard.PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
        Device(
            id: "456",
            name: "Tall Mole",
            pubkey: WireGuard.PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
        Device(
            id: "543",
            name: "Old Mole",
            pubkey: WireGuard.PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
        Device(
            id: "867",
            name: "Young Mole",
            pubkey: WireGuard.PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
        Device(
            id: "234",
            name: "Rich Mole",
            pubkey: WireGuard.PrivateKey().publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        ),
    ]

    let devicesToReturn: Int

    init(
        devicesToReturn: Int = 5,
        getDevicesCompletionHandler:
            (@Sendable () -> Result<[Device], any Error>)? = {
                .success(mockDevices)
            }
    ) {
        self.devicesToReturn = devicesToReturn
        self.getDevicesCompletionHandler = getDevicesCompletionHandler
    }

    func deleteDevice(
        _ identifier: String
    ) async -> Result<Bool, any Error> {
        do {
            try await Task.sleep(for: .seconds(2))
            try Task.checkCancellation()
            return .success(true)
        } catch {
            return .failure(error)
        }
    }

    func getDevices() async -> Result<[Device], any Error> {
        guard let getDevicesCompletionHandler else {
            return .failure(CancellationError())
        }

        return getDevicesCompletionHandler()
            .map { Array($0.prefix(devicesToReturn)) }
    }
}
