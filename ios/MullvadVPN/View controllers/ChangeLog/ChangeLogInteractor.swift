//
//  ChangeLogInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import UIKit.NSAttributedString
import UIKit.UIFont

final class ChangeLogInteractor {
    private let logger = Logger(label: "ChangeLogInteractor")
    private var bulletList = ""
    private let bullet = "•  "
    var viewModel: ChangeLogViewModel {
        let font = UIFont.preferredFont(forTextStyle: .body)
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineBreakMode = .byWordWrapping
        paragraphStyle.headIndent = bullet.size(withAttributes: [.font: font]).width

        return ChangeLogViewModel(
            body: NSAttributedString(
                string: bulletList,
                attributes: [
                    .paragraphStyle: paragraphStyle,
                    .font: font,
                    .foregroundColor: UIColor.white.withAlphaComponent(0.8),
                ]
            )
        )
    }

    init() {
        do {
            let string = try readFromFile()
            self.bulletList = string.split(whereSeparator: { $0.isNewline })
                .map { "\(bullet)\($0)" }
                .joined(separator: "\n")
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

struct ChangeLogViewModel {
    let header: String = Bundle.main.shortVersion
    let title: String = NSLocalizedString(
        "CHANGE_LOG_TITLE",
        tableName: "Account",
        value: "Changes in this version:",
        comment: ""
    )
    var body: NSAttributedString
}
