//
//  BackgroundTask.swift
//  MullvadVPN
//
//  Created by pronebird on 09/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/**
 Background tasks defined by the app.

 When adding new background tasks, don't forget to update `BGTaskSchedulerPermittedIdentifiers` key in `Info.plist` by adding a task identifier
 using the following pattern:

 ```
 $(APPLICATION_IDENTIFIER).<TaskName>
 ```

 Note that `<TaskName>` is capitalized in plist, but the label for enum case should start with a lowercase letter.
 */
enum BackgroundTask: String {
    case appRefresh, privateKeyRotation, addressCacheUpdate

    /// Returns background task identifier.
    /// Use it when registering or scheduling tasks with `BGTaskScheduler`.
    var identifier: String {
        "\(ApplicationTarget.mainApp.bundleIdentifier).\(capitalizedRawValue)"
    }

    private var capitalizedRawValue: String {
        rawValue.prefix(1).uppercased() + rawValue.dropFirst()
    }
}
