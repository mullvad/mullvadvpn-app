//
//  BlockedStateErrorMapper.swift
//  PacketTunnel
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import WireGuardKit

/**
 Struct responsible for mapping errors that may occur in the packet tunnel to the `BlockedStateReason`.
 */
public struct BlockedStateErrorMapper: BlockedStateErrorMapperProtocol {
    public func mapError(_ error: Error) -> BlockedStateReason {
        switch error {
        case let error as ReadDeviceDataError:
            // Such error is thrown by implementations of `SettingsReaderProtocol`.
            switch error {
            case .loggedOut:
                return .deviceLoggedOut
            case .revoked:
                return .deviceRevoked
            }

        case is UnsupportedSettingsVersionError:
            // Can be returned after updating the app. The tunnel is usually restarted right after but the main app
            // needs to be launched to perform settings migration.
            return .outdatedSchema

        case let keychainError as KeychainError where keychainError == .interactionNotAllowed:
            // Returned when reading device state from Keychain when it is locked on device boot.
            return .deviceLocked

        case let error as ReadSettingsVersionError:
            // Returned when reading tunnel settings from Keychain.
            // interactionNotAllowed is returned when device is locked on boot, otherwise it must be a generic error
            // when reading settings from keychain.
            if case KeychainError.interactionNotAllowed = error.underlyingError as? KeychainError {
                return .deviceLocked
            } else {
                return .readSettings
            }

        case is NoRelaysSatisfyingConstraintsError:
            // Returned by relay selector when there are no relays satisfying the given constraint.
            return .noRelaysSatisfyingConstraints

        case is WireGuardAdapterError:
            // Any errors that originate from wireguard adapter including failure to set tunnel settings using
            // packet tunnel provider.
            return .tunnelAdapter

        case is PublicKeyError:
            // Returned when there is an endpoint but its public key is invalid.
            return .invalidRelayPublicKey

        default:
            // Everything else in case we introduce new errors and forget to handle them.
            return .unknown
        }
    }
}
