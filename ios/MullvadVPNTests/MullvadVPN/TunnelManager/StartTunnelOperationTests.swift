//
//  StartTunnelOperationTests.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
import MullvadSettings
import Network
import Operations
import WireGuardKitTypes
import XCTest

class StartTunnelOperationTests: XCTestCase {
    // MARK: utility code for setting up tests

    let testQueue = DispatchQueue(label: "StartTunnelOperationTests.testQueue")
    let operationQueue = AsyncOperationQueue()
    let tunnelSettings = LatestTunnelSettings()

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
        let interactor = MockTunnelInteractor(
            isConfigurationLoaded: true,
            settings: tunnelSettings,
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
            interactor: makeInteractor(deviceState: .loggedOut),
            tunnelSettings: tunnelSettings
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
        let interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnecting(.nothing))
        nonisolated(unsafe) var tunnelStatus = TunnelStatus()
        interactor.onUpdateTunnelStatus = { status in tunnelStatus = status }
        let expectation = expectation(description: "Tunnel status set to reconnect")

        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: interactor,
            tunnelSettings: tunnelSettings
        ) { _ in
            XCTAssertEqual(tunnelStatus.state, .disconnecting(.reconnect))
            expectation.fulfill()
        }
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: .UnitTest.timeout)
    }

    func testStartsTunnelIfDisconnected() {
        let interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnected)
        let expectation = expectation(description: "Make tunnel provider and start tunnel")
        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: interactor,
            tunnelSettings: tunnelSettings
        ) { _ in
            XCTAssertNotNil(interactor.tunnel)
            XCTAssertNotNil(interactor.tunnel?.startDate)
            expectation.fulfill()
        }
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: .UnitTest.timeout)
    }
}
