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
    let viewModel: any ConfigureApiAccessMethodViewModelProtocol
    var focusedInput: FocusState<AccessMethodField?>.Binding

    var body: some View {
        switch viewModel.accessMethodViewModel.method {
        case .shadowsocks:
            ShadowSocksServerDetailsView(viewModel: viewModel, focusedInput: focusedInput)
        case .socks5:
            Socks5ServerDetailsView(viewModel: viewModel, focusedInput: focusedInput)
        default:
            EmptyView()
        }
    }

}
