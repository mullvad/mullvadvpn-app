//
//  Bundle+MullvadVersion.swift
//  MullvadVPN
//
//  Created by pronebird on 29/11/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation

private let kInfoDictionaryMullvadVersionSuffixKey = "MullvadVersionSuffix"

extension Bundle {

    var mullvadVersion: String? {
        let shortVersion = infoDictionary?["CFBundleShortVersionString"] as? String
        let versionSuffix = infoDictionary?[kInfoDictionaryMullvadVersionSuffixKey] as? String

        return [shortVersion, versionSuffix]
            .compactMap { $0 }
            .joined(separator: "-")
    }

}
