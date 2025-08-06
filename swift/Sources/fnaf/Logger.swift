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

  public static func enter_span(location: String, args: String) -> Int? {
    if Logger.shared.logger != nil {
      let id = Logger.shared.logger?.call(
        method: "span", Variant.init(location), Variant.init(args))
      return id.to()
    } else {
      return nil
    }
  }

  public static func exit_span(span: Int?) {
    if Logger.shared.logger != nil && span != nil {
      Logger.shared.logger?.call(method: "exit_span", Variant.init(span))
    }
  }
}
