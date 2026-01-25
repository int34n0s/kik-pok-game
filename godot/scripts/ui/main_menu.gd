extends CenterContainer

func _ready():
	pass

func _on_single_player_pressed():
	get_tree().change_scene_to_file("res://scenes/ui/level_select.tscn")

func _on_multiplayer_pressed():
	get_tree().change_scene_to_file("res://scenes/ui/room_select.tscn")

func _on_settings_pressed():
	get_tree().change_scene_to_file("res://scenes/ui/settings.tscn")

func _on_quit_pressed():
	get_tree().quit()
