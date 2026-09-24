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
        let state: ProblemReportViewModelNew.ModalState

        let supportEmail = "support@mullvadvpn.net"

        var body: some View {
            VStack {
                switch state {
                case .sending:
                    MullvadNoticeView(
                        viewModel: .init(
                            style: .loading,
                            title: .init(text: "Sending...", style: .headline(.bold, alignment: .leading)),
                            verticalAlignment: .top,
                            details: [],
                            actions: [
                                .init(style: .primary, state: .init(kind: .idle, message: "Cancel"))
                            ]))
                case .success:
                    MullvadNoticeView(
                        viewModel: .init(
                            style: .success,
                            title: .init(text: "Sent", style: .headline(.bold, alignment: .leading)),
                            verticalAlignment: .top,
                            details: [
                                .init(
                                    text: "Thanks! We will look into this.",
                                    style: .primary(.none, alignment: .leading))
                            ],
                            actions: [])
                    )
                case .failure:
                    MullvadNoticeView(
                        viewModel: .init(
                            style: .fail,
                            title: .init(text: "Failed to send", style: .headline(.bold, alignment: .leading)),
                            details: [
                                .init(
                                    text:
                                        "If you exit the form and try again later, the information you already entered will still be here.",
                                    style: .primary(.none, alignment: .leading)
                                ),
                                .init(
                                    text:
                                        "If you still experience issues you can email our support directly at **\(supportEmail)**. Please attach your app log to your email.",
                                    style: .primary(.none, alignment: .leading)
                                ),
                            ],
                            actions: [
                                .init(style: .primary, state: .init(kind: .idle, message: "Edit message")),
                                .init(style: .primary, state: .init(kind: .idle, message: "View app logs")),
                                .init(style: .success, state: .init(kind: .idle, message: "Try again")),
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
