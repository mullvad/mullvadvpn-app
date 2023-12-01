//
//  Actor+KeyPolicy.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /**
     Cache WG active key for a period of time, before switching to using the new one stored in settings.

     This function replaces the key policy to `.usePrior` caching the currently used key in associated value.

     That cached key is used by actor for some time until the new key is propagated across relays. Then it flips the key policy back to `.useCurrent` and
     reconnects the tunnel using new key.

     The `lastKeyRotation` passed as an argument is a simple marker value passed back to UI process with actor state. This date can be used to determine when
     the main app has to re-read device state from Keychain, since there is no other mechanism to notify other process when packet tunnel mutates keychain store.

     - Parameter lastKeyRotation: date when last key rotation took place.
     */
    func cacheActiveKey(lastKeyRotation: Date?) {
        func mutateConnectionState(_ connState: inout ConnectionState) -> Bool {
            switch connState.keyPolicy {
            case .useCurrent:
                if let currentKey = connState.currentKey {
                    connState.lastKeyRotation = lastKeyRotation

                    // Move currentKey into keyPolicy.
                    connState.keyPolicy = .usePrior(currentKey, startKeySwitchTask())
                    connState.currentKey = nil

                    return true
                } else {
                    return false
                }

            case .usePrior:
                // It's unlikely that we'll see subsequent key rotations happen frequently.
                return false
            }
        }

        switch state {
        case var .connecting(connState):
            if mutateConnectionState(&connState) {
                state = .connecting(connState)
                Task {
                    await blockAllTrafficUntilDeviceIsConnected()
                }
            }

        case var .connected(connState):
            if mutateConnectionState(&connState) {
                state = .connected(connState)
            }

        case var .reconnecting(connState):
            if mutateConnectionState(&connState) {
                state = .reconnecting(connState)
                Task {
                    await blockAllTrafficUntilDeviceIsConnected()
                }
            }

        case var .error(blockedState):
            switch blockedState.keyPolicy {
            case .useCurrent:
                // Key policy is preserved between states and key rotation may still happen while in blocked state.
                // Therefore perform the key switch as normal with one exception that it shouldn't reconnect the tunnel
                // automatically.
                if let currentKey = blockedState.currentKey {
                    blockedState.lastKeyRotation = lastKeyRotation

                    // Move currentKey into keyPolicy.
                    blockedState.keyPolicy = .usePrior(currentKey, startKeySwitchTask())
                    blockedState.currentKey = nil

                    state = .error(blockedState)
                }

            case .usePrior:
                break
            }

        case .initial, .disconnected, .disconnecting:
            break
        }
    }

    /**
     Switch key policy  from `.usePrior` to `.useCurrent` policy and reconnect the tunnel.

     Next reconnection attempt will read the new key from settings.
     */
    func switchToCurrentKey() {
        if switchToCurrentKeyInner() {
            commandChannel.send(.reconnect(.random))
        }
    }

    /**
     Start a task that will wait for the new key to propagate across relays (see `PacketTunnelActorTimings.wgKeyPropagationDelay`) and then:

     1. Switch `keyPolicy` back to `.useCurrent`.
     2. Reconnect the tunnel using the new key (currently stored in device state)
     */
    private func startKeySwitchTask() -> AutoCancellingTask {
        // Use detached task to prevent inheriting current context.
        let task = Task.detached { [weak self] in
            guard let self else { return }

            // Wait for key to propagate across relays.
            try await Task.sleepUsingContinuousClock(for: timings.wgKeyPropagationDelay)

            // Enqueue task to change key policy.
            commandChannel.send(.switchKey)
        }

        return AutoCancellingTask(task)
    }

    /**
     Switch key policy  from `.usePrior` to `.useCurrent` policy.

     - Returns: `true` if the tunnel should reconnect, otherwise `false`.
     */
    private func switchToCurrentKeyInner() -> Bool {
        switch state {
        case var .connecting(connState):
            if setCurrentKeyPolicy(&connState.keyPolicy) {
                state = .connecting(connState)
                Task {
                    await blockAllTrafficUntilDeviceIsConnected()
                }
                return true
            }

        case var .connected(connState):
            if setCurrentKeyPolicy(&connState.keyPolicy) {
                state = .connected(connState)
                return true
            }

        case var .reconnecting(connState):
            if setCurrentKeyPolicy(&connState.keyPolicy) {
                state = .reconnecting(connState)
                Task {
                    await blockAllTrafficUntilDeviceIsConnected()
                }
                return true
            }

        case var .error(blockedState):
            if setCurrentKeyPolicy(&blockedState.keyPolicy) {
                state = .error(blockedState)

                // Prevent tunnel from reconnecting when in blocked state.
                return false
            }

        case .disconnected, .disconnecting, .initial:
            break
        }
        return false
    }

    /**
     Internal helper that transitions key policy from `.usePrior` to `.useCurrent`.

     - Parameter keyPolicy: a reference to key policy held either in connection state or blocked state struct.
     - Returns: `true` when the policy was modified, otherwise `false`.
     */
    private func setCurrentKeyPolicy(_ keyPolicy: inout KeyPolicy) -> Bool {
        switch keyPolicy {
        case .useCurrent:
            return false

        case .usePrior:
            keyPolicy = .useCurrent
            return true
        }
    }
}
