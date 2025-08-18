extends Button

func _ready():
	pass

func _input(event):
	if event is InputEventKey and event.pressed and event.keycode == KEY_ENTER:
		pressed.emit()
		get_viewport().set_input_as_handled()
