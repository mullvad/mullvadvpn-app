// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadSettings
import MullvadTypes
import SwiftUI

@MainActor
protocol EditAccessMethodViewModelProtocol {
    var accessMethod: PersistentAccessMethod { get }

    var name: String { get }
    var infoHeaderConfig: InfoHeaderConfig? { get }
    var infoModalConfig: InfoModalConfig? { get }
    var isEnabled: Bool { get set }
    var isEditable: Bool { get }
    var testState: ApiAccessMethodTestState { get }

    func startTesting()
    func stopTesting()
    func delete()
}

enum ApiAccessMethodTestState {
    case idle
    case testing
    case reachable
    case unreachable
}

@Observable
final class EditAccessMethodViewModel: EditAccessMethodViewModelProtocol {
    var accessMethod: PersistentAccessMethod
    var testState: ApiAccessMethodTestState = .idle

    let interactor: EditAccessMethodInteractorProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol

    init(
        accessMethod: PersistentAccessMethod,
        interactor: EditAccessMethodInteractorProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol
    ) {
        self.interactor = interactor
        self.proxyConfigurationTester = proxyConfigurationTester
        self.isEnabled = accessMethod.isEnabled
        self.accessMethod = accessMethod
    }

    var name: String {
        accessMethod.name
    }

    var isEnabled: Bool {
        didSet {
            guard isEnabled != oldValue else { return }

            let result = interactor.setEnabled(isEnabled, for: accessMethod)

            switch result {
            case .success(let updatedMethod):
                accessMethod = updatedMethod

            case .failure:
                // The access method can only be disabled if at least one other method is enabled
                isEnabled = true
            }
        }
    }

    var isEditable: Bool {
        !accessMethod.kind.isPermanent
    }

    var infoHeaderConfig: InfoHeaderConfig? {
        switch accessMethod.id {
        case AccessMethodRepository.directId:
            InfoHeaderConfig(
                body: NSLocalizedString("The app communicates with a Mullvad API server directly.", comment: ""),
                link: String(format: NSLocalizedString("About %@ method...", comment: ""), "Direct")
            )
        case AccessMethodRepository.bridgeId:
            InfoHeaderConfig(
                body: NSLocalizedString(
                    "The app communicates with a Mullvad API server via a Mullvad bridge server.",
                    comment: ""
                ),
                link: String(format: NSLocalizedString("About %@ method...", comment: ""), "Mullvad bridges")
            )
        case AccessMethodRepository.encryptedDNSId:
            InfoHeaderConfig(
                body: NSLocalizedString(
                    "The app communicates with a Mullvad API server via a proxy address.",
                    comment: ""
                ),
                link: String(format: NSLocalizedString("About %@ method...", comment: ""), "Encrypted DNS proxy")
            )
        case AccessMethodRepository.domainFrontingId:
            InfoHeaderConfig(
                body: NSLocalizedString(
                    "The app communicates with a Mullvad API server via domain fronting.",
                    comment: ""
                ),
                link: String(format: NSLocalizedString("About %@ method...", comment: ""), "Domain fronting")
            )
        default:
            nil
        }
    }

    var infoModalConfig: InfoModalConfig? {
        switch accessMethod.id {
        case AccessMethodRepository.directId:
            InfoModalConfig(
                header: "Direct",
                preamble: NSLocalizedString("The app communicates with a Mullvad API server directly.", comment: ""),
                body: [
                    String(
                        format: NSLocalizedString(
                            "With the “%@” method, the app communicates with a Mullvad API "
                                + "server directly without any intermediate proxies.",
                            comment: ""
                        ),
                        "Direct"
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
                    String(
                        format: NSLocalizedString(
                            "With the “%@” method, the app communicates with a Mullvad API server via a "
                                + "Mullvad bridge server. It does this by sending the traffic obfuscated "
                                + "by Shadowsocks.",
                            comment: ""
                        ),
                        "Mullvad bridges"
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
                    String(
                        format: NSLocalizedString(
                            "With the “%@” method, the app will communicate with our "
                                + "Mullvad API through a proxy address. It does this by retrieving "
                                + "an address from a DNS over HTTPS (DoH) server and then using that "
                                + "to reach our API servers.",
                            comment: ""
                        ),
                        "Encrypted DNS proxy"
                    ),
                    String(
                        format: NSLocalizedString(
                            """
                            If you are not connected to our VPN, then the %@ will \
                            use your own non-VPN IP when connecting. The DoH servers are hosted by \
                            one of the following providers: Quad9 or Cloudflare.
                            """,
                            comment: ""
                        ),
                        "Encrypted DNS proxy"
                    ),
                ]
            )
        case AccessMethodRepository.domainFrontingId:
            InfoModalConfig(
                header: "Domain fronting",
                preamble: String(
                    format:
                        NSLocalizedString(
                            "The app communicates with a Mullvad API server via %@.",
                            comment: ""
                        ),
                    "domain fronting"
                ),
                body: [
                    String(
                        format: NSLocalizedString(
                            "With the %@ access method, the app reaches the Mullvad API via a CDN, "
                                + "mixing the traffic with a lot of other internet traffic, making it "
                                + "more difficult to censor.",
                            comment: ""
                        ),
                        "domain fronting"
                    ),
                    NSLocalizedString(
                        "This can be useful when direct access and other methods are blocked by censorship.",
                        comment: ""
                    ),
                    NSLocalizedString(
                        "The CDN used is Datapacket’s CDN77. The CDN can only observe the "
                            + "proxied TLS traffic, not the contents.",
                        comment: ""
                    ),
                ]
            )
        default:
            nil
        }
    }

    func startTesting() {
        testState = .testing
        proxyConfigurationTester.start(configuration: accessMethod) { [weak self] error in
            Task { @MainActor in
                guard let self else { return }

                if error != nil {
                    self.testState = .unreachable
                } else {
                    self.testState = .reachable
                }
            }
        }
    }

    func stopTesting() {
        testState = .idle
        proxyConfigurationTester.cancel()
    }

    func delete() {
        interactor.delete(accessMethod)
    }
}
