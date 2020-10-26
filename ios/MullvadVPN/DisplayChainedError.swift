//
//  DisplayChainedError.swift
//  MullvadVPN
//
//  Created by pronebird on 04/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol DisplayChainedError {
    var errorChainDescription: String? { get }
}

extension RestError: DisplayChainedError {
    var errorChainDescription: String? {
        switch self {
        case .network(let urlError):
            return String(
                format: NSLocalizedString("Network error: %@", comment: ""),
                urlError.localizedDescription
            )
        case .server(let serverError):
            if let knownErrorDescription = serverError.errorDescription {
                return knownErrorDescription
            } else {
                return String(
                    format: NSLocalizedString("Server error: %@", comment: ""),
                    serverError.error ?? "(empty)"
                )
            }
        case .encodePayload:
            return NSLocalizedString("Server request encoding error", comment: "")
        case .decodeSuccessResponse:
            return NSLocalizedString("Server success response decoding error", comment: "")
        case .decodeErrorResponse:
            return NSLocalizedString("Server error response decoding error", comment: "")
        }
    }
}

extension TunnelManager.Error: DisplayChainedError {
    var errorChainDescription: String? {
        switch self {
        case .loadAllVPNConfigurations(let systemError):
            return String(format: NSLocalizedString("Failed to load system VPN configurations: %@", comment: ""), systemError.localizedDescription)

        case .reloadVPNConfiguration(let systemError):
            return String(format: NSLocalizedString("Failed to reload a VPN configuration: %@", comment: ""), systemError.localizedDescription)

        case .saveVPNConfiguration(let systemError):
            return String(format: NSLocalizedString("Failed to save a VPN tunnel configuration: %@", comment: ""), systemError.localizedDescription)

        case .obtainPersistentKeychainReference(_):
            return NSLocalizedString("Failed to obtain the persistent keychain reference for the VPN configuration", comment: "")

        case .startVPNTunnel(let systemError):
            return String(format: NSLocalizedString("System error when starting the VPN tunnel: %@", comment: ""), systemError.localizedDescription)

        case .removeVPNConfiguration(let systemError):
            return String(format: NSLocalizedString("Failed to remove the system VPN configuration: %@", comment: ""), systemError.localizedDescription)

        case .removeInconsistentVPNConfiguration(let systemError):
            return String(format: NSLocalizedString("Failed to remove the outdated system VPN configuration: %@", comment: ""), systemError.localizedDescription)

        case .readTunnelSettings(_):
            return NSLocalizedString("Failed to read tunnel settings", comment: "")

        case .addTunnelSettings(_):
            return NSLocalizedString("Failed to add tunnel settings", comment: "")

        case .updateTunnelSettings(_):
            return NSLocalizedString("Failed to update tunnel settings", comment: "")

        case .removeTunnelSettings(_):
            return NSLocalizedString("Failed to remove tunnel settings", comment: "")

        case .pushWireguardKey(let restError):
            let reason = restError.errorChainDescription ?? ""
            var message = String(format: NSLocalizedString("Failed to send the WireGuard key to server: %@", comment: ""), reason)

            if case .server(.keyLimitReached) = restError {
                message.append("\n\n")
                message.append(NSLocalizedString("Remove unused WireGuard keys and try again", comment: ""))
            }

            return message

        case .replaceWireguardKey(let restError):
            let reason = restError.errorChainDescription ?? ""
            var message = String(format: NSLocalizedString("Failed to replace the WireGuard key on server: %@", comment: ""), reason)

            if case .server(.keyLimitReached) = restError {
                message.append("\n\n")
                message.append(NSLocalizedString("Remove unused WireGuard keys and try again", comment: ""))
            }

            return message

        case .removeWireguardKey:
            // This error is never displayed anywhere
            return nil

        case .verifyWireguardKey(let restError):
            let reason = restError.errorChainDescription ?? ""

            return String(format: NSLocalizedString("Failed to verify the WireGuard key on server: %@", comment: ""), reason)

        case .missingAccount:
            return NSLocalizedString("Internal error: missing account", comment: "")
        }
    }
}

extension Account.Error: DisplayChainedError {

    var errorChainDescription: String? {
        switch self {
        case .createAccount(let restError), .verifyAccount(let restError):
            return restError.errorChainDescription

        case .tunnelConfiguration(let tunnelError):
            return tunnelError.errorChainDescription
        }
    }

}

extension AppStorePaymentManager.Error: DisplayChainedError {
    var errorChainDescription: String? {
        switch self {
        case .noAccountSet:
            return NSLocalizedString("Internal error: account is not set", comment: "")

        case .readReceipt(let readReceiptError):
            switch readReceiptError {
            case .refresh(let storeError):
                return String(format: NSLocalizedString("Cannot refresh the AppStore receipt: %@", comment: ""), storeError.localizedDescription)
            case .io(let ioError):
                return String(format: NSLocalizedString("Cannot read the AppStore receipt from disk: %@", comment: ""), ioError.localizedDescription)
            case .doesNotExist:
                return NSLocalizedString("AppStore receipt is not found on disk.", comment: "")
            }

        case .sendReceipt(let restError):
            let reason = restError.errorChainDescription ?? ""

            return String(format: NSLocalizedString(#"Failed to send the receipt to server: %@\n\nPlease retry by using the "Restore purchases" button."#, comment: ""), reason)

        case .storePayment(let storeError):
            return storeError.localizedDescription
        }
    }
}
