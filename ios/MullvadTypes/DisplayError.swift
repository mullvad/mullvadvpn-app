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

/// A protocol that adds a formal interface for all errors displayed in user interface.
///
/// This protocol is meant to be used in place of `LocalizedError` when producing a user friendly
/// error message that requires a deeper look at the underlying cause.
///
/// Note that `Logger.error(error: Error)` picks up `errorDescription`s when unrolling
/// the underlying error chain, hence it's better to keep error descriptions relatively concise,
/// explaining what happened but without telling why that happened.
public protocol DisplayError {
    var displayErrorDescription: String? { get }
}
