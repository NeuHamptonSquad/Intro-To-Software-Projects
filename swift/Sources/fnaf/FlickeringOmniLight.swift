//
// FlickeringOmniLight.swift
//
// Created by Isaac Mills (08/12/25)
//

import Foundation
import SwiftGodot

func easeOutElastic(x: Double) -> Double {
  let c4 = (3.0 * Double.pi) / 3.0

  return x == 0.0
    ? 0.0
    : x == 1.0
      ? 1.0
      : pow(2.0, -10.0 * x) * sin((x * 10.0 - 0.75) * c4) + 1.0
}

@Godot
class FlickeringOmniLight3D: OmniLight3D {
  @Node("/root/GlobalTerminal") var terminal: Node?
  var flicker_time: Double? = nil

  override func _ready() {
    let callable = Callable(object: self, method: StringName("_onInit"))
    if self.terminal == nil {
      Logger.error(log: "This FlickeringOmniLight3D could not find the global terminal")
    }
    self.terminal?.connect(signal: "init", callable: callable)
  }

  override func _process(delta: Double) {
    if self.flicker_time != nil {
      if self.flicker_time ?? 1.0 < 1.0 {
        self.flicker_time? += delta
      } else {
        self.flicker_time = 1.0
      }
    }
    self.lightEnergy = easeOutElastic(x: self.flicker_time ?? 0.0)
  }

  @Callable
  func _onInit() {
    Logger.info(log: "Received init")
    self.flicker_time = 0.0
  }
}
