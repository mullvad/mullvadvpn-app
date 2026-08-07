//
//  UpdateAccountDataTaskTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2026-08-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Testing
@testable import MullvadMockData

struct UpdateAccountDataTaskTests {
    @Test func canDoAccountDataUpdate() async throws {
        let task = UpdateAccountDataTask(
            interactor: makeInteractor(deviceState: Device.loggedInDeviceState),
            accountsProxy: AccountsProxyStub()
        )

        let result = await task.start()

        #expect(result.isSuccess)
    }

    @Test func canCancelAccountDataUpdate() async throws {
        let task = UpdateAccountDataTask(
            interactor: makeInteractor(deviceState: Device.loggedInDeviceState),
            accountsProxy: AccountsProxyStub()
        )

        Task {
            let result = await task.start()
            result.inspectError({ error in
                #expect(error.isOperationCancellationError)
            })
        }

        await task.cancel()
    }
}

extension UpdateAccountDataTaskTests {
    private func makeInteractor(deviceState: DeviceState, tunnelState: TunnelState? = nil) -> MockTunnelInteractor {
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
}
