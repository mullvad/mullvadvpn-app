// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadMockData
import MullvadSettings
import MullvadTypes
import Network
import Operations
import XCTest

@testable import MullvadMockData

class StartTunnelOperationTests: XCTestCase {
    // MARK: utility code for setting up tests

    let testQueue = DispatchQueue(label: "StartTunnelOperationTests.testQueue")
    let operationQueue = AsyncOperationQueue()

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

    func testFailsIfNotLoggedIn() throws {
        let expectation = expectation(description: "Start tunnel operation failed")
        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: makeInteractor(deviceState: .loggedOut)
        ) { result in
            guard case .failure = result else {
                XCTFail("Operation returned \(result), not failure")
                return
            }
            expectation.fulfill()
        }

        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: .UnitTest.timeout)
    }

    func testSetsReconnectIfDisconnecting() {
        let interactor = makeInteractor(deviceState: Device.loggedInDeviceState, tunnelState: .disconnecting(.nothing))
        nonisolated(unsafe) var tunnelStatus = TunnelStatus()
        interactor.onUpdateTunnelStatus = { status in tunnelStatus = status }
        let expectation = expectation(description: "Tunnel status set to reconnect")

        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: interactor
        ) { _ in
            XCTAssertEqual(tunnelStatus.state, .disconnecting(.reconnect))
            expectation.fulfill()
        }
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: .UnitTest.timeout)
    }

    func testStartsTunnelIfDisconnected() {
        let interactor = makeInteractor(deviceState: Device.loggedInDeviceState, tunnelState: .disconnected)
        let expectation = expectation(description: "Make tunnel provider and start tunnel")
        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: interactor
        ) { _ in
            XCTAssertNotNil(interactor.tunnel)
            XCTAssertNotNil(interactor.tunnel?.startDate)
            expectation.fulfill()
        }
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: .UnitTest.timeout)
    }
}
