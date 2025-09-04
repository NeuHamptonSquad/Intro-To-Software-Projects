//
// Vent.swift
//
// Created by Isaac Mills (08/27/25)
//

import SwiftGodot

@Godot
class Vent: Node3D {
  @Node("/root/GlobalTerminal") var terminal: Node?
  @Node("Vent") var vent: CSGBox3D?
  @Node("AnimationPlayer") var animationPlayer: AnimationPlayer?
  @Export var ventNumber: Int = 0

  var isOpen: Bool = false

  override func _ready() {
    let callable = Callable(object: self, method: StringName("_onVent"))
    if self.terminal == nil {
      Logger.error(log: "This Vent could not find the global terminal")
    }
    if self.vent == nil {
      Logger.error(log: "This Vent could not find it's vent CSGCombiner3D")
    }
    if self.animationPlayer == nil {
      Logger.error(log: "This Vent could not find it's AnimationPlayer")
    }
    self.terminal?.connect(signal: "vent_\(ventNumber)", callable: callable)
  }

  // override func _process(delta: Double) {
  //   if self.flicker_time != nil {
  //     if self.flicker_time ?? 1.0 < 1.0 {
  //       self.flicker_time? += delta
  //     } else {
  //       self.flicker_time = 1.0
  //     }
  //   }
  //   self.lightEnergy = easeOutElastic(x: self.flicker_time ?? 0.0)
  // }

  @Callable
  func _onVent(opened: Bool) {
    if opened {
      if !isOpen {
        animationPlayer?.play(name: "vent/open")
        isOpen = true
      }
    } else {
      if isOpen {
        animationPlayer?.playBackwards(name: "vent/open")
        isOpen = false
      }
    }
    Logger.info(log: "Received init")
    // self.flicker_time = 0.0
  }

}
