// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

extension AccessMethodViewModel {
    /// Title suitable for navigation item.
    /// User-defined name is preferred unless it's blank, in which case the name of access method is used instead.
    var navigationItemTitle: String {
        if name.trimmingCharacters(in: .whitespaces).isEmpty {
            method.localizedDescription
        } else {
            name
        }
    }
}
