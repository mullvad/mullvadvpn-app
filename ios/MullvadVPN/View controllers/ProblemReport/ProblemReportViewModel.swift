//
//  ProblemReportViewModel.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-02-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct ProblemReportViewModel {
    let email: String
    let message: String
    let includeAccountTokenInLogs: Bool

    static let navigationTitle = NSLocalizedString("Report a problem", comment: "")

    static let subheadLabelText = NSLocalizedString(
        "To help you more effectively, your app’s log file will be attached to this message. "
            + "Your data will remain secure and private, as it is anonymised before being "
            + "sent over an encrypted channel.",
        comment: ""
    )

    static let userPrivacyWarningText = NSLocalizedString(
        "Include my account token for faster help with payment or account related issues",
        comment: ""
    )

    static let emailPlaceholderText = NSLocalizedString("Your email (optional)", comment: "")

    static let messageTextViewPlaceholder = NSLocalizedString(
        "To assist you better, please write in English or Swedish and include which country you are connecting from.",
        comment: ""
    )

    static let viewLogsButtonTitle = NSLocalizedString("View app logs", comment: "")

    static let sendLogsButtonTitle = NSLocalizedString("Send", comment: "")

    static let emptyEmailAlertWarning = NSLocalizedString(
        "You are about to send the problem report without a way for us to get back to you. "
            + "If you want an answer to your report you will have to enter an email address.",
        comment: ""
    )

    static let confirmEmptyEmailTitle = NSLocalizedString("Send anyway", comment: "")

    static let cancelEmptyEmailTitle = NSLocalizedString("Cancel", comment: "")

    init() {
        email = ""
        message = ""
        includeAccountTokenInLogs = false
    }

    init(email: String, message: String, includeAccountTokenInLogs: Bool) {
        self.email = email.trimmingCharacters(in: .whitespacesAndNewlines)
        self.message = message.trimmingCharacters(in: .whitespacesAndNewlines)
        self.includeAccountTokenInLogs = includeAccountTokenInLogs
    }

    var isValid: Bool {
        !message.isEmpty
    }
}
