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

protocol BreadcrumbsObserver: AnyObject {
    func didUpdateBreadcrumbs(_ breadcrumbs: Set<Breadcrumb>)
}

final class BreadcrumbsBlockObserver: BreadcrumbsObserver, @unchecked Sendable {
    typealias DidUpdateBreadcrumbsHandler = (Set<Breadcrumb>) -> Void

    private let didUpdateBreadcrumbsHandler: DidUpdateBreadcrumbsHandler?

    init(didUpdateBreadcrumbsHandler: DidUpdateBreadcrumbsHandler?) {
        self.didUpdateBreadcrumbsHandler = didUpdateBreadcrumbsHandler
    }

    func didUpdateBreadcrumbs(_ breadcrumbs: Set<Breadcrumb>) {
        didUpdateBreadcrumbsHandler?(breadcrumbs)
    }
}

final class BreadcrumbsProvider {
    private let observerList = ObserverList<BreadcrumbsObserver>()
    private(set) var breadcrumbs: Set<Breadcrumb> = []

    func add(breadcrumb: Breadcrumb) {
        breadcrumbs.insert(breadcrumb)
        observerList.notify {
            $0.didUpdateBreadcrumbs(breadcrumbs)
        }
    }

    func remove(breadcrumb: Breadcrumb) {
        breadcrumbs.remove(breadcrumb)
        observerList.notify {
            $0.didUpdateBreadcrumbs(breadcrumbs)
        }
    }

    func add(observer: BreadcrumbsObserver) {
        observerList.append(observer)
    }

    func remove(observer: BreadcrumbsObserver) {
        observerList.remove(observer)
    }
}
