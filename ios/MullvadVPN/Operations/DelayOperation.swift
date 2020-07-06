//
//  DelayOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum DelayTimerType {
    case deadline
    case walltime
}

class DelayOperation: AsyncOperation {
    private let delay: TimeInterval
    private let timerType: DelayTimerType
    private var timer: DispatchSourceTimer?

    init(delay: TimeInterval, timerType: DelayTimerType) {
        self.delay = delay
        self.timerType = timerType
    }

    override func main() {
        let timer = DispatchSource.makeTimerSource()
        timer.setEventHandler { [weak self] in
            self?.finish()
        }

        switch timerType {
        case .deadline:
            timer.schedule(deadline: DispatchTime.now() + delay)
        case .walltime:
            timer.schedule(wallDeadline: DispatchWallTime.now() + delay)
        }

        self.timer = timer
        timer.activate()
    }

    override func operationDidCancel() {
        timer?.cancel()
        timer = nil
        finish()
    }
}
