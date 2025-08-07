//
// LoggingNode.swift
//
// Created by Isaac Mills (08/06/25)
//

import SwiftGodot

@Godot
class LoggingNode: Node3D {
  override func _ready() {
    let span = Logger.span(location: "LogggingNode::ready", args: "")
    Logger.info(log: "Hello from Swift!")
    Logger.exit_span(span: span)
  }
}
