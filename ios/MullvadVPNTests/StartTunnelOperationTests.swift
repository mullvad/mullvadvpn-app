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
import PacketTunnelCore

struct MockTunnelInteractor: TunnelInteractor {
    var tunnel: (TunnelProtocol)?
    
    func getPersistentTunnels() -> [TunnelProtocol] {
        return []
    }
    
    func createNewTunnel() -> TunnelProtocol {
        fatalError()
    }
    
    func setTunnel(_ tunnel: (TunnelProtocol)?, shouldRefreshTunnelState: Bool) {
        
    }
    
    var tunnelStatus: TunnelStatus {
        TunnelStatus()
    }
    
    func updateTunnelStatus(_ block: (inout TunnelStatus) -> Void) -> TunnelStatus {
        TunnelStatus()
    }
    
    var isConfigurationLoaded: Bool
    
    var settings: MullvadSettings.LatestTunnelSettings
    
    var deviceState: MullvadSettings.DeviceState
    
    func setConfigurationLoaded() {
    }
    
    func setSettings(_ settings: MullvadSettings.LatestTunnelSettings, persist: Bool) {
    }
    
    func setDeviceState(_ deviceState: MullvadSettings.DeviceState, persist: Bool) {
    }
    
    func removeLastUsedAccount() {
    }
    
    func handleRestError(_ error: Error) {
    }
    
    func startTunnel() {
    }
    
    func prepareForVPNConfigurationDeletion() {
    }
    
    func selectRelay() throws -> PacketTunnelCore.SelectedRelay {
        fatalError()
    }
    
    
}

final class StartTunnelOperationTests: XCTestCase {

    override func setUpWithError() throws {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDownWithError() throws {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    func testFailsIfNotLoggedIn() throws {
        let operationQueue = AsyncOperationQueue()
        let testQueue = DispatchQueue(label: "StartTunnelOperationTests.testQueue")
        let settings = LatestTunnelSettings()
        let expectation = XCTestExpectation(description:"")
        let operation = StartTunnelOperation(
            dispatchQueue: testQueue,
            interactor: MockTunnelInteractor(isConfigurationLoaded: true, settings: settings, deviceState: .loggedOut)) { result in
                
                guard case let .failure(err) = result else {
                    XCTFail("Operation returned \(result), not failure")
                    return
                }
                
                expectation.fulfill()
                
            }
        
        operationQueue.addOperation(operation)
        wait(for: [expectation], timeout: 10.0)
    }

}
