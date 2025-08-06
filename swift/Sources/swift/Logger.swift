//
// Logger.swift
//
//
// Created by Isaac Mills on 08/16/25
//

import SwiftGodot



class Logger {
	static let shared = Logger();
	static let logger = Engine.getSingleton(name: "Logger");

	private init() {}

	public func info(log: String) {
		if (logger != nil) {
			logger.call(method: "info", Variant.init(log));
		}
	}
}
