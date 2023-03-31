//
//  ChangeLog.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum ChangeLog {
    private static let userDefaultsKey = "lastSeenChangeLogVersion"

    /**
     Returns `true` if changelog for current application version was already seen by user, otherwise
     `false`.
     */
    static var isSeen: Bool {
        let version = UserDefaults.standard.string(forKey: Self.userDefaultsKey)

        return version == Bundle.main.shortVersion
    }

    /**
     Marks changelog for current application version as seen in user defaults.
     */
    static func markAsSeen() {
        UserDefaults.standard.set(Bundle.main.shortVersion, forKey: Self.userDefaultsKey)
    }

    /**
     Marks changelog as unseen. Removes an entry from user defaults.
     */
    static func markAsUnseen() {
        UserDefaults.standard.removeObject(forKey: Self.userDefaultsKey)
    }

    /**
     Reads changelog file from bundle and returns its contents as a string.
     */
    static func readFromFile() throws -> String {
        return try String(contentsOfFile: try getPathToChangesFile())
            .split(whereSeparator: { $0.isNewline })
            .compactMap { line in
                let trimmedString = line.trimmingCharacters(in: .whitespaces)

                return trimmedString.isEmpty ? nil : trimmedString
            }
            .joined(separator: "\n")
    }

    /**
     Returns path to changelog file in bundle.
     */
    static func getPathToChangesFile() throws -> String {
        if let filePath = Bundle.main.path(forResource: "changes", ofType: "txt") {
            return filePath
        } else {
            throw CocoaError(.fileNoSuchFile)
        }
    }
}
