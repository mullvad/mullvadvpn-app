//
//  ProblemReportViewModel.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-02-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct ProblemReportViewModel {
    let email: String
    let message: String

    let navigationTitle = NSLocalizedString(
        "NAVIGATION_TITLE",
        tableName: "ProblemReport",
        value: "Report a problem",
        comment: ""
    )

    let subheadLabelText = NSLocalizedString(
        "SUBHEAD_LABEL",
        tableName: "ProblemReport",
        value: """
        To help you more effectively, your app’s log file will be attached to \
        this message. Your data will remain secure and private, as it is anonymised \
        before being sent over an encrypted channel.
        """,
        comment: ""
    )

    let emailPlaceholderText = NSLocalizedString(
        "EMAIL_TEXTFIELD_PLACEHOLDER",
        tableName: "ProblemReport",
        value: "Your email (optional)",
        comment: ""
    )

    let messageTextViewPlaceholder = NSLocalizedString(
        "DESCRIPTION_TEXTVIEW_PLACEHOLDER",
        tableName: "ProblemReport",
        value: """
        To assist you better, please write in English or Swedish and \
        include which country you are connecting from.
        """,
        comment: ""
    )

    let viewLogsButtonTitle = NSLocalizedString(
        "VIEW_APP_LOGS_BUTTON_TITLE",
        tableName: "ProblemReport",
        value: "View app logs",
        comment: ""
    )

    let sendLogsButtonTitle = NSLocalizedString(
        "SEND_BUTTON_TITLE",
        tableName: "ProblemReport",
        value: "Send",
        comment: ""
    )

    let emptyEmailAlertWarning = NSLocalizedString(
        "EMPTY_EMAIL_ALERT_MESSAGE",
        tableName: "ProblemReport",
        value: """
        You are about to send the problem report without a way for us to get back to you. \
        If you want an answer to your report you will have to enter an email address.
        """,
        comment: ""
    )

    let confirmEmptyEmailTitle = NSLocalizedString(
        "EMPTY_EMAIL_ALERT_SEND_ANYWAY_ACTION",
        tableName: "ProblemReport",
        value: "Send anyway",
        comment: ""
    )

    let cancelEmptyEmailTitle = NSLocalizedString(
        "EMPTY_EMAIL_ALERT_CANCEL_ACTION",
        tableName: "ProblemReport",
        value: "Cancel",
        comment: ""
    )

    init() {
        email = ""
        message = ""
    }

    init(email: String, message: String) {
        self.email = email.trimmingCharacters(in: .whitespacesAndNewlines)
        self.message = message.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var isValid: Bool {
        !message.isEmpty
    }
}
