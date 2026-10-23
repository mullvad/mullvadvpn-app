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

struct AccessMethodTypeView<ViewModel: AccessMethodTypeViewModelProtocol>: View {
    let viewModel: ViewModel
    var body: some View {
        SingleChoiceList(
            options: viewModel.list,
            value: viewModel.current
        )
        .toolbar(.visible, for: .navigationBar)
        .tint(Color.mullvadTextPrimary)
    }
}
