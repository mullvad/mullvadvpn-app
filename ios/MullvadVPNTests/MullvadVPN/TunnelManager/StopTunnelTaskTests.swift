//
//  StopTunnelTaskTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-11.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadREST
import MullvadSettings
import MullvadTypes
import Testing

struct StopTunnelTaskTests {

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

    @Test
    func testIgnoreIfNotLoggedIn() async throws {
        let stopTunnelTask = StopTunnelTask(interactor: makeInteractor(deviceState: .loggedOut))
        await #expect(throws: Never.self) {
            try await stopTunnelTask.start()
        }
    }

    @Test
    func testTaskCanBeCancelled() async throws {
        let selectedRelays = try RelaySelectorStub.nonFallible().selectRelays(
            tunnelSettings: LatestTunnelSettings(), connectionAttemptCount: 0)

        let stopTunnelTask = StopTunnelTask(
            interactor: makeInteractor(
                deviceState: loggedInDeviceState,
                tunnelState: .connected(selectedRelays, isPostQuantum: false, isDaita: false)))

        let task = Task {
            try await stopTunnelTask.start()
        }

        task.cancel()

        let error = await #expect(throws: Error.self) {
            try await task.value
        }

        #expect(error is CancellationError || error is UnsetTunnelError)
    }

    @Test
    func testStopTunnelIfConnected() async throws {
        let selectedRelays = try RelaySelectorStub.nonFallible().selectRelays(
            tunnelSettings: LatestTunnelSettings(), connectionAttemptCount: 0)

        let stopTunnelTask = StopTunnelTask(
            interactor: makeInteractor(
                MockTunnel(
                    tunnelProvider: SimulatorTunnelProviderManager(),
                    backgroundTaskProvider: UIApplicationStub()
                ),
                deviceState: loggedInDeviceState,
                tunnelState: .connected(selectedRelays, isPostQuantum: false, isDaita: false)))

        await #expect(throws: Never.self) {
            try await stopTunnelTask.start()
        }
    }

    private func makeInteractor(
        _ tunnel: (any TunnelProtocol)? = nil, deviceState: DeviceState, tunnelState: TunnelState? = nil
    ) -> MockTunnelInteractor {
        let interactor = MockTunnelInteractor(
            isConfigurationLoaded: true,
            settings: LatestTunnelSettings(),
            deviceState: deviceState,
            tunnel: tunnel
        )
        if let tunnelState {
            interactor.tunnelStatus = TunnelStatus(state: tunnelState)
        }
        return interactor
    }
}
