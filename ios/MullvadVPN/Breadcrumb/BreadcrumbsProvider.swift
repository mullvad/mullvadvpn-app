//
//  BreadcrumbsProvider.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-03-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
