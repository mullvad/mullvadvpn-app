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

        @State var viewModel: ProblemReportViewModelNew

        var body: some View {
            VStack {
                ZStack {
                    HStack {
                        Spacer()
                        Text("App Logs")
                            .font(Font.system(.title2, design: .default))
                        Spacer()
                    }
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
            let viewModel = ProblemReportViewModelNew()
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
