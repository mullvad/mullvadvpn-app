//
//  TunnelJobQueue.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes

protocol TunnelJobQueueInteractor: Sendable {
    var tunnel: (any TunnelProtocol)? { get }
    var deviceState: DeviceState { get async }

    func setDeviceState(_ deviceState: DeviceState, persist: Bool) async
    func setLastUsedAccount(_ accountNumber: String) async
    func setTunnelStatus(_ block: @Sendable (inout TunnelStatus) -> Void) async
    func unsetTunnelConfiguration(isRemovingProfile: Bool) async
    func removeLastUsedAccount() async
    func notifyKeyRotation() async throws
    func startPollingTunnelStatus()

    @MainActor
    func didReconnectTunnel(error: Error?)
}

actor TunnelJobQueue {
    private let interactor: any TunnelJobQueueInteractor
    private let backgroundTaskProvider: BackgroundTaskProviding
    private let exclusivityCoordinator = AsyncExclusivityCoordinator()
    private let logger = Logger(label: "TunnelJobQueue")

    private let accountsProxy: RESTAccountHandling
    private let devicesProxy: DeviceHandling
    private let apiProxy: APIQuerying

    init(
        interactor: any TunnelJobQueueInteractor,
        backgroundTaskProvider: BackgroundTaskProviding,
        accountsProxy: RESTAccountHandling,
        devicesProxy: DeviceHandling,
        apiProxy: APIQuerying
    ) {
        self.interactor = interactor
        self.backgroundTaskProvider = backgroundTaskProvider
        self.accountsProxy = accountsProxy
        self.devicesProxy = devicesProxy
        self.apiProxy = apiProxy
    }

    func reconnect(selectNewRelay: Bool) async throws {
        try await exclusivityCoordinator.withExclusiveAccess(categories: [.manageTunnel]) {
            try await performReconnect(
                selectNewRelay: selectNewRelay
            )
        }
    }

    func setNewAccount() async throws -> StoredAccountData {
        try await setAccount(action: .new)!
    }

    func setExistingAccount(accountNumber: String) async throws -> StoredAccountData {
        try await setAccount(action: .existing(accountNumber))!
    }

    func unsetAccount(isRemovingProfile: Bool = true) async throws {
        _ = try await setAccount(action: .unset(isRemovingProfile: isRemovingProfile))
    }

    public func deleteAccount(accountNumber: String) async throws {
        _ = try await setAccount(action: .delete(accountNumber))
        await interactor.removeLastUsedAccount()
    }

    public func updateAccountData() async throws {
        let deviceState = await interactor.deviceState
        guard case .loggedIn(let accountData, let deviceData) = deviceState else {
            throw InvalidDeviceStateError()
        }
        try await exclusivityCoordinator.withExclusiveAccess(
            categories: [.deviceStateUpdate]
        ) { [weak self] in
            guard let self else { return }
            let result = await accountsProxy.getAccountData(accountNumber: accountData.number, retryStrategy: .default)
            let deviceState = await interactor.deviceState
            let value = try result.get()
            guard var accountData = deviceState.accountData else { return }
            accountData.expiry = value.expiry
            await interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)
        }
    }

    public func updateDeviceData() async throws {
        let deviceState = await interactor.deviceState
        guard case .loggedIn(let accountData, let deviceData) = deviceState else {
            throw InvalidDeviceStateError()
        }
        try await exclusivityCoordinator.withExclusiveAccess(
            categories: [.deviceStateUpdate]
        ) { [weak self] in
            guard let self else { return }
            let result = await devicesProxy.getDevice(
                accountNumber: accountData.number, identifier: accountData.identifier, retryStrategy: .default)
            let deviceState = await interactor.deviceState
            let value = try result.get()
            guard var deviceData = deviceState.deviceData else { return }
            deviceData.update(from: value)
            await interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)
        }
    }

    public func rotatePrivateKey() async throws {
        let deviceState = await interactor.deviceState
        guard case .loggedIn(let accountData, let deviceData) = deviceState else {
            throw InvalidDeviceStateError()
        }

        try await exclusivityCoordinator.withExclusiveAccess(
            categories: [.deviceStateUpdate]
        ) { [weak self] in
            guard let self else { return }

            // Create key rotation.
            var keyRotation = WgKeyRotation(data: deviceData)

            // Check if key rotation can take place.
            guard keyRotation.shouldRotate else {
                logger.debug("Throttle private key rotation.")
                return
            }

            logger.debug("Private key is old enough, rotate right away.")

            // Mark the beginning of key rotation and receive the public key to push to backend.
            let publicKey = keyRotation.beginAttempt()

            let result = await devicesProxy.rotateDeviceKey(
                accountNumber: accountData.number, identifier: accountData.identifier, publicKey: publicKey,
                retryStrategy: .default)
            let deviceState = await interactor.deviceState
            let value = try result.get()
            guard var deviceData = deviceState.deviceData else { return }
            deviceData.update(from: value)
            await interactor.setDeviceState(.loggedIn(accountData, deviceData), persist: true)
            try await interactor.notifyKeyRotation()
        }
    }

    private func setAccount(action: SetAccountTaskAction) async throws -> StoredAccountData? {
        try await exclusivityCoordinator.withExclusiveAccess(
            categories: [
                .manageTunnel,
                .deviceStateUpdate,
                .settingsUpdate,
            ]
        ) {
            let deviceState = await interactor.deviceState
            let task = SetAccountTask(
                accountsProxy: accountsProxy,
                devicesProxy: devicesProxy,
                action: action,
                deviceState: deviceState,
                updateState: { [weak self] update in
                    guard let self else { return }
                    switch update {
                    case let .setLastUsedAccount(accountNumber):
                        await interactor.setLastUsedAccount(accountNumber)
                    case let .setDeviceState(state):
                        await interactor.setDeviceState(state, persist: true)
                    case let .logout(isRemovingProfile):
                        await interactor.setDeviceState(.loggedOut, persist: true)
                        await interactor.unsetTunnelConfiguration(isRemovingProfile: isRemovingProfile)
                    }
                }
            )
            return try await task.run().get()
        }
    }

    private func performReconnect(selectNewRelay: Bool) async throws {
        interactor.startPollingTunnelStatus()

        do {
            //            try await withBackgroundTask(
            //                name: "Reconnect tunnel",
            //                provider: backgroundTaskProvider
            //            ) {
            guard let tunnel = self.interactor.tunnel else {
                throw UnsetTunnelError()
            }

            let observedState = try await tunnel.reconnectTunnel(
                to: selectNewRelay ? .random : .current
            )

            if let connectionState = observedState.connectionState {
                // This makes the app feel very responsive when the user wants to reconnect.
                // If the tunnel is already connected, at worst the next tunnel status poll
                // will correct the state.
                await self.interactor.setTunnelStatus { tunnelStatus in
                    tunnelStatus.state = .reconnecting(
                        connectionState.selectedRelays,
                        isPostQuantum: connectionState.isPostQuantum,
                        isDaita: connectionState.isDaitaEnabled
                    )
                    tunnelStatus.observedState = observedState
                }
            }
            //            }

            await interactor.didReconnectTunnel(error: nil)
        } catch {
            await interactor.didReconnectTunnel(error: error)
            throw error
        }
    }

}

private enum Category: Hashable, Sendable {
    case manageTunnel
    case deviceStateUpdate
    case settingsUpdate
}

private actor AsyncExclusivityCoordinator {
    private var occupiedCategories: Set<Category> = []
    private var waiters: [Waiter] = []

    private struct Waiter {
        let categories: Set<Category>
        let continuation: CheckedContinuation<Void, Never>
    }

    func withExclusiveAccess<T: Sendable>(
        categories: Set<Category>,
        operation: @Sendable () async throws -> T
    ) async throws -> T {
        await acquire(categories: categories)

        defer {
            release(categories: categories)
        }

        try Task.checkCancellation()
        return try await operation()
    }

    private func acquire(categories: Set<Category>) async {
        guard occupiedCategories.isDisjoint(with: categories) else {
            await withCheckedContinuation { continuation in
                waiters.append(
                    Waiter(
                        categories: categories,
                        continuation: continuation
                    )
                )
            }

            return
        }

        occupiedCategories.formUnion(categories)
    }

    private func release(categories: Set<Category>) {
        occupiedCategories.subtract(categories)
        resumeAvailableWaiters()
    }

    private func resumeAvailableWaiters() {
        var index = 0

        while index < waiters.count {
            let waiter = waiters[index]

            guard occupiedCategories.isDisjoint(with: waiter.categories) else {
                index += 1
                continue
            }

            waiters.remove(at: index)
            occupiedCategories.formUnion(waiter.categories)
            waiter.continuation.resume()
        }
    }
}
