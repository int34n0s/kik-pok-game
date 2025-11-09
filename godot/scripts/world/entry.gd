extends MultiplayerManager

func _input(event: InputEvent) -> void:
	print("kek")
	if event is InputEventMouseMotion:
		var pos = event.position  # viewport (screen) coords
		print("pos", pos)
	if event is InputEventMouseButton:
		var click_pos = event.position
		print("click_pos", click_pos)
