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
import MullvadREST
import MullvadSettings
import MullvadTypes
import Testing

struct LoadTunnelConfigurationTests {
    let task: LoadTunnelConfigurationTask
    let tunnelInteractor: MockTunnelInteractor
    let settingsManager: SettingsManager
    static let tunnel = MockTunnel(
        tunnelProvider: SimulatorTunnelProviderManager(),
        backgroundTaskProvider: UIApplicationStub()
    )
    static let loggedInDevice = Device.loggedInDeviceState
    static let loggedOutDevice = DeviceState.loggedOut

    init() async throws {
        tunnelInteractor = MockTunnelInteractor(
            isConfigurationLoaded: false, settings: LatestTunnelSettings(), deviceState: .revoked)
        settingsManager = SettingsManager(store: InMemorySettingsStore<SettingNotFound>())
        task = LoadTunnelConfigurationTask(interactor: tunnelInteractor, settingsManager: settingsManager)
    }

    @Test(
        """
        Tests the following configuration for a LoadTunnelConfigurationTask
        - Happy path
        - No settings and logged out path which sets the LatestTunnelSettings
        - No device state which clears the tunnel regardless of tunnel state
        - No settings (or KeyChain doesn't find settings) sets LatestTunnelSettings and stays logged in
        """,
        arguments: [
            (
                TestScenario(
                    settings: LatestTunnelSettings(),
                    deviceState: loggedInDevice,
                    tunnel: tunnel),
                TestScenario(
                    settings: LatestTunnelSettings(),
                    deviceState: loggedInDevice,
                    tunnel: tunnel)
            ),
            (
                TestScenario(
                    settings: nil,
                    deviceState: loggedOutDevice,
                    tunnel: nil),
                TestScenario(
                    settings: LatestTunnelSettings(),
                    deviceState: loggedOutDevice,
                    tunnel: nil)
            ),
            (
                TestScenario(
                    settings: nil,
                    deviceState: nil,
                    tunnel: tunnel),
                TestScenario(
                    settings: LatestTunnelSettings(),
                    deviceState: loggedOutDevice,
                    tunnel: nil)
            ),
            (
                TestScenario(
                    settings: nil,
                    deviceState: loggedInDevice,
                    tunnel: tunnel),
                TestScenario(
                    settings: LatestTunnelSettings(),
                    deviceState: loggedInDevice,
                    tunnel: tunnel)
            ),
        ]
    )
    fileprivate func runLoadTunnelConfigurationTask(args: (input: TestScenario, expectedOutcome: TestScenario))
        async throws
    {
        let input = args.input
        let expectedOutcome = args.expectedOutcome

        tunnelInteractor.onSetSettings = { newSettings, shouldPersistSettings in
            #expect(newSettings == expectedOutcome.settings)
            #expect(shouldPersistSettings == false)
        }
        tunnelInteractor.onSetDeviceState = { newDeviceState, shouldPersistDevice in
            #expect(newDeviceState == expectedOutcome.deviceState)
            #expect(shouldPersistDevice == false)
        }

        tunnelInteractor.onSetTunnel = { newTunnel, shouldRefreshTunnelState in
            if expectedOutcome.tunnel == nil {
                #expect(newTunnel == nil)
            } else {
                /// `TunnelProtocol` is not equatable, use the `startDate` to determine whether it's the same tunnel
                let capturedStartDate = await newTunnel?.startDate
                let expectedStartDate = await expectedOutcome.tunnel?.startDate
                #expect(capturedStartDate == expectedStartDate)
            }

            #expect(shouldRefreshTunnelState == true)
        }

        if let settings = input.settings {
            try settingsManager.writeSettings(settings)
        } else {
            try settingsManager.store.delete(key: .settings)
        }

        if let deviceState = input.deviceState {
            try settingsManager.writeDeviceState(deviceState)
        } else {
            try settingsManager.store.delete(key: .deviceState)
        }
        tunnelInteractor.tunnel = input.tunnel

        await task.start()
        #expect(tunnelInteractor.isConfigurationLoaded == true)
    }
}

private struct TestScenario {
    let settings: LatestTunnelSettings?
    let deviceState: DeviceState?
    let tunnel: (any TunnelProtocol)?
}
