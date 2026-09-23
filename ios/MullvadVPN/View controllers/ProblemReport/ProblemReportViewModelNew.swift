// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

// this will replace ProblemReportViewModel and be renamed to it, in the fullness of time

import SwiftUI

@MainActor
@Observable final class ProblemReportViewModelNew {
    enum ModalState {
        case sending
        case success
        case failure
    }

    var email: String = ""
    var message: String = ""
    var includeAccountTokenInLogs: Bool = false

    var modalState: ModalState?
    var logText: String?
    var showLogs: Binding<Bool>!
    var alert: MullvadAlert?

    var interactor: ProblemReportInteractorProtocol?

    init() {
        showLogs = Binding<Bool>(
            get: { self.logText != nil },
            set: { if !$0 { self.logText = nil } }
        )
    }

    func doShowLog() {
        Task {
            self.logText = await interactor?.fetchReportString()
        }
    }

    var canSend: Bool {
        !message.isEmpty
    }

    func submitForm() {
        if email.isEmpty {
            alert = MullvadAlert(
                type: .warning,
                messages: [
                    """
                    You are about to send the problem report without a way \
                    for us to get back to you. If you want an answer to your \
                    report you will have to enter an email address.
                    """
                ],
                actions: [
                    .init(
                        type: .destructivePrimary,
                        title: "Send anyway"
                    ) {
                        [weak self] in
                        self?.alert = nil
                        self?.doSend()
                    },
                    .init(
                        type: .primary,
                        title: "Cancel"
                    ) { [weak self] in
                        self?.alert = nil
                    },
                ]
            )
        } else {
            doSend()
        }
    }

    private func doSend() {
        modalState = .sending
        Task { [self] in
            do {
                try await interactor?.sendReport(
                    email: email,
                    message: message,
                    includeAccountTokenInLogs: includeAccountTokenInLogs
                )
                modalState = .success
            } catch {
                modalState = .failure
            }
        }
    }
}
