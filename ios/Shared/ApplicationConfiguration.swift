//
//  ApplicationConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

enum ApplicationConfiguration {
    /// Shared container security group identifier.
    static var securityGroupIdentifier: String {
        // swiftlint:disable:next force_cast
        Bundle.main.object(forInfoDictionaryKey: "ApplicationSecurityGroupIdentifier") as! String
    }

    /// Container URL for security group.
    static var containerURL: URL {
        FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: securityGroupIdentifier)!
    }

    /// Returns URL for log file associated with application target and located within shared container.
    static func logFileURL(for target: ApplicationTarget) -> URL {
        containerURL.appendingPathComponent("\(target.bundleIdentifier).log", isDirectory: false)
    }

    /// Privacy policy URL.
    static let privacyPolicyURL = URL(string: "https://mullvad.net/help/privacy-policy/")!

    /// Make a start regarding  policy URL.
    static let privacyGuidesURL = URL(string: "https://mullvad.net/help/first-steps-towards-online-privacy/")!

    /// FAQ & Guides URL.
    static let faqAndGuidesURL = URL(string: "https://mullvad.net/help/tag/mullvad-app/")!

    /// Maximum number of devices per account.
    static let maxAllowedDevices = 5
}
