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

/// A protocol providing error a way to override error description when printing error chain.
public protocol CustomErrorDescriptionProtocol {
    /// A custom error description that overrides `localizedDescription` when printing error chain.
    var customErrorDescription: String? { get }
}
