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

struct ChangeLogView<ViewModel>: View where ViewModel: ChangeLogViewModelProtocol {
    @ObservedObject var viewModel: ViewModel

    init(viewModel: ViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        ZStack {
            UIColor.secondaryColor.color.ignoresSafeArea()
            VStack {
                Text(viewModel.changeLog?.title ?? "")
                    .font(.title)
                    .fontWeight(.semibold)
                    .foregroundColor(UIColor.primaryTextColor.color)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, UIMetrics.contentInsets.left)
                    .padding(.top, UIMetrics.TableView.emptyHeaderHeight)
                List {
                    ForEach(viewModel.changeLog?.changes ?? [], id: \.self) { item in
                        BulletPointText(text: item)
                            .listRowSeparator(.hidden)
                            .listRowBackground(Color.clear)
                    }
                    .padding(.horizontal, UIMetrics.contentInsets.left)
                }
                .listStyle(.plain)
                .frame(maxHeight: .infinity)
            }
        }
        .onAppear {
            viewModel.getLatestChanges()
        }
    }
}

#Preview {
    ChangeLogView(viewModel: MockChangeLogViewModel())
}
