//
//  MullvadStateView+Preview.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-05-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct MullvadStateViewPreviewWrapper: View {
    static let p1 =
        "This is sample placeholder text used to demonstrate how content will appear in a layout. It helps visualize spacing, typography, and overall design before the final content is available."

    @State private var currentPage = 0
    @State private var isLoading: Bool = true
    @StateObject private var loadingViewModel: StateViewModel

    init() {
        let actionButton = MullvadStateView.ActionItem(
            style: .default,
            state: .init(kind: .idle, message: "Stop loading")
        )

        let viewModel = StateViewModel(
            style: .loading,
            title: MullvadStateView.TextItem(
                text: "loading",
                style: .headline(.bold)
            ),
            banner: Image.mullvadUniqueFilterBanner,
            details: [
                MullvadStateView.TextItem(text: Self.p1, style: .primary(.bold)),
                MullvadStateView.TextItem(text: Self.p1, style: .primary(.bold)),
                MullvadStateView.TextItem(text: Self.p1, style: .secondary(.boldItalic)),
            ],
            actions: [actionButton]
        )

        _loadingViewModel = StateObject(wrappedValue: viewModel)

        actionButton.onTap = { [weak viewModel] in
            viewModel?.style = .success
        }
    }

    var body: some View {
        let view1 = MullvadStateView(
            viewModel: StateViewModel(
                style: .info,
                title: MullvadStateView.TextItem(text: "info", style: .headline(.bold)),
                details: [
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.none)),
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.none)),
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.none)),
                ]
            )
        )

        let view2 = MullvadStateView(
            viewModel: StateViewModel(
                style: .error,
                title: MullvadStateView.TextItem(text: "error", style: .headline(.bold)),
                details: [
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.none)),
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.bold)),
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.italic)),
                ]
            )
        )

        let view3 = MullvadStateView(
            viewModel: StateViewModel(
                style: .success,
                title: MullvadStateView.TextItem(text: "Success", style: .headline(.bold)),
                details: [
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.none)),
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.bold)),
                    MullvadStateView.TextItem(text: Self.p1, style: .primary(.boldItalic)),
                ]
            )
        )

        let view4 = MullvadStateView(viewModel: loadingViewModel)

        let view5 = MullvadStateView(
            viewModel: StateViewModel(
                style: .custom(.init(image: Image.mullvadIconMultihopWhenNeeded)),
                title: MullvadStateView.TextItem(text: "Custom state", style: .secondary(alignment: .center)),
                details: [
                    MullvadStateView.TextItem(text: Self.p1, style: .secondary(alignment: .center)),
                ],
                actions: [
                    MullvadStateView.ActionItem(style: .default, state: .init(kind: .idle, message: "Action"))
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
