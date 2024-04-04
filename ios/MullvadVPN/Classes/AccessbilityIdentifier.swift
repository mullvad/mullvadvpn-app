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
    case addAccessMethodButton
    case accessMethodAddButton
    case accountButton
    case accessMethodUnreachableBackButton
    case accessMethodUnreachableSaveButton
    case agreeButton
    case alertOkButton
    case applyButton
    case cancelButton
    case connectionPanelButton
    case collapseButton
    case expandButton
    case createAccountButton
    case deleteButton
    case disconnectButton
    case revokedDeviceLoginButton
    case dnsSettingsEditButton
    case infoButton
    case learnAboutPrivacyButton
    case loginBarButton
    case loginTextFieldButton
    case logoutButton
    case purchaseButton
    case redeemVoucherButton
    case restorePurchasesButton
    case secureConnectionButton
    case selectLocationButton
    case settingsButton
    case startUsingTheAppButton
    case problemReportAppLogsButton
    case problemReportSendButton
    case relayStatusCollapseButton
    case settingsDoneButton

    // Cells
    case accessMethodProtocolSelectionCell
    case vpnSettingsCell
    case dnsSettingsAddServerCell
    case dnsSettingsUseCustomDNSCell
    case preferencesCell
    case versionCell
    case problemReportCell
    case faqCell
    case apiAccessCell
    case relayFilterOwnershipCell
    case relayFilterProviderCell
    case wireGuardPortsCell
    case wireGuardObfuscationCell
    case udpOverTCPPortCell
    case quantumResistantTunnelCell
    case socks5ServerCell
    case socks5PortCell

    // Labels
    case accountPagePaidUntilLabel
    case addAccessMethodTestStatusReachableLabel
    case addAccessMethodTestStatusTestingLabel
    case addAccessMethodTestStatusUnreachableLabel
    case headerDeviceNameLabel
    case connectionStatusConnectedLabel
    case connectionStatusNotConnectedLabel
    case welcomeAccountNumberLabel
    case connectionPanelDetailLabel

    // Views
    case accessMethodProtocolPickerView
    case accessMethodUnreachableAlert
    case accountView
    case addLocationsView
    case addAccessMethodTableView
    case apiAccessView
    case alertContainerView
    case alertTitle
    case changeLogAlert
    case editAccessMethodView
    case headerBarView
    case loginView
    case outOfTimeView
    case termsOfServiceView
    case selectLocationView
    case selectLocationTableView
    case settingsTableView
    case vpnSettingsTableView
    case tunnelControlView
    case problemReportView
    case problemReportSubmittedView
    case revokedDeviceView
    case welcomeView
    case deleteAccountView
    case settingsContainerView

    // Other UI elements
    case accessMethodEnableSwitch
    case accessMethodNameTextField
    case logOutSpinnerAlertView
    case connectionPanelInAddressRow
    case connectionPanelOutAddressRow
    case customSwitch
    case customWireGuardPortTextField
    case dnsContentBlockersHeaderView
    case dnsSettingsEnterIPAddressTextField
    case loginStatusIconAuthenticating
    case loginStatusIconFailure
    case loginStatusIconSuccess
    case loginTextField
    case selectLocationSearchTextField
    case problemReportEmailTextField
    case problemReportMessageTextView
    case deleteAccountTextField
    case socks5AuthenticationSwitch

    // DNS settings
    case dnsSettings
    case ipOverrides
    case wireGuardCustomPort
    case wireGuardObfuscationAutomatic
    case wireGuardObfuscationPort
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

    // Quantum resistance
    case quantumResistanceAutomatic
    case quantumResistanceOff
    case quantumResistanceOn

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
