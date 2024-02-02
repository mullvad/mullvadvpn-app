//
//  StartTunnelOperationTests.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-02.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest
import MullvadSettings
import Operations
import Network
import WireGuardKitTypes


final class StartTunnelOperationTests: XCTestCase {
    
    let testQueue = DispatchQueue(label: "StartTunnelOperationTests.testQueue")

    func testFailsIfNotLoggedIn() throws {
        let operationQueue = AsyncOperationQueue()
        let settings = LatestTunnelSettings()
        let expectation = XCTestExpectation(description:"")
        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: MockTunnelInteractor(isConfigurationLoaded: true, settings: settings, deviceState: .loggedOut)) { result in
                
                guard case .failure(_) = result else {
                    XCTFail("Operation returned \(result), not failure")
                    return
                }
                expectation.fulfill()
            }
        
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: 10.0)
    }
    
    func testSetsReconnectIfDisconnecting() {
        let operationQueue = AsyncOperationQueue()
        let settings = LatestTunnelSettings()
        var interactor = MockTunnelInteractor(
            isConfigurationLoaded: true,
            settings: settings,
            deviceState: .loggedIn(
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
        )
        var tunnelStatus = TunnelStatus()
        tunnelStatus.state = .disconnecting(.nothing)
        interactor.tunnelStatus = tunnelStatus
        interactor.onUpdateTunnelStatus = { status in tunnelStatus = status }
        let expectation = XCTestExpectation(description:"")

        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: interactor) { result in
                XCTAssertEqual(tunnelStatus.state, .disconnecting(.reconnect))
                expectation.fulfill()
            }
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: 10.0)
    }
}
