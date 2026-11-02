// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Network

public enum ApplicationConfiguration {
    static var hostName: String {
        Bundle.main.object(forInfoDictionaryKey: "HostName") as! String
    }

    /// Shared container security group identifier.
    static var securityGroupIdentifier: String {
        Bundle.main.object(forInfoDictionaryKey: "ApplicationSecurityGroupIdentifier") as! String
    }

    /// Container URL for security group.
    ///
    /// On tvOS the root of the shared container is not writable — `Library/Caches` is the only
    /// place in it an app may write — so everything the app and the packet tunnel share through
    /// the container (relay cache, log files, Shadowsocks cache) lives there instead. Both
    /// processes resolve this the same way, so they still agree on the paths.
    ///
    /// - Important: tvOS may purge this directory. Nothing in it can be treated as durable
    ///   storage; the relay cache already copes by refetching, and settings live in the keychain.
    public static var containerURL: URL {
        let containerURL = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: securityGroupIdentifier)!

        #if os(tvOS)
            let writableURL =
                containerURL
                .appendingPathComponent("Library", isDirectory: true)
                .appendingPathComponent("Caches", isDirectory: true)
            try? FileManager.default.createDirectory(at: writableURL, withIntermediateDirectories: true)
            return writableURL
        #else
            return containerURL
        #endif
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

        let filteredFilePaths: [URL] =
            filePathsInDirectory?.compactMap { path in
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
    static let logMaximumFileSize: UInt64 = 131_072  // 128 kB.

    /// Privacy policy URL.
    static func privacyPolicyLink() -> String {
        "https://\(Self.hostName)/help/privacy-policy/"
    }

    /// Make a start regarding  policy URL.
    static func privacyGuidesURL() -> URL {
        URL(string: "https://\(Self.hostName)/help/first-steps-towards-online-privacy/")!
    }

    /// FAQ & Guides URL.
    static func faqAndGuidesURL() -> URL {
        URL(string: "https://\(Self.hostName)/help/tag/mullvad-app/")!
    }

    /// Maximum number of devices per account.
    static let maxAllowedDevices = 5
}
