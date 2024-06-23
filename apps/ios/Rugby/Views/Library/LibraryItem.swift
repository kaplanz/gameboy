//
//  LibraryItem.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-23.
//

import SwiftUI

struct LibraryItem: View {
    @State var game: Game

    @Binding var play: Game?

    var body: some View {
        VStack {
            LibraryIcon(icon: game.icon)
                .contextMenu {
                    Button {
                        play = game
                    } label: {
                        Label("Play", systemImage: "play")
                    }
                    Divider()
                    Button {} label: {
                        Label("Get Info", systemImage: "info.circle")
                    }
                    Button {} label: {
                        Label("Rename", systemImage: "pencil")
                    }
                    Divider()
                    Button(role: .destructive) {} label: {
                        Label("Delete", systemImage: "trash")
                    }
                }
                .onTapGesture {
                    play = game
                }
            Text(game.name)
        }
        .padding()
    }
}

#Preview {
    LibraryItem(game: Game(path: Bundle.main.url(
        forResource: "roms/test/acid2/dmg-acid2",
        withExtension: "gb"
    )!), play: .constant(nil))
}

struct LibraryIcon: View {
    @State var icon: Image?

    var body: some View {
        return (icon ?? Image(systemName: "questionmark.app"))
            .aspectRatio(contentMode: .fit)
            .frame(width: 160, height: 144)
            .background(.white)
            .clipShape(.rect(cornerRadius: 15.0))
            .shadow(radius: 10)
    }
}
