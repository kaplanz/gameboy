//
//  EmulatorView.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-20.
//

import SwiftUI

public extension Color {
    static let shell = Color(#colorLiteral(red: 0.7724417448, green: 0.7521434426, blue: 0.7400485873, alpha: 1))
    static let cover = Color(#colorLiteral(red: 0.5647058824, green: 0.5529411765, blue: 0.5725490196, alpha: 1))
    static let button = Color(#colorLiteral(red: 0.6039215686, green: 0.1333333333, blue: 0.3411764706, alpha: 1))
    static let screen = Color(#colorLiteral(red: 0.5490196078, green: 0.6274509804, blue: 0.3529411765, alpha: 1))
    static let navy = Color(#colorLiteral(red: 0.1254901961, green: 0.2784313725, blue: 0.5254901961, alpha: 1))
}

struct EmulatorView: View {
    @State var game: Game

    var body: some View {
        VStack {
            Spacer()
            Display()
            Spacer()
            Joypad()
        }
        .background(Color.shell)
    }
}

#Preview {
    EmulatorView(game: Game(path: Bundle.main.url(
        forResource: "roms/test/acid2/dmg-acid2",
        withExtension: "gb"
    )!))
}
