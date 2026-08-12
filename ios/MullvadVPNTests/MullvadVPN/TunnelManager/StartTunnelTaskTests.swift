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
import Testing

@testable import MullvadMockData

struct StartTunnelTaskTests {
    // MARK: utility code for setting up tests
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

    @Test
    func testFailsIfNotLoggedIn() async throws {
        let task = StartTunnelTask(interactor: makeInteractor(deviceState: .loggedOut))
        await #expect(throws: InvalidDeviceStateError.self) {
            try await task.start()
        }
    }

    @Test
    func testSetsReconnectIfDisconnecting() async throws {
        let interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnecting(.nothing))
        var tunnelStatus = TunnelStatus()
        interactor.onUpdateTunnelStatus = {
            status in tunnelStatus = status
        }

        let task = StartTunnelTask(interactor: interactor)
        try await task.start()

        #expect(tunnelStatus.state == .disconnecting(.reconnect))
    }

    @Test
    func testStartsTunnelIfDisconnected() async throws {
        let interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnected)

        let task = StartTunnelTask(interactor: interactor)
        try await task.start()

        #expect(interactor.tunnel != nil)
        #expect(interactor.tunnel?.startDate != nil)
    }

    @Test
    func testTaskCanBeCancelled() async throws {
        let interactor = makeInteractor(deviceState: loggedInDeviceState, tunnelState: .disconnected)
        let startTunnelTask = StartTunnelTask(interactor: interactor)

        let task = Task {
            try await startTunnelTask.start()
        }

        task.cancel()

        try await task.value

        #expect(interactor.tunnel == nil)
    }
}
