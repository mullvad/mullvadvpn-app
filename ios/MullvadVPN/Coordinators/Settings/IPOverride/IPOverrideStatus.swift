//
//  IPOverrideStatus.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum IPOverrideStatus: Equatable, CustomStringConvertible {
    case active, noImports
    case importSuccessful(Context)
    case importFailed(Context)

    enum Context: Equatable {
        case text
        case file(fileName: String)
    }

    var title: String {
        switch self {
        case .active:
            NSLocalizedString("OVERRIDES ACTIVE", comment: "")
        case .noImports, .importFailed:
            NSLocalizedString("NO OVERRIDES IMPORTED", comment: "")
        case .importSuccessful:
            NSLocalizedString("IMPORT SUCCESSFUL", comment: "")
        }
    }

    var icon: UIImage? {
        let titleConfiguration = UIImage.SymbolConfiguration(textStyle: .body)
        let weightConfiguration = UIImage.SymbolConfiguration(weight: .bold)
        let combinedConfiguration = titleConfiguration.applying(weightConfiguration)

        switch self {
        case .active, .noImports:
            return nil
        case .importFailed:
            return UIImage(systemName: "xmark", withConfiguration: combinedConfiguration)?
                .withRenderingMode(.alwaysOriginal)
                .withTintColor(.dangerColor)
        case .importSuccessful:
            return UIImage(systemName: "checkmark", withConfiguration: combinedConfiguration)?
                .withRenderingMode(.alwaysOriginal)
                .withTintColor(.successColor)
        }
    }

    var description: String {
        switch self {
        case .active, .noImports:
            ""
        case let .importFailed(context):
            switch context {
            case .file(let fileName):
                String(
                    format: NSLocalizedString("Import of %@ was unsuccessful, please try again.", comment: ""),
                    fileName
                )
            case .text:
                NSLocalizedString("Import of text was unsuccessful, please try again.", comment: "")
            }
        case let .importSuccessful(context):
            switch context {
            case .file(let fileName):
                String(
                    format: NSLocalizedString("Import of %@ was successful, overrides are now active.", comment: ""),
                    fileName
                )
            case .text:
                NSLocalizedString("Import of text was successful, overrides are now active.", comment: "")
            }
        }
    }
}
