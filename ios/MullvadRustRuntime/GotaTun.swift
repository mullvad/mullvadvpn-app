//
//  GotaTun.swift
//  MullvadRustRuntime
//
//  Created by Emils on 31/10/2025.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntimeProxy

public class GotaTun {
    public enum Error: Swift.Error {
        case startFailed(Int32)
    }
    private var handle: SwiftGotaTun = SwiftGotaTun(_0: nil)

    public init() {}

    public func start(tunnelFileDescriptor: Int32, configuration: GotaTunConfig) throws {
        let status = mullvad_ios_gotatun_start(&handle, configuration.handle, tunnelFileDescriptor, )
        if status != 0 {
            throw Error.startFailed(status)
        }
    }

    public func stop() {
        mullvad_ios_gotatun_stop(handle)
        mullvad_ios_gotatun_drop(handle)
    }

    deinit {
        mullvad_ios_gotatun_drop(handle)
    }
}

public class GotaTunConfig {
    fileprivate var handle = mullvad_ios_gotatun_config_new()

    public init() {}

    public func addEntry() {

    }

    public func addExit(privateKey: Data, preSharedKey: Data?, publicKey: Data, endpoint: String) {
        var preSharedKey = preSharedKey
        if preSharedKey == nil {
            preSharedKey = Data(repeating: 0, count: 32)
        }
        mullvad_ios_gotatun_config_set_exit(
            handle, privateKey.map { $0 }, preSharedKey?.map { $0 }, publicKey.map { $0 }, endpoint)
    }

    deinit {
        mullvad_ios_gotatun_config_drop(handle)
    }

}
