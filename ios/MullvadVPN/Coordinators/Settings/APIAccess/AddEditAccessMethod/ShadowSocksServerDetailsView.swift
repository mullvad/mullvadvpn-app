//// This Source Code Form is subject to the terms of the GPLv3 License.
//// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
////
//// This file incorporates work covered by the following copyright and
//// permission notice:
////
////   Copyright (c) Mullvad VPN AB. All rights reserved.
////
//// SPDX-License-Identifier: GPL-3.0-only
//
//import SwiftUI
//
//struct ShadowSocksServerDetailsView : some View {
//    var body: some View {
//
//    }
//
//    private func makeListItem(
//        _ item: ListAccessMethodItem,
//        at index: Int
//    ) -> some View {
//        SegmentedListItem(
//            isLastInList: index == viewModel.items.count - 1,
//            accessibilityIdentifier: item.accessibilityIdentifier,
//            leadingAndTrailingDestination: {
//                if let viewModel = factory.makeEditMethodViewModel(for: item) {
//                    EditAccessMethodView(viewModel: viewModel)
//                } else {
//                    EmptyView()
//                }
//            },
//            leading: {
//                if let detail = item.detail, !detail.isEmpty {
//                    itemFactory.leading(for: .generic(title: item.name, subtitle: detail))
//                } else {
//                    itemFactory.leading(for: .generic(title: item.name))
//                }
//            },
//            trailing: {
//                itemFactory.trailing(
//                    for: .drillDown(
//                        title: viewModel.getStatus(for: item)?.description ?? "",
//                        breadcrumb: nil
//                    )
//                )
//            }
//        )
//    }
//}
