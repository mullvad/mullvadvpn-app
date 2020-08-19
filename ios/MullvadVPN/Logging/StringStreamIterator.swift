//
//  StringStreamIterator.swift
//  MullvadVPN
//
//  Created by pronebird on 17/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if DEBUG

import Foundation

class StringStreamIterator<Codec>: IteratorProtocol where Codec: UnicodeCodec {
    let separator: Character

    private var string = ""
    private var data = [Codec.CodeUnit]()
    private var parser = Codec.ForwardParser()

    init(separator: Character) {
        self.separator = separator
    }

    func append<S>(bytes: S) where S: Sequence, S.Element == Codec.CodeUnit {
        data.append(contentsOf: bytes)
    }

    func next() -> String? {
        var dataIterator = data.makeIterator()
        var bytesRead = 0

        defer {
            if bytesRead > 0 {
                data.removeSubrange(..<bytesRead)
            }
        }

        while case .valid(let encodedScalar) = parser.parseScalar(from: &dataIterator) {
            let unicodeScalar = Codec.decode(encodedScalar)
            let character = Character(unicodeScalar)

            bytesRead += encodedScalar.count

            if character == separator {
                let returnString = string
                string = ""

                return returnString
            } else {
                string.append(character)
            }
        }

        return nil
    }
}

#endif
