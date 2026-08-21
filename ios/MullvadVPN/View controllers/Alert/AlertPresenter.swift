// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Routing

@MainActor
struct AlertPresenter {
    weak var context: (any Presenting)?

    func showAlert(presentation: AlertPresentation, animated: Bool) {
        guard let context else { return }

        context.applicationRouter?.presentAlert(
            route: .alert(presentation.id),
            animated: animated,
            metadata: AlertMetadata(presentation: presentation, context: context)
        )
    }

    func dismissAlert(presentation: AlertPresentation, animated: Bool) {
        context?.applicationRouter?.dismiss(.alert(presentation.id), animated: animated)
    }
}

extension ApplicationRouter {
    func presentAlert(route: RouteType, animated: Bool, metadata: AlertMetadata) {
        present(route, animated: animated, metadata: metadata)
    }
}
