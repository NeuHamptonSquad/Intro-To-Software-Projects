extends Node3D

func _ready() -> void:
	var span = Logger.span("Node3D::_ready", "")
	Logger.info("Hello world!")
	Logger.exit_span(span)
