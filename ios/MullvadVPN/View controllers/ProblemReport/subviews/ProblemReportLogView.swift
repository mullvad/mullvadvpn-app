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
    struct LogView: View {

        @State var viewModel: ProblemReportViewModel
        @ScaledMetric var doneButtonWidth: CGFloat = 48

        var body: some View {
            VStack {
                ZStack {
                    HStack {
                        Spacer()
                        Text("App Logs")
                            .lineLimit(1)
                            .font(.mullvadSmallSemiBold)
                        Spacer()
                    }.padding(.horizontal, doneButtonWidth)
                    HStack {
                        Spacer()
                        Button(
                            "Done",
                            action: {
                                viewModel.showLogs.wrappedValue = false
                            })
                    }
                }
                .padding(16)
                .foregroundStyle(.white)
                .background(Color.mullvadBackground)
                TextEditor(text: .constant(viewModel.logText ?? ""))
                    .padding([.leading, .trailing], 16)
                    .padding([.top, .bottom], 24)
                    .scrollContentBackground(.hidden)
                    .background(Color.MullvadLogView.backgroundColor)
                    .foregroundStyle(Color.MullvadLogView.foregroundColor)
            }.background(Color.mullvadBackground)
        }
    }
}

#Preview {
    ProblemReportView.LogView(
        viewModel: {
            let viewModel = ProblemReportViewModel()
            viewModel.logText = """
                xxxxxxxxxxxxxxxxxxxxx
                xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                xxxxxxxxxxxxxxxxxxxxx

                xxxxxxx
                xxxxxxxxxxxxxx
                xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                xxxxxxx
                xxxxxxxxxxxxxx
                xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                xxxxxxx
                xxxxxxxxxxxxxx
                xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                xxxxxxx
                xxxxxxxxxxxxxx
                xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                xxxxxxx
                xxxxxxxxxxxxxx
                xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                xxxxxxxxxxxxxx
                xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                """
            return viewModel
        }())
}
