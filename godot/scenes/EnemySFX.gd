extends Node3D

@export var velocity_source_path: NodePath
@export var footstep_path: NodePath
@export var special_paths: Array[NodePath] = []

@export var min_move_speed := 0.1
@export var special_min_interval := 6.0
@export var special_max_interval := 12.0

var vel_src: Node
var footstep: AudioStreamPlayer3D
var specials: Array[AudioStreamPlayer3D] = []
var special_timer: Timer

var _last_pos: Vector3
var _spd_from_pos: float = 0.0
var _pos_accum_time: float = 0.0

func _ready() -> void:
    vel_src = get_node_or_null(velocity_source_path)
    footstep = get_node_or_null(footstep_path)
    for p in special_paths:
        var s = get_node_or_null(p)
        if s:
            specials.append(s)

    special_timer = Timer.new()
    special_timer.one_shot = true
    add_child(special_timer)
    special_timer.timeout.connect(_on_special)

    if vel_src is Node3D:
        _last_pos = (vel_src as Node3D).global_position
    else:
        _last_pos = global_position

    _schedule_next_special()

func _physics_process(delta: float) -> void:
    var pos: Vector3 = (vel_src as Node3D).global_position if vel_src is Node3D else global_position
    var inst_spd: float = pos.distance_to(_last_pos) / max(delta, 0.0001)
    _last_pos = pos
    _pos_accum_time += delta
    if _pos_accum_time >= 0.1:
        _spd_from_pos = inst_spd
        _pos_accum_time = 0.0

    var speed := _get_speed()
    if speed > min_move_speed:
        if footstep and not footstep.playing:
            footstep.play()
    else:
        if footstep and footstep.playing:
            footstep.stop()

func _get_speed() -> float:
    if vel_src is CharacterBody3D:
        return (vel_src as CharacterBody3D).velocity.length()
    elif vel_src and vel_src.has_method("get_velocity"):
        var v = vel_src.call("get_velocity")
        if v is Vector3:
            return (v as Vector3).length()
    return _spd_from_pos

func _on_special() -> void:
    if specials.is_empty():
        _schedule_next_special()
        return

    var s: AudioStreamPlayer3D = specials.pick_random()
    if s.playing:
        s.stop()

    s.pitch_scale = randf_range(0.95, 1.05)
    s.volume_db  = randf_range(-2.0, 1.0)
    s.play()

    s.finished.connect(Callable(self, "_on_special_finished").bind(s), CONNECT_ONE_SHOT)

func _on_special_finished(_player: AudioStreamPlayer3D) -> void:
    special_timer.start(randf_range(special_min_interval, special_max_interval))

func _schedule_next_special() -> void:
    special_timer.start(randf_range(special_min_interval, special_max_interval))
