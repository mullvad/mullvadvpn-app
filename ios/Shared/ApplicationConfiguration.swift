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

    /// Returns URL for new log file associated with application target and located within shared container.
    static func newLogFileURL(for target: ApplicationTarget) -> URL {
        containerURL.appendingPathComponent(
            "\(target.bundleIdentifier)_\(Date().logFormatDeviceLog()).log",
            isDirectory: false
        )
    }

    /// Returns URLs for log files associated with application target and located within shared container.
    static func logFileURLs(for target: ApplicationTarget) -> [URL] {
        let containerUrl = containerURL

        return (try? FileManager.default.contentsOfDirectory(atPath: containerURL.relativePath))?.compactMap { file in
            if file.split(separator: ".").last == "log" {
                containerUrl.appendingPathComponent(file)
            } else {
                nil
            }
        }.sorted { $0.relativePath > $1.relativePath } ?? []
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
