//
//  Actor+KeyPolicy.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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
        state.mutateAssociatedData { stateData in
            guard
                stateData.keyPolicy == .useCurrent,
                let currentKey = stateData.currentKey
            else { return }
            // Key policy is preserved between states and key rotation may still happen while in blocked state.
            // Therefore perform the key switch as normal with one exception that it shouldn't reconnect the tunnel
            // automatically.
            stateData.lastKeyRotation = lastKeyRotation

            // Move currentKey into keyPolicy.
            stateData.keyPolicy = .usePrior(currentKey, startKeySwitchTask())
            stateData.currentKey = nil
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
            try await Task.sleep(for: timings.wgKeyPropagationDelay)

            guard !Task.isCancelled else { return }

            // Enqueue task to change key policy.
            eventChannel.send(.switchKey)
        }

        return AutoCancellingTask(task)
    }
}
