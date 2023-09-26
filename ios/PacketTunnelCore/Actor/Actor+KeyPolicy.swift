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
     Tell actor that key rotation took place.

     When that happens the actor changes key policy to `.usePrior` caching the currently used key in associated value.

     That cached key is used by actor for some time until the new key is propagated across relays. Then it flips the key policy back to `.useCurrent` and
     reconnects the tunnel using new key.

     The `date` passed as an argument is a simple marker value passed back to UI process with actor state. This date can be used to determine when
     the main app has to re-read device state from Keychain, since there is no other mechanism to notify other process when packet tunnel mutates keychain store.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.

     - Parameter date: date when last key rotation took place.
     */
    public func notifyKeyRotated(date: Date? = nil) async {
        try? await taskQueue.add(kind: .keyRotated) { [self] in
            notifyKeyRotatedInner(date: date)
        }
    }

    /**
     Cache currently used WG key and start a task that will change the policy back to use the newest key after some time and reconnect the tunnel.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.
     */
    private func notifyKeyRotatedInner(date: Date?) {
        func mutateConnectionState(_ connState: inout ConnectionState) -> Bool {
            switch connState.keyPolicy {
            case .useCurrent:
                connState.lastKeyRotation = date
                connState.keyPolicy = .usePrior(connState.currentKey, startKeySwitchTask())
                return true

            case .usePrior:
                // It's unlikely that we'll see subsequent key rotations happen frequently.
                return false
            }
        }

        switch state {
        case var .connecting(connState):
            if mutateConnectionState(&connState) {
                state = .connecting(connState)
            }

        case var .connected(connState):
            if mutateConnectionState(&connState) {
                state = .connected(connState)
            }

        case var .reconnecting(connState):
            if mutateConnectionState(&connState) {
                state = .reconnecting(connState)
            }

        case var .error(blockedState):
            switch blockedState.keyPolicy {
            case .useCurrent:
                if let currentKey = blockedState.currentKey {
                    blockedState.lastKeyRotation = date
                    blockedState.keyPolicy = .usePrior(currentKey, startKeySwitchTask())
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
     Start a task that will wait for the new key to propagate across relays (see `PacketTunnelActorTimings.wgKeyPropagationDelay`) and then:

     1. Switch `keyPolicy` back to `.useCurrent`.
     2. Reconnect the tunnel using the new key (currently stored in device state)
     */
    private func startKeySwitchTask() -> AutoCancellingTask {
        let task = Task {
            // Wait for key to propagate across relays.
            try await Task.sleepUsingContinuousClock(for: timings.wgKeyPropagationDelay)

            // Enqueue task to change key policy.
            let isKeySwitched = try await enqueueKeySwitch()

            // Enqueue task to reconnect
            if isKeySwitched {
                try await reconnect(to: .random)
            }
        }

        return AutoCancellingTask(task)
    }

    /**
     Enqueue a task to switch key policy to `.useCurrent`.

     - Returns: `true` if the key was switched, otherwise `false`.
     */
    private func enqueueKeySwitch() async throws -> Bool {
        try await taskQueue.add(kind: .switchKey) {
            self.keySwitchInner()
        }
    }

    /**
     Switch key policy  from `.usePrior` to `.useCurrent` policy.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.

     - Returns: `true` if the key policy switch happened, otherwise `false`.
     */
    private func keySwitchInner() -> Bool {
        // Switch key policy to use current key.
        switch state {
        case var .connecting(connState):
            if setCurrentKeyPolicy(&connState.keyPolicy) {
                state = .connecting(connState)
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
                return true
            }

        case var .error(blockedState):
            if setCurrentKeyPolicy(&blockedState.keyPolicy) {
                state = .error(blockedState)
                return true
            }

        case .disconnected, .disconnecting, .initial:
            break
        }
        return false
    }

    /**
     Internal helper that transitions key policy from `.usePrior` to `.useCurrent`.

     - Parameter keyPolicy: a reference to key policy hend either in connection state or blocked state struct.
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
