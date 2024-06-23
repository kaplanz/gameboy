//
//  Display.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-23.
//

import SwiftUI

struct Display: View {
    var body: some View {
        Rectangle()
            .fill(Color.cover)
            .frame(width: 260, height: 200)
            .clipShape(.rect(
                topLeadingRadius: 10,
                bottomLeadingRadius: 10,
                bottomTrailingRadius: 30,
                topTrailingRadius: 10
            ))
            .overlay {
                VStack {
                    HStack {
                        Screen()
                    }
                }
            }
    }
}

struct Screen: View {
    var body: some View {
        Rectangle()
            .fill(Color.screen)
            .frame(width: 160, height: 144)
    }
}

#Preview {
    Display()
}
