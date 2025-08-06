//
// Logger.swift
//
//
// Created by Isaac Mills on 08/16/25
//

import SwiftGodot

class Logger {
	let logger: Object?

	private init() {
		self.logger = Engine.getSingleton(name: "Logger")
	}

	static var shared: Logger {
		return Logger()
	}

	public static func info(log: String) {
		if Logger.shared.logger != nil {
			Logger.shared.logger?.call(method: "info", Variant.init(log))
		}
	}
}
