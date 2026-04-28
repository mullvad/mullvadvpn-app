//
//  LogRedaction.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public class LogRedaction {
    private var logRedactor: LogRedactor
    init() {
        logRedactor = init_log_redactor()
    }

    deinit {
        drop_log_redactor(logRedactor)
    }

    func redact(string: String) -> String {
        let ptr = redact_log(logRedactor, string)
        guard ptr != nil else {
            return string
        }
        return String(cString: ptr!)
    }
}
