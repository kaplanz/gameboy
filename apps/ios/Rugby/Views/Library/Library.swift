//
//  Library.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-21.
//

import Foundation
import SwiftUI
import UniformTypeIdentifiers

@Observable
class Library {
    var roms: [Game] = load()
}

public extension UTType {
    static let gb = UTType(exportedAs: "dev.zakhary.rugby.gb")
    static let gbc = UTType(exportedAs: "dev.zakhary.rugby.gbc")
    static let sav = UTType(exportedAs: "dev.zakhary.rugby.sav")
}

struct Game: Hashable, Identifiable {
    let path: URL

    var name: String {
        path.deletingPathExtension().lastPathComponent
    }

    var data: [UInt8] {
        do {
            return try [UInt8](Data(contentsOf: path))
        } catch let error as NSError {
            fatalError(error.localizedDescription)
        }
    }

    var icon: Image? {
        let path = path.deletingPathExtension().appendingPathExtension("png")
        let data = try? Data(contentsOf: path)
        return data.flatMap { UIImage(data: $0) }.flatMap { Image(uiImage: $0) }
    }

    // impl Identifiable
    var id: Self { self }
}

private func load() -> [Game] {
    let path = URL.documentsDirectory.appendingPathComponent("roms")

    do {
        try FileManager.default.createDirectory(
            at: path,
            withIntermediateDirectories: true
        )
        return try FileManager.default.contentsOfDirectory(
            at: path,
            includingPropertiesForKeys: nil
        )
        .filter {
            ["gb", "gbc"].contains($0.pathExtension)
        }
        .map { Game(path: $0) }
    } catch let error as NSError {
        fatalError(error.localizedDescription)
    }
}
