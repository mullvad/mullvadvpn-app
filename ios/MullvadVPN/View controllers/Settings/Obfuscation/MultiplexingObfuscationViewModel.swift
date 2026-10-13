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

protocol MultiplexObfuscationSettingsViewModel {
    var socketCount: Binding<UInt8> { get }
    func validateCount(count: UInt8) -> UInt8?
}

/// ** The live view model which interfaces with the TunnelManager  */
class MultiplexImplObfuscationSettingsViewModel: MultiplexObfuscationSettingsViewModel {
    var socketCount: Binding<UInt8>

    init(socketCount: Binding<UInt8>) {
        self.socketCount = socketCount
    }

    func validateCount(count: UInt8) -> UInt8? {
        count > 0 ? count : nil
    }
}
