//
//  StartTunnelTaskTests.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Network
import XCTest

@testable import MullvadMockData

class StartTunnelTaskTests: XCTestCase {
    // MARK: utility code for setting up tests

    let testQueue = DispatchQueue(label: "StartTunnelTaskTests.testQueue")

    let loggedInDeviceState = DeviceState.loggedIn(
        StoredAccountData(
            identifier: "",
            number: "",
            expiry: .distantFuture
        ),
        StoredDeviceData(
            creationDate: Date(),
            identifier: "",
            name: "",
            hijackDNS: false,
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!,
            wgKeyData: StoredWgKeyData(creationDate: Date(), privateKey: WireGuard.PrivateKey())
        )
    )

    func makeInteractor(deviceState: DeviceState, tunnelState: TunnelState? = nil) -> MockTunnelInteractor {
        let interactor = MockTunnelInteractor(
            isConfigurationLoaded: true,
            settings: LatestTunnelSettings(),
            deviceState: deviceState
        )
        if let tunnelState {
            interactor.tunnelStatus = TunnelStatus(state: tunnelState)
        }
        return interactor
    }

    // MARK: the tests

    func testFailsIfNotLoggedIn() async throws {
        let task = StartTunnelTask(interactor: makeInteractor(deviceState: .loggedOut))
        let result = await task.start()

        switch result {
        case .success:
            XCTFail("Task returned \(result), not failure")
        case .failure:
            XCTAssertTrue(true)
        }
    }

    func testSetsReconnectIfDisconnecting() async {
        let interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnecting(.nothing))

        nonisolated(unsafe) var tunnelStatus = TunnelStatus()
        interactor.onUpdateTunnelStatus = {
            status in tunnelStatus = status
        }

        let task = StartTunnelTask(interactor: interactor)
        _ = await task.start()

        XCTAssertEqual(tunnelStatus.state, .disconnecting(.reconnect))
    }

    func testStartsTunnelIfDisconnected() async {
        let interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnected)

        let task = StartTunnelTask(interactor: interactor)
        _ = await task.start()

        XCTAssertNotNil(interactor.tunnel)
        XCTAssertNotNil(interactor.tunnel?.startDate)
    }
}
