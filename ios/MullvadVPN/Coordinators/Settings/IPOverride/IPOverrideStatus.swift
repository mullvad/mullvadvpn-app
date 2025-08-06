//
//  IPOverrideStatus.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum IPOverrideStatus: Equatable, CustomStringConvertible {
    case active, noImports, importSuccessful(Context), importFailed(Context)

    enum Context {
        case file, text

        // Used in "statusDescription" below to form a complete sentence and therefore not localized here.
        var description: String {
            switch self {
            case .file: "of file"
            case .text: "via text"
            }
        }
    }

    var title: String {
        switch self {
        case .active:
            NSLocalizedString(
                "IP_OVERRIDE_STATUS_TITLE_ACTIVE",
                value: "Overrides active",
                comment: ""
            )
        case .noImports, .importFailed:
            NSLocalizedString(
                "IP_OVERRIDE_STATUS_TITLE_NO_IMPORTS",
                value: "No overrides imported",
                comment: ""
            )
        case .importSuccessful:
            NSLocalizedString(
                "IP_OVERRIDE_STATUS_TITLE_IMPORT_SUCCESSFUL",
                value: "Import successful",
                comment: ""
            )
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
            String(format: NSLocalizedString(
                "IP_OVERRIDE_STATUS_DESCRIPTION_INACTIVE",
                value: "Import %@ was unsuccessful, please try again.",
                comment: ""
            ), context.description)
        case let .importSuccessful(context):
            String(format: NSLocalizedString(
                "IP_OVERRIDE_STATUS_DESCRIPTION_INACTIVE",
                value: "Import %@ was successful, overrides are now active.",
                comment: ""
            ), context.description)
        }
    }
}
