// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

// this will replace ProblemReportViewModel and be renamed to it, in the fullness of time

import SwiftUI

@Observable class ProblemReportViewModelNew {
    var email: String = ""
    var message: String = ""
    var includeAccountTokenInLogs: Bool = false

    var canSend: Bool {
        !email.isEmpty && !message.isEmpty
    }
}
