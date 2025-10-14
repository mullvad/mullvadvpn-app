//
//  AccessMethodViewModel.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

/// The view model used by view controllers editing access method data.
struct AccessMethodViewModel: Identifiable {
    /// Socks configuration view model.
    struct Socks {
        /// Server IP address input.
        var server = ""
        /// Server port input.
        var port = ""
        /// Authentication username.
        var username = ""
        /// Authentication password.
        var password = ""
        /// Indicates whether authentication is enabled.
        var authenticate = false
    }

    /// Shadowsocks configuration view model.
    struct Shadowsocks {
        /// Server IP address input.
        var server = ""
        /// Server port input.
        var port = ""
        /// Server password.
        var password = ""
        /// Shadowsocks cipher.
        var cipher = ShadowsocksCipherOptions.default
    }

    /// Access method testing status view model.
    enum TestingStatus {
        /// The default state before the testing began.
        case initial
        /// Testing is in progress.
        case inProgress
        /// Testing failed.
        case failed
        /// Testing succeeded.
        case succeeded
    }

    /// The unique identifier used for referencing the access method entry in a persistent store.
    var id = UUID()

    /// The user-defined name for access method.
    var name = ""

    /// The selected access method kind.
    /// Determines which subview model is used when presenting proxy configuration in UI.
    var method: AccessMethodKind = .shadowsocks

    /// The flag indicating whether configuration is enabled.
    var isEnabled = true

    /// The flag indicating whether configuration is enabled.
    var canBeToggled = true

    /// The status of testing the entered proxy configuration.
    var testingStatus: TestingStatus = .initial

    /// Socks configuration view model.
    var socks = Socks()

    /// Shadowsocks configuration view model.
    var shadowsocks = Shadowsocks()
}

extension AccessMethodViewModel {
    var infoHeaderConfig: InfoHeaderConfig? {
        switch id {
        case AccessMethodRepository.directId:
            InfoHeaderConfig(
                body: NSLocalizedString("The app communicates with a Mullvad API server directly.", comment: ""),
                link: NSLocalizedString("About Direct method...", comment: "")
            )
        case AccessMethodRepository.bridgeId:
            InfoHeaderConfig(
                body: NSLocalizedString(
                    "The app communicates with a Mullvad API server via a Mullvad bridge server.",
                    comment: ""
                ),
                link: NSLocalizedString("About Mullvad bridges method...", comment: "")
            )
        case AccessMethodRepository.encryptedDNSId:
            InfoHeaderConfig(
                body: NSLocalizedString(
                    "The app communicates with a Mullvad API server via a proxy address.",
                    comment: ""
                ),
                link: NSLocalizedString("About Encrypted DNS proxy method...", comment: "")
            )
        default:
            nil
        }
    }

    var infoModalConfig: InfoModalConfig? {
        switch id {
        case AccessMethodRepository.directId:
            InfoModalConfig(
                header: "Direct",
                preamble: NSLocalizedString("The app communicates with a Mullvad API server directly.", comment: ""),
                body: [
                    NSLocalizedString(
                        "With the “Direct” method, the app communicates with a Mullvad API "
                            + "server directly without any intermediate proxies.",
                        comment: ""
                    ),
                    NSLocalizedString("This can be useful when you are not affected by censorship.", comment: ""),
                ]
            )
        case AccessMethodRepository.bridgeId:
            InfoModalConfig(
                header: "Mullvad bridges",
                preamble: NSLocalizedString(
                    "The app communicates with a Mullvad API server via a Mullvad bridge server.",
                    comment: ""
                ),
                body: [
                    NSLocalizedString(
                        "With the “Mullvad bridges” method, the app communicates with a Mullvad API server via a "
                            + "Mullvad bridge server. It does this by sending the traffic obfuscated by Shadowsocks.",
                        comment: ""
                    ),
                    NSLocalizedString(
                        "This can be useful if the API is censored but Mullvad’s bridge servers are not.",
                        comment: ""
                    ),
                ]
            )
        case AccessMethodRepository.encryptedDNSId:
            InfoModalConfig(
                header: "Encrypted DNS proxy",
                preamble: NSLocalizedString(
                    "The app communicates with a Mullvad API server via a proxy address.",
                    comment: ""
                ),
                body: [
                    NSLocalizedString(
                        "With the “Encrypted DNS proxy” method, the app will communicate with our "
                            + "Mullvad API through a proxy address. It does this by retrieving "
                            + "an address from a DNS over HTTPS (DoH) server and then using that "
                            + "to reach our API servers.",
                        comment: ""
                    ),
                    NSLocalizedString(
                        "If you are not connected to our VPN, then the Encrypted DNS proxy will use "
                            + "your own non-VPN IP when connecting. The DoH servers are hosted by one of the "
                            + "following providers: Quad9 or Cloudflare.",
                        comment: ""
                    ),
                ]
            )
        default:
            nil
        }
    }
}
