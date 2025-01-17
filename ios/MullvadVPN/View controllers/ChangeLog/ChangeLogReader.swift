//
//  ChangeLogReader.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-01-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation

protocol ChangeLogReaderProtocol {
    func read() throws -> [String]
}

struct ChangeLogReader: ChangeLogReaderProtocol {
    /**
     Reads change log file from bundle and returns its contents as a string.
     */
    func read() throws -> [String] {
        try String(contentsOfFile: try getPathToChangesFile())
            .split(whereSeparator: { $0.isNewline })
            .compactMap { line in
                let trimmedString = line.trimmingCharacters(in: .whitespaces)

                return trimmedString.isEmpty ? nil : trimmedString
            }
    }

    /**
     Returns path to change log file in bundle.
     */
    private func getPathToChangesFile() throws -> String {
        if let filePath = Bundle.main.path(forResource: "changes", ofType: "txt") {
            return filePath
        } else {
            throw CocoaError(.fileNoSuchFile)
        }
    }
}
