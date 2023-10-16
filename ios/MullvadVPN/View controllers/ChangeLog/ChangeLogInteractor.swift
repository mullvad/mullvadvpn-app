//
//  ChangeLogInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

final class ChangeLogInteractor {
    private let logger = Logger(label: "ChangeLogInteractor")
    private var items: [String] = []

    var hasNewChanges: Bool {
        !items.isEmpty
    }

    var viewModel: ChangeLogViewModel {
        return ChangeLogViewModel(
            body: items
        )
    }

    init() {
        do {
            let string = try readFromFile()
            items = string.split(whereSeparator: { $0.isNewline }).map { String($0) }
        } catch {
            logger.error(error: error, message: "Cannot read change log from bundle.")
        }
    }

    /**
     Reads change log file from bundle and returns its contents as a string.
     */
    private func readFromFile() throws -> String {
        try String(contentsOfFile: try getPathToChangesFile())
            .split(whereSeparator: { $0.isNewline })
            .compactMap { line in
                let trimmedString = line.trimmingCharacters(in: .whitespaces)

                return trimmedString.isEmpty ? nil : trimmedString
            }
            .joined(separator: "\n")
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
