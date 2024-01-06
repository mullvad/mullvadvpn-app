//
//  AccessMethodRepositoryStub.swift
//  MullvadRESTTests
//
//  Created by Mojgan on 2024-01-02.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

typealias PersistentAccessMethod = MullvadSettings.PersistentAccessMethod
class AccessMethodRepositoryStub: AccessMethodRepositoryDataSource {
    var accessMethods: [PersistentAccessMethod] {
        _accessMethods
    }

    var _accessMethods: [PersistentAccessMethod]

    init(accessMethods: [PersistentAccessMethod]) {
        _accessMethods = accessMethods
    }
}
