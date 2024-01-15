//
//  IPOverrideStatus.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum IPOverrideStatus {
    case active, noImports, importSuccessful(Context), importFailed(Context)

    enum Context {
        case file, text

        var description: String {
            switch self {
            case .file: "of file"
            case .text: "via text"
            }
        }
    }

    var statusTitle: String {
        switch self {
        case .active:
            NSLocalizedString(
                "IP_OVERRIDE_STATUS_TITLE_ACTIVE",
                tableName: "IPOverride",
                value: "Overrides active",
                comment: ""
            )
        case .noImports, .importFailed:
            NSLocalizedString(
                "IP_OVERRIDE_STATUS_TITLE_NO_IMPORTS",
                tableName: "IPOverride",
                value: "No overrides imported",
                comment: ""
            )
        case .importSuccessful:
            NSLocalizedString(
                "IP_OVERRIDE_STATUS_TITLE_IMPORT_SUCCESSFUL",
                tableName: "IPOverride",
                value: "Import successful",
                comment: ""
            )
        }
    }

    var statusIcon: UIImage? {
        let titleConfiguration = UIImage.SymbolConfiguration(textStyle: .title2)
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

    var statusDescription: String? {
        switch self {
        case .active, .noImports:
            nil
        case let .importFailed(context):
            NSLocalizedString(
                "IP_OVERRIDE_STATUS_DESCRIPTION_INACTIVE",
                tableName: "IPOverride",
                value: "Import \(context.description) was unsuccessful, please try again.",
                comment: ""
            )
        case let .importSuccessful(context):
            NSLocalizedString(
                "IP_OVERRIDE_STATUS_DESCRIPTION_INACTIVE",
                tableName: "IPOverride",
                value: "Import \(context.description) was successful, overrides are now active.",
                comment: ""
            )
        }
    }
}
