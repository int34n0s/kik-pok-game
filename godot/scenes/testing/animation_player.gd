extends AnimationPlayer

#func _ready() -> void:
	#animation_started.connect(_on_animation_started)
#
#func _on_animation_started(anim_name: StringName) -> void:
	#advance(10000000000035.5)

#func _input(event: InputEvent) -> void:
	#if event.is_pressed():
		#seek(1.5, true)
