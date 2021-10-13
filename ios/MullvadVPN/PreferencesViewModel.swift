//
//  PreferencesViewModel.swift
//  MullvadVPN
//
//  Created by pronebird on 11/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct PreferencesViewModel: Equatable {
    var blockAdvertising: Bool
    var blockTracking: Bool
    var enableCustomDNS: Bool
    var customDNSDomains: [String]
    var dnsServerUserInput: String = ""

    /// Returns true if custom DNS can be enabled.
    var canEnableCustomDNS: Bool {
        return !blockAdvertising && !blockTracking
    }

    /// Effective state of the custom DNS setting.
    var effectiveEnableCustomDNS: Bool {
        return canEnableCustomDNS && enableCustomDNS
    }

    init(from dnsSettings: DNSSettings = DNSSettings()) {
        blockAdvertising = dnsSettings.blockAdvertising
        blockTracking = dnsSettings.blockTracking
        enableCustomDNS = dnsSettings.enableCustomDNS
        customDNSDomains = dnsSettings.customDNSDomains.map { ipAddress in
            return "\(ipAddress)"
        }
    }

    /// Compare view models ignoring `dnsServerUserInput` field.
    func compare(_ other: PreferencesViewModel) -> Bool {
        return blockAdvertising == other.blockAdvertising &&
            blockTracking == other.blockTracking &&
            enableCustomDNS == other.enableCustomDNS &&
            customDNSDomains == other.customDNSDomains
    }

    /// Merge view models ignoring `dnsServerUserInput` field.
    mutating func merge(_ other: PreferencesViewModel) {
        blockAdvertising = other.blockAdvertising
        blockTracking = other.blockTracking
        enableCustomDNS = other.enableCustomDNS
        customDNSDomains = other.customDNSDomains
    }

    /// Reset user input and sanitize custom DNS entries.
    mutating func endEditing() {
        // Reset user input
        dnsServerUserInput = ""

        // Santize DNS domains, drop invalid entries.
        customDNSDomains = customDNSDomains.compactMap { ipAddressString in
            return AnyIPAddress(ipAddressString).map { ipAddress in
                return "\(ipAddress)"
            }
        }

        // Toggle off custom DNS when no domains specified.
        if customDNSDomains.isEmpty {
            enableCustomDNS = false
        }
    }

    /// Converts view model into `DNSSettings`.
    func asDNSSettings() -> DNSSettings {
        var dnsSettings = DNSSettings()
        dnsSettings.blockAdvertising = blockAdvertising
        dnsSettings.blockTracking = blockTracking
        dnsSettings.enableCustomDNS = enableCustomDNS
        dnsSettings.customDNSDomains = customDNSDomains.compactMap { ipAddressString in
            return AnyIPAddress(ipAddressString)
        }
        return dnsSettings
    }

    /// Returns true if the given string is empty or a valid IP address.
    func isValidDNSDomainInput(_ string: String) -> Bool {
        return string.isEmpty || AnyIPAddress(string) != nil
    }
}
