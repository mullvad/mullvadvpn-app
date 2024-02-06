//
//  StartTunnelOperationTests.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-02.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import Network
import Operations
import WireGuardKitTypes
import XCTest

class StartTunnelOperationTests: XCTestCase {
    // MARK: utility code for setting up tests

    let testQueue = DispatchQueue(label: "StartTunnelOperationTests.testQueue")
    let operationQueue = AsyncOperationQueue()

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
            wgKeyData: StoredWgKeyData(creationDate: Date(), privateKey: PrivateKey())
        )
    )

    func makeInteractor(deviceState: DeviceState, tunnelState: TunnelState? = nil) -> MockTunnelInteractor {
        var interactor = MockTunnelInteractor(
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
        let settings = LatestTunnelSettings()
        let exp = expectation(description: "Start tunnel operation failed")
        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: makeInteractor(deviceState: .loggedOut)
        ) { result in
            guard case .failure = result else {
                XCTFail("Operation returned \(result), not failure")
                return
            }
            exp.fulfill()
        }

        operationQueue.addOperation(operation)
        wait(for: [exp], timeout: 1.0)
    }

    func testSetsReconnectIfDisconnecting() {
        let settings = LatestTunnelSettings()
        var interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnecting(.nothing))
        var tunnelStatus = TunnelStatus()
        interactor.onUpdateTunnelStatus = { status in tunnelStatus = status }
        let expectation = expectation(description: "Tunnel status set to reconnect")

        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: interactor
        ) { result in
            XCTAssertEqual(tunnelStatus.state, .disconnecting(.reconnect))
            expectation.fulfill()
        }
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: 1.0)
    }

    func testStartsTunnelIfDisconnected() {
        let settings = LatestTunnelSettings()
        var interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnected)
        let expectation = expectation(description: "Make tunnel provider and start tunnel")
        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: interactor
        ) { result in
            XCTAssertNotNil(interactor.tunnel)
            XCTAssertNotNil(interactor.tunnel?.startDate)
            expectation.fulfill()
        }
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: 1.0)
    }
}
