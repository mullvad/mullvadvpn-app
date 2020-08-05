//
//  TextFileStream.swift
//  MullvadVPN
//
//  Created by pronebird on 05/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Darwin

class TextFileStream<Codec> where Codec: UnicodeCodec {
    private let readSource: DispatchSourceRead
    private let fileDescriptor: Int32
    private let queue = DispatchQueue(label: "net.mullvad.MullvadVPN.TextFileStream<\(Codec.self)>")
    private let stringStream: StringStreamIterator<Codec>

    init?(fileURL: URL, separator: Character, encoding: String.Encoding = .utf8) {
        let filePath = fileURL.path.utf8CString.map { $0 }

        let fileDescriptor = open(filePath, O_RDONLY)
        if (fileDescriptor == -1) {
            return nil
        }

        // Avoid blocking the read operation
        _ = fcntl(fileDescriptor, F_SETFL, O_NONBLOCK);

        let readSource = DispatchSource.makeReadSource(fileDescriptor: fileDescriptor, queue: queue)
        readSource.setCancelHandler {
            close(fileDescriptor)
        }

        stringStream = StringStreamIterator(separator: separator)

        self.readSource = readSource
        self.fileDescriptor = fileDescriptor
    }

    deinit {
        readSource.cancel()
    }

    func read(_ handler: @escaping (String) -> Void) {
        readSource.setEventHandler { [weak self] in
            guard let self = self else { return }

            let estimated = Int(self.readSource.data + 1)
            var buffer = [Codec.CodeUnit](repeating: 0, count: estimated)
            let actual = Darwin.read(self.fileDescriptor, &buffer, estimated)

            if actual == -1 {
                print("TextFileInputStream: read error: \(errno)")
            }

            if actual > 0 {
                let bytes = buffer[..<actual]
                self.stringStream.append(bytes: bytes)

                while let s = self.stringStream.next() {
                    handler(s)
                }
            }
        }
        readSource.activate()
    }

    func cancel() {
        readSource.cancel()
    }

}

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
