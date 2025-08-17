//
// ProtoController.swift
//
// Created by Isaac Mills (08/13/25)
//

import SwiftGodot

func degToRad(_ angle: Float) -> Float {
  return angle * (Float.pi / 180.0)
}

func clamp(_ value: Float, min: Float, max: Float) -> Float {
  if value > max {
    return max
  } else if value < min {
    return min
  } else {
    return value
  }
}

@Godot
class ProtoController: CharacterBody3D {
  @Signal var positionChanged: SignalWithArguments<Float, Float>

  @Export var canMove = true
  @Export var hasGravity = true
  @Export var canJump = true
  @Export var canSprint = false
  @Export var canFreefly = false

  #exportGroup("Speeds")
  @Export var lookSpeed: Float = 0.002
  @Export var baseSpeed: Float = 7.0
  @Export var jumpVelocity: Float = 4.5
  @Export var sprintSpeed: Float = 10.0
  @Export var freeflySpeed: Float = 25.0

  #exportGroup("Input Actions")
  @Export var inputLeft: StringName = "ui_left"
  @Export var inputRight: StringName = "ui_right"
  @Export var inputForward: StringName = "ui_up"
  @Export var inputBack: StringName = "ui_down"
  @Export var inputJump: StringName = "ui_accept"
  @Export var inputSprint: StringName = "sprint"
  @Export var inputFreefly: StringName = "freefly"

  var mouseCaptured = false
  var lookRotation: Vector2 = Vector2(x: 0.0, y: 0.0)
  var moveSpeed: Float = 0.0
  var freeflying = false

  @Node("Head") var head: Node3D
  @Node("Collider") var collider: CollisionShape3D
  @Node("/root/GlobalTerminal") var terminal: Object

  override func _ready() {
    self.checkInputMappings()
    self.lookRotation.y = self.rotation.y
    self.lookRotation.x = self.head.rotation.x
    let callable = Callable(object: terminal, method: StringName("_on_player_pos_changed"))
    self.connect(signal: "position_changed", callable: callable)
  }

  override func _unhandledInput(event: InputEvent?) {
    if Input.isMouseButtonPressed(button: .left) {
      self.captureMouse()
    }
    if Input.isKeyPressed(keycode: .escape) {
      self.releaseMouse()
    }

    let event: InputEventMouseMotion? = event as? InputEventMouseMotion
    if self.mouseCaptured && event != nil {
      // Event is not nil
      self.rotateLook(rotInput: event!.relative)
    }

    if self.canFreefly && Input.isActionPressed(action: self.inputFreefly) {
      if !self.freeflying {
        self.enableFreefly()
      } else {
        self.disableFreefly()
      }
    }

  }

  override func _physicsProcess(delta: Double) {
    let delta = Vector3(x: Float(delta), y: Float(delta), z: Float(delta))
    if self.canFreefly && self.freeflying {
      var inputDir = Input.getVector(
        negativeX: self.inputLeft, positiveX: self.inputRight, negativeY: self.inputForward,
        positiveY: self.inputBack)
      var motion = (self.head.globalBasis * Vector3(x: inputDir.x, y: 0.0, z: inputDir.y))
        .normalized()
      motion *= Vector3(x: self.freeflySpeed, y: self.freeflySpeed, z: self.freeflySpeed) * delta
      self.moveAndCollide(motion: motion)
      return
    }

    if self.hasGravity {
      if !self.isOnFloor() {
        self.velocity += self.getGravity() * delta
      }
    }

    if self.canJump {
      if Input.isActionJustPressed(action: self.inputJump) && self.isOnFloor() {
        velocity.y = self.jumpVelocity
      }
    }

    if self.canSprint && Input.isActionPressed(action: self.inputSprint) {
      self.moveSpeed = self.sprintSpeed
    } else {
      self.moveSpeed = self.baseSpeed
    }

    if self.canMove {
      var inputDir = Input.getVector(
        negativeX: self.inputLeft, positiveX: self.inputRight, negativeY: self.inputForward,
        positiveY: self.inputBack)
      var moveDir = (self.transform.basis * Vector3(x: inputDir.x, y: 0.0, z: inputDir.y))
        .normalized()
      self.velocity.x = moveDir.x * self.moveSpeed
      self.velocity.z = moveDir.z * self.moveSpeed
    } else {
      self.velocity.x = 0
      self.velocity.y = 0
    }

    let willMove = self.velocity.x != 0 || self.velocity.y != 0
    self.moveAndSlide()

    if willMove {
      self.positionChanged.emit(self.position.x, self.position.z)
    }

  }

  func rotateLook(rotInput: Vector2) {
    self.lookRotation.x -= rotInput.y * self.lookSpeed
    self.lookRotation.x = clamp(self.lookRotation.x, min: degToRad(-85), max: degToRad(85))
    self.lookRotation.y -= rotInput.x * self.lookSpeed
    transform.basis = Basis()
    self.rotateY(angle: Double(self.lookRotation.y))
    self.head.transform.basis = Basis()
    self.head.rotateX(angle: Double(self.lookRotation.x))
  }

  func enableFreefly() {
    self.collider.disabled = true
    self.freeflying = true
    self.velocity = Vector3.zero
  }

  func disableFreefly() {
    self.collider.disabled = false
    self.freeflying = false
  }

  func captureMouse() {
    Input.mouseMode = .captured
    self.mouseCaptured = true
  }

  func releaseMouse() {
    Input.mouseMode = .visible
    self.mouseCaptured = false
  }

  func checkInputMappings() {
    if self.canMove && !InputMap.hasAction(self.inputLeft) {
      Logger.error(log: "Movement disabled. No InputAction found for inputLeft: " + inputLeft)
      self.canMove = false
    }
    if self.canMove && !InputMap.hasAction(self.inputRight) {
      Logger.error(log: "Movement disabled. No InputAction found for inputRight: " + inputRight)
      self.canMove = false
    }
    if self.canMove && !InputMap.hasAction(self.inputForward) {
      Logger.error(log: "Movement disabled. No InputAction found for inputForward: " + inputForward)
      self.canMove = false
    }
    if self.canMove && !InputMap.hasAction(self.inputBack) {
      Logger.error(log: "Movement disabled. No InputAction found for inputBack: " + inputBack)
      self.canMove = false
    }
    if self.canJump && !InputMap.hasAction(self.inputJump) {
      Logger.error(log: "Jumping disabled. No InputAction found for inputJump: " + inputJump)
      self.canJump = false
    }
    if self.canSprint && !InputMap.hasAction(self.inputSprint) {
      Logger.error(log: "Sprinting disabled. No InputAction found for inputSprint: " + inputSprint)
      self.canSprint = false
    }
    if self.canFreefly && !InputMap.hasAction(self.inputFreefly) {
      Logger.error(log: "Freefly disabled. No InputAction found for inputFreefly: " + inputFreefly)
      self.canFreefly = false
    }
  }
}
