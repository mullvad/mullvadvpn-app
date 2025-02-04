//
//  ApplicationConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

enum ApplicationConfiguration {
    static var hostName: String {
        // swiftlint:disable:next force_cast
        Bundle.main.object(forInfoDictionaryKey: "HostName") as! String
    }

    /// Shared container security group identifier.
    static var securityGroupIdentifier: String {
        // swiftlint:disable:next force_cast
        Bundle.main.object(forInfoDictionaryKey: "ApplicationSecurityGroupIdentifier") as! String
    }

    /// Container URL for security group.
    static var containerURL: URL {
        FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: securityGroupIdentifier)!
    }

    /// Returns URL for new log file associated with application target and located within the specified container.
    static func newLogFileURL(for target: ApplicationTarget, in containerURL: URL) -> URL {
        containerURL.appendingPathComponent(
            "\(target.bundleIdentifier)_\(Date().logFileFormatted).log",
            isDirectory: false
        )
    }

    /// Returns URLs for log files associated with application target and located within the specified container.
    static func logFileURLs(for target: ApplicationTarget, in containerURL: URL) -> [URL] {
        let fileManager = FileManager.default
        let filePathsInDirectory = try? fileManager.contentsOfDirectory(atPath: containerURL.relativePath)

        let filteredFilePaths: [URL] = filePathsInDirectory?.compactMap { path in
            let pathIsLog = path.split(separator: ".").last == "log"
            // Pattern should be "<Target Bundle ID>_", eg. "net.mullvad.MullvadVPN_".
            let pathBelongsToTarget = path.contains("\(target.bundleIdentifier)_")

            return pathIsLog && pathBelongsToTarget ? containerURL.appendingPathComponent(path) : nil
        } ?? []

        let sortedFilePaths = try? filteredFilePaths.sorted { path1, path2 in
            let path1Attributes = try fileManager.attributesOfItem(atPath: path1.relativePath)
            let date1 = (path1Attributes[.creationDate] as? Date) ?? Date.distantPast

            let path2Attributes = try fileManager.attributesOfItem(atPath: path2.relativePath)
            let date2 = (path2Attributes[.creationDate] as? Date) ?? Date.distantPast

            return date1 > date2
        }

        return sortedFilePaths ?? []
    }

    // Maximum file size for writing and reading logs.
    static let logMaximumFileSize: UInt64 = 131_072 // 128 kB.

    /// Privacy policy URL.
    static let privacyPolicyURL = URL(string: "https://\(Self.hostName)/help/privacy-policy/")!

    /// Make a start regarding  policy URL.
    static let privacyGuidesURL = URL(string: "https://\(Self.hostName)/help/first-steps-towards-online-privacy/")!

    /// FAQ & Guides URL.
    static let faqAndGuidesURL = URL(string: "https://\(Self.hostName)/help/tag/mullvad-app/")!

    /// Maximum number of devices per account.
    static let maxAllowedDevices = 5
}
