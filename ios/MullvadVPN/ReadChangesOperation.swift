//
//  ReadChangesOperation.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-15.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Operations

/// Change log file name.
private let fileName = "changes.txt"

final class ReadChangesOperation: ResultOperation<[String], Error> {
    override func main() {
        dispatchPrecondition(condition: .notOnQueue(.main))

        do {
            guard let changesFileURL = Bundle.main.url(forResource: fileName, withExtension: nil)
            else {
                finish(completion: .failure(URLError(.fileDoesNotExist)))
                return
            }

            let changes = try String(contentsOf: changesFileURL, encoding: .utf8)
                // Consider each line as a new entry.
                .split(separator: "\n")
                // Remove white spaces.
                .map { $0.trimmingCharacters(in: .whitespaces) }
                // Remove empty entries.
                .filter { !$0.isEmpty }

            finish(completion: .success(changes))
        } catch {
            finish(completion: .failure(error))
        }
    }
}
