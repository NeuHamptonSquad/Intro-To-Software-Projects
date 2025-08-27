//
// Gate.swift
//
// Created by Isaac Mills (08/27/25)
//

import SwiftGodot

@Godot
class Gate: Node3D {
  @Node("/root/GlobalTerminal") var terminal: Node?
  @Node("Gate") var gate: CSGCombiner3D?
  @Node("AnimationPlayer") var animationPlayer: AnimationPlayer?
  @Export var gateNumber: Int = 0

  override func _ready() {
    let callable = Callable(object: self, method: StringName("_onGate"))
    if self.terminal == nil {
      Logger.error(log: "This Gate could not find the global terminal")
    }
    if self.gate == nil {
      Logger.error(log: "This Gate could not find it's gate CSGCombiner3D")
    }
    if self.animationPlayer == nil {
      Logger.error(log: "This Gate could not find it's AnimationPlayer")
    }
    self.terminal?.connect(signal: "gate_\(gateNumber)", callable: callable)
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
  func _onGate(opened: Bool) {
    if opened {
      animationPlayer?.play(name: "gate/open")
    } else {
      animationPlayer?.playBackwards(name: "gate/open")
    }
    Logger.info(log: "Received init")
    // self.flicker_time = 0.0
  }

}
