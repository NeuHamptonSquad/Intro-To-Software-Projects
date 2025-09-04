extends AnimationPlayer

@onready var sfx_open  : AudioStreamPlayer3D = get_node("../VentSfxOpen")
@onready var sfx_close : AudioStreamPlayer3D = get_node("../VentSfxClose")

func _ready() -> void:
	animation_started.connect(_on_animation_started)

func _on_animation_started(name: StringName) -> void:
	if name == "vent/open": 
		var spd: float = get_playing_speed()
		if spd >= 0.0:
			if sfx_open.playing: sfx_open.stop()
			sfx_open.play()
		else:
			if sfx_close.playing: sfx_close.stop()
			sfx_close.play()
