extends CenterContainer

func _ready():
	pass

func _on_level_pressed(level_number: int):
	print("Level %d selected" % level_number)
	# TODO: Load level scene when available

func _on_back_pressed():
	get_tree().change_scene_to_file("res://scenes/ui/main_menu.tscn")
