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
    case appLogsDoneButton
    case appLogsShareButton
    case applyButton
    case cancelButton
    case continueWithLoginButton
    case collapseButton
    case expandButton
    case createAccountButton
    case deleteButton
    case deviceCellRemoveButton
    case disconnectButton
    case revokedDeviceLoginButton
    case dnsSettingsEditButton
    case infoButton
    case learnAboutPrivacyButton
    case logOutDeviceConfirmButton
    case logOutDeviceCancelButton
    case loginBarButton
    case loginTextFieldButton
    case logoutButton
    case purchaseButton
    case redeemVoucherButton
    case restorePurchasesButton
    case secureConnectionButton
    case selectLocationButton
    case closeSelectLocationButton
    case settingsButton
    case startUsingTheAppButton
    case problemReportAppLogsButton
    case problemReportSendButton
    case relayStatusCollapseButton
    case settingsDoneButton
    case openCustomListsMenuButton
    case addNewCustomListButton
    case editCustomListButton
    case saveCreateCustomListButton
    case confirmDeleteCustomListButton
    case cancelDeleteCustomListButton
    case customListLocationCheckmarkButton
    case listCustomListDoneButton
    case selectLocationFilterButton
    case relayFilterChipCloseButton

    // Cells
    case deviceCell
    case accessMethodProtocolSelectionCell
    case vpnSettingsCell
    case dnsSettingsAddServerCell
    case dnsSettingsUseCustomDNSCell
    case preferencesCell
    case versionCell
    case problemReportCell
    case faqCell
    case apiAccessCell
    case relayFilterProviderCell
    case wireGuardPortsCell
    case wireGuardObfuscationCell
    case udpOverTCPPortCell
    case quantumResistantTunnelCell
    case customListEditNameFieldCell
    case customListEditAddOrEditLocationCell
    case customListEditDeleteListCell
    case locationFilterOwnershipHeaderCell
    case locationFilterProvidersHeaderCell
    case ownershipMullvadOwnedCell
    case ownershipRentedCell
    case ownershipAnyCell
    case countryLocationCell
    case cityLocationCell
    case relayLocationCell
    case customListLocationCell
    case daitaConfirmAlertBackButton
    case daitaConfirmAlertEnableButton

    // Labels
    case accountPageDeviceNameLabel
    case socks5ServerCell
    case socks5PortCell
    case accountPagePaidUntilLabel
    case addAccessMethodTestStatusReachableLabel
    case addAccessMethodTestStatusTestingLabel
    case addAccessMethodTestStatusUnreachableLabel
    case headerDeviceNameLabel
    case connectionStatusConnectedLabel
    case connectionStatusConnectingLabel
    case connectionStatusNotConnectedLabel
    case welcomeAccountNumberLabel
    case connectionPanelDetailLabel
    case relayFilterChipLabel

    // Views
    case accessMethodProtocolPickerView
    case accessMethodUnreachableAlert
    case accountView
    case addLocationsView
    case addAccessMethodTableView
    case apiAccessView
    case alertContainerView
    case alertTitle
    case appLogsView
    case changeLogAlert
    case deviceManagementView
    case editAccessMethodView
    case headerBarView
    case loginView
    case outOfTimeView
    case termsOfServiceView
    case selectLocationView
    case selectLocationViewWrapper
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
    case newCustomListView
    case customListEditTableView
    case listCustomListsView
    case listCustomListsTableView
    case editCustomListEditLocationsView
    case editCustomListEditLocationsTableView
    case relayFilterChipView
    case dnsSettingsTableView

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
    case problemReportAppLogsTextView
    case problemReportEmailTextField
    case problemReportMessageTextView
    case deleteAccountTextField
    case socks5AuthenticationSwitch
    case statusImageView

    // DNS settings
    case dnsSettings
    case ipOverrides
    case wireGuardCustomPort
    case wireGuardObfuscationAutomatic
    case wireGuardObfuscationPort
    case wireGuardObfuscationOff
    case wireGuardObfuscationUDPTCP
    case wireGuardObfuscationShadowsocks
    case wireGuardPort

    // Custom DNS
    case blockAll
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

    // DAITA
    case daitaSwitch
    case daitaPromptAlert
    case daitaDirectOnlySwitch

    // Quantum resistance
    case quantumResistanceAutomatic
    case quantumResistanceOff
    case quantumResistanceOn

    // Multihop
    case multihopSwitch

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
