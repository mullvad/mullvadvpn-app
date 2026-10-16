// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

extension ProblemReportView {
    struct ModalOverlay: View {
        let state: ProblemReportViewModel.ModalState

        let supportEmail = "support@mullvadvpn.net"

        var body: some View {
            VStack {
                switch state {
                case .sending:
                    MullvadNoticeView(
                        viewModel: .init(
                            style: .loading,
                            title: .init(
                                text: NSLocalizedString("Sending...", comment: ""),
                                style: .headline(.bold, alignment: .leading)
                            ),
                            verticalAlignment: .top,
                            details: [],
                            actions: [
                                .init(style: .primary, state: .init(kind: .idle, message: "Cancel"))
                            ]))
                case .success:
                    MullvadNoticeView(
                        viewModel: .init(
                            style: .success,
                            title: .init(
                                text: NSLocalizedString("Sent", comment: ""),
                                style: .headline(.bold, alignment: .leading)
                            ),
                            verticalAlignment: .top,
                            details: [
                                .init(
                                    text: NSLocalizedString("Thanks! We will look into this.", comment: ""),
                                    style: .primary(.none, alignment: .leading))
                            ],
                            actions: [])
                    )
                case .failure:
                    MullvadNoticeView(
                        viewModel: .init(
                            style: .fail,
                            title: .init(
                                text: NSLocalizedString("Failed to send", comment: ""),
                                style: .headline(.bold, alignment: .leading)
                            ),
                            details: [
                                .init(
                                    text:
                                        NSLocalizedString(
                                            "If you exit the form and try again later, the information you already entered will still be here.",
                                            comment: ""),
                                    style: .primary(.none, alignment: .leading)
                                ),
                                .init(
                                    text:
                                        NSLocalizedString(
                                            """
                                            If you still experience issues you can email our support directly at **\(supportEmail)**. \
                                            Please attach your app log to your email.
                                            """,
                                            comment: ""),
                                    style: .primary(.none, alignment: .leading)
                                ),
                            ],
                            actions: [
                                .init(
                                    style: .primary,
                                    state: .init(kind: .idle, message: NSLocalizedString("Edit message", comment: ""))
                                ),
                                .init(
                                    style: .primary,
                                    state: .init(kind: .idle, message: NSLocalizedString("View app logs", comment: ""))
                                ),
                                .init(
                                    style: .success,
                                    state: .init(kind: .idle, message: NSLocalizedString("Try again", comment: ""))
                                ),
                            ])
                    )
                }
            }.background(Color.mullvadBackground)

        }
    }
}

#Preview("sending") {
    ProblemReportView.ModalOverlay(state: .sending)
}

#Preview("sent") {
    ProblemReportView.ModalOverlay(state: .success)
}

#Preview("error") {
    ProblemReportView.ModalOverlay(state: .failure)
}
