//
//  LibraryView.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-20.
//

import SwiftUI

struct LibraryView: View {
    @Environment(Library.self) var lib
    @State private var showEmulator = false
    @State private var showImporter = false

    @State private var play: Game?

    var body: some View {
        ScrollView {
            LazyVGrid(columns: Array(
                repeating: .init(.flexible()),
                count: 2
            )) {
                ForEach(lib.roms, id: \.self) { game in
                    LibraryItem(game: game, play: $play)
                }
            }
        }
        .navigationTitle("Library")
        .sheet(item: $play) { game in
            EmulatorView(game: game)
        }
        .toolbar {
            Button {
                showImporter.toggle()
            } label: {
                Label("Import", systemImage: "plus")
            }
            .fileImporter(
                isPresented: $showImporter,
                allowedContentTypes: [.gb, .gbc],
                allowsMultipleSelection: true
            ) { res in
                switch res {
                case let .success(file):
                    print(file)
                case let .failure(error):
                    print(error.localizedDescription)
                }
            }
        }
    }
}

#Preview {
    NavigationStack {
        LibraryView()
            .environment(Library())
    }
}
