//
// LoggingNode.swift
//
// Created by Isaac Mills (08/06/25)
//

import SwiftGodot

@Godot
class LoggingNode: Node3D {
	override func _ready() {
		Logger.info(log: "Hello World!")
	}
}
