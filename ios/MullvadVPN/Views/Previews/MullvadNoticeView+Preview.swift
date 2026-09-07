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

struct MullvadNoticeViewPreviewWrapper: View {
    static let p1 =
        "This is sample placeholder text used to demonstrate how content will appear in a layout. It helps visualize spacing, typography, and overall design before the final content is available."

    @State private var currentPage = 0
    @State private var isLoading: Bool = true
    @StateObject private var loadingViewModel: NoticeViewModel

    init() {
        let actionButton = MullvadNoticeView.ActionItem(
            style: .primary,
            state: .init(kind: .idle, message: "Stop loading")
        )

        let viewModel = NoticeViewModel(
            style: .loading,
            title: MullvadNoticeView.TextItem(
                text: "loading",
                style: .headline(.bold)
            ),
            banner: Image.mullvadUniqueFilterBanner,
            details: [
                MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.bold)),
                MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.bold)),
                MullvadNoticeView.TextItem(text: Self.p1, style: .secondary(.boldItalic)),
            ],
            actions: [actionButton]
        )

        _loadingViewModel = StateObject(wrappedValue: viewModel)

        actionButton.onTap = { [weak viewModel] in
            viewModel?.style = .success
        }
    }

    var body: some View {
        let view1 = MullvadNoticeView(
            viewModel: NoticeViewModel(
                style: .success,
                title: MullvadNoticeView.TextItem(text: "info", style: .headline(.bold)),
                details: [
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.none)),
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.none)),
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.none)),
                ]
            )
        )

        let view2 = MullvadNoticeView(
            viewModel: NoticeViewModel(
                style: .error,
                title: MullvadNoticeView.TextItem(text: "error", style: .headline(.bold)),
                details: [
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.none)),
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.bold)),
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.italic)),
                ]
            )
        )

        let view3 = MullvadNoticeView(
            viewModel: NoticeViewModel(
                style: .success,
                title: MullvadNoticeView.TextItem(text: "Success", style: .headline(.bold)),
                details: [
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.none)),
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.bold)),
                    MullvadNoticeView.TextItem(text: Self.p1, style: .primary(.boldItalic)),
                ]
            )
        )

        let view4 = MullvadNoticeView(viewModel: loadingViewModel)

        let view5 = MullvadNoticeView(
            viewModel: NoticeViewModel(
                style: .custom(.init(image: Image.mullvadIconMultihopWhenNeeded)),
                title: MullvadNoticeView.TextItem(text: "Custom state", style: .secondary(alignment: .center)),
                details: [
                    MullvadNoticeView.TextItem(text: Self.p1, style: .secondary(alignment: .center))
                ],
                actions: [
                    MullvadNoticeView.ActionItem(style: .primary, state: .init(kind: .idle, message: "Action"))
                ]
            )
        )

        return MullvadPaginationView(
            pages: [view1, view2, view3, view4, view5],
            currentPage: $currentPage
        )
        .background(Color.mullvadBackground)
    }
}
