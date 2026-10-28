// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import SwiftUI

struct ServerDetailsSection: View {
    let itemFactory = SegmentedListItemFactory()
    let viewModel: any ConfigureApiAccessMethodViewModelProtocol

    private var typeBinding: Binding<AccessMethodKind> {
        Binding(
            get: { viewModel.accessMethodViewModel.method },
            set: { viewModel.accessMethodViewModel.method = $0 }
        )
    }

    var body: some View {
        switch typeBinding.wrappedValue {
        case .shadowsocks:
            ShadowSocksServerDetailsView(viewModel: viewModel)
        case .socks5:
            Socks5ServerDeitalsView(viewModel: viewModel)
        default:
            EmptyView()
        }
    }

}
