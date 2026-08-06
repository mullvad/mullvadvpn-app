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

        let runner = Task {
            let result = await task.start()

            if case .failure(let error) = result {
                #expect(error.isTaskCancellationError, "Expected TaskError.cancelled")
            } else {
                Issue.record("Expected TaskError.cancelled")
            }
        }

        runner.cancel()
        await runner.value
    }

    @Test func cancellingAccountDataUpdateDoesNotUpdateDeviceState() async throws {
        let interactor = makeInteractor(deviceState: Device.loggedInDeviceState)
        let oldExpiry = interactor.deviceState.accountData?.expiry

        let task = UpdateAccountDataTask(
            interactor: interactor,
            accountsProxy: AccountsProxyStub()
        )

        let runner = Task {
            let result = await task.start()

            let newExpiry = interactor.deviceState.accountData?.expiry
            #expect(oldExpiry == newExpiry, "Expected account expiry to not change")
        }

        task.cancel()
        await runner.value
    }

    @Test func canCancelParentTask() async throws {
        let task = UpdateAccountDataTask(
            interactor: makeInteractor(deviceState: Device.loggedInDeviceState),
            accountsProxy: AccountsProxyStub()
        )

        let runner = Task {
            let result = await task.start()

            if case .failure(let error) = result {
                #expect(error.isTaskCancellationError, "Expected TaskError.cancelled")
            } else {
                Issue.record("Expected TaskError.cancelled")
            }
        }

        runner.cancel()
        await runner.value
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
