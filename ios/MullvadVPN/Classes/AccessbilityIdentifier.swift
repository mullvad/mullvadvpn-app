public enum AccessibilityIdentifier: String {
    // Buttons
    case accountButton
    case agreeButton
    case alertOkButton
    case applyButton
    case cancelButton
    case collapseButton
    case deleteButton
    case disconnectButton
    case infoButton
    case learnAboutPrivacyButton
    case loginBarButton
    case logoutButton
    case purchaseButton
    case redeemVoucherButton
    case selectLocationButton
    case settingsButton
    case startUsingTheAppButton

    // Cells
    case preferencesCell
    case relayFilterOwnershipCell
    case relayFilterProviderCell

    // Other UI elements
    case loginTextField

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
}
