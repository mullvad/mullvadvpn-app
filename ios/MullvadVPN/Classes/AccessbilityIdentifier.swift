//
//  RelayFilter.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-12-20.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

public enum AccessibilityIdentifier: String {
    // Buttons
    case accountButton
    case agreeButton
    case alertOkButton
    case applyButton
    case cancelButton
    case connectionPanelButton
    case collapseButton
    case deleteButton
    case disconnectButton
    case infoButton
    case learnAboutPrivacyButton
    case loginBarButton
    case loginTextFieldButton
    case logoutButton
    case purchaseButton
    case redeemVoucherButton
    case restorePurchasesButton
    case selectLocationButton
    case settingsButton
    case startUsingTheAppButton

    // Cells
    case preferencesCell
    case versionCell
    case problemReportCell
    case faqCell
    case apiAccessCell
    case ipOverrideCell
    case relayFilterOwnershipCell
    case relayFilterProviderCell

    // Labels
    case headerDeviceNameLabel

    // Views
    case accountView
    case alertContainerView
    case alertTitle
    case headerBarView
    case loginView
    case termsOfServiceView
    case selectLocationView
    case selectLocationTableView
    case settingsTableView
    case tunnelControlView

    // Other UI elements
    case connectionPanelInAddressRow
    case connectionPanelOutAddressRow
    case customSwitch
    case dnsContentBlockersHeaderView
    case loginTextField
    case selectLocationSearchTextField

    // DNS settings
    case dnsSettings
    case wireGuardCustomPort
    case wireGuardObfuscationAutomatic
    case wireGuardObfuscationOff
    case wireGuardObfuscationOn
    case wireGuardPort

    // Custom DNS
    case blockAdvertising
    case blockTracking
    case blockMalware
    case blockGambling
    case blockAdultContent
    case blockSocialMedia
    case useCustomDNS
    case addDNSServer
    case dnsServer
    case dnsServerInfo

    // Error
    case unknown
}

extension UIAccessibilityIdentification {
    var accessibilityIdentifier: AccessibilityIdentifier? {
        get {
            guard let accessibilityIdentifier else { return nil }
            return AccessibilityIdentifier(rawValue: accessibilityIdentifier)
        }
        set {
            accessibilityIdentifier = newValue?.rawValue
        }
    }
}
