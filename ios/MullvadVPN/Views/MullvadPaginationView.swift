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

struct MullvadPaginationView: View {
    let pages: [AnyView]
    @Binding var currentPage: Int

    init(pages: [any View], currentPage: Binding<Int>) {
        self.pages = pages.map { AnyView($0) }
        self._currentPage = currentPage
    }

    var body: some View {
        VStack(spacing: 8) {
            TabView(selection: $currentPage) {
                ForEach(pages.indices, id: \.self) { index in
                    pages[index]
                        .tag(index)
                }
            }
            .tabViewStyle(.page(indexDisplayMode: .never))

            if pages.count > 1 {
                HStack(spacing: 8) {
                    ForEach(pages.indices, id: \.self) { index in
                        Circle()
                            .fill(currentPage == index ? Color.mullvadTextPrimary : Color.mullvadTextSecondary)
                            .frame(width: 8, height: 8)
                    }
                }
            }
        }
        .padding(.bottom, 8)
    }
}
